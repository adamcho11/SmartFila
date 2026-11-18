//! Cuerpo de error común a todos los servicios (documento v2, 8.2):
//! `{ code, message, details, traceId }`. Nunca se devuelven trazas de pila.

use axum::extract::rejection::JsonRejection;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::aplicacion::ErrorAplicacion;
use crate::dominio::ErrorDominio;

tokio::task_local! {
    /// X-Correlation-Id de la petición en curso; lo pone el middleware de `api`.
    pub(crate) static CORRELACION: String;
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CuerpoError {
    #[schema(example = "VALIDATION_ERROR")]
    pub code: String,
    #[schema(example = "Datos inválidos")]
    pub message: String,
    pub details: Vec<Detalle>,
    /// Igual a la cabecera X-Correlation-Id, para buscar la petición en los logs.
    #[schema(example = "4bf92f35-77b3-4da6-a3ce-929d0e0e4736")]
    pub trace_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Detalle {
    #[schema(example = "ci")]
    pub field: String,
    #[schema(example = "solo dígitos")]
    pub issue: String,
}

#[derive(Debug)]
pub enum ErrorApi {
    Aplicacion(ErrorAplicacion),
    /// Falta el JWT, es inválido o venció.
    NoAutenticado(&'static str),
    /// El cuerpo no es JSON o un campo tiene el tipo equivocado.
    CuerpoInvalido(String),
}

impl From<ErrorAplicacion> for ErrorApi {
    fn from(e: ErrorAplicacion) -> Self {
        Self::Aplicacion(e)
    }
}

impl From<ErrorDominio> for ErrorApi {
    fn from(e: ErrorDominio) -> Self {
        Self::Aplicacion(e.into())
    }
}

impl From<JsonRejection> for ErrorApi {
    fn from(r: JsonRejection) -> Self {
        Self::CuerpoInvalido(r.body_text())
    }
}

impl IntoResponse for ErrorApi {
    fn into_response(self) -> Response {
        let (estado, code, message, details) = match self {
            Self::Aplicacion(ErrorAplicacion::Dominio(e)) => (
                StatusCode::from_u16(e.estado_http()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                e.codigo(),
                capitalizar(&e.to_string()),
                e.detalles()
                    .iter()
                    .map(|d| Detalle {
                        field: d.campo.to_owned(),
                        issue: d.problema.clone(),
                    })
                    .collect(),
            ),
            Self::Aplicacion(ErrorAplicacion::BaseNoDisponible(e)) => {
                tracing::error!(error = %e, "base de datos no disponible");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "DEPENDENCY_UNAVAILABLE",
                    "Servicio temporalmente no disponible".to_owned(),
                    Vec::new(),
                )
            }
            Self::Aplicacion(ErrorAplicacion::Interno(e)) => {
                tracing::error!(error = %format!("{e:#}"), "error interno");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Error interno".to_owned(),
                    Vec::new(),
                )
            }
            Self::NoAutenticado(m) => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHENTICATED",
                m.to_owned(),
                Vec::new(),
            ),
            Self::CuerpoInvalido(m) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", m, Vec::new()),
        };

        let cuerpo = CuerpoError {
            code: code.to_owned(),
            message,
            details,
            trace_id: CORRELACION.try_with(Clone::clone).unwrap_or_default(),
        };
        let mut respuesta = (estado, Json(cuerpo)).into_response();
        if estado == StatusCode::UNAUTHORIZED {
            respuesta
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        respuesta
    }
}

fn capitalizar(texto: &str) -> String {
    let mut letras = texto.chars();
    match letras.next() {
        Some(primera) => primera.to_uppercase().chain(letras).collect(),
        None => String::new(),
    }
}
