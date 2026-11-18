namespace Inventory.Application;

public sealed record CreateProductRequest(string Name, string Sku, int Stock, decimal UnitPrice);

public sealed record UpdateProductRequest(string Name, string Sku, int Stock, decimal UnitPrice);

public sealed record ProductResponse(Guid Id, string Name, string Sku, int Stock, decimal UnitPrice);
