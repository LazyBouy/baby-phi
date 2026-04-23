// Wire translators for the Agents endpoints (M4/P4 list + M4/P5 edit).
//
// Mirrors `server/src/handlers/agents.rs`. The roster (list) payload
// is thin phi-governance data. The edit endpoints surface phi-core
// types (`AgentProfile`, `ExecutionLimits`) via `Record<string, unknown>`
// — the web tier never re-specifies phi-core field layouts per the
// leverage-checklist Q3 ("no phi-core imports on the web tier").

const API_BASE = process.env.PHI_API_URL ?? "http://127.0.0.1:8080";

export type AgentKindWire = "human" | "llm";
export type AgentRoleWire =
  | "executive"
  | "admin"
  | "member"
  | "intern"
  | "contract"
  | "system";

export type ExecutionLimitsSourceWire = "inherit" | "override";

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

/** Opaque pass-through for `phi_core::ExecutionLimits` serde shape. */
export type ExecutionLimitsWire = Record<string, unknown>;

export type CreateAgentBody = {
  display_name: string;
  kind: AgentKindWire;
  role?: AgentRoleWire | null;
  /** Opaque phi-core blueprint — full `phi_core::AgentProfile` shape. */
  blueprint: Record<string, unknown>;
  parallelize: number;
  initial_execution_limits_override?: ExecutionLimitsWire | null;
};

export type CreateAgentResponseWire = {
  agent_id: string;
  owning_org_id: string;
  inbox_id: string;
  outbox_id: string;
  profile_id: string | null;
  default_grant_ids: string[];
  execution_limits_override_id: string | null;
  audit_event_id: string;
};

/** The three ExecutionLimits patch modes — externally-tagged. */
export type ExecutionLimitsPatchWire =
  | { unchanged: null }
  | { revert: null }
  | { set: ExecutionLimitsWire };

export type UpdateAgentProfileBody = {
  display_name?: string | null;
  parallelize?: number | null;
  /** Opaque phi-core blueprint, full replacement. */
  blueprint?: Record<string, unknown> | null;
  execution_limits?: ExecutionLimitsPatchWire;
};

export type UpdateAgentProfileResponseWire = {
  agent_id: string;
  audit_event_id: string | null;
  execution_limits_source: ExecutionLimitsSourceWire;
};

export type RevertLimitsResponseWire = {
  agent_id: string;
  execution_limits_source: ExecutionLimitsSourceWire;
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

// ---- List (M4/P4) --------------------------------------------------------

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

// ---- Create (M4/P5) ------------------------------------------------------

export async function createAgentApi(
  headers: Headers,
  orgId: string,
  body: CreateAgentBody,
): Promise<ApiResult<CreateAgentResponseWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/orgs/${encodeURIComponent(orgId)}/agents`,
    {
      method: "POST",
      headers: { "content-type": "application/json", ...headers },
      body: JSON.stringify(body),
      cache: "no-store",
    },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as CreateAgentResponseWire };
}

// ---- Update profile (M4/P5) ---------------------------------------------

export async function updateAgentProfileApi(
  headers: Headers,
  agentId: string,
  body: UpdateAgentProfileBody,
): Promise<ApiResult<UpdateAgentProfileResponseWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/agents/${encodeURIComponent(agentId)}/profile`,
    {
      method: "PATCH",
      headers: { "content-type": "application/json", ...headers },
      body: JSON.stringify(body),
      cache: "no-store",
    },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return {
    ok: true,
    value: (await res.json()) as UpdateAgentProfileResponseWire,
  };
}

// ---- Revert ExecutionLimits override (M4/P5) ----------------------------

export async function revertExecutionLimitsOverrideApi(
  headers: Headers,
  agentId: string,
): Promise<ApiResult<RevertLimitsResponseWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/agents/${encodeURIComponent(agentId)}/execution-limits-override`,
    { method: "DELETE", headers, cache: "no-store" },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as RevertLimitsResponseWire };
}
