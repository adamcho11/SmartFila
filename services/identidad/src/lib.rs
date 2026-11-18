//! Microservicio de Identidad de SmartFila.
//!
//! Por ahora solo contiene el dominio: las reglas de negocio puras, sin base de
//! datos ni HTTP. Las capas de API (axum), persistencia (PostgreSQL) y mensajería
//! (RabbitMQ) se agregan encima y llaman a estas funciones.

pub mod dominio;
