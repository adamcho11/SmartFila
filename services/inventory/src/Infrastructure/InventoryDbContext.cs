using Inventory.Domain;
using Microsoft.EntityFrameworkCore;

namespace Inventory.Infrastructure;

public sealed class InventoryDbContext(DbContextOptions<InventoryDbContext> options) : DbContext(options)
{
    public DbSet<Product> Products => Set<Product>();
    public DbSet<ProcessedMessage> ProcessedMessages => Set<ProcessedMessage>();

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        var product = modelBuilder.Entity<Product>();
        product.ToTable("products");
        product.HasKey(x => x.Id);
        product.Property(x => x.Name).HasMaxLength(200).IsRequired();
        product.Property(x => x.Sku).HasMaxLength(100).IsRequired();
        product.HasIndex(x => x.Sku).IsUnique();
        product.Property(x => x.UnitPrice).HasPrecision(18, 2);

        var processedMessage = modelBuilder.Entity<ProcessedMessage>();
        processedMessage.ToTable("processed_messages");
        processedMessage.HasKey(x => x.EventId);
        processedMessage.Property(x => x.EventType).HasMaxLength(100).IsRequired();
        processedMessage.Property(x => x.OrderId).HasMaxLength(100);
        processedMessage.Property(x => x.ProcessedAt).IsRequired();
    }
}
