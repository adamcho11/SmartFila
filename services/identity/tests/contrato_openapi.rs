//! El contrato versionado en `contracts/openapi/identity.v1.yaml` debe coincidir con
//! lo que genera el código. Si esta prueba falla, se cambió la API sin actualizar el
//! contrato (o al revés). Para regenerarlo, desde `services/identity/`:
//!
//! ```bash
//! cargo run --bin openapi > ../../contracts/openapi/identity.v1.yaml
//! ```

use utoipa::OpenApi;

#[test]
fn el_contrato_openapi_esta_al_dia() {
    let ruta = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../contracts/openapi/identity.v1.yaml"
    );
    let versionado =
        std::fs::read_to_string(ruta).expect("falta contracts/openapi/identity.v1.yaml");
    let generado = identity::api::ApiDoc::openapi().to_yaml().unwrap();
    assert!(
        versionado == generado,
        "contracts/openapi/identity.v1.yaml está desactualizado; regenéralo con `cargo run --bin openapi`"
    );
}
