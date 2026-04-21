// Wire translators for the model-providers endpoints (M2/P5).
//
// Mirrors `server/src/handlers/platform_model_providers.rs`. The
// persisted `config` field is phi-core's `ModelConfig` verbatim, so
// the web tier never has to know about per-provider field layouts —
// it ships whatever JSON shape the operator typed/pasted.

const API_BASE = process.env.BABY_PHI_API_URL ?? "http://127.0.0.1:8080";
const BASE_PATH = "/api/v0/platform/model-providers";
const KINDS_PATH = "/api/v0/platform/provider-kinds";

// ---- Wire shapes (server DTOs) --------------------------------------------

export type TenantSetWire =
  | { mode: "all" }
  | { mode: "only"; orgs: string[] };

export type ProviderSummaryWire = {
  id: string;
  // phi-core ModelConfig — accepted verbatim (`config.id`,
  // `config.name`, `config.api`, etc.). We expose it as `unknown`
  // because consuming components only read a handful of fields.
  config: Record<string, unknown>;
  secret_ref: string;
  tenants_allowed: TenantSetWire;
  status: string;
  archived_at: string | null;
  created_at: string;
};

export type ListWire = { providers: ProviderSummaryWire[] };

export type RegisterBody = {
  config: Record<string, unknown>;
  secret_ref: string;
  tenants_allowed?: TenantSetWire;
};

export type RegisterWire = {
  provider_id: string;
  auth_request_id: string;
  audit_event_id: string;
};

export type ArchiveWire = {
  provider_id: string;
  audit_event_id: string;
};

export type ProviderKindsWire = { kinds: string[] };

export type ApiErrorWire = { code: string; message: string };

// ---- Domain shapes --------------------------------------------------------

export type ProviderSummary = {
  id: string;
  modelId: string;
  modelName: string;
  providerKind: string;
  providerLabel: string; // e.g. "anthropic"
  baseUrl: string;
  secretRef: string;
  tenantsAllowed: TenantSetWire;
  status: string;
  archivedAt: string | null;
  createdAt: string;
};

export type ApiResult<T> =
  | { ok: true; value: T }
  | { ok: false; httpStatus: number; code: string; message: string };

// ---- Pure translators -----------------------------------------------------

function strOr(val: unknown, fallback: string): string {
  return typeof val === "string" ? val : fallback;
}

export function parseProviderSummary(w: ProviderSummaryWire): ProviderSummary {
  return {
    id: w.id,
    modelId: strOr(w.config["id"], "(unknown)"),
    modelName: strOr(w.config["name"], "(unknown)"),
    providerKind: strOr(w.config["api"], "(unknown)"),
    providerLabel: strOr(w.config["provider"], "(unknown)"),
    baseUrl: strOr(w.config["base_url"], "(unknown)"),
    secretRef: w.secret_ref,
    tenantsAllowed: w.tenants_allowed,
    status: w.status,
    archivedAt: w.archived_at,
    createdAt: w.created_at,
  };
}

export function parseListBody(w: ListWire): ProviderSummary[] {
  return w.providers.map(parseProviderSummary);
}

// ---- HTTP (server-side only) ----------------------------------------------

type Headers = Record<string, string>;

async function readError(res: Response): Promise<ApiErrorWire> {
  try {
    return (await res.json()) as ApiErrorWire;
  } catch {
    return { code: "UNKNOWN", message: `HTTP ${res.status}` };
  }
}

export async function listProvidersApi(
  headers: Headers,
  includeArchived: boolean,
): Promise<ApiResult<ProviderSummary[]>> {
  const q = includeArchived ? "?include_archived=true" : "";
  const res = await fetch(`${API_BASE}${BASE_PATH}${q}`, {
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

export async function registerProviderApi(
  headers: Headers,
  body: RegisterBody,
): Promise<ApiResult<RegisterWire>> {
  const res = await fetch(`${API_BASE}${BASE_PATH}`, {
    method: "POST",
    headers: { ...headers, "content-type": "application/json" },
    body: JSON.stringify(body),
    cache: "no-store",
  });
  if (res.status !== 201) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as RegisterWire };
}

export async function archiveProviderApi(
  headers: Headers,
  id: string,
): Promise<ApiResult<ArchiveWire>> {
  const res = await fetch(
    `${API_BASE}${BASE_PATH}/${encodeURIComponent(id)}/archive`,
    {
      method: "POST",
      headers,
      cache: "no-store",
    },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as ArchiveWire };
}

export async function listProviderKindsApi(
  headers: Headers,
): Promise<ApiResult<string[]>> {
  const res = await fetch(`${API_BASE}${KINDS_PATH}`, {
    headers,
    cache: "no-store",
  });
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  const body = (await res.json()) as ProviderKindsWire;
  return { ok: true, value: body.kinds };
}
