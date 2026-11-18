using Queue.Domain;

namespace Queue.Application;

public interface ITurnoRepository
{
    Task<Turno?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default);
    Task<IReadOnlyList<Turno>> GetAllAsync(CancellationToken cancellationToken = default);
    Task<int> GetNextPositionAsync(CancellationToken cancellationToken = default);
    Task AddAsync(Turno turno, CancellationToken cancellationToken = default);
    Task UpdateAsync(Turno turno, CancellationToken cancellationToken = default);
}
