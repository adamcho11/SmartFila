//! Errores del dominio y su traducción al catálogo común de errores (contratos v2, 8.2).
//!
//! La capa HTTP arma el cuerpo `{ code, message, details, traceId }` con
//! [`ErrorDominio::codigo`], [`ErrorDominio::estado_http`] y, en los errores de
//! validación, la lista de [`DetalleCampo`].

use serde::Serialize;
use thiserror::Error;

/// Un campo inválido. Se serializa como `{ "field": ..., "issue": ... }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetalleCampo {
    /// Nombre del campo tal como viaja en el JSON (camelCase).
    #[serde(rename = "field")]
    pub campo: &'static str,
    #[serde(rename = "issue")]
    pub problema: String,
}

impl DetalleCampo {
    pub fn new(campo: &'static str, problema: impl Into<String>) -> Self {
        Self {
            campo,
            problema: problema.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorDominio {
    /// Uno o más campos con formato inválido. Se juntan todos para responder una sola vez.
    #[error("datos inválidos")]
    Validacion(Vec<DetalleCampo>),

    /// C.I. desconocido o contraseña incorrecta. Es el mismo error en los dos casos
    /// para no revelar qué C.I. están registrados.
    #[error("C.I. o contraseña incorrectos")]
    CredencialesInvalidas,

    #[error("la cuenta está deshabilitada")]
    CuentaDeshabilitada,

    /// Autenticado, pero sin permiso o el recurso es de otro organizador (OWASP API1).
    #[error("no tiene permiso para esta operación")]
    SinPermiso,

    #[error("no existe")]
    NoEncontrado,

    #[error("el C.I. ya está registrado")]
    CiYaRegistrado,

    #[error("el correo ya está registrado")]
    EmailYaRegistrado,

    #[error("la sanción ya fue levantada")]
    SancionYaLevantada,

    /// La sesión del token fue revocada (otro inicio de sesión) o venció.
    #[error("la sesión ya no está activa")]
    SesionInactiva,
}

impl ErrorDominio {
    /// Código del catálogo común. Los que no están en el documento v2 son
    /// propios de Identidad y deben agregarse a `contracts/openapi/identity.v1.yaml`.
    pub fn codigo(&self) -> &'static str {
        match self {
            Self::Validacion(_) => "VALIDATION_ERROR",
            Self::CredencialesInvalidas => "INVALID_CREDENTIALS",
            Self::CuentaDeshabilitada => "ACCOUNT_DISABLED",
            Self::SinPermiso => "FORBIDDEN",
            Self::NoEncontrado => "NOT_FOUND",
            Self::CiYaRegistrado => "CI_ALREADY_REGISTERED",
            Self::EmailYaRegistrado => "EMAIL_ALREADY_REGISTERED",
            Self::SancionYaLevantada => "SANCTION_ALREADY_LIFTED",
            Self::SesionInactiva => "SESSION_REVOKED",
        }
    }

    pub fn estado_http(&self) -> u16 {
        match self {
            Self::Validacion(_) => 400,
            Self::CredencialesInvalidas | Self::SesionInactiva => 401,
            Self::CuentaDeshabilitada | Self::SinPermiso => 403,
            Self::NoEncontrado => 404,
            Self::CiYaRegistrado | Self::EmailYaRegistrado | Self::SancionYaLevantada => 409,
        }
    }

    pub fn detalles(&self) -> &[DetalleCampo] {
        match self {
            Self::Validacion(d) => d,
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_validacion_es_400_con_detalles_en_camel_case() {
        let e = ErrorDominio::Validacion(vec![DetalleCampo::new("firstNames", "obligatorio")]);
        assert_eq!(e.estado_http(), 400);
        assert_eq!(e.codigo(), "VALIDATION_ERROR");
        let json = serde_json::to_string(e.detalles()).unwrap();
        assert_eq!(json, r#"[{"field":"firstNames","issue":"obligatorio"}]"#);
    }

    #[test]
    fn diferencia_401_y_403() {
        assert_eq!(ErrorDominio::CredencialesInvalidas.estado_http(), 401);
        assert_eq!(ErrorDominio::SesionInactiva.estado_http(), 401);
        assert_eq!(ErrorDominio::SinPermiso.estado_http(), 403);
    }

    #[test]
    fn los_duplicados_son_409() {
        assert_eq!(ErrorDominio::CiYaRegistrado.estado_http(), 409);
        assert_eq!(ErrorDominio::EmailYaRegistrado.estado_http(), 409);
        assert_eq!(ErrorDominio::SancionYaLevantada.estado_http(), 409);
    }
}
