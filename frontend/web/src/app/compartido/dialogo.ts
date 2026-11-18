import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  effect,
  input,
  model,
  output,
  viewChild,
} from '@angular/core';
import { Icono, NombreIcono } from './icono';

/**
 * Modal de confirmación sobre <dialog> nativo: atrapa el foco, cierra con Esc
 * y con clic en el fondo. Uso: <app-dialogo [(abierto)]="señal" (confirmar)="...">.
 */
@Component({
  selector: 'app-dialogo',
  imports: [Icono],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <dialog
      #dialogo
      class="dialogo"
      [class.dialogo--peligro]="tono() === 'peligro'"
      [attr.aria-labelledby]="'dialogo-titulo-' + id"
      (close)="abierto.set(false)"
      (click)="clicEnFondo($event)"
    >
      <div class="dialogo__cuerpo">
        <span class="dialogo__icono"><app-icono [nombre]="icono()" /></span>
        <h2 [id]="'dialogo-titulo-' + id">{{ titulo() }}</h2>
        <div class="dialogo__texto"><ng-content /></div>
        <div class="dialogo__acciones">
          <button type="button" class="boton" (click)="abierto.set(false)">
            {{ textoCancelar() }}
          </button>
          <button
            type="button"
            class="boton"
            [class.boton--primario]="tono() !== 'peligro'"
            [class.boton--peligro-solido]="tono() === 'peligro'"
            (click)="aceptar()"
          >
            {{ textoConfirmar() }}
          </button>
        </div>
      </div>
    </dialog>
  `,
})
export class Dialogo {
  private static siguiente = 0;
  protected readonly id = ++Dialogo.siguiente;

  readonly abierto = model(false);
  readonly titulo = input.required<string>();
  readonly icono = input<NombreIcono>('check');
  readonly tono = input<'normal' | 'peligro'>('normal');
  readonly textoConfirmar = input('Aceptar');
  readonly textoCancelar = input('Cancelar');
  readonly confirmar = output<void>();

  private readonly dialogo = viewChild.required<ElementRef<HTMLDialogElement>>('dialogo');

  constructor() {
    effect(() => {
      const d = this.dialogo().nativeElement;
      if (this.abierto() && !d.open) d.showModal();
      if (!this.abierto() && d.open) d.close();
    });
  }

  protected aceptar(): void {
    this.confirmar.emit();
    this.abierto.set(false);
  }

  /** El clic en el ::backdrop llega al propio <dialog>. */
  protected clicEnFondo(evento: MouseEvent): void {
    if (evento.target === this.dialogo().nativeElement) this.abierto.set(false);
  }
}
