using Microsoft.EntityFrameworkCore;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddControllers();
builder.Services.AddOpenApi();
builder.Services.AddHealthChecks();
builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen(options =>
{
    options.SwaggerDoc("v1", new Microsoft.OpenApi.OpenApiInfo
    {
        Title = "SmartFila Inventory API",
        Version = "v1",
        Description = "Inventory and product management API"
    });
});

builder.Services.AddDbContext<Inventory.Infrastructure.InventoryDbContext>(options =>
{
    var connectionString = builder.Configuration.GetConnectionString("InventoryDatabase");
    options.UseNpgsql(connectionString);
});

builder.Services.AddScoped<Inventory.Application.IProductRepository, Inventory.Infrastructure.ProductRepository>();
builder.Services.AddScoped<Inventory.Application.ProductService>();
builder.Services.AddHostedService<Inventory.Api.Messaging.VentaConfirmadaConsumer>();

var app = builder.Build();

using (var scope = app.Services.CreateScope())
{
    var db = scope.ServiceProvider.GetRequiredService<Inventory.Infrastructure.InventoryDbContext>();
    await db.Database.EnsureCreatedAsync();
    await db.Database.ExecuteSqlRawAsync("""
        CREATE TABLE IF NOT EXISTS processed_messages (
            "EventId" uuid PRIMARY KEY,
            "EventType" character varying(100) NOT NULL,
            "OrderId" character varying(100),
            "ProcessedAt" timestamp with time zone NOT NULL
        );
        """);
}

app.MapOpenApi();
app.UseSwagger();
app.UseSwaggerUI(options =>
{
    options.SwaggerEndpoint("/swagger/v1/swagger.json", "Inventory API v1");
    options.RoutePrefix = "docs";
});

app.UseAuthorization();

app.MapControllers();
app.MapHealthChecks("/health");

app.Run();
