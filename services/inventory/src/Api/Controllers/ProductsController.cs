using Inventory.Application;
using Inventory.Domain;
using Inventory.Infrastructure;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;

namespace Inventory.Api.Controllers;

[ApiController]
[Route("api/v1/products")]
[Route("api/products")]
public sealed class ProductsController(ProductService service, InventoryDbContext db) : ControllerBase
{
    [HttpGet("processed-events")]
    public async Task<ActionResult<IReadOnlyList<ProcessedMessage>>> GetProcessedEvents(CancellationToken cancellationToken)
    {
        var events = await db.ProcessedMessages
            .AsNoTracking()
            .OrderByDescending(x => x.ProcessedAt)
            .ToListAsync(cancellationToken);
        return Ok(events);
    }
    [HttpGet]
    public async Task<ActionResult<IReadOnlyList<ProductResponse>>> GetAll(CancellationToken cancellationToken)
    {
        return Ok(await service.GetAllAsync(cancellationToken));
    }

    [HttpGet("{id:guid}")]
    public async Task<ActionResult<ProductResponse>> GetById(Guid id, CancellationToken cancellationToken)
    {
        var product = await service.GetByIdAsync(id, cancellationToken);
        return product is null ? NotFound() : Ok(product);
    }

    [HttpPost]
    public async Task<ActionResult<ProductResponse>> Create(CreateProductRequest request, CancellationToken cancellationToken)
    {
        var product = await service.CreateAsync(request, cancellationToken);
        return CreatedAtAction(nameof(GetById), new { id = product.Id }, product);
    }

    [HttpPut("{id:guid}")]
    public async Task<ActionResult<ProductResponse>> Update(Guid id, UpdateProductRequest request, CancellationToken cancellationToken)
    {
        var product = await service.UpdateAsync(id, request, cancellationToken);
        return product is null ? NotFound() : Ok(product);
    }

    [HttpDelete("{id:guid}")]
    public async Task<IActionResult> Delete(Guid id, CancellationToken cancellationToken)
    {
        return await service.DeleteAsync(id, cancellationToken) ? NoContent() : NotFound();
    }
}
