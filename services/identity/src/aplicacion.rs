//! Casos de uso de Identidad. Cada uno toma las decisiones con el dominio y usa la
//! infraestructura para guardar, hashear y firmar. REST y gRPC llaman a estas
//! funciones; ninguno de los dos tiene reglas de negocio propias.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dominio::ci::Ci;
use crate::dominio::registro::{registro_publico, SolicitudCuenta, CONTRASENA_MAX};
use crate::dominio::sesion::Sesion;
use crate::dominio::token::{claims_para, Claims};
use crate::dominio::usuario::{autorizar_inicio_de_sesion, EstadoCuenta, Usuario};
use crate::dominio::ErrorDominio;
use crate::infra::claves::Claves;
use crate::infra::{contrasenas, repositorio};

#[derive(Debug, thiserror::Error)]
pub enum ErrorAplicacion {
    /// Regla de negocio: se responde con el código del catálogo (400/401/403/404/409).
    #[error(transparent)]
    Dominio(#[from] ErrorDominio),

    /// PostgreSQL no responde. REST responde 503 y gRPC `UNAVAILABLE`.
    #[error("base de datos no disponible: {0}")]
    BaseNoDisponible(#[source] sqlx::Error),

    /// Cualquier otra falla. Se registra en el log y el cliente recibe un 500 sin detalles.
    #[error("error interno: {0:#}")]
    Interno(#[from] anyhow::Error),
}

impl From<sqlx::Error> for ErrorAplicacion {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed | sqlx::Error::Io(_) => {
                Self::BaseNoDisponible(e)
            }
            otro => Self::Interno(otro.into()),
        }
    }
}

/// Resultado del login: el JWT firmado, sus claims y la sesión que se abrió.
pub struct TokenEmitido {
    pub token: String,
    pub claims: Claims,
    pub sesion: Sesion,
}

/// Lo que hay detrás de un `sid`. La consultan el extractor de REST y `ValidateSession`.
pub enum EstadoSesion {
    Activa {
        sesion: Sesion,
        usuario: Box<Usuario>,
    },
    /// Otro login la reemplazó (RN-07), se cerró con logout o la cuenta se deshabilitó.
    Revocada,
    /// Pasaron las 12 h de vida de la sesión.
    Vencida,
    NoExiste,
}

#[derive(Clone)]
pub struct Servicio {
    pool: PgPool,
    claves: Arc<Claves>,
}

impl Servicio {
    pub fn new(pool: PgPool, claves: Claves) -> Self {
        Self {
            pool,
            claves: Arc::new(claves),
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn claves(&self) -> &Claves {
        &self.claves
    }

    /// `POST /users` (RN-06, RN-08): el registro público siempre crea un ciudadano.
    pub async fn registrar_ciudadano(
        &self,
        solicitud: SolicitudCuenta,
    ) -> Result<Usuario, ErrorAplicacion> {
        let cuenta = registro_publico(solicitud)?;
        let contrasena = cuenta.contrasena;
        let hash = en_segundo_plano(move || contrasenas::hashear(&contrasena))
            .await?
            .map_err(|e| anyhow::anyhow!("argon2: {e}"))?;

        let usuario = Usuario {
            id: Uuid::new_v4(),
            ci: cuenta.ci,
            nombres: cuenta.nombres,
            apellidos: cuenta.apellidos,
            email: cuenta.email,
            celular: cuenta.celular,
            rol: cuenta.rol,
            estado: EstadoCuenta::Activa,
        };
        repositorio::insertar_cuenta(&self.pool, &usuario, &hash).await?;
        Ok(usuario)
    }

    /// `POST /sessions` (RN-07): abre una sesión nueva, revoca la anterior y emite el JWT.
    pub async fn iniciar_sesion(
        &self,
        ci: &str,
        complemento: Option<&str>,
        contrasena: String,
    ) -> Result<TokenEmitido, ErrorAplicacion> {
        let ci = Ci::nuevo(ci, complemento).map_err(ErrorDominio::Validacion)?;
        // Ninguna contraseña válida es más larga; así nadie manda megabytes a Argon2.
        if contrasena.chars().count() > CONTRASENA_MAX {
            return Err(ErrorDominio::CredencialesInvalidas.into());
        }

        // Si el C.I. no existe se verifica contra un hash ficticio: el login tarda lo
        // mismo y no revela qué C.I. están registrados.
        let (usuario, hash) = match repositorio::buscar_por_ci(&self.pool, &ci).await? {
            Some((usuario, hash)) => (Some(usuario), hash),
            None => (None, contrasenas::HASH_FICTICIO.clone()),
        };
        let correcta = en_segundo_plano(move || contrasenas::verificar(&contrasena, &hash)).await?;
        let usuario = autorizar_inicio_de_sesion(usuario.as_ref(), correcta)?.clone();

        let ahora = Utc::now();
        let sesion =
            repositorio::abrir_sesion(&self.pool, usuario.id, Uuid::new_v4(), ahora).await?;
        self.emitir_token(&usuario, sesion, ahora)
    }

    pub async fn estado_sesion(&self, sid: Uuid) -> Result<EstadoSesion, ErrorAplicacion> {
        let Some((sesion, usuario)) = repositorio::sesion_con_usuario(&self.pool, sid).await?
        else {
            return Ok(EstadoSesion::NoExiste);
        };
        Ok(
            if sesion.revocada_en.is_some() || usuario.estado != EstadoCuenta::Activa {
                EstadoSesion::Revocada
            } else if !sesion.activa(Utc::now()) {
                EstadoSesion::Vencida
            } else {
                EstadoSesion::Activa {
                    sesion,
                    usuario: Box::new(usuario),
                }
            },
        )
    }

    /// `DELETE /sessions/current` (logout).
    pub async fn cerrar_sesion(&self, sid: Uuid) -> Result<(), ErrorAplicacion> {
        repositorio::revocar_sesion(&self.pool, sid, Utc::now()).await
    }

    fn emitir_token(
        &self,
        usuario: &Usuario,
        sesion: Sesion,
        ahora: DateTime<Utc>,
    ) -> Result<TokenEmitido, ErrorAplicacion> {
        let claims = claims_para(usuario, &sesion, ahora)?;
        let token = self
            .claves
            .firmar(&claims)
            .map_err(|e| ErrorAplicacion::Interno(e.into()))?;
        Ok(TokenEmitido {
            token,
            claims,
            sesion,
        })
    }
}

/// Argon2 ocupa la CPU: se ejecuta en el pool de hilos bloqueantes de tokio.
async fn en_segundo_plano<T: Send + 'static>(
    trabajo: impl FnOnce() -> T + Send + 'static,
) -> Result<T, ErrorAplicacion> {
    tokio::task::spawn_blocking(trabajo)
        .await
        .map_err(|e| ErrorAplicacion::Interno(e.into()))
}
