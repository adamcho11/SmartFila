import { createSchema } from "graphql-yoga";
import { fetchIdentity, fetchInventory, fetchQueue } from "./clients.js";

export function createGatewaySchema({ identityUrl, inventoryUrl, queueUrl }) {
  return createSchema({
    typeDefs: /* GraphQL */ `
      type IdentityStatus {
        status: String!
        database: String!
        signingKey: String
      }

      type Product {
        id: ID!
        name: String!
        sku: String!
        stock: Int!
        unitPrice: Float!
      }

      enum TurnoEstado {
        EN_ESPERA
        ADMITIDO
        LIBERADO
      }

      type Turno {
        id: ID!
        usuarioId: ID!
        puestoId: String!
        posicion: Int!
        estado: TurnoEstado!
      }

      type SmartFilaOverview {
        identity: IdentityStatus!
        inventory: [Product!]!
        queue: [Turno!]!
      }

      type Query {
        overview: SmartFilaOverview!
      }
    `,
    resolvers: {
      Query: {
        overview: async () => {
          const [identity, inventory, queue] = await Promise.all([
            fetchIdentity(identityUrl),
            fetchInventory(inventoryUrl),
            fetchQueue(queueUrl)
          ]);

          return { identity, inventory, queue };
        }
      }
    }
  });
}
