import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';
import { FondoAnimado } from './compartido/fondo-animado';
import { Icono } from './compartido/icono';
import { Logo } from './compartido/logo';
import { AuthService } from './core/auth.service';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet, RouterLink, RouterLinkActive, FondoAnimado, Icono, Logo],
  templateUrl: './app.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class App {
  protected readonly auth = inject(AuthService);

  /** Iniciales para el avatar de la barra. */
  protected readonly iniciales = computed(() => {
    const u = this.auth.usuario();
    if (!u) return '';
    return `${u.firstNames.trim()[0] ?? ''}${u.lastNames.trim()[0] ?? ''}`.toUpperCase();
  });

  protected salir(): void {
    void this.auth.cerrarSesion();
  }
}
