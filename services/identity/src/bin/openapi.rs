//! Exporta el contrato OpenAPI que genera el código (utoipa) en YAML.
//!
//! El contrato versionado vive en `contracts/openapi/identity.v1.yaml`; la prueba
//! `contrato_openapi` falla si el código y el archivo dejan de coincidir. Para
//! actualizarlo, desde `services/identity/`:
//!
//! ```bash
//! cargo run --bin openapi > ../../contracts/openapi/identity.v1.yaml
//! ```

use utoipa::OpenApi;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("{}", identity::api::ApiDoc::openapi().to_yaml()?);
    Ok(())
}
