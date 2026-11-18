# Queue microservice

ASP.NET Core 10 microservice for turn and queue management.

## Endpoints

- `GET /health`
- `GET /api/turnos`
- `POST /api/turnos`
- `GET /api/turnos/{id}`
- `POST /api/turnos/{id}/admitir`
- `POST /api/turnos/{id}/liberar`

## Run

```bash
dotnet run --project src/Api/Queue.Api.csproj
```
