using Microsoft.AspNetCore.Mvc;
using Queue.Application;

namespace Queue.Api.Controllers;

[ApiController]
[Route("api/[controller]")]
public sealed class TurnosController(TurnoService service) : ControllerBase
{
    [HttpGet]
    public async Task<ActionResult<IReadOnlyList<TurnoResponse>>> GetAll(CancellationToken cancellationToken) =>
        Ok(await service.GetAllAsync(cancellationToken));

    [HttpGet("{id:guid}")]
    public async Task<ActionResult<TurnoResponse>> GetById(Guid id, CancellationToken cancellationToken)
    {
        var turno = await service.GetByIdAsync(id, cancellationToken);
        return turno is null ? NotFound() : Ok(turno);
    }

    [HttpPost]
    public async Task<ActionResult<TurnoResponse>> Create(CreateTurnoRequest request, CancellationToken cancellationToken)
    {
        var turno = await service.CreateAsync(request, cancellationToken);
        return CreatedAtAction(nameof(GetById), new { id = turno.Id }, turno);
    }

    [HttpPost("{id:guid}/admitir")]
    public async Task<ActionResult<TurnoResponse>> Admit(Guid id, CancellationToken cancellationToken)
    {
        var turno = await service.AdmitAsync(id, cancellationToken);
        return turno is null ? NotFound() : Ok(turno);
    }

    [HttpPost("{id:guid}/liberar")]
    public async Task<ActionResult<TurnoResponse>> Release(Guid id, CancellationToken cancellationToken)
    {
        var turno = await service.ReleaseAsync(id, cancellationToken);
        return turno is null ? NotFound() : Ok(turno);
    }
}
