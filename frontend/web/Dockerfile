# SmartFila · frontend Angular
# Contexto de construcción: raíz del repo (docker compose build frontend).

# 1) Compilación
FROM node:24-alpine AS build
WORKDIR /app
COPY frontend/web/package.json frontend/web/package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY frontend/web/ ./
RUN npx ng build --configuration production

# 2) Archivos estáticos con NGINX sin root (uid 101), puerto 8080
FROM nginxinc/nginx-unprivileged:1.28-alpine
COPY frontend/web/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/dist/web/browser /usr/share/nginx/html
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -q --spider http://127.0.0.1:8080/ || exit 1
