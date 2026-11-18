FROM mcr.microsoft.com/dotnet/sdk:10.0 AS build
WORKDIR /src
COPY src/Domain/Inventory.Domain.csproj src/Domain/
COPY src/Application/Inventory.Application.csproj src/Application/
COPY src/Infrastructure/Inventory.Infrastructure.csproj src/Infrastructure/
COPY src/Api/Inventory.Api.csproj src/Api/
RUN dotnet restore src/Api/Inventory.Api.csproj
COPY src/ src/
RUN dotnet publish src/Api/Inventory.Api.csproj -c Release -o /app/publish --no-restore

FROM mcr.microsoft.com/dotnet/aspnet:10.0 AS runtime
WORKDIR /app
COPY --from=build /app/publish .
EXPOSE 8080
ENTRYPOINT ["dotnet", "Inventory.Api.dll"]
