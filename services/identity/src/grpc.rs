//! Servidor gRPC de Identidad. Contrato:
//! `contracts/proto/smartfila/identity/v1/identity.proto`.
//!
//! El gateway llama a `ValidateSession` antes de cada mutación para respetar la
//! sesión única (RN-07). Los códigos de estado están documentados en el `.proto`.

use tonic::{Request, Response, Status};
use tracing::Instrument;
use uuid::Uuid;

use crate::api::CABECERA_CORRELACION;
use crate::aplicacion::{ErrorAplicacion, EstadoSesion, Servicio};

pub mod pb {
    tonic::include_proto!("smartfila.identity.v1");

    /// Descriptor del contrato para la reflexión gRPC (grpcurl y Postman lo usan
    /// para listar los métodos sin tener el `.proto`).
    pub const DESCRIPTOR: &[u8] = tonic::include_file_descriptor_set!("identity_descriptor");
}

use pb::identity_service_server::{IdentityService, IdentityServiceServer};
use pb::{ValidateSessionRequest, ValidateSessionResponse};

pub struct IdentidadGrpc {
    servicio: Servicio,
}

impl IdentidadGrpc {
    pub fn new(servicio: Servicio) -> Self {
        Self { servicio }
    }

    pub fn servidor(servicio: Servicio) -> IdentityServiceServer<Self> {
        IdentityServiceServer::new(Self::new(servicio))
    }

    async fn validar(
        &self,
        peticion: ValidateSessionRequest,
    ) -> Result<ValidateSessionResponse, Status> {
        let sid = Uuid::parse_str(peticion.session_id.trim())
            .map_err(|_| Status::invalid_argument("session_id debe ser un UUID"))?;

        match self.servicio.estado_sesion(sid).await {
            Ok(EstadoSesion::Activa { sesion, usuario }) => Ok(ValidateSessionResponse {
                session_id: sesion.id.to_string(),
                user_id: usuario.id.to_string(),
                roles: vec![usuario.rol.nombre().to_owned()],
                organizer_id: usuario
                    .rol
                    .organizador_id()
                    .map(|id| id.to_string())
                    .unwrap_or_default(),
                expires_at: Some(prost_types::Timestamp {
                    seconds: sesion.expira_en.timestamp(),
                    nanos: sesion.expira_en.timestamp_subsec_nanos() as i32,
                }),
            }),
            Ok(EstadoSesion::Revocada) => Err(Status::unauthenticated("SESSION_REVOKED")),
            Ok(EstadoSesion::Vencida) => Err(Status::unauthenticated("SESSION_EXPIRED")),
            Ok(EstadoSesion::NoExiste) => Err(Status::not_found("SESSION_NOT_FOUND")),
            Err(ErrorAplicacion::BaseNoDisponible(e)) => {
                tracing::error!(error = %e, "base de datos no disponible");
                Err(Status::unavailable("DEPENDENCY_UNAVAILABLE"))
            }
            Err(e) => {
                tracing::error!(error = %e, "error interno");
                Err(Status::internal("INTERNAL_ERROR"))
            }
        }
    }
}

#[tonic::async_trait]
impl IdentityService for IdentidadGrpc {
    async fn validate_session(
        &self,
        peticion: Request<ValidateSessionRequest>,
    ) -> Result<Response<ValidateSessionResponse>, Status> {
        let correlacion = peticion
            .metadata()
            .get(CABECERA_CORRELACION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-")
            .to_owned();
        let span = tracing::info_span!(
            "grpc",
            rpc = "ValidateSession",
            correlation_id = %correlacion
        );

        let resultado = self
            .validar(peticion.into_inner())
            .instrument(span.clone())
            .await;
        span.in_scope(|| match &resultado {
            Ok(_) => tracing::info!(code = "OK", "rpc atendida"),
            Err(s) => tracing::info!(code = ?s.code(), detail = s.message(), "rpc atendida"),
        });
        resultado.map(Response::new)
    }
}
