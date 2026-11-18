//! API REST con axum. Swagger UI en `/docs` y el contrato en `/docs/openapi.json`.

mod error;
mod extractores;
mod sesiones;
mod usuarios;

pub use error::{CuerpoError, ErrorApi};

use std::time::Instant;

use axum::extract::{Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use tracing::Instrument;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use crate::aplicacion::Servicio;
use crate::dominio::ErrorDominio;
use crate::infra::claves::Jwks;
use error::CORRELACION;

/// NGINX la genera si falta y todos los servicios la propagan (documento v2, 8.1).
pub const CABECERA_CORRELACION: &str = "x-correlation-id";

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SmartFila · Identidad",
        version = "1.0.0",
        description = "Cuentas, sesión única (RN-07), roles y JWT RS256. \
                       Errores con el cuerpo común { code, message, details, traceId }."
    ),
    paths(
        usuarios::registrar,
        usuarios::perfil,
        sesiones::iniciar,
        sesiones::actual,
        sesiones::cerrar,
        jwks,
        salud,
    ),
    modifiers(&SeguridadBearer),
    tags(
        (name = "usuarios", description = "Registro y perfil"),
        (name = "sesiones", description = "Login, sesión actual y logout"),
        (name = "operación", description = "Clave pública y estado del servicio"),
    )
)]
pub struct ApiDoc;

struct SeguridadBearer;

impl Modify for SeguridadBearer {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi
            .components
            .get_or_insert_with(Default::default)
            .add_security_scheme(
                "bearer",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
    }
}

pub fn router(servicio: Servicio) -> Router {
    Router::new()
        .route("/api/v1/users", post(usuarios::registrar))
        .route("/api/v1/users/me", get(usuarios::perfil))
        .route("/api/v1/sessions", post(sesiones::iniciar))
        .route(
            "/api/v1/sessions/current",
            get(sesiones::actual).delete(sesiones::cerrar),
        )
        .route("/.well-known/jwks.json", get(jwks))
        .route("/health", get(salud))
        .fallback(no_encontrado)
        .with_state(servicio)
        .merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
        .layer(middleware::from_fn(correlacion))
}

/// Lee (o genera) el X-Correlation-Id, lo pone en el span de los logs y en la
/// respuesta, y registra una línea por petición.
async fn correlacion(peticion: Request, siguiente: Next) -> Response {
    let id = peticion
        .headers()
        .get(CABECERA_CORRELACION)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty() && v.len() <= 128 && v.chars().all(|c| c.is_ascii_graphic()))
        .map(str::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // El healthcheck de Docker llama cada pocos segundos: no ensucia los logs.
    let registrar = peticion.uri().path() != "/health";
    let span = tracing::info_span!(
        "http",
        correlation_id = %id,
        method = %peticion.method(),
        path = %peticion.uri().path(),
    );
    let inicio = Instant::now();
    let mut respuesta = CORRELACION
        .scope(id.clone(), siguiente.run(peticion))
        .instrument(span.clone())
        .await;

    if registrar {
        span.in_scope(|| {
            tracing::info!(
                status = respuesta.status().as_u16(),
                duration_ms = inicio.elapsed().as_millis() as u64,
                "petición atendida"
            )
        });
    }
    if let Ok(valor) = HeaderValue::from_str(&id) {
        respuesta.headers_mut().insert(CABECERA_CORRELACION, valor);
    }
    respuesta
}

async fn no_encontrado() -> ErrorApi {
    ErrorDominio::NoEncontrado.into()
}

/// Clave pública para validar el JWT sin llamar a Identidad.
#[utoipa::path(
    get,
    path = "/.well-known/jwks.json",
    tag = "operación",
    responses((status = 200, body = Jwks))
)]
async fn jwks(State(servicio): State<Servicio>) -> impl IntoResponse {
    (
        [(header::CACHE_CONTROL, "public, max-age=300")],
        Json(servicio.claves().jwks()),
    )
}

#[derive(Serialize, ToSchema)]
struct Salud {
    #[schema(example = "UP")]
    status: String,
    #[schema(example = "UP")]
    database: String,
}

/// Estado del servicio y de su base (lo usan el healthcheck de Docker y Kubernetes).
#[utoipa::path(
    get,
    path = "/health",
    tag = "operación",
    responses(
        (status = 200, description = "Servicio y base de datos responden", body = Salud),
        (status = 503, description = "La base de datos no responde", body = Salud),
    )
)]
async fn salud(State(servicio): State<Servicio>) -> (StatusCode, Json<Salud>) {
    let base_ok = sqlx::query("SELECT 1")
        .execute(servicio.pool())
        .await
        .is_ok();
    let (estado, texto) = if base_ok {
        (StatusCode::OK, "UP")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "DOWN")
    };
    (
        estado,
        Json(Salud {
            status: texto.to_owned(),
            database: texto.to_owned(),
        }),
    )
}
