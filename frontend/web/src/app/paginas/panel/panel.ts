import { CurrencyPipe, DatePipe, DecimalPipe } from '@angular/common';
import { ChangeDetectionStrategy, Component, computed, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { print } from 'graphql';
import { Subject, catchError, map, merge, of, switchMap, tap, timer } from 'rxjs';
import { Icono } from '../../compartido/icono';
import { AuthService } from '../../core/auth.service';
import { leerError } from '../../core/errores';
import {
  Evento,
  agruparPorEvento,
  etiquetaDisponible,
  eventoDe,
  numeroEnEvento,
  porLlegada,
  tituloUnidad,
  tramoDe,
} from '../../core/eventos';
import { CONSULTA_OVERVIEW, GatewayService } from '../../core/gateway.service';
import { EstadoTurno, Overview, Turno } from '../../core/modelos';

/** El panel se vuelve a consultar solo cada 15 s. */
const REFRESCO_MS = 15_000;

const ETIQUETA_ESTADO: Record<EstadoTurno, string> = {
  EN_ESPERA: 'En espera',
  ADMITIDO: 'Admitido',
  LIBERADO: 'Liberado',
};

type Resultado = { ok: true; datos: Overview } | { ok: false; mensaje: string };

/**
 * Vista inicial con datos reales (P1): una sola consulta GraphQL al gateway,
 * que la resuelve contra Identity, Inventario y Cola.
 */
@Component({
  selector: 'app-panel',
  imports: [CurrencyPipe, DatePipe, DecimalPipe, Icono],
  templateUrl: './panel.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelPagina {
  private readonly gateway = inject(GatewayService);
  private readonly auth = inject(AuthService);
  private readonly recargar$ = new Subject<void>();

  protected readonly datos = signal<Overview | null>(null);
  protected readonly cargando = signal(false);
  protected readonly error = signal<string | null>(null);
  protected readonly actualizado = signal<Date | null>(null);
  protected readonly consulta = print(CONSULTA_OVERVIEW);
  protected readonly etiquetaEstado = ETIQUETA_ESTADO;
  protected readonly refrescoSegundos = REFRESCO_MS / 1000;

  protected readonly nombre = computed(() => this.auth.usuario()?.firstNames.split(' ')[0] ?? '');
  protected readonly stockTotal = computed(
    () => this.datos()?.inventory.reduce((suma, p) => suma + p.stock, 0) ?? 0,
  );
  protected readonly enEspera = computed(() => this.contar('EN_ESPERA'));
  protected readonly admitidos = computed(() => this.contar('ADMITIDO'));
  /** Turnos ordenados por llegada; los liberados al final. */
  protected readonly turnos = computed(() =>
    [...(this.datos()?.queue ?? [])].sort(
      (a, b) =>
        Number(a.estado === 'LIBERADO') - Number(b.estado === 'LIBERADO') || porLlegada(a, b),
    ),
  );
  private readonly nombres = computed(
    () => new Map((this.datos()?.inventory ?? []).map((p) => [p.id, p.name])),
  );
  private readonly eventosPorPuesto = computed(
    () => new Map((this.datos()?.inventory ?? []).map((p) => [p.id, eventoDe(p.sku)])),
  );

  /** Puestos y turnos agrupados por evento; lo que no es de ningún evento, al final. */
  protected readonly gruposPuestos = computed(() =>
    agruparPorEvento(this.datos()?.inventory ?? [], (p) => eventoDe(p.sku)),
  );
  protected readonly gruposTurnos = computed(() =>
    agruparPorEvento(this.turnos(), (t) => this.eventosPorPuesto().get(t.puestoId) ?? null),
  );
  protected readonly eventosConVenta = computed(
    () => this.gruposPuestos().filter((g) => g.evento).length,
  );

  protected readonly tituloUnidad = tituloUnidad;
  protected readonly tramoDe = tramoDe;
  protected readonly etiquetaDisponible = etiquetaDisponible;

  constructor() {
    merge(timer(0, REFRESCO_MS), this.recargar$)
      .pipe(
        tap(() => this.cargando.set(true)),
        switchMap(() =>
          this.gateway.overview().pipe(
            map((datos): Resultado => ({ ok: true, datos })),
            catchError((err) => of<Resultado>({ ok: false, mensaje: leerError(err).mensaje })),
          ),
        ),
        takeUntilDestroyed(),
      )
      .subscribe((r) => {
        this.cargando.set(false);
        if (r.ok) {
          this.datos.set(r.datos);
          this.actualizado.set(new Date());
          this.error.set(null);
        } else {
          this.error.set(r.mensaje);
        }
      });
  }

  protected cargar(): void {
    this.recargar$.next();
  }

  protected nombrePuesto(id: string): string {
    const nombre = this.nombres().get(id);
    return nombre ? tituloUnidad(nombre) : id;
  }

  /** Número del turno dentro de su evento (la Cola repite los suyos). */
  protected numero(turno: Turno): number {
    return numeroEnEvento(
      turno,
      this.datos()?.queue ?? [],
      (id) => this.eventosPorPuesto().get(id) ?? null,
    );
  }

  protected sumaStock(puestos: { stock: number }[]): number {
    return puestos.reduce((suma, p) => suma + p.stock, 0);
  }

  protected contarEnEspera(turnos: Turno[]): number {
    return turnos.filter((t) => t.estado === 'EN_ESPERA').length;
  }

  /** "3 tramos", "1 sector" o "1 puesto", según el tipo de unidad del evento. */
  protected etiquetaGrupo(evento: Evento | null, cantidad: number): string {
    const [una, varias] =
      evento?.unidad === 'tramo'
        ? ['tramo', 'tramos']
        : evento?.unidad === 'asiento'
          ? ['sector', 'sectores']
          : ['puesto', 'puestos'];
    return cantidad === 1 ? una : varias;
  }

  protected nombreEvento(evento: Evento | null): string {
    return evento?.nombre ?? 'Sin evento asignado';
  }

  protected esMio(turno: Turno): boolean {
    return turno.usuarioId === this.auth.usuario()?.id;
  }

  private contar(estado: EstadoTurno): number {
    return this.datos()?.queue.filter((t) => t.estado === estado).length ?? 0;
  }
}
