// Tipos de los contratos que consume el frontend.
// Identity: contracts/openapi/identity.v1.yaml · Gateway: gateway/graphql/src/schema.js

export type Rol = 'ADMIN' | 'ORGANIZADOR' | 'INSPECTOR' | 'CIUDADANO';

/** Respuesta de POST /api/v1/sessions. */
export interface TokenDto {
  accessToken: string;
  tokenType: string;
  /** Segundos de vida del JWT (600). */
  expiresIn: number;
  sessionId: string;
  sessionExpiresAt: string;
}

/** Respuesta de GET /api/v1/users/me. */
export interface Usuario {
  id: string;
  ci: string;
  firstNames: string;
  lastNames: string;
  fullName: string;
  email: string;
  phone: string;
  role: Rol;
  status: string;
  organizerId?: string | null;
}

/** Cuerpo de POST /api/v1/users. */
export interface RegistroBody {
  ci: string;
  ciComplement: string | null;
  firstNames: string;
  lastNames: string;
  email: string;
  phone: string;
  password: string;
}

/** Cuerpo de error común de los servicios (ADR de errores). */
export interface CuerpoError {
  code: string;
  message: string;
  details?: { field: string; issue: string }[];
  traceId?: string;
}

export type EstadoTurno = 'EN_ESPERA' | 'ADMITIDO' | 'LIBERADO';

export interface Turno {
  id: string;
  usuarioId: string;
  puestoId: string;
  /** Número de llegada (FIFO); no es la cantidad de personas delante. */
  posicion: number;
  estado: EstadoTurno;
}

export interface Producto {
  id: string;
  name: string;
  sku: string;
  stock: number;
  unitPrice: number;
}

export interface EstadoIdentity {
  status: string;
  database: string;
  signingKey: string | null;
}

export interface Overview {
  identity: EstadoIdentity;
  inventory: Producto[];
  queue: Turno[];
}
