import { ChangeDetectionStrategy, Component, input } from '@angular/core';

export type NombreIcono =
  | 'escudo'
  | 'puestos'
  | 'personas'
  | 'ticket'
  | 'reloj'
  | 'salir'
  | 'check'
  | 'actualizar'
  | 'flecha'
  | 'codigo'
  | 'persona'
  | 'ojo'
  | 'ojo-tachado';

/** Íconos de línea propios (24×24, trazo con currentColor). */
@Component({
  selector: 'app-icono',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'icono', 'aria-hidden': 'true' },
  template: `
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      @switch (nombre()) {
        @case ('escudo') {
          <path d="M12 3 4.5 6v5.5c0 4.6 3.1 8.2 7.5 9.5 4.4-1.3 7.5-4.9 7.5-9.5V6L12 3Z" />
          <path d="m8.8 12 2.2 2.2 4.2-4.4" />
        }
        @case ('puestos') {
          <rect x="3.5" y="3.5" width="7" height="7" rx="1.5" />
          <rect x="13.5" y="3.5" width="7" height="7" rx="1.5" />
          <rect x="3.5" y="13.5" width="7" height="7" rx="1.5" />
          <rect x="13.5" y="13.5" width="7" height="7" rx="1.5" />
        }
        @case ('personas') {
          <circle cx="9" cy="8" r="3.2" />
          <path d="M3.5 19.5c.6-3.2 2.8-5 5.5-5s4.9 1.8 5.5 5" />
          <circle cx="16.8" cy="9" r="2.4" />
          <path d="M16 14.6c2.3.1 4 1.6 4.5 4.4" />
        }
        @case ('ticket') {
          <path
            d="M4 7.5A1.5 1.5 0 0 1 5.5 6h13A1.5 1.5 0 0 1 20 7.5V10a2 2 0 0 0 0 4v2.5a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 4 16.5V14a2 2 0 0 0 0-4V7.5Z"
          />
          <path d="M14.5 6v12" stroke-dasharray="1.6 2.2" />
        }
        @case ('reloj') {
          <circle cx="12" cy="12" r="8.5" />
          <path d="M12 7.5V12l3 2" />
        }
        @case ('salir') {
          <path d="M14 4.5h3.5A1.5 1.5 0 0 1 19 6v12a1.5 1.5 0 0 1-1.5 1.5H14" />
          <path d="M10 8.5 6.5 12l3.5 3.5M6.5 12H15" />
        }
        @case ('check') {
          <path d="m5 12.5 4.5 4.5L19 7.5" />
        }
        @case ('actualizar') {
          <path d="M19.5 12a7.5 7.5 0 1 1-2.2-5.3" />
          <path d="M19.5 4.5v4h-4" />
        }
        @case ('flecha') {
          <path d="M5 12h14M13 6l6 6-6 6" />
        }
        @case ('ojo') {
          <path d="M2.8 12S6.2 5.8 12 5.8 21.2 12 21.2 12 17.8 18.2 12 18.2 2.8 12 2.8 12Z" />
          <circle cx="12" cy="12" r="2.8" />
        }
        @case ('ojo-tachado') {
          <path d="M9.9 6.1A9.6 9.6 0 0 1 12 5.8c5.8 0 9.2 6.2 9.2 6.2a16 16 0 0 1-2.3 3.1" />
          <path d="M6.6 7.7C4.1 9.4 2.8 12 2.8 12s3.4 6.2 9.2 6.2c1.7 0 3.2-.5 4.4-1.2" />
          <path d="M10 10.1a2.8 2.8 0 0 0 3.9 3.9" />
          <path d="m4 4 16 16" />
        }
        @case ('persona') {
          <circle cx="12" cy="7.2" r="3.7" fill="currentColor" stroke="none" />
          <path
            d="M4.8 21c0-4.4 3.2-7.6 7.2-7.6s7.2 3.2 7.2 7.6Z"
            fill="currentColor"
            stroke="none"
          />
        }
        @case ('codigo') {
          <path d="m8.5 8-4 4 4 4M15.5 8l4 4-4 4" />
        }
      }
    </svg>
  `,
})
export class Icono {
  readonly nombre = input.required<NombreIcono>();
}
