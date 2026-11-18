//! Roles y permisos (RN-01, RN-08 y la matriz de permisos de la sección 8.7).
//!
//! El organizador va dentro del rol: un `ORGANIZADOR` o un `INSPECTOR` sin
//! organizador no se puede construir, y un `ADMIN` o `CIUDADANO` con organizador
//! tampoco. Es la misma restricción de la tabla `user_roles`, pero comprobada al compilar.

use uuid::Uuid;

use super::error::ErrorDominio;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rol {
    /// Administrador de la plataforma. Se siembra; nadie se registra como admin.
    Admin,
    Organizador {
        organizador_id: Uuid,
    },
    Inspector {
        organizador_id: Uuid,
    },
    Ciudadano,
}

impl Rol {
    /// Nombre en la columna `user_roles.role` y en el claim `roles` del JWT.
    pub fn nombre(&self) -> &'static str {
        match self {
            Self::Admin => "ADMIN",
            Self::Organizador { .. } => "ORGANIZADOR",
            Self::Inspector { .. } => "INSPECTOR",
            Self::Ciudadano => "CIUDADANO",
        }
    }

    /// Valor del claim `org`.
    pub fn organizador_id(&self) -> Option<Uuid> {
        match self {
            Self::Organizador { organizador_id } | Self::Inspector { organizador_id } => {
                Some(*organizador_id)
            }
            Self::Admin | Self::Ciudadano => None,
        }
    }

    /// Reconstruye el rol desde la base (`user_roles.role`, `user_roles.organizer_id`).
    /// Devuelve `None` si la fila es incoherente, por ejemplo un INSPECTOR sin organizador.
    pub fn desde_bd(nombre: &str, organizador_id: Option<Uuid>) -> Option<Self> {
        match (nombre, organizador_id) {
            ("ADMIN", None) => Some(Self::Admin),
            ("CIUDADANO", None) => Some(Self::Ciudadano),
            ("ORGANIZADOR", Some(id)) => Some(Self::Organizador { organizador_id: id }),
            ("INSPECTOR", Some(id)) => Some(Self::Inspector { organizador_id: id }),
            _ => None,
        }
    }

    /// RN-08: admin, organizador e inspector no compran. Los demás servicios lo
    /// deciden con el claim `roles`; aquí queda la regla escrita en un solo lugar.
    pub fn puede_comprar(&self) -> bool {
        matches!(self, Self::Ciudadano)
    }
}

/// Quién hace la petición. La capa HTTP lo arma con los claims de un JWT ya validado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Actor {
    pub usuario_id: Uuid,
    pub rol: Rol,
}

/// RN-08: solo el admin da de alta organizadores.
pub fn autorizar_alta_de_organizador(actor: &Actor) -> Result<(), ErrorDominio> {
    match actor.rol {
        Rol::Admin => Ok(()),
        _ => Err(ErrorDominio::SinPermiso),
    }
}

/// RN-08: un organizador crea inspectores, siempre de su propio organizador. El
/// organizador del inspector no viene en la petición, se toma del token (OWASP API1/API3).
pub fn rol_para_nuevo_inspector(actor: &Actor) -> Result<Rol, ErrorDominio> {
    match actor.rol {
        Rol::Organizador { organizador_id } => Ok(Rol::Inspector { organizador_id }),
        _ => Err(ErrorDominio::SinPermiso),
    }
}

/// Escritura sobre un recurso de un organizador (inspectores, sanciones): solo el
/// organizador dueño. Un organizador sobre recursos de otro recibe 403.
pub fn autorizar_gestion(actor: &Actor, organizador_del_recurso: Uuid) -> Result<(), ErrorDominio> {
    match actor.rol {
        Rol::Organizador { organizador_id } if organizador_id == organizador_del_recurso => Ok(()),
        _ => Err(ErrorDominio::SinPermiso),
    }
}

/// Lectura de un recurso de un organizador: el organizador dueño, o el admin, que ve
/// todo en modo lectura.
pub fn autorizar_lectura(actor: &Actor, organizador_del_recurso: Uuid) -> Result<(), ErrorDominio> {
    match actor.rol {
        Rol::Admin => Ok(()),
        _ => autorizar_gestion(actor, organizador_del_recurso),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORG_A: Uuid = Uuid::from_u128(0xA);
    const ORG_B: Uuid = Uuid::from_u128(0xB);

    fn actor(rol: Rol) -> Actor {
        Actor {
            usuario_id: Uuid::from_u128(1),
            rol,
        }
    }

    #[test]
    fn nombres_y_claim_org() {
        assert_eq!(Rol::Admin.nombre(), "ADMIN");
        assert_eq!(Rol::Ciudadano.organizador_id(), None);
        let insp = Rol::Inspector {
            organizador_id: ORG_A,
        };
        assert_eq!(insp.nombre(), "INSPECTOR");
        assert_eq!(insp.organizador_id(), Some(ORG_A));
    }

    #[test]
    fn desde_bd_rechaza_filas_incoherentes() {
        assert_eq!(
            Rol::desde_bd("INSPECTOR", Some(ORG_A)),
            Some(Rol::Inspector {
                organizador_id: ORG_A
            })
        );
        assert_eq!(Rol::desde_bd("INSPECTOR", None), None);
        assert_eq!(Rol::desde_bd("ORGANIZADOR", None), None);
        assert_eq!(Rol::desde_bd("CIUDADANO", Some(ORG_A)), None);
        assert_eq!(Rol::desde_bd("ROOT", None), None);
    }

    #[test]
    fn solo_el_ciudadano_compra() {
        assert!(Rol::Ciudadano.puede_comprar());
        assert!(!Rol::Admin.puede_comprar());
        assert!(!Rol::Organizador {
            organizador_id: ORG_A
        }
        .puede_comprar());
        assert!(!Rol::Inspector {
            organizador_id: ORG_A
        }
        .puede_comprar());
    }

    #[test]
    fn solo_el_admin_crea_organizadores() {
        assert!(autorizar_alta_de_organizador(&actor(Rol::Admin)).is_ok());
        for rol in [
            Rol::Ciudadano,
            Rol::Organizador {
                organizador_id: ORG_A,
            },
        ] {
            assert_eq!(
                autorizar_alta_de_organizador(&actor(rol)),
                Err(ErrorDominio::SinPermiso)
            );
        }
    }

    #[test]
    fn el_inspector_hereda_el_organizador_del_que_lo_crea() {
        let rol = rol_para_nuevo_inspector(&actor(Rol::Organizador {
            organizador_id: ORG_A,
        }))
        .unwrap();
        assert_eq!(
            rol,
            Rol::Inspector {
                organizador_id: ORG_A
            }
        );
    }

    #[test]
    fn ni_admin_ni_inspector_crean_inspectores() {
        for rol in [
            Rol::Admin,
            Rol::Ciudadano,
            Rol::Inspector {
                organizador_id: ORG_A,
            },
        ] {
            assert_eq!(
                rol_para_nuevo_inspector(&actor(rol)),
                Err(ErrorDominio::SinPermiso)
            );
        }
    }

    #[test]
    fn un_organizador_no_gestiona_lo_de_otro() {
        let org_a = actor(Rol::Organizador {
            organizador_id: ORG_A,
        });
        assert!(autorizar_gestion(&org_a, ORG_A).is_ok());
        assert_eq!(
            autorizar_gestion(&org_a, ORG_B),
            Err(ErrorDominio::SinPermiso)
        );
    }

    #[test]
    fn el_admin_lee_pero_no_gestiona() {
        let admin = actor(Rol::Admin);
        assert!(autorizar_lectura(&admin, ORG_A).is_ok());
        assert_eq!(
            autorizar_gestion(&admin, ORG_A),
            Err(ErrorDominio::SinPermiso)
        );
    }

    #[test]
    fn el_inspector_no_gestiona_ni_en_su_organizador() {
        let insp = actor(Rol::Inspector {
            organizador_id: ORG_A,
        });
        assert_eq!(
            autorizar_gestion(&insp, ORG_A),
            Err(ErrorDominio::SinPermiso)
        );
        assert_eq!(
            autorizar_lectura(&insp, ORG_A),
            Err(ErrorDominio::SinPermiso)
        );
    }
}
