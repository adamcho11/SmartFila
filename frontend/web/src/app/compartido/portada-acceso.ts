import { DecimalPipe } from '@angular/common';
import { ChangeDetectionStrategy, Component, computed, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { catchError, of, switchMap, timer } from 'rxjs';
import { EVENTO_DESTACADO, agruparPorEvento, eventoDe, paradasDelEvento } from '../core/eventos';
import { GatewayService } from '../core/gateway.service';
import { Producto } from '../core/modelos';

/** Cada cuánto se vuelve a leer la disponibilidad de los tramos. */
const REFRESCO_MS = 30_000;

/** Lado izquierdo de login y registro: el evento, su recorrido y los tramos a la venta. */
@Component({
  selector: 'app-portada-acceso',
  imports: [DecimalPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'portada' },
  template: `
    <p class="portada__lugar">{{ evento.ciudad }} · {{ evento.nombre }}</p>
    <h1 class="titulo-grande">Saca tu puesto sin madrugar en la fila.</h1>
    <p class="portada__bajada">
      Entras a la fila desde el celular y esperas tu turno donde estés. Cuando te toca, eliges tramo
      y pagas; el comprobante sale a tu nombre.
    </p>

    <section class="ruta" aria-labelledby="ruta-titulo">
      <h2 id="ruta-titulo" class="ruta__titulo">Recorrido y tramos a la venta</h2>
      <ol class="ruta__paradas">
        @for (
          parada of paradas();
          track parada.nombre;
          let i = $index, primera = $first, ultima = $last
        ) {
          <li
            class="ruta__parada"
            [class.ruta__parada--venta]="parada.tramos.length"
            [style.--i]="i"
          >
            <span class="ruta__calle">
              {{ parada.nombre }}
              @if (primera) {
                <small>salida</small>
              } @else if (ultima) {
                <small>llegada</small>
              }
            </span>
            @for (t of parada.tramos; track t.letra) {
              <span class="ruta__tramo">Tramo {{ t.letra }} · {{ t.desde }} → {{ t.hasta }}</span>
              <span class="ruta__cifras">
                @if (t.metros === null) {
                  <span>{{ estado() === 'error' ? 'Sin datos' : 'Consultando…' }}</span>
                } @else if (t.metros === 0) {
                  <span>Agotado</span>
                } @else {
                  <strong>Bs {{ t.precio | number: '1.0-2' }}</strong> el metro
                  <span>{{ t.metros | number }} m libres</span>
                }
              </span>
            }
          </li>
        }
      </ol>
    </section>

    @if (otros().length) {
      <p class="portada__otros">
        También a la venta:
        @for (o of otros(); track o.nombre; let ultimo = $last) {
          <strong>{{ o.nombre }}</strong> · desde Bs {{ o.desde | number: '1.0-2'
          }}{{ ultimo ? '.' : ';' }}
        }
      </p>
    }

    <p class="portada__reglas">
      Un turno por C.I. · El comprobante va a nombre del titular y no se transfiere.
    </p>
  `,
})
export class PortadaAcceso {
  protected readonly evento = EVENTO_DESTACADO;

  private readonly productos = signal<Producto[] | null>(null);
  protected readonly estado = signal<'cargando' | 'listo' | 'error'>('cargando');
  protected readonly paradas = computed(() => paradasDelEvento(this.evento, this.productos()));
  /** Los demás eventos con venta abierta y su precio más bajo. */
  protected readonly otros = computed(() =>
    agruparPorEvento(this.productos() ?? [], (p) => eventoDe(p.sku))
      .filter((g) => g.evento && g.evento !== this.evento)
      .map((g) => ({
        nombre: g.evento!.nombre,
        desde: Math.min(...g.elementos.map((p) => p.unitPrice)),
      })),
  );

  constructor() {
    const gateway = inject(GatewayService);
    // Inventario se lee sin sesión: la portada muestra la disponibilidad real.
    timer(0, REFRESCO_MS)
      .pipe(
        switchMap(() => gateway.puestos().pipe(catchError(() => of(null)))),
        takeUntilDestroyed(),
      )
      .subscribe((productos) => {
        if (productos) {
          this.productos.set(productos);
          this.estado.set('listo');
        } else if (!this.productos()) {
          this.estado.set('error');
        }
      });
  }
}
