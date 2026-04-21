// Wire translators for the MCP-servers endpoints (M2/P6).
//
// Mirrors `server/src/handlers/platform_mcp_servers.rs`. The persisted
// `endpoint` is phi-core's transport argument verbatim
// (`stdio:///cmd args…` or `http[s]://…`); the web tier never parses
// it — the server's health-probe is the only code that inspects the
// scheme.

const API_BASE = process.env.BABY_PHI_API_URL ?? "http://127.0.0.1:8080";
const BASE_PATH = "/api/v0/platform/mcp-servers";

// ---- Wire shapes (server DTOs) --------------------------------------------

export type TenantSetWire =
  | { mode: "all" }
  | { mode: "only"; orgs: string[] };

export type ServerSummaryWire = {
  id: string;
  display_name: string;
  kind: string;
  endpoint: string;
  secret_ref: string | null;
  tenants_allowed: TenantSetWire;
  status: string;
  archived_at: string | null;
  created_at: string;
};

export type ListWire = { servers: ServerSummaryWire[] };

export type RegisterBody = {
  display_name: string;
  kind: string;
  endpoint: string;
  secret_ref?: string | null;
  tenants_allowed?: TenantSetWire;
};

export type RegisterWire = {
  mcp_server_id: string;
  auth_request_id: string;
  audit_event_id: string;
};

export type ArchiveWire = {
  mcp_server_id: string;
  audit_event_id: string;
};

export type TenantRevocationWire = {
  org: string;
  auth_request: string;
  revoked_grants: string[];
};

export type PatchWire = {
  mcp_server_id: string;
  cascade: TenantRevocationWire[];
  audit_event_id: string | null;
};

export type ApiErrorWire = { code: string; message: string };

// ---- Domain shapes --------------------------------------------------------

export type ServerSummary = {
  id: string;
  displayName: string;
  kind: string;
  endpoint: string;
  secretRef: string | null;
  tenantsAllowed: TenantSetWire;
  status: string;
  archivedAt: string | null;
  createdAt: string;
};

export type CascadePreview = {
  orgCount: number;
  arCount: number;
  grantCount: number;
  isNarrowing: boolean;
};

export type ApiResult<T> =
  | { ok: true; value: T }
  | { ok: false; httpStatus: number; code: string; message: string };

// ---- Pure translators -----------------------------------------------------

export function parseServerSummary(w: ServerSummaryWire): ServerSummary {
  return {
    id: w.id,
    displayName: w.display_name,
    kind: w.kind,
    endpoint: w.endpoint,
    secretRef: w.secret_ref,
    tenantsAllowed: w.tenants_allowed,
    status: w.status,
    archivedAt: w.archived_at,
    createdAt: w.created_at,
  };
}

export function parseListBody(w: ListWire): ServerSummary[] {
  return w.servers.map(parseServerSummary);
}

/// Compute whether `next` is a strict subset of `current` — the client's
/// pre-flight check for "is this PATCH going to trigger a cascade?".
/// Mirrors the Rust `is_narrowing` in
/// `server/src/platform/mcp_servers/patch_tenants.rs` exactly.
export function isNarrowing(
  current: TenantSetWire,
  next: TenantSetWire,
): boolean {
  if (current.mode === "all" && next.mode === "only") return true;
  if (current.mode === "only" && next.mode === "only") {
    return current.orgs.some((o) => !next.orgs.includes(o));
  }
  return false;
}

export function cascadeSummary(w: PatchWire): CascadePreview {
  const orgs = new Set<string>();
  let grantCount = 0;
  for (const rev of w.cascade) {
    orgs.add(rev.org);
    grantCount += rev.revoked_grants.length;
  }
  return {
    orgCount: orgs.size,
    arCount: w.cascade.length,
    grantCount,
    isNarrowing: w.cascade.length > 0,
  };
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

export async function listServersApi(
  headers: Headers,
  includeArchived: boolean,
): Promise<ApiResult<ServerSummary[]>> {
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

export async function registerServerApi(
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

export async function patchTenantsApi(
  headers: Headers,
  id: string,
  tenantsAllowed: TenantSetWire,
): Promise<ApiResult<PatchWire>> {
  const res = await fetch(
    `${API_BASE}${BASE_PATH}/${encodeURIComponent(id)}/tenants`,
    {
      method: "PATCH",
      headers: { ...headers, "content-type": "application/json" },
      body: JSON.stringify({ tenants_allowed: tenantsAllowed }),
      cache: "no-store",
    },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as PatchWire };
}

export async function archiveServerApi(
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
