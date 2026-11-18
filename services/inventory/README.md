# Inventory microservice

ASP.NET Core 10 microservice organized with Clean Architecture.

## Projects

- `src/Domain`: entities and domain rules.
- `src/Application`: use cases and repository contracts.
- `src/Infrastructure`: EF Core and PostgreSQL implementation.
- `src/Api`: HTTP controllers and application startup.
- `tests`: unit tests.

## Run locally

```bash
dotnet run --project src/Api/Inventory.Api.csproj
```

The API listens on `http://localhost:8082`.

## Docker

```bash
docker build -t smartfila-inventory .
docker run --rm -p 8082:8080 -e ConnectionStrings__InventoryDatabase='Host=host.docker.internal;Port=5432;Database=smartfila_inventory;Username=postgres;Password=postgres' smartfila-inventory
```
