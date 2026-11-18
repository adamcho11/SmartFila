import { HttpErrorResponse } from '@angular/common/http';
import { leerError } from './errores';

describe('leerError', () => {
  it('lee el cuerpo de error común y agrupa los problemas por campo', () => {
    const err = new HttpErrorResponse({
      status: 400,
      error: {
        code: 'VALIDATION_ERROR',
        message: 'Datos inválidos',
        details: [
          { field: 'ci', issue: 'solo dígitos' },
          { field: 'ci', issue: 'mínimo 5' },
          { field: 'phone', issue: 'empieza en 6 o 7' },
        ],
        traceId: 'abc',
      },
    });
    expect(leerError(err)).toEqual({
      mensaje: 'Datos inválidos',
      campos: { ci: 'solo dígitos; mínimo 5', phone: 'empieza en 6 o 7' },
      traceId: 'abc',
    });
  });

  it('explica cuando no hay conexión', () => {
    expect(leerError(new HttpErrorResponse({ status: 0 })).mensaje).toContain(
      'No se pudo conectar',
    );
  });

  it('cubre respuestas sin cuerpo (la Cola devuelve 500 vacío)', () => {
    expect(leerError(new HttpErrorResponse({ status: 500 })).mensaje).toBe(
      'El servidor respondió 500.',
    );
  });
});
