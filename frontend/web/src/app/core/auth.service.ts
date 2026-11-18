import { HttpClient } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';
import { Router } from '@angular/router';
import { firstValueFrom } from 'rxjs';
import { RegistroBody, TokenDto, Usuario } from './modelos';

const CLAVE = 'smartfila.sesion';

interface SesionGuardada {
  token: string;
  /** Instante (ms) en que vence el JWT. */
  venceEn: number;
  usuario: Usuario | null;
}

function leerGuardada(): SesionGuardada | null {
  try {
    const crudo = sessionStorage.getItem(CLAVE);
    if (!crudo) return null;
    const sesion = JSON.parse(crudo) as SesionGuardada;
    return sesion.venceEn > Date.now() ? sesion : null;
  } catch {
    return null;
  }
}

/**
 * Sesión del usuario contra Identity (REST por NGINX).
 * El JWT se guarda en sessionStorage: dura lo que la pestaña y no se comparte
 * entre pestañas, en línea con la sesión única.
 */
@Injectable({ providedIn: 'root' })
export class AuthService {
  private readonly http = inject(HttpClient);
  private readonly router = inject(Router);
  private readonly sesion = signal<SesionGuardada | null>(leerGuardada());
  private temporizador: ReturnType<typeof setTimeout> | undefined;

  readonly usuario = computed(() => this.sesion()?.usuario ?? null);
  readonly conSesion = computed(() => this.sesion() !== null);

  constructor() {
    const actual = this.sesion();
    if (actual) this.programarVencimiento(actual.venceEn);
  }

  /** JWT vigente o null. */
  token(): string | null {
    const s = this.sesion();
    return s && s.venceEn > Date.now() ? s.token : null;
  }

  async iniciarSesion(ci: string, ciComplement: string | null, password: string): Promise<void> {
    const t = await firstValueFrom(
      this.http.post<TokenDto>('/api/v1/sessions', { ci, ciComplement, password }),
    );
    // Se guarda el token antes de pedir /me para que el interceptor lo envíe.
    this.guardar({ token: t.accessToken, venceEn: Date.now() + t.expiresIn * 1000, usuario: null });
    try {
      const usuario = await firstValueFrom(this.http.get<Usuario>('/api/v1/users/me'));
      this.guardar({ ...this.sesion()!, usuario });
    } catch (err) {
      this.limpiar();
      throw err;
    }
  }

  async registrar(datos: RegistroBody): Promise<void> {
    await firstValueFrom(this.http.post<Usuario>('/api/v1/users', datos));
    await this.iniciarSesion(datos.ci, datos.ciComplement, datos.password);
  }

  async cerrarSesion(): Promise<void> {
    if (this.token()) {
      try {
        await firstValueFrom(this.http.delete('/api/v1/sessions/current'));
      } catch {
        // Si la sesión ya no existe en Identity, igual se limpia la local.
      }
    }
    this.limpiar();
    await this.router.navigate(['/login']);
  }

  /** El JWT venció o Identity respondió 401: se vuelve al login. */
  sesionVencida(): void {
    if (!this.sesion()) return;
    this.limpiar();
    void this.router.navigate(['/login'], { queryParams: { motivo: 'vencida' } });
  }

  private guardar(sesion: SesionGuardada): void {
    this.sesion.set(sesion);
    try {
      sessionStorage.setItem(CLAVE, JSON.stringify(sesion));
    } catch {
      // Sin almacenamiento (modo privado): la sesión vive solo en memoria.
    }
    this.programarVencimiento(sesion.venceEn);
  }

  private limpiar(): void {
    clearTimeout(this.temporizador);
    this.sesion.set(null);
    try {
      sessionStorage.removeItem(CLAVE);
    } catch {
      // nada que limpiar
    }
  }

  private programarVencimiento(venceEn: number): void {
    clearTimeout(this.temporizador);
    this.temporizador = setTimeout(() => this.sesionVencida(), Math.max(0, venceEn - Date.now()));
  }
}
