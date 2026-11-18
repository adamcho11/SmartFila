import { Injectable, inject } from '@angular/core';
import { Apollo, gql } from 'apollo-angular';
import { Observable, map } from 'rxjs';
import { Overview, Producto } from './modelos';

/** Una sola consulta que el gateway resuelve contra Identity, Inventario y Cola. */
export const CONSULTA_OVERVIEW = gql`
  query Overview {
    overview {
      identity {
        status
        database
        signingKey
      }
      inventory {
        id
        name
        sku
        stock
        unitPrice
      }
      queue {
        id
        usuarioId
        puestoId
        posicion
        estado
      }
    }
  }
`;

const CONSULTA_PUESTOS = gql`
  query Puestos {
    overview {
      inventory {
        id
        name
        sku
        stock
        unitPrice
      }
    }
  }
`;

/** Lecturas que pasan por el gateway GraphQL (ADR-003: el navegador no orquesta). */
@Injectable({ providedIn: 'root' })
export class GatewayService {
  private readonly apollo = inject(Apollo);

  overview(): Observable<Overview> {
    return this.apollo
      .query<{ overview: Overview }>({ query: CONSULTA_OVERVIEW, fetchPolicy: 'network-only' })
      .pipe(map((r) => exigirDatos(r.data, r.error).overview));
  }

  puestos(): Observable<Producto[]> {
    return this.apollo
      .query<{ overview: Pick<Overview, 'inventory'> }>({
        query: CONSULTA_PUESTOS,
        fetchPolicy: 'network-only',
      })
      .pipe(map((r) => exigirDatos(r.data, r.error).overview.inventory));
  }
}

function exigirDatos<T>(data: T | undefined, error: unknown): T {
  if (data) return data;
  if (error instanceof Error) throw error;
  throw new Error('El gateway no devolvió datos.');
}
