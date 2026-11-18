//! Arranque de Identidad: lee la configuración, conecta a PostgreSQL, aplica las
//! migraciones y levanta REST (axum) y gRPC (tonic) en paralelo.
//!
//! `identity healthcheck` hace un GET a `/health` y termina con código 0 o 1. Lo usa
//! el HEALTHCHECK del Dockerfile, porque la imagen final no trae curl.

use std::sync::LazyLock;
use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use identity::aplicacion::Servicio;
use identity::config::Config;
use identity::grpc::{pb, IdentidadGrpc};
use identity::infra::{claves::Claves, contrasenas};

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::args().nth(1).as_deref() == Some("healthcheck") {
        return healthcheck();
    }

    tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .with_current_span(true)
        .with_span_list(false)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn")),
        )
        .init();

    let config = Config::desde_entorno()?;

    let pem = std::fs::read_to_string(&config.ruta_clave_privada).with_context(|| {
        format!(
            "no se pudo leer la clave privada en {}",
            config.ruta_clave_privada
        )
    })?;
    let claves = Claves::desde_pem(&pem, &config.kid)?;
    LazyLock::force(&contrasenas::HASH_FICTICIO);

    let pool = conectar(&config.database_url).await?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("no se pudieron aplicar las migraciones")?;

    let servicio = Servicio::new(pool, claves);

    let reflexion_v1 = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(pb::DESCRIPTOR)
        .build_v1()?;
    let reflexion_v1alpha = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(pb::DESCRIPTOR)
        .build_v1alpha()?;

    let oyente_http = TcpListener::bind(config.direccion_http()).await?;
    tracing::info!(
        http = %config.direccion_http(),
        grpc = %config.direccion_grpc(),
        "Identidad escuchando"
    );

    let http = axum::serve(oyente_http, identity::api::router(servicio.clone()))
        .with_graceful_shutdown(senal_de_apagado());
    let grpc = tonic::transport::Server::builder()
        .add_service(IdentidadGrpc::servidor(servicio))
        .add_service(reflexion_v1)
        .add_service(reflexion_v1alpha)
        .serve_with_shutdown(config.direccion_grpc(), senal_de_apagado());

    tokio::try_join!(async { http.await.context("servidor HTTP") }, async {
        grpc.await.context("servidor gRPC")
    },)?;
    tracing::info!("Identidad detenida");
    Ok(())
}

/// Reintenta mientras PostgreSQL arranca (en Compose también hay `depends_on`).
async fn conectar(url: &str) -> Result<PgPool> {
    let opciones = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(3));
    let mut intento = 1;
    loop {
        match opciones.clone().connect(url).await {
            Ok(pool) => return Ok(pool),
            Err(e) if intento < 10 => {
                tracing::warn!(intento, error = %e, "PostgreSQL no responde; reintento en 2 s");
                tokio::time::sleep(Duration::from_secs(2)).await;
                intento += 1;
            }
            Err(e) => return Err(e).context("no se pudo conectar a PostgreSQL"),
        }
    }
}

/// Ctrl+C o SIGTERM (`docker stop`, Kubernetes): deja de aceptar conexiones y
/// termina las peticiones en curso.
async fn senal_de_apagado() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminar = async {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminar = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminar => {},
    }
}

fn healthcheck() -> Result<()> {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let puerto: u16 = std::env::var("HTTP_PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()?;
    let mut conexion = TcpStream::connect(("127.0.0.1", puerto))?;
    conexion.set_read_timeout(Some(Duration::from_secs(2)))?;
    conexion.write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;
    let mut respuesta = String::new();
    conexion.read_to_string(&mut respuesta)?;
    anyhow::ensure!(
        respuesta.starts_with("HTTP/1.1 200"),
        "GET /health no respondió 200"
    );
    Ok(())
}
