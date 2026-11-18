import { Routes } from '@angular/router';
import { requiereSesion, soloSinSesion } from './core/guardias';

export const routes: Routes = [
  { path: '', pathMatch: 'full', redirectTo: 'panel' },
  {
    path: 'login',
    title: 'Ingresar · SmartFila',
    canActivate: [soloSinSesion],
    loadComponent: () => import('./paginas/login/login').then((m) => m.LoginPagina),
  },
  {
    path: 'registro',
    title: 'Crear cuenta · SmartFila',
    canActivate: [soloSinSesion],
    loadComponent: () => import('./paginas/registro/registro').then((m) => m.RegistroPagina),
  },
  {
    path: 'panel',
    title: 'Panel · SmartFila',
    canActivate: [requiereSesion],
    loadComponent: () => import('./paginas/panel/panel').then((m) => m.PanelPagina),
  },
  {
    path: 'turno',
    title: 'Mi turno · SmartFila',
    canActivate: [requiereSesion],
    loadComponent: () => import('./paginas/turno/turno').then((m) => m.TurnoPagina),
  },
  { path: '**', redirectTo: 'panel' },
];
