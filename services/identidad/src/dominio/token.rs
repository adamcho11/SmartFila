//! Contenido del JWT (ADR-012, contratos v2 8.7).
//!
//! Aquí solo se decide qué va en el token y cuánto dura. La firma RS256 con la clave
//! privada la hace la capa de infraestructura (crate `jsonwebtoken`); Cola, Inventario,
//! Ventas y el gateway validan con la clave pública sin llamar a Identidad.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::ErrorDominio;
use super::sesion::Sesion;
use super::usuario::Usuario;

pub const EMISOR: &str = "smartfila-identidad";
pub const AUDIENCIA: &str = "smartfila";
pub const VIDA_TOKEN: Duration = Duration::minutes(10);

/// Mismos nombres y orden que el ejemplo del documento v2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub aud: String,
    pub sub: Uuid,
    pub sid: Uuid,
    pub roles: Vec<String>,
    /// Organizador del ORGANIZADOR o INSPECTOR; `null` para ADMIN y CIUDADANO.
    pub org: Option<Uuid>,
    /// C.I. canónico (`1234567` o `1234567-1A`): Ventas lo usa para pagador = titular.
    pub ci: String,
    pub name: String,
    pub iat: i64,
    pub exp: i64,
}

/// Arma los claims para el login y para la renovación del token.
///
/// - La sesión tiene que ser del usuario y estar activa: una sesión revocada por otro
///   login (RN-07) ya no obtiene tokens nuevos.
/// - `exp` nunca pasa del vencimiento de la sesión.
pub fn claims_para(
    usuario: &Usuario,
    sesion: &Sesion,
    ahora: DateTime<Utc>,
) -> Result<Claims, ErrorDominio> {
    if sesion.usuario_id != usuario.id {
        return Err(ErrorDominio::SesionInactiva);
    }
    sesion.verificar_activa(ahora)?;

    let exp = (ahora + VIDA_TOKEN).min(sesion.expira_en);
    Ok(Claims {
        iss: EMISOR.to_owned(),
        aud: AUDIENCIA.to_owned(),
        sub: usuario.id,
        sid: sesion.id,
        roles: vec![usuario.rol.nombre().to_owned()],
        org: usuario.rol.organizador_id(),
        ci: usuario.ci.to_string(),
        name: usuario.nombre_completo(),
        iat: ahora.timestamp(),
        exp: exp.timestamp(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::rol::Rol;
    use crate::dominio::sesion::{iniciar_sesion, DURACION_SESION};
    use crate::dominio::usuario::tests::usuario_con_rol;
    use chrono::TimeZone;

    fn ahora() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 12, 8, 10, 0, 0).unwrap()
    }

    fn sesion_de(u: &Usuario) -> Sesion {
        iniciar_sesion(u.id, Uuid::from_u128(9), ahora(), &[]).nueva
    }

    #[test]
    fn claims_de_un_ciudadano() {
        let u = usuario_con_rol(Rol::Ciudadano);
        let c = claims_para(&u, &sesion_de(&u), ahora()).unwrap();
        assert_eq!(c.iss, "smartfila-identidad");
        assert_eq!(c.aud, "smartfila");
        assert_eq!(c.sub, u.id);
        assert_eq!(c.sid, Uuid::from_u128(9));
        assert_eq!(c.roles, ["CIUDADANO"]);
        assert_eq!(c.org, None);
        assert_eq!(c.ci, "1234567-1A");
        assert_eq!(c.name, "Ana María Pérez Cruz");
        assert_eq!(c.exp - c.iat, 600);
    }

    #[test]
    fn el_inspector_lleva_su_organizador_en_org() {
        let org = Uuid::from_u128(0xA);
        let u = usuario_con_rol(Rol::Inspector {
            organizador_id: org,
        });
        let c = claims_para(&u, &sesion_de(&u), ahora()).unwrap();
        assert_eq!(c.roles, ["INSPECTOR"]);
        assert_eq!(c.org, Some(org));
    }

    #[test]
    fn org_nulo_se_serializa_como_null() {
        let u = usuario_con_rol(Rol::Ciudadano);
        let json = serde_json::to_value(claims_para(&u, &sesion_de(&u), ahora()).unwrap()).unwrap();
        assert!(json["org"].is_null());
        assert_eq!(json["sub"], "00000000-0000-0000-0000-000000000064");
    }

    #[test]
    fn una_sesion_revocada_no_obtiene_token() {
        let u = usuario_con_rol(Rol::Ciudadano);
        let mut s = sesion_de(&u);
        s.revocar(ahora());
        assert_eq!(
            claims_para(&u, &s, ahora()),
            Err(ErrorDominio::SesionInactiva)
        );
    }

    #[test]
    fn no_sirve_la_sesion_de_otro_usuario() {
        let u = usuario_con_rol(Rol::Ciudadano);
        let mut s = sesion_de(&u);
        s.usuario_id = Uuid::from_u128(999);
        assert_eq!(
            claims_para(&u, &s, ahora()),
            Err(ErrorDominio::SesionInactiva)
        );
    }

    #[test]
    fn el_token_no_dura_mas_que_la_sesion() {
        let u = usuario_con_rol(Rol::Ciudadano);
        let s = sesion_de(&u);
        let casi_al_final = ahora() + DURACION_SESION - Duration::minutes(3);
        let c = claims_para(&u, &s, casi_al_final).unwrap();
        assert_eq!(c.exp, s.expira_en.timestamp());
        assert_eq!(c.exp - c.iat, 180);
    }
}
