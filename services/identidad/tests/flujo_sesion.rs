//! Prueba de integración con PostgreSQL real: registro → login → ValidateSession
//! (gRPC) → segundo login revoca el primero (RN-07) → logout.
//!
//! Necesita una base vacía o desechable; borra todos los datos de Identidad:
//!
//! ```bash
//! IDENTIDAD_TEST_DATABASE_URL=postgres://... cargo test -- --include-ignored
//! ```
//!
//! El README del servicio tiene los comandos para levantar la base con Docker.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use serde_json::{json, Value};
use sqlx::PgPool;
use tonic::Code;
use tower::ServiceExt;

use identidad::aplicacion::Servicio;
use identidad::dominio::token::Claims;
use identidad::grpc::pb::identity_service_server::IdentityService;
use identidad::grpc::pb::ValidateSessionRequest;
use identidad::grpc::IdentidadGrpc;
use identidad::infra::claves::Claves;

async fn preparar() -> (Router, IdentidadGrpc) {
    let url = std::env::var("IDENTIDAD_TEST_DATABASE_URL")
        .expect("define IDENTIDAD_TEST_DATABASE_URL con una base desechable");
    let pool = PgPool::connect(&url).await.expect("conectar a PostgreSQL");
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    sqlx::query("TRUNCATE users, organizers, processed_messages CASCADE")
        .execute(&pool)
        .await
        .unwrap();

    let clave = rsa::RsaPrivateKey::new(&mut rand::rngs::OsRng, 2048).unwrap();
    let pem = clave.to_pkcs8_pem(LineEnding::LF).unwrap();
    let servicio = Servicio::new(pool, Claves::desde_pem(&pem, "prueba").unwrap());
    (
        identidad::api::router(servicio.clone()),
        IdentidadGrpc::new(servicio),
    )
}

async fn llamar(
    app: &Router,
    metodo: &str,
    ruta: &str,
    token: Option<&str>,
    cuerpo: Option<Value>,
) -> (StatusCode, Value) {
    let mut peticion = Request::builder()
        .method(metodo)
        .uri(ruta)
        .header("x-correlation-id", "prueba-integracion");
    if let Some(t) = token {
        peticion = peticion.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    let peticion = match cuerpo {
        Some(c) => peticion
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(c.to_string())),
        None => peticion.body(Body::empty()),
    }
    .unwrap();

    let respuesta = app.clone().oneshot(peticion).await.unwrap();
    let estado = respuesta.status();
    let bytes = respuesta.into_body().collect().await.unwrap().to_bytes();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (estado, json)
}

async fn validar(grpc: &IdentidadGrpc, sid: &str) -> Result<Vec<String>, (Code, String)> {
    grpc.validate_session(tonic::Request::new(ValidateSessionRequest {
        session_id: sid.to_owned(),
    }))
    .await
    .map(|r| r.into_inner().roles)
    .map_err(|s| (s.code(), s.message().to_owned()))
}

fn ana() -> Value {
    json!({
        "ci": "1234567", "ciComplement": "1a",
        "firstNames": "Ana María", "lastNames": "Pérez Cruz",
        "email": "Ana@Example.com", "phone": "71234567",
        "password": "una-clave-segura"
    })
}

fn login_ana(password: &str) -> Value {
    json!({ "ci": "1234567", "ciComplement": "1A", "password": password })
}

#[tokio::test]
#[ignore = "necesita PostgreSQL: IDENTIDAD_TEST_DATABASE_URL"]
async fn flujo_completo_de_sesion_unica() {
    let (app, grpc) = preparar().await;

    // Estado del servicio y contrato publicado.
    let (estado, salud) = llamar(&app, "GET", "/health", None, None).await;
    assert_eq!(
        (estado, salud["status"].as_str()),
        (StatusCode::OK, Some("UP"))
    );
    let (estado, _) = llamar(&app, "GET", "/docs/openapi.json", None, None).await;
    assert_eq!(estado, StatusCode::OK);

    // Registro: datos inválidos devuelven todos los errores juntos.
    let (estado, error) = llamar(
        &app,
        "POST",
        "/api/v1/users",
        None,
        Some(json!({ "ci": "12a" })),
    )
    .await;
    assert_eq!(estado, StatusCode::BAD_REQUEST);
    assert_eq!(error["code"], "VALIDATION_ERROR");
    assert_eq!(error["traceId"], "prueba-integracion");
    let campos: Vec<&str> = error["details"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["field"].as_str().unwrap())
        .collect();
    for campo in [
        "ci",
        "firstNames",
        "lastNames",
        "email",
        "phone",
        "password",
    ] {
        assert!(
            campos.contains(&campo),
            "falta el detalle de {campo}: {campos:?}"
        );
    }

    // Registro correcto: complemento en mayúsculas, correo en minúsculas, rol CIUDADANO.
    let (estado, usuario) = llamar(&app, "POST", "/api/v1/users", None, Some(ana())).await;
    assert_eq!(estado, StatusCode::CREATED, "{usuario}");
    assert_eq!(usuario["ci"], "1234567-1A");
    assert_eq!(usuario["email"], "ana@example.com");
    assert_eq!(usuario["role"], "CIUDADANO");
    assert!(usuario["organizerId"].is_null());
    let user_id = usuario["id"].as_str().unwrap().to_owned();

    // Duplicados: primero choca el C.I., después el correo.
    let (estado, error) = llamar(&app, "POST", "/api/v1/users", None, Some(ana())).await;
    assert_eq!(
        (estado, error["code"].as_str()),
        (StatusCode::CONFLICT, Some("CI_ALREADY_REGISTERED"))
    );
    let mut otro_ci = ana();
    otro_ci["ci"] = json!("7654321");
    let (estado, error) = llamar(&app, "POST", "/api/v1/users", None, Some(otro_ci)).await;
    assert_eq!(
        (estado, error["code"].as_str()),
        (StatusCode::CONFLICT, Some("EMAIL_ALREADY_REGISTERED"))
    );

    // Login: contraseña mala y C.I. inexistente dan el mismo error.
    let (estado, error) = llamar(
        &app,
        "POST",
        "/api/v1/sessions",
        None,
        Some(login_ana("otra-clave")),
    )
    .await;
    assert_eq!(
        (estado, error["code"].as_str()),
        (StatusCode::UNAUTHORIZED, Some("INVALID_CREDENTIALS"))
    );
    let (_, error2) = llamar(
        &app,
        "POST",
        "/api/v1/sessions",
        None,
        Some(json!({ "ci": "9999999", "password": "una-clave-segura" })),
    )
    .await;
    assert_eq!(error2["code"], error["code"]);

    // Primer login.
    let (estado, sesion1) = llamar(
        &app,
        "POST",
        "/api/v1/sessions",
        None,
        Some(login_ana("una-clave-segura")),
    )
    .await;
    assert_eq!(estado, StatusCode::CREATED, "{sesion1}");
    assert_eq!(sesion1["tokenType"], "Bearer");
    assert_eq!(sesion1["expiresIn"], 600);
    let token1 = sesion1["accessToken"].as_str().unwrap().to_owned();
    let sid1 = sesion1["sessionId"].as_str().unwrap().to_owned();

    // El JWT se valida con la clave publicada en el JWKS y trae los claims del v2.
    let (_, jwks) = llamar(&app, "GET", "/.well-known/jwks.json", None, None).await;
    let jwk = &jwks["keys"][0];
    assert_eq!(jwk["kid"], "prueba");
    let publica =
        DecodingKey::from_rsa_components(jwk["n"].as_str().unwrap(), jwk["e"].as_str().unwrap())
            .unwrap();
    let mut validacion = Validation::new(Algorithm::RS256);
    validacion.set_audience(&["smartfila"]);
    let claims = jsonwebtoken::decode::<Claims>(&token1, &publica, &validacion)
        .unwrap()
        .claims;
    assert_eq!(claims.sub.to_string(), user_id);
    assert_eq!(claims.sid.to_string(), sid1);
    assert_eq!(claims.ci, "1234567-1A");
    assert_eq!(claims.name, "Ana María Pérez Cruz");
    assert_eq!(claims.roles, ["CIUDADANO"]);

    // Con el token se lee el perfil, y gRPC confirma la sesión.
    let (estado, perfil) = llamar(&app, "GET", "/api/v1/users/me", Some(&token1), None).await;
    assert_eq!(
        (estado, perfil["id"].as_str()),
        (StatusCode::OK, Some(user_id.as_str()))
    );
    assert_eq!(
        validar(&grpc, &sid1).await,
        Ok(vec!["CIUDADANO".to_owned()])
    );

    // RN-07: un segundo login revoca el primero.
    let (_, sesion2) = llamar(
        &app,
        "POST",
        "/api/v1/sessions",
        None,
        Some(login_ana("una-clave-segura")),
    )
    .await;
    let token2 = sesion2["accessToken"].as_str().unwrap().to_owned();
    let sid2 = sesion2["sessionId"].as_str().unwrap().to_owned();
    assert_ne!(sid1, sid2);

    assert_eq!(
        validar(&grpc, &sid1).await,
        Err((Code::Unauthenticated, "SESSION_REVOKED".into()))
    );
    let (estado, error) =
        llamar(&app, "GET", "/api/v1/sessions/current", Some(&token1), None).await;
    assert_eq!(
        (estado, error["code"].as_str()),
        (StatusCode::UNAUTHORIZED, Some("SESSION_REVOKED"))
    );
    let (estado, actual) =
        llamar(&app, "GET", "/api/v1/sessions/current", Some(&token2), None).await;
    assert_eq!(
        (estado, actual["sessionId"].as_str()),
        (StatusCode::OK, Some(sid2.as_str()))
    );

    // Logout: la sesión 2 queda revocada para REST y para gRPC.
    let (estado, _) = llamar(
        &app,
        "DELETE",
        "/api/v1/sessions/current",
        Some(&token2),
        None,
    )
    .await;
    assert_eq!(estado, StatusCode::NO_CONTENT);
    assert_eq!(
        validar(&grpc, &sid2).await,
        Err((Code::Unauthenticated, "SESSION_REVOKED".into()))
    );
    let (estado, _) = llamar(&app, "GET", "/api/v1/users/me", Some(&token2), None).await;
    assert_eq!(estado, StatusCode::UNAUTHORIZED);

    // Códigos gRPC de las peticiones mal formadas o desconocidas.
    assert_eq!(
        validar(&grpc, "no-es-un-uuid").await.unwrap_err().0,
        Code::InvalidArgument
    );
    assert_eq!(
        validar(&grpc, &uuid::Uuid::new_v4().to_string())
            .await
            .unwrap_err()
            .0,
        Code::NotFound
    );

    // Sin token, token basura y ruta inexistente: siempre el cuerpo de error común.
    let (estado, error) = llamar(&app, "GET", "/api/v1/users/me", None, None).await;
    assert_eq!(
        (estado, error["code"].as_str()),
        (StatusCode::UNAUTHORIZED, Some("UNAUTHENTICATED"))
    );
    let (estado, _) = llamar(&app, "GET", "/api/v1/users/me", Some("basura"), None).await;
    assert_eq!(estado, StatusCode::UNAUTHORIZED);
    let (estado, error) = llamar(&app, "GET", "/api/v1/no-existe", None, None).await;
    assert_eq!(
        (estado, error["code"].as_str()),
        (StatusCode::NOT_FOUND, Some("NOT_FOUND"))
    );
}
