//! Alta de cuentas (RN-06 y RN-08).
//!
//! Hay tres caminos y ninguno deja elegir el rol en la petición (OWASP API3):
//! - `POST /users` (público)                → siempre `CIUDADANO`.
//! - `POST /inspectors` (organizador)       → `INSPECTOR` del organizador del token.
//! - `POST /organizers` (admin)             → el organizador y su primera cuenta `ORGANIZADOR`.
//!
//! Primero se autoriza y después se valida: un ciudadano que llama a `POST /inspectors`
//! con datos basura recibe 403, no 400.
//!
//! La unicidad del C.I. y del correo la garantiza la base (`UNIQUE`); la capa de
//! persistencia traduce la violación a [`ErrorDominio::CiYaRegistrado`] o
//! [`ErrorDominio::EmailYaRegistrado`].

use uuid::Uuid;

use super::ci::Ci;
use super::error::{DetalleCampo, ErrorDominio};
use super::rol::{autorizar_alta_de_organizador, rol_para_nuevo_inspector, Actor, Rol};

pub const NOMBRE_MIN: usize = 2;
pub const NOMBRE_MAX: usize = 80;
pub const CONTRASENA_MIN: usize = 8;
/// Tope para que nadie mande megabytes a Argon2 y sature la CPU.
pub const CONTRASENA_MAX: usize = 128;

/// Lo que llega en el cuerpo de la petición, sin validar.
#[derive(Debug, Clone, Default)]
pub struct SolicitudCuenta {
    pub ci: String,
    pub ci_complemento: Option<String>,
    pub nombres: String,
    pub apellidos: String,
    pub email: String,
    pub celular: String,
    pub contrasena: String,
}

/// Cuenta lista para guardar. La contraseña sigue en texto plano: la capa de
/// infraestructura la pasa por Argon2id y solo guarda el hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuevaCuenta {
    pub ci: Ci,
    pub nombres: String,
    pub apellidos: String,
    pub email: String,
    pub celular: String,
    pub contrasena: String,
    pub rol: Rol,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuevoOrganizador {
    pub id: Uuid,
    pub nombre: String,
    pub email_contacto: String,
}

/// `POST /users`: registro público. Siempre crea un ciudadano.
pub fn registro_publico(s: SolicitudCuenta) -> Result<NuevaCuenta, ErrorDominio> {
    validar_cuenta(s, Rol::Ciudadano)
}

/// `POST /inspectors`: el organizador del token crea un inspector propio.
pub fn alta_de_inspector(actor: &Actor, s: SolicitudCuenta) -> Result<NuevaCuenta, ErrorDominio> {
    let rol = rol_para_nuevo_inspector(actor)?;
    validar_cuenta(s, rol)
}

/// `POST /organizers`: el admin crea un organizador junto con su primera cuenta.
/// `nuevo_id` lo genera la capa de aplicación (UUID v4, o v5 en la siembra).
pub fn alta_de_organizador(
    actor: &Actor,
    nuevo_id: Uuid,
    nombre: &str,
    email_contacto: &str,
    primera_cuenta: SolicitudCuenta,
) -> Result<(NuevoOrganizador, NuevaCuenta), ErrorDominio> {
    autorizar_alta_de_organizador(actor)?;

    let mut errores = Vec::new();
    let nombre = limpiar_espacios(nombre);
    let largo = nombre.chars().count();
    if !(NOMBRE_MIN..=120).contains(&largo) {
        errores.push(DetalleCampo::new(
            "name",
            format!("entre {NOMBRE_MIN} y 120 caracteres"),
        ));
    }
    let email_contacto = juntar(validar_email(email_contacto, "contactEmail"), &mut errores);

    // La primera cuenta se valida igual que cualquier otra; sus errores se suman a los del organizador.
    let cuenta = validar_cuenta(
        primera_cuenta,
        Rol::Organizador {
            organizador_id: nuevo_id,
        },
    );
    match (cuenta, errores.is_empty()) {
        (Ok(cuenta), true) => Ok((
            NuevoOrganizador {
                id: nuevo_id,
                nombre,
                email_contacto: email_contacto.unwrap_or_default(),
            },
            cuenta,
        )),
        (Ok(_), false) => Err(ErrorDominio::Validacion(errores)),
        (Err(ErrorDominio::Validacion(mas)), _) => {
            errores.extend(mas);
            Err(ErrorDominio::Validacion(errores))
        }
        (Err(otro), _) => Err(otro),
    }
}

/// Valida todos los campos y junta todos los errores en una sola respuesta 400.
fn validar_cuenta(s: SolicitudCuenta, rol: Rol) -> Result<NuevaCuenta, ErrorDominio> {
    let mut errores = Vec::new();

    let ci = juntar(Ci::nuevo(&s.ci, s.ci_complemento.as_deref()), &mut errores);
    let nombres = juntar(validar_nombre(&s.nombres, "firstNames"), &mut errores);
    let apellidos = juntar(validar_nombre(&s.apellidos, "lastNames"), &mut errores);
    let email = juntar(validar_email(&s.email, "email"), &mut errores);
    let celular = juntar(validar_celular(&s.celular), &mut errores);

    let largo = s.contrasena.chars().count();
    if !(CONTRASENA_MIN..=CONTRASENA_MAX).contains(&largo) {
        errores.push(DetalleCampo::new(
            "password",
            format!("entre {CONTRASENA_MIN} y {CONTRASENA_MAX} caracteres"),
        ));
    }

    match (ci, nombres, apellidos, email, celular) {
        (Some(ci), Some(nombres), Some(apellidos), Some(email), Some(celular))
            if errores.is_empty() =>
        {
            Ok(NuevaCuenta {
                ci,
                nombres,
                apellidos,
                email,
                celular,
                contrasena: s.contrasena,
                rol,
            })
        }
        _ => Err(ErrorDominio::Validacion(errores)),
    }
}

/// Guarda los errores de un campo y devuelve el valor si era válido.
fn juntar<T>(r: Result<T, Vec<DetalleCampo>>, errores: &mut Vec<DetalleCampo>) -> Option<T> {
    r.map_err(|e| errores.extend(e)).ok()
}

/// Quita espacios de los extremos y deja uno solo entre palabras.
fn limpiar_espacios(texto: &str) -> String {
    texto.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Letras (con tildes y ñ), espacios, apóstrofo, guion y punto.
fn validar_nombre(texto: &str, campo: &'static str) -> Result<String, Vec<DetalleCampo>> {
    let limpio = limpiar_espacios(texto);
    let largo = limpio.chars().count();
    if largo == 0 {
        return Err(vec![DetalleCampo::new(campo, "obligatorio")]);
    }
    if !(NOMBRE_MIN..=NOMBRE_MAX).contains(&largo) {
        return Err(vec![DetalleCampo::new(
            campo,
            format!("entre {NOMBRE_MIN} y {NOMBRE_MAX} caracteres"),
        )]);
    }
    if !limpio
        .chars()
        .all(|c| c.is_alphabetic() || matches!(c, ' ' | '\'' | '-' | '.'))
    {
        return Err(vec![DetalleCampo::new(
            campo,
            "solo letras, espacios, apóstrofo, guion o punto",
        )]);
    }
    Ok(limpio)
}

/// Comprobación de forma, no de existencia: no se envían correos de verificación.
fn validar_email(texto: &str, campo: &'static str) -> Result<String, Vec<DetalleCampo>> {
    let email = texto.trim().to_lowercase();
    if email.is_empty() {
        return Err(vec![DetalleCampo::new(campo, "obligatorio")]);
    }
    let bien_formado = email.len() <= 254
        && !email.chars().any(char::is_whitespace)
        && match email.split_once('@') {
            Some((local, dominio)) => {
                !local.is_empty()
                    && local.len() <= 64
                    && !dominio.contains('@')
                    && dominio.contains('.')
                    && !dominio.starts_with('.')
                    && !dominio.ends_with('.')
                    && !dominio.contains("..")
            }
            None => false,
        };
    if bien_formado {
        Ok(email)
    } else {
        Err(vec![DetalleCampo::new(campo, "correo inválido")])
    }
}

/// Celular boliviano: 8 dígitos que empiezan con 6 o 7. Acepta el prefijo +591 y
/// espacios o guiones; se guarda solo con los 8 dígitos.
fn validar_celular(texto: &str) -> Result<String, Vec<DetalleCampo>> {
    let digitos: String = texto.chars().filter(|c| !matches!(c, ' ' | '-')).collect();
    if digitos.is_empty() {
        return Err(vec![DetalleCampo::new("phone", "obligatorio")]);
    }
    let local = digitos.strip_prefix("+591").unwrap_or(&digitos);
    let valido = local.len() == 8
        && local.chars().all(|c| c.is_ascii_digit())
        && matches!(local.as_bytes()[0], b'6' | b'7');
    if valido {
        Ok(local.to_owned())
    } else {
        Err(vec![DetalleCampo::new(
            "phone",
            "celular de 8 dígitos que empiece con 6 o 7",
        )])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORG_A: Uuid = Uuid::from_u128(0xA);

    fn solicitud() -> SolicitudCuenta {
        SolicitudCuenta {
            ci: "1234567".into(),
            ci_complemento: None,
            nombres: "  Ana   María ".into(),
            apellidos: "Pérez Cruz".into(),
            email: " Ana.Perez@Example.COM ".into(),
            celular: "+591 712-34567".into(),
            contrasena: "clave-segura-1".into(),
        }
    }

    fn campos(e: ErrorDominio) -> Vec<&'static str> {
        e.detalles().iter().map(|d| d.campo).collect()
    }

    fn actor(rol: Rol) -> Actor {
        Actor {
            usuario_id: Uuid::from_u128(1),
            rol,
        }
    }

    #[test]
    fn el_registro_publico_crea_un_ciudadano_normalizado() {
        let c = registro_publico(solicitud()).unwrap();
        assert_eq!(c.rol, Rol::Ciudadano);
        assert_eq!(c.nombres, "Ana María");
        assert_eq!(c.email, "ana.perez@example.com");
        assert_eq!(c.celular, "71234567");
    }

    #[test]
    fn junta_todos_los_errores_en_una_respuesta() {
        let s = SolicitudCuenta {
            ci: "abc".into(),
            email: "sin-arroba".into(),
            contrasena: "corta".into(),
            ..solicitud()
        };
        let e = registro_publico(s).unwrap_err();
        assert_eq!(e.estado_http(), 400);
        assert_eq!(campos(e), ["ci", "email", "password"]);
    }

    #[test]
    fn todos_los_datos_son_obligatorios() {
        let e = registro_publico(SolicitudCuenta::default()).unwrap_err();
        assert_eq!(
            campos(e),
            [
                "ci",
                "firstNames",
                "lastNames",
                "email",
                "phone",
                "password"
            ]
        );
    }

    #[test]
    fn nombres_aceptan_tildes_y_rechazan_digitos() {
        assert!(validar_nombre("Ñuflo D'Ávila-Rojas", "firstNames").is_ok());
        assert!(validar_nombre("Ana2", "firstNames").is_err());
        assert!(validar_nombre("A", "firstNames").is_err());
    }

    #[test]
    fn correos() {
        for bueno in ["a@b.co", "ana.perez+cola@uni.edu.bo"] {
            assert!(validar_email(bueno, "email").is_ok(), "{bueno}");
        }
        for malo in [
            "a@b",
            "@b.com",
            "a@@b.com",
            "a@b..com",
            "a b@c.com",
            "a@.com",
        ] {
            assert!(validar_email(malo, "email").is_err(), "{malo}");
        }
    }

    #[test]
    fn celulares() {
        assert_eq!(validar_celular("71234567").unwrap(), "71234567");
        assert_eq!(validar_celular("+591 61234567").unwrap(), "61234567");
        for malo in ["41234567", "7123456", "712345678", "7123456a"] {
            assert!(validar_celular(malo).is_err(), "{malo}");
        }
    }

    #[test]
    fn limites_de_contrasena() {
        let con = |p: &str| {
            registro_publico(SolicitudCuenta {
                contrasena: p.into(),
                ..solicitud()
            })
        };
        assert!(con("1234567").is_err());
        assert!(con("12345678").is_ok());
        assert!(con(&"x".repeat(CONTRASENA_MAX)).is_ok());
        assert!(con(&"x".repeat(CONTRASENA_MAX + 1)).is_err());
    }

    #[test]
    fn el_inspector_queda_en_el_organizador_del_token() {
        let c = alta_de_inspector(
            &actor(Rol::Organizador {
                organizador_id: ORG_A,
            }),
            solicitud(),
        )
        .unwrap();
        assert_eq!(
            c.rol,
            Rol::Inspector {
                organizador_id: ORG_A
            }
        );
    }

    #[test]
    fn primero_autoriza_y_despues_valida() {
        let basura = SolicitudCuenta::default();
        assert_eq!(
            alta_de_inspector(&actor(Rol::Ciudadano), basura.clone()),
            Err(ErrorDominio::SinPermiso)
        );
        assert_eq!(
            alta_de_organizador(
                &actor(Rol::Organizador {
                    organizador_id: ORG_A
                }),
                ORG_A,
                "",
                "",
                basura
            ),
            Err(ErrorDominio::SinPermiso)
        );
    }

    #[test]
    fn el_admin_crea_organizador_con_su_primera_cuenta() {
        let nuevo = Uuid::from_u128(0xC);
        let (org, cuenta) = alta_de_organizador(
            &actor(Rol::Admin),
            nuevo,
            " GAMS  Sucre ",
            "Eventos@Sucre.gob.bo",
            solicitud(),
        )
        .unwrap();
        assert_eq!(org.nombre, "GAMS Sucre");
        assert_eq!(org.email_contacto, "eventos@sucre.gob.bo");
        assert_eq!(
            cuenta.rol,
            Rol::Organizador {
                organizador_id: nuevo
            }
        );
    }

    #[test]
    fn errores_del_organizador_y_de_su_cuenta_salen_juntos() {
        let s = SolicitudCuenta {
            ci: "".into(),
            ..solicitud()
        };
        let e = alta_de_organizador(&actor(Rol::Admin), ORG_A, "X", "malo", s).unwrap_err();
        assert_eq!(campos(e), ["name", "contactEmail", "ci"]);
    }
}
