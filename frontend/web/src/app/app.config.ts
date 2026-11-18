import { registerLocaleData } from '@angular/common';
import { provideHttpClient, withFetch, withInterceptors } from '@angular/common/http';
import localeEsBo from '@angular/common/locales/es-BO';
import {
  ApplicationConfig,
  LOCALE_ID,
  inject,
  provideBrowserGlobalErrorListeners,
} from '@angular/core';
import { provideRouter, withComponentInputBinding, withViewTransitions } from '@angular/router';
import { InMemoryCache } from '@apollo/client';
import { provideApollo } from 'apollo-angular';
import { HttpLink } from 'apollo-angular/http';
import { routes } from './app.routes';
import { authInterceptor } from './core/auth.interceptor';

registerLocaleData(localeEsBo);

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    // View Transitions: fundido suave al cambiar de pantalla.
    provideRouter(routes, withComponentInputBinding(), withViewTransitions()),
    // Apollo usa este HttpClient, así que /graphql también lleva el JWT.
    provideHttpClient(withFetch(), withInterceptors([authInterceptor])),
    provideApollo(() => ({
      link: inject(HttpLink).create({ uri: '/graphql' }),
      cache: new InMemoryCache(),
    })),
    { provide: LOCALE_ID, useValue: 'es-BO' },
  ],
};
