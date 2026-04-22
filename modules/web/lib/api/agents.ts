// Wire translators for the Agents endpoints (M4/P4).
//
// Mirrors `server/src/handlers/agents.rs`. The roster payload is
// deliberately thin: phi governance fields only. `AgentProfile`
// (which wraps `phi_core::AgentProfile`) is surfaced by the page 09
// editor endpoints that land at M4/P5, not here.

const API_BASE = process.env.PHI_API_URL ?? "http://127.0.0.1:8080";

export type AgentKindWire = "human" | "llm";
export type AgentRoleWire =
  | "executive"
  | "admin"
  | "member"
  | "intern"
  | "contract"
  | "system";

export type AgentRosterItemWire = {
  id: string;
  kind: AgentKindWire;
  display_name: string;
  owning_org: string | null;
  role: AgentRoleWire | null;
  created_at: string;
};

export type ListAgentsWire = {
  org_id: string;
  agents: AgentRosterItemWire[];
};

export type ApiErrorWire = { code: string; message: string };

export type ApiResult<T> =
  | { ok: true; value: T }
  | { ok: false; httpStatus: number; code: string; message: string };

type Headers = Record<string, string>;

async function readError(res: Response): Promise<ApiErrorWire> {
  try {
    return (await res.json()) as ApiErrorWire;
  } catch {
    return { code: "UNKNOWN", message: `HTTP ${res.status}` };
  }
}

export type ListAgentsQuery = {
  role?: AgentRoleWire | null;
  search?: string | null;
};

export function buildListAgentsUrl(
  orgId: string,
  q: ListAgentsQuery | undefined,
): string {
  const url = `${API_BASE}/api/v0/orgs/${encodeURIComponent(orgId)}/agents`;
  const params = new URLSearchParams();
  if (q?.role) params.set("role", q.role);
  if (q?.search) params.set("search", q.search);
  const qs = params.toString();
  return qs.length ? `${url}?${qs}` : url;
}

export async function listAgentsApi(
  headers: Headers,
  orgId: string,
  q?: ListAgentsQuery,
): Promise<ApiResult<ListAgentsWire>> {
  const res = await fetch(buildListAgentsUrl(orgId, q), {
    method: "GET",
    headers,
    cache: "no-store",
  });
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as ListAgentsWire };
}
