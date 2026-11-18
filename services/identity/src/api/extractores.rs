//! Extractores propios de axum.

use axum::extract::{FromRequest, FromRequestParts};
use axum::http::{header, request::Parts};

use super::error::ErrorApi;
use crate::aplicacion::{EstadoSesion, Servicio};
use crate::dominio::sesion::Sesion;
use crate::dominio::token::Claims;
use crate::dominio::usuario::Usuario;
use crate::dominio::ErrorDominio;

/// Como `axum::Json`, pero un cuerpo inválido responde con el error común (400).
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ErrorApi))]
pub struct CuerpoJson<T>(pub T);

/// Petición con `Authorization: Bearer <jwt>` válido y sesión activa.
///
/// Los demás servicios solo validan el JWT con la clave pública. Identidad, que es
/// el dueño de las sesiones, además comprueba que la sesión no esté revocada.
pub struct Autenticado {
    pub claims: Claims,
    pub sesion: Sesion,
    pub usuario: Usuario,
}

impl FromRequestParts<Servicio> for Autenticado {
    type Rejection = ErrorApi;

    async fn from_request_parts(
        parts: &mut Parts,
        servicio: &Servicio,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split_once(' '))
            .filter(|(esquema, _)| esquema.eq_ignore_ascii_case("bearer"))
            .map(|(_, token)| token.trim())
            .filter(|token| !token.is_empty())
            .ok_or(ErrorApi::NoAutenticado("Falta el token Bearer"))?;

        let claims = servicio
            .claves()
            .verificar(token)
            .map_err(|_| ErrorApi::NoAutenticado("Token inválido o vencido"))?;

        match servicio.estado_sesion(claims.sid).await? {
            EstadoSesion::Activa { sesion, usuario } if usuario.id == claims.sub => Ok(Self {
                claims,
                sesion,
                usuario: *usuario,
            }),
            _ => Err(ErrorDominio::SesionInactiva.into()),
        }
    }
}
