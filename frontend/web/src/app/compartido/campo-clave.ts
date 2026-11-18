import { ChangeDetectionStrategy, Component, ElementRef, inject, signal } from '@angular/core';
import { Icono } from './icono';

/**
 * Envuelve un input de contraseña y le agrega el botón para verla u ocultarla.
 * Uso: <app-campo-clave><input type="password" formControlName="password" /></app-campo-clave>
 */
@Component({
  selector: 'app-campo-clave',
  imports: [Icono],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'clave' },
  template: `
    <ng-content />
    <button
      type="button"
      class="clave__ver"
      [attr.aria-label]="visible() ? 'Ocultar contraseña' : 'Mostrar contraseña'"
      [attr.aria-pressed]="visible()"
      [title]="visible() ? 'Ocultar contraseña' : 'Mostrar contraseña'"
      (click)="alternar()"
    >
      <app-icono [nombre]="visible() ? 'ojo-tachado' : 'ojo'" />
    </button>
  `,
})
export class CampoClave {
  private readonly host = inject<ElementRef<HTMLElement>>(ElementRef);
  protected readonly visible = signal(false);

  protected alternar(): void {
    const input = this.host.nativeElement.querySelector('input');
    if (!input) return;
    this.visible.update((v) => !v);
    input.type = this.visible() ? 'text' : 'password';
    // El cursor sigue donde estaba: se puede seguir escribiendo sin otro clic.
    input.focus();
  }
}
