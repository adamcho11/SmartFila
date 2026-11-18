import { ChangeDetectionStrategy, Component, inject, input, signal } from '@angular/core';
import { NonNullableFormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { Router, RouterLink } from '@angular/router';
import { CampoClave } from '../../compartido/campo-clave';
import { Icono } from '../../compartido/icono';
import { PortadaAcceso } from '../../compartido/portada-acceso';
import { AuthService } from '../../core/auth.service';
import { ErrorVista, leerError } from '../../core/errores';

@Component({
  selector: 'app-login',
  imports: [ReactiveFormsModule, RouterLink, PortadaAcceso, Icono, CampoClave],
  templateUrl: './login.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class LoginPagina {
  private readonly auth = inject(AuthService);
  private readonly router = inject(Router);

  /** Query params (?motivo=vencida&volver=/turno). */
  readonly motivo = input<string>();
  readonly volver = input<string>();

  protected readonly enviando = signal(false);
  protected readonly error = signal<ErrorVista | null>(null);

  protected readonly form = inject(NonNullableFormBuilder).group({
    // Se aceptan espacios alrededor (C.I. pegado) y se recortan al enviar.
    ci: ['', [Validators.required, Validators.pattern(/^\s*\d{5,10}\s*$/)]],
    complemento: ['', Validators.pattern(/^\s*[0-9A-Za-z]{2}\s*$/)],
    password: ['', Validators.required],
  });

  protected async ingresar(): Promise<void> {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }
    const { ci, complemento, password } = this.form.getRawValue();
    this.enviando.set(true);
    this.error.set(null);
    try {
      await this.auth.iniciarSesion(
        ci.trim(),
        complemento.trim() ? complemento.trim().toUpperCase() : null,
        password,
      );
      await this.router.navigateByUrl(rutaSegura(this.volver()));
    } catch (err) {
      this.error.set(leerError(err));
    } finally {
      this.enviando.set(false);
    }
  }
}

/** Solo rutas internas: evita usar ?volver= para salir del sitio. */
function rutaSegura(ruta: string | undefined): string {
  return ruta && ruta.startsWith('/') && !ruta.startsWith('//') ? ruta : '/panel';
}
