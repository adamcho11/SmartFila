//! `/api/v1/users`: registro público y perfil propio.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::error::{CuerpoError, ErrorApi};
use super::extractores::{Autenticado, CuerpoJson};
use crate::aplicacion::Servicio;
use crate::dominio::registro::SolicitudCuenta;
use crate::dominio::usuario::Usuario;

/// Los campos que faltan llegan vacíos y el dominio responde "obligatorio" para
/// cada uno, todos en la misma respuesta 400.
#[derive(Default, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct RegistroBody {
    #[schema(example = "1234567")]
    pub ci: String,
    /// Complemento de 2 caracteres (opcional).
    #[schema(example = "1A")]
    pub ci_complement: Option<String>,
    #[schema(example = "Ana María")]
    pub first_names: String,
    #[schema(example = "Pérez Cruz")]
    pub last_names: String,
    #[schema(example = "ana@example.com")]
    pub email: String,
    /// Celular boliviano de 8 dígitos que empieza en 6 o 7.
    #[schema(example = "71234567")]
    pub phone: String,
    #[schema(example = "una-clave-segura", min_length = 8, max_length = 128)]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsuarioDto {
    pub id: Uuid,
    /// Forma canónica: número y complemento separados por guion.
    #[schema(example = "1234567-1A")]
    pub ci: String,
    #[schema(example = "Ana María")]
    pub first_names: String,
    #[schema(example = "Pérez Cruz")]
    pub last_names: String,
    #[schema(example = "Ana María Pérez Cruz")]
    pub full_name: String,
    #[schema(example = "ana@example.com")]
    pub email: String,
    #[schema(example = "71234567")]
    pub phone: String,
    /// ADMIN, ORGANIZADOR, INSPECTOR o CIUDADANO.
    #[schema(example = "CIUDADANO")]
    pub role: String,
    /// Solo para ORGANIZADOR e INSPECTOR.
    pub organizer_id: Option<Uuid>,
    #[schema(example = "ACTIVE")]
    pub status: String,
}

impl From<&Usuario> for UsuarioDto {
    fn from(u: &Usuario) -> Self {
        Self {
            id: u.id,
            ci: u.ci.to_string(),
            first_names: u.nombres.clone(),
            last_names: u.apellidos.clone(),
            full_name: u.nombre_completo(),
            email: u.email.clone(),
            phone: u.celular.clone(),
            role: u.rol.nombre().to_owned(),
            organizer_id: u.rol.organizador_id(),
            status: u.estado.nombre().to_owned(),
        }
    }
}

/// Registro de ciudadano (RN-06). El rol no se elige: siempre es CIUDADANO.
#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "usuarios",
    request_body = RegistroBody,
    responses(
        (status = 201, description = "Ciudadano registrado", body = UsuarioDto),
        (status = 400, description = "VALIDATION_ERROR: lista de campos inválidos", body = CuerpoError),
        (status = 409, description = "CI_ALREADY_REGISTERED o EMAIL_ALREADY_REGISTERED", body = CuerpoError),
    )
)]
pub async fn registrar(
    State(servicio): State<Servicio>,
    CuerpoJson(b): CuerpoJson<RegistroBody>,
) -> Result<(StatusCode, Json<UsuarioDto>), ErrorApi> {
    let usuario = servicio
        .registrar_ciudadano(SolicitudCuenta {
            ci: b.ci,
            ci_complemento: b.ci_complement,
            nombres: b.first_names,
            apellidos: b.last_names,
            email: b.email,
            celular: b.phone,
            contrasena: b.password,
        })
        .await?;
    tracing::info!(user_id = %usuario.id, "ciudadano registrado");
    Ok((StatusCode::CREATED, Json(UsuarioDto::from(&usuario))))
}

/// Perfil del usuario del token.
#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    tag = "usuarios",
    security(("bearer" = [])),
    responses(
        (status = 200, body = UsuarioDto),
        (status = 401, description = "UNAUTHENTICATED o SESSION_REVOKED", body = CuerpoError),
    )
)]
pub async fn perfil(autenticado: Autenticado) -> Json<UsuarioDto> {
    Json(UsuarioDto::from(&autenticado.usuario))
}
