using Queue.Domain;

namespace Queue.Application;

public sealed class TurnoService(ITurnoRepository repository)
{
    public async Task<TurnoResponse> CreateAsync(CreateTurnoRequest request, CancellationToken cancellationToken = default)
    {
        var position = await repository.GetNextPositionAsync(cancellationToken);
        var turno = new Turno(request.UsuarioId, request.PuestoId, position);
        await repository.AddAsync(turno, cancellationToken);
        return Map(turno);
    }

    public async Task<TurnoResponse?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default) =>
        (await repository.GetByIdAsync(id, cancellationToken)) is { } turno ? Map(turno) : null;

    public async Task<IReadOnlyList<TurnoResponse>> GetAllAsync(CancellationToken cancellationToken = default) =>
        (await repository.GetAllAsync(cancellationToken)).Select(Map).ToArray();

    public async Task<TurnoResponse?> AdmitAsync(Guid id, CancellationToken cancellationToken = default)
    {
        var turno = await repository.GetByIdAsync(id, cancellationToken);
        if (turno is null) return null;
        turno.Admitir();
        await repository.UpdateAsync(turno, cancellationToken);
        return Map(turno);
    }

    public async Task<TurnoResponse?> ReleaseAsync(Guid id, CancellationToken cancellationToken = default)
    {
        var turno = await repository.GetByIdAsync(id, cancellationToken);
        if (turno is null) return null;
        turno.Liberar();
        await repository.UpdateAsync(turno, cancellationToken);
        return Map(turno);
    }

    private static TurnoResponse Map(Turno turno) =>
        new(turno.Id, turno.UsuarioId, turno.PuestoId, turno.Posicion, turno.Estado);
}
