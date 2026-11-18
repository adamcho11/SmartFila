import { HttpErrorResponse, HttpInterceptorFn } from '@angular/common/http';
import { inject } from '@angular/core';
import { catchError, throwError } from 'rxjs';
import { AuthService } from './auth.service';

/**
 * Agrega `Authorization: Bearer <jwt>` a las llamadas del mismo origen
 * (REST de los servicios y /graphql, todo detrás de NGINX) y cierra la sesión
 * local si el backend responde 401.
 */
export const authInterceptor: HttpInterceptorFn = (req, next) => {
  const auth = inject(AuthService);
  const mismoOrigen = req.url.startsWith('/');
  const esLogin = req.method === 'POST' && req.url === '/api/v1/sessions';
  const token = auth.token();

  const peticion =
    mismoOrigen && token && !esLogin
      ? req.clone({ setHeaders: { Authorization: `Bearer ${token}` } })
      : req;

  return next(peticion).pipe(
    catchError((err: unknown) => {
      if (err instanceof HttpErrorResponse && err.status === 401 && token && !esLogin) {
        auth.sesionVencida();
      }
      return throwError(() => err);
    }),
  );
};
