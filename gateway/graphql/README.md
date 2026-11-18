# GraphQL Gateway

Gateway GraphQL for the SmartFila microservices.

## Query

```graphql
query {
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
```

The gateway combines data from Identity, Inventory and Queue.

## Validate a session over gRPC

```graphql
query {
  validateSession(sessionId: "11111111-1111-1111-1111-111111111111") {
    active
    status
    errorCode
    sessionId
    userId
    roles
    organizerId
    expiresAt
  }
}
```

The gateway calls `IdentityService.ValidateSession` at
`identity:50051` with a configurable deadline:

```bash
GRPC_DEADLINE_MS=200
```
