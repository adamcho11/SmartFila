import { ChangeDetectionStrategy, Component } from '@angular/core';

/**
 * Marca de SmartFila: una fila de personas (puntos) que forma la "S" y avanza
 * hacia su puesto; la primera, en dorado, es la que ya tiene turno.
 * El mismo dibujo está en public/favicon.svg con colores fijos.
 */
@Component({
  selector: 'app-logo',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'logo', 'aria-hidden': 'true' },
  template: `
    <svg viewBox="0 0 40 40">
      <rect class="logo__fondo" width="40" height="40" rx="11" />
      <path
        class="logo__fila"
        d="M29.5 10.5H16.75a5.25 5.25 0 0 0 0 10.5h6.5a5.25 5.25 0 0 1 0 10.5H14"
      />
      <circle class="logo__frente" cx="10.5" cy="31.5" r="3.3" />
    </svg>
  `,
})
export class Logo {}
