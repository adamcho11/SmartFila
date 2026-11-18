# SmartFila · frontend web

Angular 22 (componentes standalone, signals, sin zone.js) con Apollo Angular para GraphQL.
El navegador habla solo con NGINX; no hay CORS ni URLs de servicios en el código.

## Pantallas (P1)

| Ruta | Qué hace | Consume |
|------|----------|---------|
| `/registro` | Alta de ciudadano y login automático | Identity REST `POST /api/v1/users` |
| `/login` | Login con C.I. y contraseña (JWT RS256) | Identity REST `POST /api/v1/sessions`, `GET /api/v1/users/me` |
| `/panel` | Vista inicial con datos reales: estado de Identity, puestos de Inventario y turnos de la Cola | Gateway GraphQL `query Overview` (una consulta, tres servicios) |
| `/turno` | Pedir turno, ver la posición cada 5 s y salir de la fila | Gateway (`Puestos`) + Cola REST `/api/v1/turnos` (ADR-003: el polling no pasa por el gateway) |

El JWT se guarda en `sessionStorage` y el interceptor lo envía como `Authorization: Bearer`
a `/api/...` y `/graphql`. Un 401 o el vencimiento del token (10 min) devuelven al login.

## Levantar

Con todo el sistema (recomendado):

```bash
cp .env.example .env
docker compose up -d --build
# http://localhost:8080
```

En desarrollo, con recarga en caliente, contra los puertos que publica compose:

```bash
cd frontend/web
npm ci
npm start            # http://127.0.0.1:4200, proxy en proxy.conf.json
npm test             # Vitest
```

## Estructura

```
src/app/
  core/       sesión (AuthService, interceptor, guardias), GatewayService (Apollo), ColaService (REST)
  paginas/    login, registro, panel, turno
```
