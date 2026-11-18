import { createSchema } from "graphql-yoga";
import { fetchIdentity, fetchInventory, fetchQueue } from "./clients.js";
import { validateIdentitySession } from "./grpc-client.js";

export function createGatewaySchema({
  identityUrl,
  identityGrpcTarget,
  identityGrpcDeadlineMs,
  inventoryUrl,
  queueUrl
}) {
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

      type SessionValidation {
        active: Boolean!
        status: String!
        errorCode: String
        sessionId: ID
        userId: ID
        roles: [String!]!
        organizerId: ID
        expiresAt: String
      }

      type Query {
        overview: SmartFilaOverview!
        validateSession(sessionId: ID!): SessionValidation!
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
        },
        validateSession: async (_parent, args, context) =>
          validateIdentitySession({
            target: identityGrpcTarget,
            sessionId: args.sessionId,
            deadlineMs: identityGrpcDeadlineMs,
            correlationId: context?.correlationId
          })
      }
    }
  });
}
