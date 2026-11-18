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
