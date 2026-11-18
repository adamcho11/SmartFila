//! Microservicio de Identidad de SmartFila.
//!
//! Capas, de adentro hacia afuera:
//! - [`dominio`]: reglas de negocio puras, sin E/S (probadas con pruebas unitarias).
//! - [`infra`]: PostgreSQL (sqlx), Argon2id y firma RS256 del JWT.
//! - [`aplicacion`]: casos de uso; une el dominio con la infraestructura.
//! - [`api`]: REST con axum y Swagger (utoipa) en el puerto HTTP.
//! - [`grpc`]: `IdentityService` con tonic en el puerto gRPC.

pub mod api;
pub mod aplicacion;
pub mod config;
pub mod dominio;
pub mod grpc;
pub mod infra;
