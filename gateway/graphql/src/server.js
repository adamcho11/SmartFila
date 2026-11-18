import express from "express";
import { createYoga } from "graphql-yoga";
import { createGatewaySchema } from "./schema.js";

const identityUrl = process.env.IDENTITY_BASE_URL || "http://localhost:8081";
const inventoryUrl = process.env.INVENTORY_BASE_URL || "http://localhost:8082";
const queueUrl = process.env.QUEUE_BASE_URL || "http://localhost:8083";
const identityGrpcTarget = process.env.IDENTITY_GRPC_TARGET || "localhost:50051";
const identityGrpcDeadlineMs = process.env.GRPC_DEADLINE_MS || "200";
const port = Number(process.env.PORT || 8085);

const app = express();

app.get("/health", (_request, response) => {
  response.json({ status: "UP" });
});

const yoga = createYoga({
  schema: createGatewaySchema({
    identityUrl,
    identityGrpcTarget,
    identityGrpcDeadlineMs,
    inventoryUrl,
    queueUrl
  }),
  graphqlEndpoint: "/graphql",
  landingPage: true
});

app.use(yoga.graphqlEndpoint, yoga);

app.listen(port, "0.0.0.0", () => {
  console.log(`SmartFila GraphQL gateway listening on port ${port}`);
});
