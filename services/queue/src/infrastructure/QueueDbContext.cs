using Microsoft.EntityFrameworkCore;
using Queue.Domain;

namespace Queue.Infrastructure;

public sealed class QueueDbContext(DbContextOptions<QueueDbContext> options) : DbContext(options)
{
    public DbSet<Turno> Turnos => Set<Turno>();

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        var turno = modelBuilder.Entity<Turno>();
        turno.ToTable("turnos");
        turno.HasKey(x => x.Id);
        turno.Property(x => x.PuestoId).HasMaxLength(100).IsRequired();
        turno.Property(x => x.Estado).HasConversion<string>().HasMaxLength(20);
        turno.Property(x => x.Posicion).IsRequired();
        turno.HasIndex(x => x.Posicion);
    }
}
