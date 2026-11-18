namespace Inventory.Domain;

public sealed class ProcessedMessage
{
    public Guid EventId { get; private set; }
    public string EventType { get; private set; } = string.Empty;
    public string? OrderId { get; private set; }
    public DateTimeOffset ProcessedAt { get; private set; }

    private ProcessedMessage() { }

    public ProcessedMessage(Guid eventId, string eventType, string? orderId, DateTimeOffset processedAt)
    {
        EventId = eventId;
        EventType = eventType;
        OrderId = orderId;
        ProcessedAt = processedAt;
    }
}
