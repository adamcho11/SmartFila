using System.Collections.Concurrent;
using Queue.Application;
using Queue.Domain;

namespace Queue.Infrastructure;

public sealed class InMemoryTurnoRepository : ITurnoRepository
{
    private readonly ConcurrentDictionary<Guid, Turno> turnos = new();

    public Task<Turno?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default)
    {
        turnos.TryGetValue(id, out var turno);
        return Task.FromResult(turno);
    }

    public Task<IReadOnlyList<Turno>> GetAllAsync(CancellationToken cancellationToken = default) =>
        Task.FromResult<IReadOnlyList<Turno>>(turnos.Values.OrderBy(x => x.Posicion).ToArray());

    public Task<int> GetNextPositionAsync(CancellationToken cancellationToken = default)
    {
        var waiting = turnos.Values.Count(x => x.Estado == EstadoTurno.EnEspera);
        return Task.FromResult(waiting + 1);
    }

    public Task AddAsync(Turno turno, CancellationToken cancellationToken = default)
    {
        turnos[turno.Id] = turno;
        return Task.CompletedTask;
    }

    public Task UpdateAsync(Turno turno, CancellationToken cancellationToken = default)
    {
        turnos[turno.Id] = turno;
        return Task.CompletedTask;
    }
}
