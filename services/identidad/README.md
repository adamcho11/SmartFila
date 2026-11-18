# Identidad (Rust)

Cuentas, sesión única, roles, JWT RS256 y sanciones de SmartFila.

## Estado

Solo está el **dominio** (`src/dominio/`): las reglas de negocio puras, sin base de datos ni HTTP, con pruebas unitarias.

| Módulo | Regla |
|---|---|
| `ci.rs` | RN-06: C.I. de 5 a 10 dígitos + complemento opcional de 2 caracteres; forma canónica `1234567-1A` |
| `registro.rs` | RN-06 y RN-08: registro público = CIUDADANO; el organizador crea inspectores; el admin crea organizadores |
| `rol.rs` | RN-08 y matriz de permisos: un organizador solo gestiona lo suyo (403) |
| `usuario.rs` | Login sin revelar qué C.I. existen; baja de inspectores |
| `sesion.rs` | RN-07: un login nuevo revoca la sesión anterior; la sesión dura 12 h |
| `sancion.rs` | RN-26: la sanción nace de `sancion.confirmada` y solo el organizador dueño la levanta |
| `token.rs` | ADR-012: claims del JWT, vida de 10 min sin pasar el vencimiento de la sesión |
| `error.rs` | Traducción al catálogo común de errores (400/401/403/404/409) |

## Pruebas

Sin Rust instalado, con Docker (con tu usuario, para que `target/` no quede como root):

```bash
docker run --rm -u "$(id -u):$(id -g)" -e CARGO_HOME=/app/target/.cargo \
  -v "$PWD":/app -w /app rust:1.90-slim cargo test
```

Con Rust instalado: `cargo test`. Antes de un PR: `cargo fmt --check` y `cargo clippy --all-targets -- -D warnings`.

## Siguiente

1. API REST con axum + utoipa (Swagger) según `contracts/openapi/identidad.v1.yaml`.
2. PostgreSQL con sqlx y migraciones (`users`, `credentials`, `organizers`, `user_roles`, `sessions`, `sanctions`, `processed_messages`).
3. Argon2id para contraseñas y firma RS256 con `jsonwebtoken`; `/.well-known/jwks.json`.
4. Consumidor de `sancion.confirmada` y outbox de `sancion.levantada` (lapin).
5. Dockerfile multietapa con usuario no root.
