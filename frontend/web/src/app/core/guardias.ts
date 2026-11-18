import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';
import { AuthService } from './auth.service';

/** Rutas que necesitan sesión: sin JWT vigente se va al login y luego se vuelve. */
export const requiereSesion: CanActivateFn = (_ruta, estado) => {
  if (inject(AuthService).token()) return true;
  return inject(Router).createUrlTree(['/login'], { queryParams: { volver: estado.url } });
};

/** Login y registro: quien ya tiene sesión va directo al panel. */
export const soloSinSesion: CanActivateFn = () =>
  inject(AuthService).token() ? inject(Router).createUrlTree(['/panel']) : true;
