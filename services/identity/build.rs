//! Genera el código gRPC desde el contrato compartido en `contracts/proto/`.
//! `protoc` viene empaquetado (protoc-bin-vendored): no hace falta instalarlo.

use std::path::PathBuf;

const RAIZ_PROTO: &str = "../../contracts/proto";
const CONTRATO: &str = "../../contracts/proto/smartfila/identity/v1/identity.proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={CONTRATO}");

    let salida = PathBuf::from(std::env::var("OUT_DIR")?);
    let mut config = tonic_prost_build::Config::new();
    config.protoc_executable(protoc_bin_vendored::protoc_bin_path()?);

    let incluir_google = protoc_bin_vendored::include_path()?;
    tonic_prost_build::configure()
        .build_client(true) // lo usan las pruebas; el gateway genera el suyo en C#
        .file_descriptor_set_path(salida.join("identity_descriptor.bin"))
        .compile_with_config(
            config,
            &[PathBuf::from(CONTRATO)],
            &[PathBuf::from(RAIZ_PROTO), incluir_google],
        )?;
    Ok(())
}
