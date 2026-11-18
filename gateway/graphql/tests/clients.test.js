import test from "node:test";
import assert from "node:assert/strict";
import { fetchIdentity, fetchInventory, fetchQueue } from "../src/clients.js";
import { createGatewaySchema } from "../src/schema.js";

test("combines data shapes from all three upstreams", async (t) => {
  const originalFetch = globalThis.fetch;
  t.after(() => {
    globalThis.fetch = originalFetch;
  });

  globalThis.fetch = async (url) => {
    if (url.includes(":8081")) {
      return url.endsWith("/health")
        ? new Response(JSON.stringify({ status: "UP", database: "UP" }), { status: 200 })
        : new Response(JSON.stringify({ keys: [{ kid: "identity-1" }] }), { status: 200 });
    }
    if (url.includes(":8082")) {
      return new Response(JSON.stringify([
        { id: "p1", name: "Entrada", sku: "E-1", stock: 10, unitPrice: 25 }
      ]), { status: 200 });
    }
    return new Response(JSON.stringify([
      { id: "t1", usuarioId: "u1", puestoId: "puerta-norte", posicion: 1, estado: 0 }
    ]), { status: 200 });
  };

  const [identity, inventory, queue] = await Promise.all([
    fetchIdentity("http://identity:8081"),
    fetchInventory("http://inventory:8082"),
    fetchQueue("http://queue:8083")
  ]);

  assert.equal(identity.signingKey, "identity-1");
  assert.equal(inventory[0].sku, "E-1");
  assert.equal(queue[0].estado, "EN_ESPERA");
});

test("GraphQL overview combines all upstream resolvers", async (t) => {
  const originalFetch = globalThis.fetch;
  t.after(() => {
    globalThis.fetch = originalFetch;
  });

  globalThis.fetch = async (url) => {
    if (url.includes("/health")) {
      return new Response(JSON.stringify({ status: "UP", database: "UP" }), { status: 200 });
    }
    if (url.includes("/jwks")) {
      return new Response(JSON.stringify({ keys: [{ kid: "identity-1" }] }), { status: 200 });
    }
    if (url.includes("/api/products")) {
      return new Response(JSON.stringify([
        { id: "p1", name: "Entrada", sku: "E-1", stock: 10, unitPrice: 25 }
      ]), { status: 200 });
    }
    return new Response(JSON.stringify([
      { id: "t1", usuarioId: "u1", puestoId: "puerta-norte", posicion: 1, estado: 0 }
    ]), { status: 200 });
  };

  const schema = createGatewaySchema({
    identityUrl: "http://identity:8080",
    inventoryUrl: "http://inventory:8080",
    queueUrl: "http://queue:8080"
  });

  const result = await import("graphql").then(({ graphql }) =>
    graphql({
      schema,
      source: `{ overview { identity { status signingKey } inventory { sku stock } queue { puestoId estado } } }`
    })
  );

  assert.equal(result.errors, undefined);
  assert.equal(result.data.overview.identity.signingKey, "identity-1");
  assert.equal(result.data.overview.inventory[0].sku, "E-1");
  assert.equal(result.data.overview.queue[0].estado, "EN_ESPERA");
});
