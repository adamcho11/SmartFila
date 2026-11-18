import { HttpClient } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable, map } from 'rxjs';
import { EstadoTurno, Turno } from './modelos';

/** Así responde hoy la Cola: el estado llega como número del enum de C#. */
interface TurnoApi extends Omit<Turno, 'estado'> {
  estado: number | string;
}

const ESTADOS: EstadoTurno[] = ['EN_ESPERA', 'ADMITIDO', 'LIBERADO'];

export function normalizarEstado(estado: number | string): EstadoTurno {
  if (typeof estado === 'number') return ESTADOS[estado] ?? 'EN_ESPERA';
  const texto = estado.toUpperCase();
  if (texto.includes('ADMIT')) return 'ADMITIDO';
  if (texto.includes('LIBER')) return 'LIBERADO';
  return 'EN_ESPERA';
}

const aTurno = (t: TurnoApi): Turno => ({ ...t, estado: normalizarEstado(t.estado) });

/**
 * La posición en la fila se consulta directo a la Cola por REST (ADR-003):
 * es el camino caliente y no debe pasar por el gateway.
 */
@Injectable({ providedIn: 'root' })
export class ColaService {
  private readonly http = inject(HttpClient);
  private readonly base = '/api/v1/turnos';

  listar(): Observable<Turno[]> {
    return this.http.get<TurnoApi[]>(this.base).pipe(map((ts) => ts.map(aTurno)));
  }

  pedirTurno(usuarioId: string, puestoId: string): Observable<Turno> {
    return this.http.post<TurnoApi>(this.base, { usuarioId, puestoId }).pipe(map(aTurno));
  }

  liberar(id: string): Observable<Turno> {
    return this.http.post<TurnoApi>(`${this.base}/${id}/liberar`, {}).pipe(map(aTurno));
  }
}
