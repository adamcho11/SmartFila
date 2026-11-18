using Microsoft.EntityFrameworkCore;
using Queue.Application;
using Queue.Domain;

namespace Queue.Infrastructure;

public sealed class TurnoRepository(QueueDbContext dbContext) : ITurnoRepository
{
    public Task<Turno?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default) =>
        dbContext.Turnos.SingleOrDefaultAsync(x => x.Id == id, cancellationToken);

    public async Task<IReadOnlyList<Turno>> GetAllAsync(CancellationToken cancellationToken = default) =>
        await dbContext.Turnos.AsNoTracking().OrderBy(x => x.Posicion).ToListAsync(cancellationToken);

    public async Task<int> GetNextPositionAsync(CancellationToken cancellationToken = default)
    {
        var waiting = await dbContext.Turnos.CountAsync(
            x => x.Estado == EstadoTurno.EnEspera,
            cancellationToken);
        return waiting + 1;
    }

    public async Task AddAsync(Turno turno, CancellationToken cancellationToken = default)
    {
        dbContext.Turnos.Add(turno);
        await dbContext.SaveChangesAsync(cancellationToken);
    }

    public async Task UpdateAsync(Turno turno, CancellationToken cancellationToken = default)
    {
        dbContext.Turnos.Update(turno);
        await dbContext.SaveChangesAsync(cancellationToken);
    }

    public async Task DeleteAsync(Guid id, CancellationToken cancellationToken = default)
    {
        var turno = await dbContext.Turnos.SingleOrDefaultAsync(x => x.Id == id, cancellationToken);
        if (turno is null) return;
        dbContext.Turnos.Remove(turno);
        await dbContext.SaveChangesAsync(cancellationToken);
    }
}
