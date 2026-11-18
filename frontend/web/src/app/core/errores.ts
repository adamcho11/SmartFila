import { HttpErrorResponse } from '@angular/common/http';
import { CuerpoError } from './modelos';

export interface ErrorVista {
  mensaje: string;
  /** Problemas por campo del formulario (details[].field → issue). */
  campos: Record<string, string>;
  traceId?: string;
}

function esCuerpoError(valor: unknown): valor is CuerpoError {
  return typeof valor === 'object' && valor !== null && 'code' in valor && 'message' in valor;
}

/** Convierte cualquier error HTTP o GraphQL en un mensaje para la vista. */
export function leerError(err: unknown): ErrorVista {
  if (err instanceof HttpErrorResponse) {
    if (err.status === 0) {
      return {
        mensaje: 'No se pudo conectar con el servidor. Revisa que docker compose esté levantado.',
        campos: {},
      };
    }
    if (esCuerpoError(err.error)) {
      const campos: Record<string, string> = {};
      for (const d of err.error.details ?? []) {
        campos[d.field] = campos[d.field] ? `${campos[d.field]}; ${d.issue}` : d.issue;
      }
      return { mensaje: err.error.message, campos, traceId: err.error.traceId };
    }
    if (err.status === 502 || err.status === 503 || err.status === 504) {
      return { mensaje: 'El servicio no está disponible en este momento.', campos: {} };
    }
    return { mensaje: `El servidor respondió ${err.status}.`, campos: {} };
  }
  if (err instanceof Error && err.message) {
    return { mensaje: err.message, campos: {} };
  }
  return { mensaje: 'Ocurrió un error inesperado.', campos: {} };
}
