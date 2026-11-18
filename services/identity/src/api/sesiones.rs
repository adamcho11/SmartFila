//! `/api/v1/sessions`: login, sesión actual y logout (RN-07, sesión única).

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::error::{CuerpoError, ErrorApi};
use super::extractores::{Autenticado, CuerpoJson};
use crate::aplicacion::Servicio;

#[derive(Default, Deserialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct LoginBody {
    #[schema(example = "1234567")]
    pub ci: String,
    #[schema(example = "1A")]
    pub ci_complement: Option<String>,
    #[schema(example = "una-clave-segura")]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenDto {
    /// JWT RS256 (ADR-012). Se envía como `Authorization: Bearer <token>`.
    pub access_token: String,
    #[schema(example = "Bearer")]
    pub token_type: String,
    /// Segundos de vida del token (600, o menos si la sesión vence antes).
    #[schema(example = 600)]
    pub expires_in: i64,
    /// Igual al claim `sid`.
    pub session_id: Uuid,
    /// La sesión dura 12 h desde el login.
    pub session_expires_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SesionDto {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Login con C.I. y contraseña. Revoca la sesión anterior del mismo usuario.
#[utoipa::path(
    post,
    path = "/api/v1/sessions",
    tag = "sesiones",
    request_body = LoginBody,
    responses(
        (status = 201, description = "Sesión abierta; la anterior queda revocada", body = TokenDto),
        (status = 400, description = "VALIDATION_ERROR: C.I. con formato inválido", body = CuerpoError),
        (status = 401, description = "INVALID_CREDENTIALS: C.I. o contraseña incorrectos", body = CuerpoError),
        (status = 403, description = "ACCOUNT_DISABLED", body = CuerpoError),
    )
)]
pub async fn iniciar(
    State(servicio): State<Servicio>,
    CuerpoJson(b): CuerpoJson<LoginBody>,
) -> Result<impl IntoResponse, ErrorApi> {
    let emitido = servicio
        .iniciar_sesion(&b.ci, b.ci_complement.as_deref(), b.password)
        .await?;
    tracing::info!(
        user_id = %emitido.claims.sub,
        session_id = %emitido.sesion.id,
        "sesión iniciada"
    );
    Ok((
        StatusCode::CREATED,
        [(header::CACHE_CONTROL, "no-store")],
        Json(TokenDto {
            access_token: emitido.token,
            token_type: "Bearer".to_owned(),
            expires_in: emitido.claims.exp - emitido.claims.iat,
            session_id: emitido.sesion.id,
            session_expires_at: emitido.sesion.expira_en,
        }),
    ))
}

/// Comprueba que la sesión del token siga activa.
#[utoipa::path(
    get,
    path = "/api/v1/sessions/current",
    tag = "sesiones",
    security(("bearer" = [])),
    responses(
        (status = 200, body = SesionDto),
        (status = 401, description = "UNAUTHENTICATED o SESSION_REVOKED", body = CuerpoError),
    )
)]
pub async fn actual(autenticado: Autenticado) -> Json<SesionDto> {
    let s = autenticado.sesion;
    Json(SesionDto {
        session_id: s.id,
        user_id: s.usuario_id,
        created_at: s.creada_en,
        expires_at: s.expira_en,
    })
}

/// Logout: revoca la sesión del token.
#[utoipa::path(
    delete,
    path = "/api/v1/sessions/current",
    tag = "sesiones",
    security(("bearer" = [])),
    responses(
        (status = 204, description = "Sesión cerrada"),
        (status = 401, description = "UNAUTHENTICATED o SESSION_REVOKED", body = CuerpoError),
    )
)]
pub async fn cerrar(
    State(servicio): State<Servicio>,
    autenticado: Autenticado,
) -> Result<StatusCode, ErrorApi> {
    servicio.cerrar_sesion(autenticado.sesion.id).await?;
    tracing::info!(
        user_id = %autenticado.claims.sub,
        session_id = %autenticado.sesion.id,
        "sesión cerrada"
    );
    Ok(StatusCode::NO_CONTENT)
}
