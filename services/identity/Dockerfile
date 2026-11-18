# syntax=docker/dockerfile:1
#
# Identidad (Rust). El contexto de construcción es la RAÍZ del repositorio, porque
# build.rs compila el contrato compartido contracts/proto/:
#
#   docker build -f services/identity/Dockerfile -t smartfila/identity .
#
# Las dos etapas usan Debian 13 (trixie): el binario enlaza contra la glibc de la
# etapa de construcción y la imagen final debe tener la misma o una más nueva.

FROM rust:1.90-slim AS build
WORKDIR /app/services/identity
COPY contracts/proto /app/contracts/proto
COPY services/identity/ ./
# Las cachés de BuildKit guardan las dependencias entre construcciones.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/services/identity/target \
    cargo build --release --locked --bin identity \
    && cp target/release/identity /usr/local/bin/identity

FROM debian:trixie-slim
RUN useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin identity
COPY --from=build /usr/local/bin/identity /usr/local/bin/identity
USER 10001
# 8080: REST + Swagger (/docs) · 50051: gRPC
EXPOSE 8080 50051
HEALTHCHECK --interval=10s --timeout=3s --start-period=15s --retries=3 \
    CMD ["identity", "healthcheck"]
ENTRYPOINT ["identity"]
