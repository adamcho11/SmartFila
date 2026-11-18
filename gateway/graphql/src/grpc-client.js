import grpc from "@grpc/grpc-js";
import protoLoader from "@grpc/proto-loader";
import path from "node:path";
import { fileURLToPath } from "node:url";

const moduleDirectory = path.dirname(fileURLToPath(import.meta.url));
const protoPath = process.env.IDENTITY_PROTO_PATH ||
  path.resolve(moduleDirectory, "../contracts/identity.proto");

let loadedPackage;

export function loadIdentityClient(target) {
  if (!loadedPackage) {
    const packageDefinition = protoLoader.loadSync(protoPath, {
      keepCase: true,
      longs: String,
      enums: String,
      defaults: true,
      oneofs: true
    });
    loadedPackage = grpc.loadPackageDefinition(packageDefinition);
  }

  const definition = loadedPackage.smartfila.identity.v1;
  return new definition.IdentityService(target, grpc.credentials.createInsecure());
}

export function validateIdentitySession({
  target,
  sessionId,
  deadlineMs = 200,
  correlationId
}) {
  const client = loadIdentityClient(target);
  const deadline = new Date(Date.now() + normalizeDeadline(deadlineMs));
  const metadata = new grpc.Metadata();

  if (correlationId) {
    metadata.set("x-correlation-id", correlationId);
  }

  return new Promise((resolve) => {
    client.validateSession(
      { session_id: sessionId },
      metadata,
      { deadline },
      (error, response) => {
        if (error) {
          resolve({
            active: false,
            status: grpcStatusName(error.code),
            errorCode: error.details || error.message,
            sessionId: null,
            userId: null,
            roles: [],
            organizerId: null,
            expiresAt: null
          });
          return;
        }

        resolve({
          active: true,
          status: "OK",
          errorCode: null,
          sessionId: response.session_id,
          userId: response.user_id,
          roles: response.roles || [],
          organizerId: response.organizer_id || null,
          expiresAt: response.expires_at?.seconds
            ? new Date(Number(response.expires_at.seconds) * 1000).toISOString()
            : null
        });
      }
    );
  });
}

export function normalizeDeadline(deadlineMs) {
  const value = Number(deadlineMs);
  if (!Number.isFinite(value) || value <= 0) {
    throw new Error("GRPC_DEADLINE_MS must be a positive number");
  }
  return value;
}

export function grpcStatusName(code) {
  return grpc.status[code] ? grpc.status[code] : "UNKNOWN";
}
