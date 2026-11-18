//! Sanciones (RN-26), la parte que le toca a Identidad.
//!
//! 1. Ventas publica `sancion.confirmada` cuando el organizador confirma un incidente.
//! 2. Identidad la consume y crea una sanción `ACTIVE` para ese C.I. y ese organizador.
//!    (Cola consume el mismo evento y bloquea los turnos por su cuenta.)
//! 3. Solo el organizador dueño la levanta (`DELETE /sanctions/{id}`); Identidad
//!    publica `sancion.levantada` y Cola quita el bloqueo.
//!
//! La idempotencia del consumidor (mismo `eventId` o mismo incidente entregado dos
//! veces) la dan `processed_messages` y `UNIQUE(incident_id)` en la base.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::ErrorDominio;
use super::rol::{autorizar_gestion, Actor};

/// `data` del evento `sancion.confirmada` (productor: Ventas).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SancionConfirmada {
    pub incidente_id: Uuid,
    pub organizador_id: Uuid,
    pub usuario_id: Uuid,
}

/// `data` del evento `sancion.levantada` (productor: Identidad).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SancionLevantada {
    pub sancion_id: Uuid,
    pub organizador_id: Uuid,
    pub usuario_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EstadoSancion {
    Activa,
    Levantada { en: DateTime<Utc>, por: Uuid },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sancion {
    pub id: Uuid,
    pub usuario_id: Uuid,
    pub organizador_id: Uuid,
    pub incidente_id: Uuid,
    pub creada_en: DateTime<Utc>,
    pub estado: EstadoSancion,
}

impl Sancion {
    /// Paso 2: la sanción nace activa a partir del evento de Ventas.
    pub fn desde_incidente(id: Uuid, evento: &SancionConfirmada, ahora: DateTime<Utc>) -> Self {
        Self {
            id,
            usuario_id: evento.usuario_id,
            organizador_id: evento.organizador_id,
            incidente_id: evento.incidente_id,
            creada_en: ahora,
            estado: EstadoSancion::Activa,
        }
    }

    pub fn activa(&self) -> bool {
        self.estado == EstadoSancion::Activa
    }

    /// Paso 3: devuelve los datos del evento `sancion.levantada`, que se guardan en el
    /// outbox en la misma transacción.
    pub fn levantar(
        &mut self,
        actor: &Actor,
        ahora: DateTime<Utc>,
    ) -> Result<SancionLevantada, ErrorDominio> {
        autorizar_gestion(actor, self.organizador_id)?;
        if !self.activa() {
            return Err(ErrorDominio::SancionYaLevantada);
        }
        self.estado = EstadoSancion::Levantada {
            en: ahora,
            por: actor.usuario_id,
        };
        Ok(SancionLevantada {
            sancion_id: self.id,
            organizador_id: self.organizador_id,
            usuario_id: self.usuario_id,
        })
    }
}

/// La sanción vale solo para los eventos de ese organizador: el mismo C.I. puede
/// seguir comprando en los eventos de otros.
pub fn esta_sancionado(sanciones: &[Sancion], usuario_id: Uuid, organizador_id: Uuid) -> bool {
    sanciones
        .iter()
        .any(|s| s.activa() && s.usuario_id == usuario_id && s.organizador_id == organizador_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::rol::Rol;
    use chrono::TimeZone;

    const ORG_A: Uuid = Uuid::from_u128(0xA);
    const ORG_B: Uuid = Uuid::from_u128(0xB);
    const CIUDADANO: Uuid = Uuid::from_u128(100);

    fn ahora() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 12, 8, 10, 0, 0).unwrap()
    }

    fn sancion_de(org: Uuid) -> Sancion {
        let evento = SancionConfirmada {
            incidente_id: Uuid::from_u128(50),
            organizador_id: org,
            usuario_id: CIUDADANO,
        };
        Sancion::desde_incidente(Uuid::from_u128(7), &evento, ahora())
    }

    fn organizador(org: Uuid) -> Actor {
        Actor {
            usuario_id: Uuid::from_u128(1),
            rol: Rol::Organizador {
                organizador_id: org,
            },
        }
    }

    #[test]
    fn nace_activa_desde_el_evento() {
        let s = sancion_de(ORG_A);
        assert!(s.activa());
        assert_eq!(s.organizador_id, ORG_A);
        assert_eq!(s.usuario_id, CIUDADANO);
    }

    #[test]
    fn solo_bloquea_en_los_eventos_de_ese_organizador() {
        let sanciones = [sancion_de(ORG_A)];
        assert!(esta_sancionado(&sanciones, CIUDADANO, ORG_A));
        assert!(!esta_sancionado(&sanciones, CIUDADANO, ORG_B));
    }

    #[test]
    fn el_organizador_dueno_la_levanta_y_se_publica_el_evento() {
        let mut s = sancion_de(ORG_A);
        let evento = s.levantar(&organizador(ORG_A), ahora()).unwrap();
        assert_eq!(
            evento,
            SancionLevantada {
                sancion_id: s.id,
                organizador_id: ORG_A,
                usuario_id: CIUDADANO
            }
        );
        assert_eq!(
            s.estado,
            EstadoSancion::Levantada {
                en: ahora(),
                por: Uuid::from_u128(1)
            }
        );
        assert!(!esta_sancionado(&[s], CIUDADANO, ORG_A));
    }

    #[test]
    fn otro_organizador_no_la_levanta() {
        let mut s = sancion_de(ORG_A);
        assert_eq!(
            s.levantar(&organizador(ORG_B), ahora()),
            Err(ErrorDominio::SinPermiso)
        );
        assert!(s.activa());
    }

    #[test]
    fn ni_el_admin_ni_un_inspector_la_levantan() {
        let mut s = sancion_de(ORG_A);
        for rol in [
            Rol::Admin,
            Rol::Inspector {
                organizador_id: ORG_A,
            },
        ] {
            let actor = Actor {
                usuario_id: Uuid::from_u128(2),
                rol,
            };
            assert_eq!(s.levantar(&actor, ahora()), Err(ErrorDominio::SinPermiso));
        }
    }

    #[test]
    fn levantarla_dos_veces_es_409_y_no_publica_otro_evento() {
        let mut s = sancion_de(ORG_A);
        s.levantar(&organizador(ORG_A), ahora()).unwrap();
        let e = s.levantar(&organizador(ORG_A), ahora()).unwrap_err();
        assert_eq!(e, ErrorDominio::SancionYaLevantada);
        assert_eq!(e.estado_http(), 409);
    }
}
