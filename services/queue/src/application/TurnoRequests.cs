using Queue.Domain;

namespace Queue.Application;

public sealed record CreateTurnoRequest(Guid UsuarioId, string PuestoId);

public sealed record TurnoResponse(Guid Id, Guid UsuarioId, string PuestoId, int Posicion, EstadoTurno Estado);
