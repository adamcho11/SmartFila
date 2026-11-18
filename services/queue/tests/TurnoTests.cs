using Queue.Application;
using Queue.Infrastructure;

namespace Queue.Tests;

public sealed class TurnoTests
{
    [Fact]
    public async Task Create_and_admit_turno()
    {
        var service = new TurnoService(new InMemoryTurnoRepository());
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
}
