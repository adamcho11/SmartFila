//! Sesión única (RN-07): un nuevo inicio de sesión revoca la sesión anterior.
//!
//! La sesión vive en la tabla `sessions` y su id viaja en el claim `sid`. El JWT dura
//! 10 min (ADR-012); la sesión dura más ([`DURACION_SESION`]) para que el frontend
//! pueda pedir un token nuevo sin volver a escribir la contraseña mientras espera en la
//! cola (ver [`Sesion::verificar_activa`] y `token::claims_para`).

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use super::error::ErrorDominio;

/// Vida máxima de una sesión desde el login. Ningún token se emite más allá de esta hora.
pub const DURACION_SESION: Duration = Duration::hours(12);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sesion {
    pub id: Uuid,
    pub usuario_id: Uuid,
    pub creada_en: DateTime<Utc>,
    pub expira_en: DateTime<Utc>,
    pub revocada_en: Option<DateTime<Utc>>,
}

impl Sesion {
    pub fn activa(&self, ahora: DateTime<Utc>) -> bool {
        self.revocada_en.is_none() && ahora < self.expira_en
    }

    /// La usan `GET /sessions/current` (el gateway la consulta en cada mutación) y la
    /// renovación del token.
    pub fn verificar_activa(&self, ahora: DateTime<Utc>) -> Result<(), ErrorDominio> {
        if self.activa(ahora) {
            Ok(())
        } else {
            Err(ErrorDominio::SesionInactiva)
        }
    }

    /// Logout (`DELETE /sessions/current`). Repetirlo no cambia la hora de revocación.
    pub fn revocar(&mut self, ahora: DateTime<Utc>) {
        self.revocada_en.get_or_insert(ahora);
    }
}

/// Resultado de un login: la sesión nueva y las que hay que marcar como revocadas en
/// la misma transacción.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InicioDeSesion {
    pub nueva: Sesion,
    pub revocar: Vec<Uuid>,
}

/// RN-07. Recibe todas las sesiones del usuario que siguen sin revocar.
///
/// Revoca también las vencidas: la base tiene `UNIQUE(user_id) WHERE revoked_at IS NULL`,
/// así que una sesión vencida sin revocar impediría insertar la nueva.
pub fn iniciar_sesion(
    usuario_id: Uuid,
    nuevo_id: Uuid,
    ahora: DateTime<Utc>,
    sin_revocar: &[Sesion],
) -> InicioDeSesion {
    let revocar = sin_revocar
        .iter()
        .filter(|s| s.usuario_id == usuario_id && s.revocada_en.is_none())
        .map(|s| s.id)
        .collect();
    InicioDeSesion {
        nueva: Sesion {
            id: nuevo_id,
            usuario_id,
            creada_en: ahora,
            expira_en: ahora + DURACION_SESION,
            revocada_en: None,
        },
        revocar,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const USUARIO: Uuid = Uuid::from_u128(100);

    fn t(h: u32, m: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 12, 8, h, m, 0).unwrap()
    }

    fn sesion(id: u128, creada: DateTime<Utc>) -> Sesion {
        iniciar_sesion(USUARIO, Uuid::from_u128(id), creada, &[]).nueva
    }

    #[test]
    fn la_primera_sesion_no_revoca_nada() {
        let inicio = iniciar_sesion(USUARIO, Uuid::from_u128(1), t(8, 0), &[]);
        assert!(inicio.revocar.is_empty());
        assert_eq!(inicio.nueva.expira_en, t(20, 0));
        assert!(inicio.nueva.activa(t(8, 0)));
    }

    #[test]
    fn un_segundo_login_revoca_el_primero() {
        let primera = sesion(1, t(8, 0));
        let inicio = iniciar_sesion(
            USUARIO,
            Uuid::from_u128(2),
            t(9, 0),
            std::slice::from_ref(&primera),
        );
        assert_eq!(inicio.revocar, vec![primera.id]);
        assert_eq!(inicio.nueva.id, Uuid::from_u128(2));
    }

    #[test]
    fn revoca_tambien_las_vencidas_sin_revocar() {
        let vieja = sesion(1, t(8, 0) - Duration::days(2));
        assert!(!vieja.activa(t(8, 0)));
        let inicio = iniciar_sesion(
            USUARIO,
            Uuid::from_u128(2),
            t(8, 0),
            std::slice::from_ref(&vieja),
        );
        assert_eq!(inicio.revocar, vec![vieja.id]);
    }

    #[test]
    fn no_toca_sesiones_ya_revocadas_ni_de_otro_usuario() {
        let mut revocada = sesion(1, t(7, 0));
        revocada.revocar(t(7, 30));
        let mut ajena = sesion(3, t(7, 0));
        ajena.usuario_id = Uuid::from_u128(999);
        let inicio = iniciar_sesion(USUARIO, Uuid::from_u128(2), t(8, 0), &[revocada, ajena]);
        assert!(inicio.revocar.is_empty());
    }

    #[test]
    fn una_sesion_revocada_deja_de_estar_activa() {
        let mut s = sesion(1, t(8, 0));
        s.revocar(t(8, 5));
        assert_eq!(
            s.verificar_activa(t(8, 6)),
            Err(ErrorDominio::SesionInactiva)
        );
    }

    #[test]
    fn revocar_dos_veces_conserva_la_primera_hora() {
        let mut s = sesion(1, t(8, 0));
        s.revocar(t(8, 5));
        s.revocar(t(9, 0));
        assert_eq!(s.revocada_en, Some(t(8, 5)));
    }

    #[test]
    fn la_sesion_vence_exactamente_a_las_12_horas() {
        let s = sesion(1, t(8, 0));
        assert!(s.activa(t(19, 59)));
        assert!(!s.activa(t(20, 0)));
    }
}
