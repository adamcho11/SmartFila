using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using Queue.Application;
using Queue.Domain;
using Queue.Infrastructure;

namespace Queue.Api.Controllers;

[ApiController]
[Route("api/v1/turnos")]
[Route("api/turnos")]
public sealed class TurnosController(TurnoService service, QueueDbContext db) : ControllerBase
{
    [HttpGet("processed-events")]
    public async Task<ActionResult<IReadOnlyList<ProcessedMessage>>> GetProcessedEvents(CancellationToken cancellationToken)
    {
        var events = await db.ProcessedMessages
            .AsNoTracking()
            .OrderByDescending(x => x.ProcessedAt)
            .ToListAsync(cancellationToken);
        return Ok(events);
    }
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

    [HttpPut("{id:guid}")]
    public async Task<ActionResult<TurnoResponse>> Update(Guid id, CreateTurnoRequest request, CancellationToken cancellationToken)
    {
        var turno = await service.UpdateAsync(id, request, cancellationToken);
        return turno is null ? NotFound() : Ok(turno);
    }

    [HttpDelete("{id:guid}")]
    public async Task<IActionResult> Delete(Guid id, CancellationToken cancellationToken)
    {
        return await service.DeleteAsync(id, cancellationToken) ? NoContent() : NotFound();
    }
}
