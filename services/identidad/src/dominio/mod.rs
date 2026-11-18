//! Reglas de negocio de Identidad (documento técnico v2, sección de reglas):
//!
//! - RN-06 Registro con C.I. + complemento, datos obligatorios y contraseña. → [`registro`], [`ci`]
//! - RN-07 Sesión única: un nuevo inicio de sesión revoca la anterior.       → [`sesion`]
//! - RN-08 El registro público crea solo ciudadanos; el admin crea
//!   organizadores y cada organizador crea sus inspectores.                → [`rol`], [`usuario`]
//! - RN-26 Sanciones: se crean al confirmarse un incidente y solo el
//!   organizador dueño las levanta.                                         → [`sancion`]
//! - ADR-012 JWT RS256 con claims `sub`, `sid`, `roles`, `org`, `ci`, `name`. → [`token`]
//!
//! Nada de este módulo hace E/S ni genera valores aleatorios: los IDs y la hora
//! llegan como parámetros, así las pruebas son deterministas.

pub mod ci;
pub mod error;
pub mod registro;
pub mod rol;
pub mod sancion;
pub mod sesion;
pub mod token;
pub mod usuario;

pub use error::{DetalleCampo, ErrorDominio};
