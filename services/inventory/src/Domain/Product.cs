namespace Inventory.Domain;

public sealed class Product
{
    public Guid Id { get; private set; }
    public string Name { get; private set; } = string.Empty;
    public string Sku { get; private set; } = string.Empty;
    public int Stock { get; private set; }
    public decimal UnitPrice { get; private set; }

    private Product() { }

    public Product(string name, string sku, int stock, decimal unitPrice)
    {
        SetDetails(name, sku, stock, unitPrice);
        Id = Guid.CreateVersion7();
    }

    public void Update(string name, string sku, int stock, decimal unitPrice)
    {
        SetDetails(name, sku, stock, unitPrice);
    }

    public bool DecrementStock(int quantity = 1)
    {
        if (quantity <= 0 || Stock < quantity)
            return false;

        Stock -= quantity;
        return true;
    }

    private void SetDetails(string name, string sku, int stock, decimal unitPrice)
    {
        if (string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("The product name is required.", nameof(name));
        if (string.IsNullOrWhiteSpace(sku))
            throw new ArgumentException("The SKU is required.", nameof(sku));
        if (stock < 0)
            throw new ArgumentOutOfRangeException(nameof(stock), "Stock cannot be negative.");
        if (unitPrice < 0)
            throw new ArgumentOutOfRangeException(nameof(unitPrice), "Unit price cannot be negative.");

        Name = name.Trim();
        Sku = sku.Trim();
        Stock = stock;
        UnitPrice = unitPrice;
    }
}
