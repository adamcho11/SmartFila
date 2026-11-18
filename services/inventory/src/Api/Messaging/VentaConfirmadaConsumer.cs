using System.Text;
using System.Text.Json;
using Inventory.Domain;
using Inventory.Infrastructure;
using Microsoft.EntityFrameworkCore;
using RabbitMQ.Client;
using RabbitMQ.Client.Events;

namespace Inventory.Api.Messaging;

public sealed class VentaConfirmadaConsumer(
    IServiceScopeFactory scopeFactory,
    IConfiguration configuration,
    ILogger<VentaConfirmadaConsumer> logger) : BackgroundService
{
    private const string DomainExchange = "smartfila.domain";
    private const string DeadLetterExchange = "smartfila.dlx";
    private const string QueueName = "inventario.venta-confirmada";
    private const string DeadLetterQueueName = "inventario.venta-confirmada.dlq";
    private const string RoutingKey = "venta.confirmada";

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        if (string.Equals(configuration["RABBITMQ_ENABLED"], "false", StringComparison.OrdinalIgnoreCase))
        {
            logger.LogInformation("Consumidor RabbitMQ de Inventario deshabilitado por configuración.");
            return;
        }

        var host = configuration["RABBITMQ_HOST"] ?? "rabbitmq";
        var port = int.TryParse(configuration["RABBITMQ_PORT"], out var parsedPort) ? parsedPort : 5672;
        var user = configuration["RABBITMQ_USER"] ?? "smartfila";
        var password = configuration["RABBITMQ_PASSWORD"] ?? "cambiar-esta-clave";

        while (!stoppingToken.IsCancellationRequested)
        {
            try
            {
                var factory = new ConnectionFactory
                {
                    HostName = host,
                    Port = port,
                    UserName = user,
                    Password = password,
                    DispatchConsumersAsync = true,
                    AutomaticRecoveryEnabled = true
                };

                using var connection = factory.CreateConnection();
                using var channel = connection.CreateModel();

                channel.ExchangeDeclare(DomainExchange, ExchangeType.Topic, durable: true, autoDelete: false);
                channel.ExchangeDeclare(DeadLetterExchange, ExchangeType.Topic, durable: true, autoDelete: false);

                channel.QueueDeclare(
                    queue: DeadLetterQueueName,
                    durable: true,
                    exclusive: false,
                    autoDelete: false,
                    arguments: null);
                channel.QueueBind(DeadLetterQueueName, DeadLetterExchange, DeadLetterQueueName);

                var queueArgs = new Dictionary<string, object>
                {
                    ["x-queue-type"] = "quorum",
                    ["x-delivery-limit"] = 3,
                    ["x-dead-letter-exchange"] = DeadLetterExchange,
                    ["x-dead-letter-routing-key"] = DeadLetterQueueName
                };

                channel.QueueDeclare(
                    queue: QueueName,
                    durable: true,
                    exclusive: false,
                    autoDelete: false,
                    arguments: queueArgs);
                channel.QueueBind(QueueName, DomainExchange, RoutingKey);
                channel.BasicQos(prefetchSize: 0, prefetchCount: 1, global: false);

                var consumer = new AsyncEventingBasicConsumer(channel);
                consumer.Received += async (_, ea) =>
                {
                    var deliveryCount = ObtenerConteoEntrega(ea.BasicProperties);
                    try
                    {
                        await ProcesarMensajeAsync(ea.Body.ToArray(), stoppingToken);
                        channel.BasicAck(ea.DeliveryTag, multiple: false);
                    }
                    catch (Exception ex)
                    {
                        var shouldRequeue = deliveryCount < 2;
                        logger.LogWarning(
                            ex,
                            "Error procesando mensaje en {Queue} (intento {Attempt}/3, requeue={Requeue})",
                            QueueName,
                            deliveryCount + 1,
                            shouldRequeue);
                        channel.BasicNack(ea.DeliveryTag, multiple: false, requeue: shouldRequeue);
                    }
                };

                channel.BasicConsume(queue: QueueName, autoAck: false, consumer: consumer);
                logger.LogInformation(
                    "Consumidor de Inventario escuchando cola {Queue} en {Host}:{Port}",
                    QueueName,
                    host,
                    port);

                await Task.Delay(Timeout.Infinite, stoppingToken);
            }
            catch (OperationCanceledException) when (stoppingToken.IsCancellationRequested)
            {
                break;
            }
            catch (Exception ex)
            {
                logger.LogWarning(ex, "Conexión a RabbitMQ interrumpida en Inventario; reintentando en 3 s");
                try
                {
                    await Task.Delay(TimeSpan.FromSeconds(3), stoppingToken);
                }
                catch (OperationCanceledException)
                {
                    break;
                }
            }
        }
    }

    private async Task ProcesarMensajeAsync(byte[] body, CancellationToken cancellationToken)
    {
        var json = Encoding.UTF8.GetString(body);
        using var document = JsonDocument.Parse(json);
        var root = document.RootElement;

        if (!root.TryGetProperty("eventId", out var eventIdElem)
            || !Guid.TryParse(eventIdElem.GetString(), out var eventId))
        {
            throw new InvalidOperationException("El evento no contiene un eventId UUID válido.");
        }

        var eventType = root.TryGetProperty("eventType", out var eventTypeElem)
            ? eventTypeElem.GetString() ?? RoutingKey
            : RoutingKey;

        if (!root.TryGetProperty("data", out var dataElem) || dataElem.ValueKind != JsonValueKind.Object)
        {
            throw new InvalidOperationException("El evento no contiene objeto data.");
        }

        if (dataElem.TryGetProperty("simulateFailure", out var simFailElem)
            && simFailElem.ValueKind == JsonValueKind.True)
        {
            throw new InvalidOperationException("Fallo simulado para demostración de reintentos y DLQ.");
        }

        var orderId = dataElem.TryGetProperty("orderId", out var orderIdElem) ? orderIdElem.GetString() : null;
        var sku = dataElem.TryGetProperty("sku", out var skuElem) ? skuElem.GetString() : null;
        var unit = dataElem.TryGetProperty("unit", out var unitElem) ? unitElem.GetString() : null;

        using var scope = scopeFactory.CreateScope();
        var db = scope.ServiceProvider.GetRequiredService<InventoryDbContext>();

        var alreadyProcessed = await db.ProcessedMessages
            .AsNoTracking()
            .AnyAsync(x => x.EventId == eventId, cancellationToken);

        if (alreadyProcessed)
        {
            logger.LogInformation(
                "Evento {EventId} ya procesado previamente en Inventario (idempotencia); ACK sin efecto repetido.",
                eventId);
            return;
        }

        Product? product = null;
        if (!string.IsNullOrWhiteSpace(sku) || !string.IsNullOrWhiteSpace(unit))
        {
            product = await db.Products.FirstOrDefaultAsync(
                p => (!string.IsNullOrWhiteSpace(sku) && p.Sku == sku)
                     || (!string.IsNullOrWhiteSpace(unit) && p.Name == unit),
                cancellationToken);
        }

        product ??= await db.Products
            .OrderBy(p => p.Name)
            .FirstOrDefaultAsync(p => p.Stock > 0, cancellationToken);

        if (product is not null && product.DecrementStock(1))
        {
            logger.LogInformation(
                "Stock actualizado en Inventario para SKU={Sku} tras evento {EventId}: nuevo stock={Stock}",
                product.Sku,
                eventId,
                product.Stock);
        }

        db.ProcessedMessages.Add(new ProcessedMessage(eventId, eventType, orderId, DateTimeOffset.UtcNow));
        await db.SaveChangesAsync(cancellationToken);
    }

    private static long ObtenerConteoEntrega(IBasicProperties? properties)
    {
        if (properties?.Headers is null)
            return 0;

        if (!properties.Headers.TryGetValue("x-delivery-count", out var raw) || raw is null)
            return 0;

        return raw switch
        {
            byte b => b,
            sbyte sb => sb,
            short s => s,
            int i => i,
            long l => l,
            _ => long.TryParse(raw.ToString(), out var parsed) ? parsed : 0
        };
    }
}
