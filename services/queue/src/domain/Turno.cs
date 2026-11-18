namespace Queue.Domain;

public enum EstadoTurno
{
    EnEspera,
    Admitido,
    Liberado
}

public sealed class Turno
{
    public Guid Id { get; private init; } = Guid.CreateVersion7();
    public Guid UsuarioId { get; private set; }
    public string PuestoId { get; private set; } = string.Empty;
    public int Posicion { get; private init; }
    public EstadoTurno Estado { get; private set; } = EstadoTurno.EnEspera;

    private Turno() { }

    public Turno(Guid usuarioId, string puestoId, int posicion)
    {
        if (usuarioId == Guid.Empty)
            throw new ArgumentException("UsuarioId is required.", nameof(usuarioId));
        if (string.IsNullOrWhiteSpace(puestoId))
            throw new ArgumentException("PuestoId is required.", nameof(puestoId));
        if (posicion < 1)
            throw new ArgumentOutOfRangeException(nameof(posicion), "Position must be positive.");

        UsuarioId = usuarioId;
        PuestoId = puestoId.Trim();
        Posicion = posicion;
    }

    public void Admitir() => Estado = EstadoTurno.Admitido;
    public void Liberar() => Estado = EstadoTurno.Liberado;

    public void Actualizar(Guid usuarioId, string puestoId)
    {
        if (Estado != EstadoTurno.EnEspera)
            throw new InvalidOperationException("Only waiting turns can be updated.");
        if (usuarioId == Guid.Empty)
            throw new ArgumentException("UsuarioId is required.", nameof(usuarioId));
        if (string.IsNullOrWhiteSpace(puestoId))
            throw new ArgumentException("PuestoId is required.", nameof(puestoId));

        UsuarioId = usuarioId;
        PuestoId = puestoId.Trim();
    }
}
