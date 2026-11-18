export async function getJson(baseUrl, path, timeoutMs = 3000) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const response = await fetch(`${baseUrl}${path}`, {
      signal: controller.signal,
      headers: { accept: "application/json" }
    });

    if (!response.ok) {
      throw new Error(`Upstream ${baseUrl}${path} returned ${response.status}`);
    }

    return await response.json();
  } finally {
    clearTimeout(timeout);
  }
}

export async function fetchIdentity(baseUrl) {
  const [health, jwks] = await Promise.all([
    getJson(baseUrl, "/health"),
    getJson(baseUrl, "/.well-known/jwks.json")
  ]);

  return {
    status: health.status,
    database: health.database,
    signingKey: jwks.keys?.[0]?.kid ?? null
  };
}

export async function fetchInventory(baseUrl) {
  const products = await getJson(baseUrl, "/api/products");
  return products.map((product) => ({
    id: product.id,
    name: product.name,
    sku: product.sku,
    stock: product.stock,
    unitPrice: product.unitPrice
  }));
}

export async function fetchQueue(baseUrl) {
  const turnos = await getJson(baseUrl, "/api/turnos");
  return turnos.map((turno) => ({
    id: turno.id,
    usuarioId: turno.usuarioId,
    puestoId: turno.puestoId,
    posicion: turno.posicion,
    estado: EstadoTurnoToGraphQL(turno.estado)
  }));
}

function EstadoTurnoToGraphQL(estado) {
  const values = ["EN_ESPERA", "ADMITIDO", "LIBERADO"];
  if (typeof estado === "number") return values[estado] ?? "EN_ESPERA";

  const normalized = String(estado).trim().toUpperCase();
  if (normalized.includes("ADMIT")) return "ADMITIDO";
  if (normalized.includes("LIBER")) return "LIBERADO";
  return "EN_ESPERA";
}
