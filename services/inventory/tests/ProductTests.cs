using Inventory.Application;
using Inventory.Domain;

namespace Inventory.Tests;

public sealed class ProductTests
{
    [Fact]
    public void Creating_product_with_negative_stock_throws()
    {
        Assert.Throws<ArgumentOutOfRangeException>(() =>
            new Product("Ticket", "TICKET-001", -1, 25m));
    }

    [Fact]
    public async Task Creating_product_returns_response()
    {
        var repository = new InMemoryProductRepository();
        var service = new ProductService(repository);

        var response = await service.CreateAsync(
            new CreateProductRequest("Ticket General", "GENERAL-001", 100, 25m));

        Assert.Equal("Ticket General", response.Name);
        Assert.Equal("GENERAL-001", response.Sku);
        Assert.Equal(100, response.Stock);
    }

    private sealed class InMemoryProductRepository : IProductRepository
    {
        private readonly Dictionary<Guid, Product> products = [];

        public Task<Product?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default)
        {
            products.TryGetValue(id, out var product);
            return Task.FromResult(product);
        }

        public Task<IReadOnlyList<Product>> GetAllAsync(CancellationToken cancellationToken = default) =>
            Task.FromResult<IReadOnlyList<Product>>(products.Values.ToArray());

        public Task AddAsync(Product product, CancellationToken cancellationToken = default)
        {
            products.Add(product.Id, product);
            return Task.CompletedTask;
        }

        public Task UpdateAsync(Product product, CancellationToken cancellationToken = default)
        {
            products[product.Id] = product;
            return Task.CompletedTask;
        }

        public Task DeleteAsync(Guid id, CancellationToken cancellationToken = default)
        {
            products.Remove(id);
            return Task.CompletedTask;
        }
    }
}
