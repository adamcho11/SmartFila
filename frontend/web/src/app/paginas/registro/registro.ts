import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import {
  AbstractControl,
  NonNullableFormBuilder,
  ReactiveFormsModule,
  ValidationErrors,
  Validators,
} from '@angular/forms';
import { Router, RouterLink } from '@angular/router';
import { CampoClave } from '../../compartido/campo-clave';
import { Icono } from '../../compartido/icono';
import { PortadaAcceso } from '../../compartido/portada-acceso';
import { AuthService } from '../../core/auth.service';
import { ErrorVista, leerError } from '../../core/errores';

function clavesIguales(grupo: AbstractControl): ValidationErrors | null {
  const clave = grupo.get('password')?.value;
  const repetida = grupo.get('confirmacion')?.value;
  return clave && repetida && clave !== repetida ? { clavesDistintas: true } : null;
}

@Component({
  selector: 'app-registro',
  imports: [ReactiveFormsModule, RouterLink, PortadaAcceso, Icono, CampoClave],
  templateUrl: './registro.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class RegistroPagina {
  private readonly auth = inject(AuthService);
  private readonly router = inject(Router);

  protected readonly enviando = signal(false);
  protected readonly error = signal<ErrorVista | null>(null);

  // Mismas reglas que el dominio de Identity; el servidor vuelve a validar.
  protected readonly form = inject(NonNullableFormBuilder).group(
    {
      // Se aceptan espacios alrededor (C.I. pegado) y se recortan al enviar.
      ci: ['', [Validators.required, Validators.pattern(/^\s*\d{5,10}\s*$/)]],
      ciComplement: ['', Validators.pattern(/^\s*[0-9A-Za-z]{2}\s*$/)],
      firstNames: ['', [Validators.required, Validators.maxLength(100)]],
      lastNames: ['', [Validators.required, Validators.maxLength(100)]],
      email: ['', [Validators.required, Validators.email]],
      phone: ['', [Validators.required, Validators.pattern(/^[67]\d{7}$/)]],
      password: ['', [Validators.required, Validators.minLength(8), Validators.maxLength(128)]],
      confirmacion: ['', Validators.required],
    },
    { validators: clavesIguales },
  );

  /** Mensaje del servidor para un campo (details[].field). */
  protected errorServidor(campo: string): string | undefined {
    return this.error()?.campos[campo];
  }

  protected mostrarError(campo: string): boolean {
    const control = this.form.get(campo);
    return !!control && control.touched && control.invalid;
  }

  protected async registrar(): Promise<void> {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }
    const { confirmacion: _, ciComplement, ...resto } = this.form.getRawValue();
    this.enviando.set(true);
    this.error.set(null);
    try {
      await this.auth.registrar({
        ...resto,
        ci: resto.ci.trim(),
        email: resto.email.trim(),
        ciComplement: ciComplement.trim() ? ciComplement.trim().toUpperCase() : null,
      });
      await this.router.navigate(['/panel']);
    } catch (err) {
      this.error.set(leerError(err));
    } finally {
      this.enviando.set(false);
    }
  }
}
