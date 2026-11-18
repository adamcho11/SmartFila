# Contexto de construcción: raíz del repositorio.
FROM eclipse-temurin:25-jdk AS build
WORKDIR /workspace

COPY services/sales/.mvn/ services/sales/.mvn/
COPY services/sales/mvnw services/sales/pom.xml services/sales/
RUN chmod +x services/sales/mvnw
RUN cd services/sales && ./mvnw -B -DskipTests dependency:go-offline

COPY services/sales/src/ services/sales/src/
RUN cd services/sales && ./mvnw -B -DskipTests package \
    && cp target/ventas-service-0.0.1-SNAPSHOT.jar /workspace/app.jar

FROM eclipse-temurin:25-jre
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 smartfila \
    && useradd --system --uid 10001 --gid smartfila --no-create-home smartfila
WORKDIR /app
COPY --from=build --chown=10001:10001 /workspace/app.jar /app/app.jar
USER 10001:10001
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=30s --retries=5 \
    CMD ["curl", "-fsS", "http://localhost:8080/actuator/health"]
ENTRYPOINT ["java", "-jar", "/app/app.jar"]
