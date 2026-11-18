# syntax=docker/dockerfile:1
#
# Identidad (Rust). El contexto de construcción es la RAÍZ del repositorio, porque
# build.rs compila el contrato compartido contracts/proto/:
#
#   docker build -f services/identidad/Dockerfile -t smartfila/identidad .
#
# Las dos etapas usan Debian 13 (trixie): el binario enlaza contra la glibc de la
# etapa de construcción y la imagen final debe tener la misma o una más nueva.

FROM rust:1.90-slim AS build
WORKDIR /app/services/identidad
COPY contracts/proto /app/contracts/proto
COPY services/identidad/ ./
# Las cachés de BuildKit guardan las dependencias entre construcciones.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/services/identidad/target \
    cargo build --release --locked --bin identidad \
    && cp target/release/identidad /usr/local/bin/identidad

FROM debian:trixie-slim
RUN useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin identidad
COPY --from=build /usr/local/bin/identidad /usr/local/bin/identidad
USER 10001
# 8080: REST + Swagger (/docs) · 50051: gRPC
EXPOSE 8080 50051
HEALTHCHECK --interval=10s --timeout=3s --start-period=15s --retries=3 \
    CMD ["identidad", "healthcheck"]
ENTRYPOINT ["identidad"]
