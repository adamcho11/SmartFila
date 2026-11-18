import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { firstValueFrom } from 'rxjs';
import { ColaService, normalizarEstado } from './cola.service';

describe('normalizarEstado', () => {
  it('traduce el número del enum de C#', () => {
    expect(normalizarEstado(0)).toBe('EN_ESPERA');
    expect(normalizarEstado(1)).toBe('ADMITIDO');
    expect(normalizarEstado(2)).toBe('LIBERADO');
  });

  it('acepta el estado como texto', () => {
    expect(normalizarEstado('Admitido')).toBe('ADMITIDO');
    expect(normalizarEstado('LIBERADO')).toBe('LIBERADO');
    expect(normalizarEstado('EnEspera')).toBe('EN_ESPERA');
  });
});

describe('ColaService', () => {
  let servicio: ColaService;
  let http: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    });
    servicio = TestBed.inject(ColaService);
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => http.verify());

  it('lista los turnos con el estado normalizado', async () => {
    const lista = firstValueFrom(servicio.listar());
    http
      .expectOne('/api/v1/turnos')
      .flush([{ id: 't1', usuarioId: 'u1', puestoId: 'p1', posicion: 1, estado: 1 }]);
    expect(await lista).toEqual([
      { id: 't1', usuarioId: 'u1', puestoId: 'p1', posicion: 1, estado: 'ADMITIDO' },
    ]);
  });

  it('pide turno con el usuario y el puesto', async () => {
    const turno = firstValueFrom(servicio.pedirTurno('u1', 'p1'));
    const req = http.expectOne('/api/v1/turnos');
    expect(req.request.method).toBe('POST');
    expect(req.request.body).toEqual({ usuarioId: 'u1', puestoId: 'p1' });
    req.flush({ id: 't9', usuarioId: 'u1', puestoId: 'p1', posicion: 9, estado: 0 });
    expect((await turno).estado).toBe('EN_ESPERA');
  });
});
