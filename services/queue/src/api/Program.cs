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
        Title = "SmartFila Queue API",
        Version = "v1",
        Description = "Queue and turn management API"
    });
});
builder.Services.AddDbContext<Queue.Infrastructure.QueueDbContext>(options =>
    options.UseNpgsql(builder.Configuration.GetConnectionString("QueueDatabase")));
builder.Services.AddScoped<Queue.Application.ITurnoRepository, Queue.Infrastructure.TurnoRepository>();
builder.Services.AddScoped<Queue.Application.TurnoService>();

var app = builder.Build();

app.MapOpenApi();
app.UseSwagger();
app.UseSwaggerUI(options =>
{
    options.SwaggerEndpoint("/swagger/v1/swagger.json", "Queue API v1");
    options.RoutePrefix = "docs";
});

using (var scope = app.Services.CreateScope())
{
    var db = scope.ServiceProvider.GetRequiredService<Queue.Infrastructure.QueueDbContext>();
    await db.Database.EnsureCreatedAsync();
}
app.UseAuthorization();
app.MapControllers();
app.MapHealthChecks("/health");

app.Run();
