using Microsoft.EntityFrameworkCore;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddControllers();
builder.Services.AddOpenApi();
builder.Services.AddHealthChecks();

builder.Services.AddDbContext<Inventory.Infrastructure.InventoryDbContext>(options =>
{
    var connectionString = builder.Configuration.GetConnectionString("InventoryDatabase");
    options.UseNpgsql(connectionString);
});

builder.Services.AddScoped<Inventory.Application.IProductRepository, Inventory.Infrastructure.ProductRepository>();
builder.Services.AddScoped<Inventory.Application.ProductService>();

var app = builder.Build();

using (var scope = app.Services.CreateScope())
{
    var db = scope.ServiceProvider.GetRequiredService<Inventory.Infrastructure.InventoryDbContext>();
    await db.Database.EnsureCreatedAsync();
}

if (app.Environment.IsDevelopment())
{
    app.MapOpenApi();
}

app.UseAuthorization();

app.MapControllers();
app.MapHealthChecks("/health");

app.Run();
