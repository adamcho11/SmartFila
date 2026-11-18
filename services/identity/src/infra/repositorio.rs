//! Consultas a PostgreSQL con sqlx. Las consultas se arman en tiempo de ejecución
//! (`sqlx::query`, no `query!`), así compilar no exige una base levantada.
//!
//! Aquí no se decide nada: las reglas viven en `dominio`. Este módulo solo traduce
//! filas ↔ tipos del dominio y las violaciones de UNIQUE ↔ errores del catálogo.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::aplicacion::ErrorAplicacion;
use crate::dominio::ci::Ci;
use crate::dominio::rol::Rol;
use crate::dominio::sesion::{iniciar_sesion, Sesion};
use crate::dominio::usuario::{EstadoCuenta, Usuario};
use crate::dominio::ErrorDominio;

#[derive(FromRow)]
struct FilaUsuario {
    id: Uuid,
    ci: String,
    ci_complement: String,
    first_names: String,
    last_names: String,
    email: String,
    phone: String,
    status: String,
    role: String,
    organizer_id: Option<Uuid>,
}

#[derive(FromRow)]
struct FilaConHash {
    #[sqlx(flatten)]
    usuario: FilaUsuario,
    password_hash: String,
}

#[derive(FromRow)]
struct FilaSesion {
    id: Uuid,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

#[derive(FromRow)]
struct FilaSesionConUsuario {
    session_id: Uuid,
    session_created_at: DateTime<Utc>,
    session_expires_at: DateTime<Utc>,
    session_revoked_at: Option<DateTime<Utc>>,
    #[sqlx(flatten)]
    usuario: FilaUsuario,
}

impl From<FilaSesion> for Sesion {
    fn from(f: FilaSesion) -> Self {
        Sesion {
            id: f.id,
            usuario_id: f.user_id,
            creada_en: f.created_at,
            expira_en: f.expires_at,
            revocada_en: f.revoked_at,
        }
    }
}

/// Reconstruye el usuario. Una fila que el dominio no acepta (un INSPECTOR sin
/// organizador, un estado desconocido) es un error interno, no del cliente.
fn a_usuario(f: FilaUsuario) -> Result<Usuario, ErrorAplicacion> {
    let id = f.id;
    let incoherente = |campo: &str| {
        ErrorAplicacion::Interno(anyhow::anyhow!(
            "fila incoherente para el usuario {id}: {campo}"
        ))
    };
    let complemento = Some(f.ci_complement.as_str()).filter(|c| !c.is_empty());
    let ci = Ci::nuevo(&f.ci, complemento).map_err(|_| incoherente("ci"))?;
    let rol = Rol::desde_bd(&f.role, f.organizer_id).ok_or_else(|| incoherente("role"))?;
    let estado = match f.status.as_str() {
        "ACTIVE" => EstadoCuenta::Activa,
        "DISABLED" => EstadoCuenta::Deshabilitada,
        _ => return Err(incoherente("status")),
    };
    Ok(Usuario {
        id,
        ci,
        nombres: f.first_names,
        apellidos: f.last_names,
        email: f.email,
        celular: f.phone,
        rol,
        estado,
    })
}

/// Guarda usuario, credencial y rol en una sola transacción.
pub async fn insertar_cuenta(
    pool: &PgPool,
    u: &Usuario,
    hash: &str,
) -> Result<(), ErrorAplicacion> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO users (id, ci, ci_complement, first_names, last_names, email, phone, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(u.id)
    .bind(u.ci.numero())
    .bind(u.ci.complemento_bd())
    .bind(&u.nombres)
    .bind(&u.apellidos)
    .bind(&u.email)
    .bind(&u.celular)
    .bind(u.estado.nombre())
    .execute(&mut *tx)
    .await
    .map_err(traducir_unicidad)?;

    sqlx::query("INSERT INTO credentials (user_id, password_hash) VALUES ($1, $2)")
        .bind(u.id)
        .bind(hash)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO user_roles (user_id, role, organizer_id) VALUES ($1, $2, $3)")
        .bind(u.id)
        .bind(u.rol.nombre())
        .bind(u.rol.organizador_id())
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// El C.I. y el correo son únicos en la base; la violación llega con el nombre de la
/// restricción (ver la migración) y se traduce al error del catálogo.
fn traducir_unicidad(e: sqlx::Error) -> ErrorAplicacion {
    let restriccion = e
        .as_database_error()
        .and_then(|d| d.constraint())
        .map(str::to_owned);
    match restriccion.as_deref() {
        Some("users_ci_unico") => ErrorDominio::CiYaRegistrado.into(),
        Some("users_email_unico") => ErrorDominio::EmailYaRegistrado.into(),
        _ => e.into(),
    }
}

/// Usuario y hash de su contraseña, para el login.
pub async fn buscar_por_ci(
    pool: &PgPool,
    ci: &Ci,
) -> Result<Option<(Usuario, String)>, ErrorAplicacion> {
    let fila = sqlx::query_as::<_, FilaConHash>(
        "SELECT u.id, u.ci, u.ci_complement, u.first_names, u.last_names, u.email, u.phone,
                u.status, r.role, r.organizer_id, c.password_hash
         FROM users u
         JOIN user_roles r ON r.user_id = u.id
         JOIN credentials c ON c.user_id = u.id
         WHERE u.ci = $1 AND u.ci_complement = $2
         ORDER BY r.role
         LIMIT 1",
    )
    .bind(ci.numero())
    .bind(ci.complemento_bd())
    .fetch_optional(pool)
    .await?;

    fila.map(|f| Ok((a_usuario(f.usuario)?, f.password_hash)))
        .transpose()
}

/// RN-07 en una transacción: bloquea al usuario (dos logins simultáneos se atienden
/// uno tras otro), revoca sus sesiones sin revocar e inserta la nueva. El índice
/// único parcial `sessions_una_activa_por_usuario` respalda la regla en la base.
pub async fn abrir_sesion(
    pool: &PgPool,
    usuario_id: Uuid,
    nueva_id: Uuid,
    ahora: DateTime<Utc>,
) -> Result<Sesion, ErrorAplicacion> {
    let mut tx = pool.begin().await?;

    sqlx::query("SELECT 1 FROM users WHERE id = $1 FOR UPDATE")
        .bind(usuario_id)
        .execute(&mut *tx)
        .await?;

    let sin_revocar: Vec<Sesion> = sqlx::query_as::<_, FilaSesion>(
        "SELECT id, user_id, created_at, expires_at, revoked_at
         FROM sessions WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(usuario_id)
    .fetch_all(&mut *tx)
    .await?
    .into_iter()
    .map(Sesion::from)
    .collect();

    let inicio = iniciar_sesion(usuario_id, nueva_id, ahora, &sin_revocar);
    if !inicio.revocar.is_empty() {
        sqlx::query("UPDATE sessions SET revoked_at = $2 WHERE id = ANY($1)")
            .bind(&inicio.revocar)
            .bind(ahora)
            .execute(&mut *tx)
            .await?;
    }

    let s = &inicio.nueva;
    sqlx::query(
        "INSERT INTO sessions (id, user_id, created_at, expires_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(s.id)
    .bind(s.usuario_id)
    .bind(s.creada_en)
    .bind(s.expira_en)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(inicio.nueva)
}

/// La sesión y su dueño en una sola consulta (la usan REST y `ValidateSession`).
pub async fn sesion_con_usuario(
    pool: &PgPool,
    sid: Uuid,
) -> Result<Option<(Sesion, Usuario)>, ErrorAplicacion> {
    let fila = sqlx::query_as::<_, FilaSesionConUsuario>(
        "SELECT s.id AS session_id, s.created_at AS session_created_at,
                s.expires_at AS session_expires_at, s.revoked_at AS session_revoked_at,
                u.id, u.ci, u.ci_complement, u.first_names, u.last_names, u.email, u.phone,
                u.status, r.role, r.organizer_id
         FROM sessions s
         JOIN users u ON u.id = s.user_id
         JOIN user_roles r ON r.user_id = u.id
         WHERE s.id = $1
         ORDER BY r.role
         LIMIT 1",
    )
    .bind(sid)
    .fetch_optional(pool)
    .await?;

    fila.map(|f| {
        let usuario = a_usuario(f.usuario)?;
        let sesion = Sesion {
            id: f.session_id,
            usuario_id: usuario.id,
            creada_en: f.session_created_at,
            expira_en: f.session_expires_at,
            revocada_en: f.session_revoked_at,
        };
        Ok((sesion, usuario))
    })
    .transpose()
}

/// Logout. Repetirlo no cambia la hora de revocación.
pub async fn revocar_sesion(
    pool: &PgPool,
    sid: Uuid,
    ahora: DateTime<Utc>,
) -> Result<(), ErrorAplicacion> {
    sqlx::query("UPDATE sessions SET revoked_at = $2 WHERE id = $1 AND revoked_at IS NULL")
        .bind(sid)
        .bind(ahora)
        .execute(pool)
        .await?;
    Ok(())
}
