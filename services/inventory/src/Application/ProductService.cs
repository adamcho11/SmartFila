using Inventory.Domain;

namespace Inventory.Application;

public sealed class ProductService(IProductRepository repository)
{
    public async Task<IReadOnlyList<ProductResponse>> GetAllAsync(CancellationToken cancellationToken = default)
    {
        var products = await repository.GetAllAsync(cancellationToken);
        return products.Select(Map).ToArray();
    }

    public async Task<ProductResponse?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default)
    {
        var product = await repository.GetByIdAsync(id, cancellationToken);
        return product is null ? null : Map(product);
    }

    public async Task<ProductResponse> CreateAsync(CreateProductRequest request, CancellationToken cancellationToken = default)
    {
        var product = new Product(request.Name, request.Sku, request.Stock, request.UnitPrice);
        await repository.AddAsync(product, cancellationToken);
        return Map(product);
    }

    public async Task<ProductResponse?> UpdateAsync(Guid id, UpdateProductRequest request, CancellationToken cancellationToken = default)
    {
        var product = await repository.GetByIdAsync(id, cancellationToken);
        if (product is null)
            return null;

        product.Update(request.Name, request.Sku, request.Stock, request.UnitPrice);
        await repository.UpdateAsync(product, cancellationToken);
        return Map(product);
    }

    public async Task<bool> DeleteAsync(Guid id, CancellationToken cancellationToken = default)
    {
        var product = await repository.GetByIdAsync(id, cancellationToken);
        if (product is null)
            return false;

        await repository.DeleteAsync(id, cancellationToken);
        return true;
    }

    private static ProductResponse Map(Product product) =>
        new(product.Id, product.Name, product.Sku, product.Stock, product.UnitPrice);
}
