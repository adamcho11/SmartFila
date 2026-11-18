//! Configuración por variables de entorno (ver `.env.example` en la raíz del repo).

use std::net::{Ipv4Addr, SocketAddr};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub puerto_http: u16,
    pub puerto_grpc: u16,
    pub database_url: String,
    /// PEM de la clave privada RSA (PKCS#8 o PKCS#1). Solo Identidad la tiene.
    pub ruta_clave_privada: String,
    /// `kid` que va en la cabecera del JWT y en el JWKS.
    pub kid: String,
}

impl Config {
    pub fn desde_entorno() -> Result<Self> {
        Ok(Self {
            puerto_http: puerto("HTTP_PORT", 8080)?,
            puerto_grpc: puerto("GRPC_PORT", 50051)?,
            database_url: std::env::var("DATABASE_URL").context("falta DATABASE_URL")?,
            ruta_clave_privada: var_o("JWT_PRIVATE_KEY_PATH", "/keys/jwt.key"),
            kid: var_o("JWT_KID", "identity-1"),
        })
    }

    pub fn direccion_http(&self) -> SocketAddr {
        (Ipv4Addr::UNSPECIFIED, self.puerto_http).into()
    }

    pub fn direccion_grpc(&self) -> SocketAddr {
        (Ipv4Addr::UNSPECIFIED, self.puerto_grpc).into()
    }
}

fn var_o(nombre: &str, por_defecto: &str) -> String {
    std::env::var(nombre).unwrap_or_else(|_| por_defecto.to_owned())
}

fn puerto(nombre: &str, por_defecto: u16) -> Result<u16> {
    match std::env::var(nombre) {
        Ok(v) => v
            .parse()
            .with_context(|| format!("{nombre} no es un puerto válido: {v}")),
        Err(_) => Ok(por_defecto),
    }
}
