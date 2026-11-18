# Identidad (Rust)

Cuentas, sesión única, roles, JWT RS256 y sanciones de SmartFila.

**Stack:** Rust · axum (REST) · tonic (gRPC) · sqlx + PostgreSQL 16 · utoipa (Swagger) · Argon2id · jsonwebtoken (RS256).

| Puerto | Qué expone |
|---|---|
| 8080 | REST `/api/v1/...`, Swagger UI en `/docs`, contrato en `/docs/openapi.json`, `/.well-known/jwks.json`, `/health` |
| 50051 | gRPC `smartfila.identity.v1.IdentityService` (con reflexión) |

## Estado

| Hecho | Pendiente |
|---|---|
| Dominio con las reglas RN-06, RN-07, RN-08, RN-26 y ADR-012 (`src/dominio/`) | Organizadores e inspectores (`/organizers`, `/inspectors`) y siembra del admin |
| Registro de ciudadano, login, sesión actual, logout y perfil | Listado de usuarios y lote para DataLoader (`GET /users?ids=`) |
| JWT RS256 con `kid` y JWKS | Renovación del token sin volver a escribir la contraseña |
| gRPC `ValidateSession` para el gateway | Consumidor de `sancion.confirmada` y outbox de `sancion.levantada` (RabbitMQ) |
| PostgreSQL con migraciones (esquema completo de la sección 7.1) | Endpoints de sanciones (`/sanctions`) |
| Dockerfile multietapa no root, healthcheck, compose | Métricas y trazas (OpenTelemetry) |
| Logs JSON con `correlation_id` | |

## Estructura

```
src/
  dominio/       reglas de negocio puras, sin E/S (pruebas unitarias)
  infra/         claves.rs (RS256 + JWKS) · contrasenas.rs (Argon2id) · repositorio.rs (sqlx)
  aplicacion.rs  casos de uso: unen dominio e infraestructura
  api/           REST (axum) + Swagger (utoipa) + cuerpo de error común
  grpc.rs        IdentityService (tonic)
  main.rs        arranque: config, migraciones, REST y gRPC en paralelo
migrations/      SQL de sqlx (se aplican solas al arrancar)
tests/           integración con PostgreSQL y verificación del contrato OpenAPI
```

Contratos compartidos en la raíz del repo:

- gRPC: `contracts/proto/smartfila/identity/v1/identity.proto`
- REST: `contracts/openapi/identidad.v1.yaml` (se genera desde el código)

## Levantar con Docker Compose

Desde la raíz del repositorio:

```bash
cp .env.example .env          # y cambiar la clave
docker compose up -d --build
```

Compose genera las claves RS256 una sola vez (servicio `keygen`, volumen `jwt-keys`). Mientras NGINX no esté en el compose, Identidad publica sus puertos solo en localhost:

- REST y Swagger: http://localhost:8081/docs
- gRPC: `localhost:50052`

## Endpoints REST

| Método y ruta | Auth | Respuesta |
|---|---|---|
| `POST /api/v1/users` | — | 201 ciudadano · 400 `VALIDATION_ERROR` · 409 `CI_ALREADY_REGISTERED` / `EMAIL_ALREADY_REGISTERED` |
| `POST /api/v1/sessions` | — | 201 `{ accessToken, expiresIn, sessionId, sessionExpiresAt }` · 401 `INVALID_CREDENTIALS` · 403 `ACCOUNT_DISABLED` |
| `GET /api/v1/sessions/current` | Bearer | 200 · 401 `SESSION_REVOKED` |
| `DELETE /api/v1/sessions/current` | Bearer | 204 |
| `GET /api/v1/users/me` | Bearer | 200 perfil |
| `GET /.well-known/jwks.json` | — | clave pública para validar el JWT |
| `GET /health` | — | 200 `UP` · 503 si la base no responde |

Los errores usan el cuerpo común `{ code, message, details, traceId }`. `traceId` es igual a la cabecera `X-Correlation-Id`.

Ejemplo:

```bash
curl -X POST localhost:8081/api/v1/users -H 'content-type: application/json' -d '{
  "ci":"7654321","firstNames":"Juan","lastNames":"Quispe Mamani",
  "email":"juan@example.com","phone":"67654321","password":"clave-segura-1"}'

curl -X POST localhost:8081/api/v1/sessions -H 'content-type: application/json' \
  -d '{"ci":"7654321","password":"clave-segura-1"}'
```

## gRPC: `ValidateSession`

El gateway lo llama antes de cada mutación con el `sid` del JWT y un **deadline de 200 ms**.

| Código | Cuándo |
|---|---|
| `OK` | Sesión activa: devuelve `user_id`, `roles`, `organizer_id` y `expires_at` |
| `INVALID_ARGUMENT` | `session_id` vacío o que no es UUID |
| `NOT_FOUND` | No existe la sesión |
| `UNAUTHENTICATED` | `SESSION_REVOKED` (otro login, logout o cuenta deshabilitada) o `SESSION_EXPIRED` |
| `UNAVAILABLE` | Identidad no llega a su base |

Probarlo con grpcurl (la reflexión lista los métodos sin el `.proto`):

```bash
# desde el host
grpcurl -plaintext -max-time 0.2 localhost:50052 list
grpcurl -plaintext -max-time 0.2 -d '{"session_id":"<sessionId del login>"}' \
  localhost:50052 smartfila.identity.v1.IdentityService/ValidateSession

# desde la red de Docker, como lo hace el gateway
docker run --rm --network smartfila_backend fullstorydev/grpcurl:v1.9.1 \
  -plaintext -max-time 0.2 identidad:50051 list
```

En C#, el gateway genera el cliente desde el mismo `.proto` (`<Protobuf Include="../../contracts/proto/smartfila/identity/v1/identity.proto" GrpcServices="Client" />`).

## Pruebas

Sin Rust instalado, con Docker y **desde la raíz del repositorio** (el build necesita `contracts/proto`):

```bash
alias cargo-docker='docker run --rm -u "$(id -u):$(id -g)" \
  -e CARGO_HOME=/repo/services/identidad/target/.cargo \
  -v "$PWD":/repo -w /repo/services/identidad rust:1.90-slim cargo'

cargo-docker test        # unitarias + contrato OpenAPI
```

La prueba de integración (`tests/flujo_sesion.rs`) usa un PostgreSQL desechable:

```bash
docker network create identidad-test
docker run -d --rm --name identidad-test-db --network identidad-test \
  -e POSTGRES_PASSWORD=test postgres:16-alpine

docker run --rm -u "$(id -u):$(id -g)" --network identidad-test \
  -e CARGO_HOME=/repo/services/identidad/target/.cargo \
  -e IDENTIDAD_TEST_DATABASE_URL=postgres://postgres:test@identidad-test-db/postgres \
  -v "$PWD":/repo -w /repo/services/identidad rust:1.90-slim \
  cargo test -- --include-ignored

docker rm -f identidad-test-db && docker network rm identidad-test
```

Con Rust instalado: `cargo test`. Antes de un PR: `cargo fmt --check` y `cargo clippy --all-targets -- -D warnings`.

Si cambias la API REST, regenera el contrato (la prueba `contrato_openapi` falla si queda desactualizado):

```bash
cargo run --bin openapi > ../../contracts/openapi/identidad.v1.yaml
```

## Variables de entorno

| Variable | Por defecto | Uso |
|---|---|---|
| `DATABASE_URL` | (obligatoria) | `postgres://identidad:<clave>@identidad-db:5432/identidad` |
| `JWT_PRIVATE_KEY_PATH` | `/keys/jwt.key` | Clave privada RSA en PEM (PKCS#8 o PKCS#1) |
| `JWT_KID` | `identidad-1` | `kid` del JWT y del JWKS |
| `HTTP_PORT` / `GRPC_PORT` | `8080` / `50051` | Puertos |
| `RUST_LOG` | `info,sqlx=warn` | Nivel de logs |
