import { CurrencyPipe, DatePipe, DecimalPipe } from '@angular/common';
import { ChangeDetectionStrategy, Component, computed, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { EMPTY, catchError, switchMap, timer } from 'rxjs';
import { Dialogo } from '../../compartido/dialogo';
import { Icono } from '../../compartido/icono';
import { AuthService } from '../../core/auth.service';
import { ColaService } from '../../core/cola.service';
import { leerError } from '../../core/errores';
import {
  Evento,
  GrupoEvento,
  agruparPorEvento,
  delanteEnEvento,
  etiquetaDisponible,
  numeroEnEvento,
  porLlegada,
  eventoDe,
  tituloUnidad,
  tramoDe,
} from '../../core/eventos';
import { GatewayService } from '../../core/gateway.service';
import { Producto, Turno } from '../../core/modelos';

/** Cada cuánto se consulta la posición (decisión v2: polling de 5 s). */
const INTERVALO_MS = 5000;
/** Personas que se dibujan delante de ti; el resto se resume en "+N". */
const MAX_VISIBLES = 12;

@Component({
  selector: 'app-turno',
  imports: [CurrencyPipe, DatePipe, DecimalPipe, Icono, Dialogo],
  templateUrl: './turno.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class TurnoPagina {
  private readonly auth = inject(AuthService);
  private readonly cola = inject(ColaService);
  private readonly gateway = inject(GatewayService);

  protected readonly intervaloSegundos = INTERVALO_MS / 1000;
  protected readonly puestos = signal<Producto[]>([]);
  protected readonly cargandoPuestos = signal(true);
  protected readonly errorPuestos = signal<string | null>(null);
  protected readonly puestoElegido = signal('');
  /** Clave del evento elegido (GUA, COL…). */
  protected readonly eventoElegido = signal('');

  protected readonly tituloUnidad = tituloUnidad;
  protected readonly tramoDe = tramoDe;
  protected readonly etiquetaDisponible = etiquetaDisponible;

  protected readonly turnos = signal<Turno[] | null>(null);
  protected readonly ultimaConsulta = signal<Date | null>(null);
  protected readonly errorConsulta = signal<string | null>(null);

  protected readonly enviando = signal(false);
  protected readonly errorAccion = signal<string | null>(null);
  protected readonly confirmarSalida = signal(false);

  /** Personas delante la primera vez que vimos cada turno (para la barra de avance). */
  private readonly delanteInicial = new Map<string, number>();

  /** Turno activo más reciente del usuario (en espera o admitido). */
  protected readonly miTurno = computed(() => {
    const yo = this.auth.usuario()?.id;
    const activos = (this.turnos() ?? []).filter(
      (t) => t.usuarioId === yo && t.estado !== 'LIBERADO',
    );
    return activos.sort(porLlegada).at(-1) ?? null;
  });

  /** Evento de cada puesto, según su SKU. */
  private readonly eventosPorPuesto = computed(
    () => new Map(this.puestos().map((p) => [p.id, eventoDe(p.sku)])),
  );
  private readonly eventoDePuesto = (id: string): Evento | null =>
    this.eventosPorPuesto().get(id) ?? null;

  /** Eventos con puestos a la venta; los productos sin evento no se ofrecen. */
  protected readonly eventos = computed(() =>
    agruparPorEvento(this.puestos(), (p) => eventoDe(p.sku)).filter(
      (g): g is GrupoEvento<Producto> & { evento: Evento } => g.evento !== null,
    ),
  );
  protected readonly grupoElegido = computed(
    () => this.eventos().find((g) => g.evento.clave === this.eventoElegido()) ?? null,
  );

  protected readonly eventoDelTurno = computed(() => {
    const mio = this.miTurno();
    return mio ? this.eventoDePuesto(mio.puestoId) : null;
  });

  /** Mi número dentro del evento; null hasta conocer el evento de cada puesto. */
  protected readonly numero = computed(() => {
    const mio = this.miTurno();
    if (!mio || this.cargandoPuestos()) return null;
    return numeroEnEvento(mio, this.turnos() ?? [], this.eventoDePuesto);
  });

  /** Personas en espera que llegaron antes que yo, para el mismo evento. */
  protected readonly delante = computed(() => {
    const mio = this.miTurno();
    if (!mio || mio.estado !== 'EN_ESPERA') return 0;
    return delanteEnEvento(mio, this.turnos() ?? [], this.eventoDePuesto);
  });

  /** Personas en espera para el evento de mi turno o, si no tengo, el elegido. */
  protected readonly enEspera = computed(() => {
    const evento = this.eventoDelTurno() ?? this.grupoElegido()?.evento ?? null;
    return (this.turnos() ?? []).filter(
      (t) => t.estado === 'EN_ESPERA' && this.eventoDePuesto(t.puestoId) === evento,
    ).length;
  });

  /** Fila dibujada: "+N" lejos, luego las personas más cercanas, luego tú. */
  protected readonly filaVisual = computed(() => {
    const n = this.delante();
    const visibles = Math.min(n, MAX_VISIBLES);
    return { personas: Array.from({ length: visibles }, (_, i) => i), resto: n - visibles };
  });

  /** Porcentaje avanzado desde que entraste a la fila. */
  protected readonly avance = computed(() => {
    const mio = this.miTurno();
    if (!mio) return 0;
    if (mio.estado === 'ADMITIDO') return 100;
    // Sin los puestos no se sabe el evento de cada turno y se contaría a todos.
    if (this.cargandoPuestos()) return 0;
    const actual = this.delante();
    if (!this.delanteInicial.has(mio.id)) this.delanteInicial.set(mio.id, actual);
    const inicial = this.delanteInicial.get(mio.id) ?? actual;
    return inicial === 0 ? 100 : Math.round(((inicial - actual) / inicial) * 100);
  });

  protected readonly puestoDelTurno = computed(() => {
    const mio = this.miTurno();
    return mio ? (this.puestos().find((p) => p.id === mio.puestoId) ?? null) : null;
  });

  constructor() {
    this.gateway
      .puestos()
      .pipe(takeUntilDestroyed())
      .subscribe({
        next: (puestos) => {
          this.puestos.set(puestos);
          this.cargandoPuestos.set(false);
          const primero = this.eventos()[0]?.evento;
          if (primero && !this.eventoElegido()) this.elegirEvento(primero.clave);
        },
        error: (err) => {
          this.errorPuestos.set(leerError(err).mensaje);
          this.cargandoPuestos.set(false);
        },
      });

    timer(0, INTERVALO_MS)
      .pipe(
        switchMap(() =>
          this.cola.listar().pipe(
            catchError((err) => {
              this.errorConsulta.set(leerError(err).mensaje);
              return EMPTY;
            }),
          ),
        ),
        takeUntilDestroyed(),
      )
      .subscribe((turnos) => {
        this.turnos.set(turnos);
        this.ultimaConsulta.set(new Date());
        this.errorConsulta.set(null);
      });
  }

  /** Cambia de evento y deja elegido su primer puesto. */
  protected elegirEvento(clave: string): void {
    this.eventoElegido.set(clave);
    const grupo = this.eventos().find((g) => g.evento.clave === clave);
    this.puestoElegido.set(grupo?.elementos[0]?.id ?? '');
  }

  protected pedirTurno(): void {
    const usuario = this.auth.usuario();
    const puesto = this.puestoElegido();
    if (!usuario || !puesto || this.miTurno()) return;
    this.ejecutar(this.cola.pedirTurno(usuario.id, puesto));
  }

  protected salirDeLaFila(): void {
    const mio = this.miTurno();
    if (mio) this.ejecutar(this.cola.liberar(mio.id));
  }

  /** Aplica la respuesta al instante, sin esperar la siguiente consulta. */
  private ejecutar(accion: ReturnType<ColaService['liberar']>): void {
    this.enviando.set(true);
    this.errorAccion.set(null);
    accion.subscribe({
      next: (turno) => {
        this.turnos.update((ts) => [...(ts ?? []).filter((t) => t.id !== turno.id), turno]);
        this.enviando.set(false);
      },
      error: (err) => {
        this.errorAccion.set(leerError(err).mensaje);
        this.enviando.set(false);
      },
    });
  }
}
