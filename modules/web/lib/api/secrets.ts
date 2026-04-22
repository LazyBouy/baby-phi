// Wire translators for the credentials-vault endpoints (M2/P4).
//
// Mirrors `server/src/handlers/platform_secrets.rs`. Pure translator
// functions are exported so `__tests__/secrets.test.tsx` can exercise
// wire-shape mapping without standing up a mock server.

const API_BASE = process.env.PHI_API_URL ?? "http://127.0.0.1:8080";
const SECRETS_PATH = "/api/v0/platform/secrets";

// ---- Wire shapes ----------------------------------------------------------

export type SecretSummaryWire = {
  id: string;
  slug: string;
  custodian_id: string;
  sensitive: boolean;
  last_rotated_at: string | null;
  created_at: string;
};

export type ListWire = { secrets: SecretSummaryWire[] };

export type AddBody = {
  slug: string;
  material_b64: string;
  sensitive?: boolean;
};

export type AddWire = {
  secret_id: string;
  slug: string;
  auth_request_id: string;
  audit_event_id: string;
};

export type WriteWire = {
  secret_id: string;
  slug: string;
  audit_event_id: string;
};

export type RevealBody = { justification: string };

export type RevealWire = {
  secret_id: string;
  slug: string;
  material_b64: string;
  audit_event_id: string;
};

export type ApiErrorWire = { code: string; message: string };

// ---- Domain shapes --------------------------------------------------------

export type SecretSummary = {
  id: string;
  slug: string;
  custodianId: string;
  sensitive: boolean;
  lastRotatedAt: string | null;
  createdAt: string;
};

export type SecretOpResult<T> =
  | { ok: true; value: T }
  | { ok: false; httpStatus: number; code: string; message: string };

// ---- Pure translators (exercised by unit tests) ---------------------------

export function parseSecretSummary(w: SecretSummaryWire): SecretSummary {
  return {
    id: w.id,
    slug: w.slug,
    custodianId: w.custodian_id,
    sensitive: w.sensitive,
    lastRotatedAt: w.last_rotated_at,
    createdAt: w.created_at,
  };
}

export function parseListBody(w: ListWire): SecretSummary[] {
  return w.secrets.map(parseSecretSummary);
}

// Standard-alphabet base64 with no padding — matches the server's
// `STANDARD_NO_PAD` engine.
type Base64Globals = {
  btoa?: (s: string) => string;
  atob?: (s: string) => string;
};

export function encodeB64NoPad(bytes: Uint8Array): string {
  let s = "";
  if (typeof Buffer !== "undefined") {
    s = Buffer.from(bytes).toString("base64");
  } else {
    // Browser path — TextDecoder doesn't cover base64, so fall back to
    // btoa on a latin-1 string.
    let bin = "";
    for (const b of bytes) bin += String.fromCharCode(b);
    const g = globalThis as unknown as Base64Globals;
    if (!g.btoa) throw new Error("base64 encode: btoa not available");
    s = g.btoa(bin);
  }
  return s.replace(/=+$/u, "");
}

export function decodeB64NoPad(s: string): Uint8Array {
  const padded = s + "=".repeat((4 - (s.length % 4)) % 4);
  if (typeof Buffer !== "undefined") {
    return new Uint8Array(Buffer.from(padded, "base64"));
  }
  const g = globalThis as unknown as Base64Globals;
  if (!g.atob) throw new Error("base64 decode: atob not available");
  const bin = g.atob(padded);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

// ---- HTTP entry points (run server-side only) -----------------------------

type Headers = Record<string, string>;

async function readError(res: Response): Promise<ApiErrorWire> {
  try {
    return (await res.json()) as ApiErrorWire;
  } catch {
    return { code: "UNKNOWN", message: `HTTP ${res.status}` };
  }
}

export async function listSecretsApi(
  headers: Headers,
): Promise<SecretOpResult<SecretSummary[]>> {
  const res = await fetch(`${API_BASE}${SECRETS_PATH}`, {
    headers,
    cache: "no-store",
  });
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  const body = (await res.json()) as ListWire;
  return { ok: true, value: parseListBody(body) };
}

export async function addSecretApi(
  headers: Headers,
  body: AddBody,
): Promise<SecretOpResult<AddWire>> {
  const res = await fetch(`${API_BASE}${SECRETS_PATH}`, {
    method: "POST",
    headers: { ...headers, "content-type": "application/json" },
    body: JSON.stringify(body),
    cache: "no-store",
  });
  if (res.status !== 201) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as AddWire };
}

export async function rotateSecretApi(
  headers: Headers,
  slug: string,
  body: { material_b64: string },
): Promise<SecretOpResult<WriteWire>> {
  const res = await fetch(
    `${API_BASE}${SECRETS_PATH}/${encodeURIComponent(slug)}/rotate`,
    {
      method: "POST",
      headers: { ...headers, "content-type": "application/json" },
      body: JSON.stringify(body),
      cache: "no-store",
    },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as WriteWire };
}

export async function revealSecretApi(
  headers: Headers,
  slug: string,
  body: RevealBody,
): Promise<SecretOpResult<RevealWire>> {
  const res = await fetch(
    `${API_BASE}${SECRETS_PATH}/${encodeURIComponent(slug)}/reveal`,
    {
      method: "POST",
      headers: { ...headers, "content-type": "application/json" },
      body: JSON.stringify(body),
      cache: "no-store",
    },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as RevealWire };
}

export async function reassignCustodyApi(
  headers: Headers,
  slug: string,
  body: { new_custodian_agent_id: string },
): Promise<SecretOpResult<WriteWire>> {
  const res = await fetch(
    `${API_BASE}${SECRETS_PATH}/${encodeURIComponent(slug)}/reassign-custody`,
    {
      method: "POST",
      headers: { ...headers, "content-type": "application/json" },
      body: JSON.stringify(body),
      cache: "no-store",
    },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as WriteWire };
}
