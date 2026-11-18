using Microsoft.EntityFrameworkCore;
using Queue.Domain;

namespace Queue.Infrastructure;

public sealed class QueueDbContext(DbContextOptions<QueueDbContext> options) : DbContext(options)
{
    public DbSet<Turno> Turnos => Set<Turno>();
    public DbSet<ProcessedMessage> ProcessedMessages => Set<ProcessedMessage>();

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        var turno = modelBuilder.Entity<Turno>();
        turno.ToTable("turnos");
        turno.HasKey(x => x.Id);
        turno.Property(x => x.PuestoId).HasMaxLength(100).IsRequired();
        turno.Property(x => x.Estado).HasConversion<string>().HasMaxLength(20);
        turno.Property(x => x.Posicion).IsRequired();
        turno.HasIndex(x => x.Posicion);

        var processedMessage = modelBuilder.Entity<ProcessedMessage>();
        processedMessage.ToTable("processed_messages");
        processedMessage.HasKey(x => x.EventId);
        processedMessage.Property(x => x.EventType).HasMaxLength(100).IsRequired();
        processedMessage.Property(x => x.OrderId).HasMaxLength(100);
        processedMessage.Property(x => x.ProcessedAt).IsRequired();
    }
}
