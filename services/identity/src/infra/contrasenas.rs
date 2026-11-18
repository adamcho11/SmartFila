//! Contraseñas con Argon2id (RN-06). Parámetros por defecto del crate: m = 19 MiB,
//! t = 2, p = 1, los mínimos que recomienda OWASP.
//!
//! Argon2 ocupa la CPU unos milisegundos: la capa de aplicación lo llama dentro de
//! `spawn_blocking` para no frenar al resto de las peticiones.

use std::sync::LazyLock;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Se compara contra este hash cuando el C.I. no existe, para que el login tarde lo
/// mismo y no revele qué C.I. están registrados.
pub static HASH_FICTICIO: LazyLock<String> =
    LazyLock::new(|| hashear("contrasena-que-nadie-usa").expect("hash ficticio"));

/// Devuelve el hash en formato PHC (`$argon2id$v=19$m=19456,t=2,p=1$...`), con la sal adentro.
pub fn hashear(contrasena: &str) -> Result<String, argon2::password_hash::Error> {
    let sal = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(contrasena.as_bytes(), &sal)?
        .to_string())
}

/// `false` también si el hash guardado está corrupto.
pub fn verificar(contrasena: &str, hash: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|h| {
        Argon2::default()
            .verify_password(contrasena.as_bytes(), &h)
            .is_ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifica_la_contrasena_correcta_y_rechaza_otra() {
        let hash = hashear("secreta-123").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(verificar("secreta-123", &hash));
        assert!(!verificar("secreta-124", &hash));
    }

    #[test]
    fn la_misma_contrasena_da_hashes_distintos() {
        assert_ne!(hashear("igual").unwrap(), hashear("igual").unwrap());
    }

    #[test]
    fn un_hash_corrupto_no_verifica() {
        assert!(!verificar("lo-que-sea", "no-es-un-hash"));
    }
}
