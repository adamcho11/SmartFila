//! Cuenta de usuario y las reglas que dependen de ella: quién puede iniciar
//! sesión (RN-07) y la baja de inspectores (RN-08).

use uuid::Uuid;

use super::ci::Ci;
use super::error::ErrorDominio;
use super::rol::{autorizar_gestion, Actor, Rol};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoCuenta {
    Activa,
    /// Inspector dado de baja por su organizador. Las filas no se borran, porque
    /// las verificaciones de Ventas guardan quién las hizo.
    Deshabilitada,
}

impl EstadoCuenta {
    /// Valor de la columna `users.status`.
    pub fn nombre(&self) -> &'static str {
        match self {
            Self::Activa => "ACTIVE",
            Self::Deshabilitada => "DISABLED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usuario {
    pub id: Uuid,
    pub ci: Ci,
    pub nombres: String,
    pub apellidos: String,
    pub email: String,
    pub celular: String,
    pub rol: Rol,
    pub estado: EstadoCuenta,
}

impl Usuario {
    /// Va en el claim `name` y en el comprobante nominativo.
    pub fn nombre_completo(&self) -> String {
        format!("{} {}", self.nombres, self.apellidos)
    }
}

/// Decide si se puede abrir una sesión, después de buscar al usuario y verificar la
/// contraseña con Argon2id (eso lo hace la capa de infraestructura).
///
/// Un C.I. inexistente y una contraseña incorrecta dan el mismo error. Para que el
/// tiempo de respuesta tampoco los distinga, la infraestructura debe calcular un hash
/// de todas formas cuando el usuario no existe.
pub fn autorizar_inicio_de_sesion(
    usuario: Option<&Usuario>,
    contrasena_correcta: bool,
) -> Result<&Usuario, ErrorDominio> {
    match usuario {
        Some(u) if contrasena_correcta => match u.estado {
            EstadoCuenta::Activa => Ok(u),
            EstadoCuenta::Deshabilitada => Err(ErrorDominio::CuentaDeshabilitada),
        },
        _ => Err(ErrorDominio::CredencialesInvalidas),
    }
}

/// `DELETE /inspectors/{id}`: el organizador deshabilita a uno de sus inspectores.
/// Si el id no es de un inspector, responde 404, como si no existiera. Después, la
/// capa de aplicación revoca sus sesiones.
pub fn deshabilitar_inspector(actor: &Actor, usuario: &mut Usuario) -> Result<(), ErrorDominio> {
    let Rol::Inspector { organizador_id } = usuario.rol else {
        return Err(ErrorDominio::NoEncontrado);
    };
    autorizar_gestion(actor, organizador_id)?;
    usuario.estado = EstadoCuenta::Deshabilitada;
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn usuario_con_rol(rol: Rol) -> Usuario {
        Usuario {
            id: Uuid::from_u128(100),
            ci: Ci::nuevo("1234567", Some("1A")).unwrap(),
            nombres: "Ana María".into(),
            apellidos: "Pérez Cruz".into(),
            email: "ana@example.com".into(),
            celular: "71234567".into(),
            rol,
            estado: EstadoCuenta::Activa,
        }
    }

    const ORG_A: Uuid = Uuid::from_u128(0xA);
    const ORG_B: Uuid = Uuid::from_u128(0xB);

    #[test]
    fn nombre_completo() {
        assert_eq!(
            usuario_con_rol(Rol::Ciudadano).nombre_completo(),
            "Ana María Pérez Cruz"
        );
    }

    #[test]
    fn inicio_de_sesion_correcto() {
        let u = usuario_con_rol(Rol::Ciudadano);
        assert_eq!(autorizar_inicio_de_sesion(Some(&u), true), Ok(&u));
    }

    #[test]
    fn usuario_inexistente_y_contrasena_mala_dan_el_mismo_error() {
        let u = usuario_con_rol(Rol::Ciudadano);
        assert_eq!(
            autorizar_inicio_de_sesion(None, false),
            Err(ErrorDominio::CredencialesInvalidas)
        );
        assert_eq!(
            autorizar_inicio_de_sesion(Some(&u), false),
            Err(ErrorDominio::CredencialesInvalidas)
        );
    }

    #[test]
    fn cuenta_deshabilitada_no_inicia_sesion() {
        let mut u = usuario_con_rol(Rol::Inspector {
            organizador_id: ORG_A,
        });
        u.estado = EstadoCuenta::Deshabilitada;
        assert_eq!(
            autorizar_inicio_de_sesion(Some(&u), true),
            Err(ErrorDominio::CuentaDeshabilitada)
        );
    }

    #[test]
    fn deshabilitada_y_contrasena_mala_no_revela_el_estado() {
        let mut u = usuario_con_rol(Rol::Inspector {
            organizador_id: ORG_A,
        });
        u.estado = EstadoCuenta::Deshabilitada;
        assert_eq!(
            autorizar_inicio_de_sesion(Some(&u), false),
            Err(ErrorDominio::CredencialesInvalidas)
        );
    }

    #[test]
    fn el_organizador_deshabilita_a_su_inspector() {
        let org = Actor {
            usuario_id: Uuid::from_u128(1),
            rol: Rol::Organizador {
                organizador_id: ORG_A,
            },
        };
        let mut insp = usuario_con_rol(Rol::Inspector {
            organizador_id: ORG_A,
        });
        deshabilitar_inspector(&org, &mut insp).unwrap();
        assert_eq!(insp.estado, EstadoCuenta::Deshabilitada);
    }

    #[test]
    fn no_deshabilita_inspectores_de_otro_organizador() {
        let org_b = Actor {
            usuario_id: Uuid::from_u128(1),
            rol: Rol::Organizador {
                organizador_id: ORG_B,
            },
        };
        let mut insp = usuario_con_rol(Rol::Inspector {
            organizador_id: ORG_A,
        });
        assert_eq!(
            deshabilitar_inspector(&org_b, &mut insp),
            Err(ErrorDominio::SinPermiso)
        );
        assert_eq!(insp.estado, EstadoCuenta::Activa);
    }

    #[test]
    fn un_ciudadano_no_es_un_inspector() {
        let org = Actor {
            usuario_id: Uuid::from_u128(1),
            rol: Rol::Organizador {
                organizador_id: ORG_A,
            },
        };
        let mut ciudadano = usuario_con_rol(Rol::Ciudadano);
        assert_eq!(
            deshabilitar_inspector(&org, &mut ciudadano),
            Err(ErrorDominio::NoEncontrado)
        );
    }
}
