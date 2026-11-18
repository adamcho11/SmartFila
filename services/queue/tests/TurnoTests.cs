using Queue.Application;
using Queue.Domain;

namespace Queue.Tests;

public sealed class TurnoTests
{
    [Fact]
    public async Task Create_and_admit_turno()
    {
        var service = new TurnoService(new FakeTurnoRepository());
        var usuarioId = Guid.CreateVersion7();

        var turno = await service.CreateAsync(new CreateTurnoRequest(usuarioId, "puerta-norte"));
        var admitido = await service.AdmitAsync(turno.Id);

        Assert.Equal(1, turno.Posicion);
        Assert.Equal("puerta-norte", turno.PuestoId);
        Assert.Equal(Domain.EstadoTurno.Admitido, admitido?.Estado);
    }

    [Fact]
    public void Turno_rejects_empty_puesto()
    {
        Assert.Throws<ArgumentException>(() =>
            new Domain.Turno(Guid.CreateVersion7(), " ", 1));
    }

    [Fact]
    public async Task Update_preserves_turno_id()
    {
        var service = new TurnoService(new FakeTurnoRepository());
        var usuarioId = Guid.CreateVersion7();
        var created = await service.CreateAsync(new CreateTurnoRequest(usuarioId, "puerta-norte"));

        var updated = await service.UpdateAsync(created.Id,
            new CreateTurnoRequest(usuarioId, "puerta-sur"));

        Assert.Equal(created.Id, updated?.Id);
        Assert.Equal("puerta-sur", updated?.PuestoId);
    }

    private sealed class FakeTurnoRepository : ITurnoRepository
    {
        private readonly Dictionary<Guid, Turno> turnos = [];

        public Task<Turno?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default)
        {
            turnos.TryGetValue(id, out var turno);
            return Task.FromResult(turno);
        }

        public Task<IReadOnlyList<Turno>> GetAllAsync(CancellationToken cancellationToken = default) =>
            Task.FromResult<IReadOnlyList<Turno>>(turnos.Values.ToArray());

        public Task<int> GetNextPositionAsync(CancellationToken cancellationToken = default) =>
            Task.FromResult(turnos.Values.Count(x => x.Estado == EstadoTurno.EnEspera) + 1);

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

        public Task DeleteAsync(Guid id, CancellationToken cancellationToken = default)
        {
            turnos.Remove(id);
            return Task.CompletedTask;
        }
    }
}
