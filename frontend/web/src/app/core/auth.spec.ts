import { HttpClient, provideHttpClient, withInterceptors } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { firstValueFrom } from 'rxjs';
import { authInterceptor } from './auth.interceptor';
import { AuthService } from './auth.service';
import { Usuario } from './modelos';

const usuario: Usuario = {
  id: 'u1',
  ci: '1234567',
  firstNames: 'Ana',
  lastNames: 'Pérez',
  fullName: 'Ana Pérez',
  email: 'ana@example.com',
  phone: '71234567',
  role: 'CIUDADANO',
  status: 'ACTIVE',
};

describe('AuthService + authInterceptor', () => {
  let auth: AuthService;
  let http: HttpClient;
  let backend: HttpTestingController;

  beforeEach(() => {
    sessionStorage.clear();
    TestBed.configureTestingModule({
      providers: [
        provideRouter([{ path: 'login', children: [] }]),
        provideHttpClient(withInterceptors([authInterceptor])),
        provideHttpClientTesting(),
      ],
    });
    auth = TestBed.inject(AuthService);
    http = TestBed.inject(HttpClient);
    backend = TestBed.inject(HttpTestingController);
  });

  afterEach(() => backend.verify());

  async function ingresar(): Promise<void> {
    const login = auth.iniciarSesion('1234567', null, 'clave-segura');
    const req = backend.expectOne('/api/v1/sessions');
    expect(req.request.headers.has('Authorization')).toBe(false);
    expect(req.request.body).toEqual({
      ci: '1234567',
      ciComplement: null,
      password: 'clave-segura',
    });
    req.flush({
      accessToken: 'jwt-de-prueba',
      tokenType: 'Bearer',
      expiresIn: 600,
      sessionId: 's1',
      sessionExpiresAt: new Date(Date.now() + 3_600_000).toISOString(),
    });
    await Promise.resolve();
    const me = backend.expectOne('/api/v1/users/me');
    expect(me.request.headers.get('Authorization')).toBe('Bearer jwt-de-prueba');
    me.flush(usuario);
    await login;
  }

  it('inicia sesión, pide /me con el JWT y guarda el usuario', async () => {
    await ingresar();
    expect(auth.usuario()?.fullName).toBe('Ana Pérez');
    expect(auth.token()).toBe('jwt-de-prueba');
    expect(sessionStorage.getItem('smartfila.sesion')).toContain('jwt-de-prueba');
  });

  it('agrega el JWT a /graphql y no a otros orígenes', async () => {
    await ingresar();
    void firstValueFrom(http.post('/graphql', {}));
    expect(backend.expectOne('/graphql').request.headers.get('Authorization')).toBe(
      'Bearer jwt-de-prueba',
    );
    void firstValueFrom(http.get('https://otro.example.com/x'));
    expect(
      backend.expectOne('https://otro.example.com/x').request.headers.has('Authorization'),
    ).toBe(false);
  });

  it('un 401 del backend cierra la sesión local', async () => {
    await ingresar();
    const peticion = firstValueFrom(http.get('/api/v1/turnos')).catch(() => 'rechazada');
    backend.expectOne('/api/v1/turnos').flush(null, { status: 401, statusText: 'Unauthorized' });
    expect(await peticion).toBe('rechazada');
    expect(auth.token()).toBeNull();
    expect(sessionStorage.getItem('smartfila.sesion')).toBeNull();
  });
});
