// Wire translators for the System Agents endpoints (M5/P7 D6.2).
//
// Mirrors `server/src/handlers/system_agents.rs`. Payloads are pure
// phi-governance — the `AgentProfile` blueprint is opaque to the web
// tier (Part 1.5 Q1: 0 imports on this surface; phi-core only surfaces
// via the `profile_ref` string).

const API_BASE = process.env.PHI_API_URL ?? "http://127.0.0.1:8080";

export type SystemAgentRowWire = {
  agent_id: string;
  display_name: string;
  profile_ref?: string | null;
  parallelize: number;
  trigger?: string | null;
  effective_parallelize?: number | null;
  queue_depth?: number | null;
  last_fired_at?: string | null;
  last_error?: string | null;
  active?: boolean;
};

export type RecentEventWire = {
  agent_id: string;
  at: string;
};

export type SystemAgentsListingWire = {
  standard: SystemAgentRowWire[];
  org_specific: SystemAgentRowWire[];
  recent_events: RecentEventWire[];
};

export type TuneOutcomeWire = {
  agent_id: string;
  updated_at: string;
  audit_event_id: string | null;
};

export type AddBodyWire = {
  display_name: string;
  profile_ref: string;
  parallelize: number;
  trigger: string;
};

export type AddOutcomeWire = {
  agent_id: string;
  audit_event_id: string;
};

export type DisableOutcomeWire = {
  agent_id: string;
  disabled_at: string;
  audit_event_id: string;
  was_standard: boolean;
};

export type ArchiveOutcomeWire = {
  agent_id: string;
  archived_at: string;
  audit_event_id: string;
};

type Headers = Record<string, string>;

export type ApiErrorWire = { code: string; message: string };
export type ApiResult<T> =
  | { ok: true; value: T }
  | { ok: false; httpStatus: number; code: string; message: string };

async function readError(res: Response): Promise<ApiErrorWire> {
  try {
    return (await res.json()) as ApiErrorWire;
  } catch {
    return { code: "UNKNOWN", message: `HTTP ${res.status}` };
  }
}

export async function listSystemAgentsApi(
  headers: Headers,
  orgId: string,
): Promise<ApiResult<SystemAgentsListingWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/orgs/${encodeURIComponent(orgId)}/system-agents`,
    { method: "GET", headers, cache: "no-store" },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as SystemAgentsListingWire };
}

export async function tuneSystemAgentApi(
  headers: Headers,
  orgId: string,
  agentId: string,
  parallelize: number,
): Promise<ApiResult<TuneOutcomeWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/orgs/${encodeURIComponent(orgId)}/system-agents/${encodeURIComponent(agentId)}`,
    {
      method: "PATCH",
      headers: { "content-type": "application/json", ...headers },
      body: JSON.stringify({ parallelize }),
      cache: "no-store",
    },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as TuneOutcomeWire };
}

export async function addSystemAgentApi(
  headers: Headers,
  orgId: string,
  body: AddBodyWire,
): Promise<ApiResult<AddOutcomeWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/orgs/${encodeURIComponent(orgId)}/system-agents`,
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
  return { ok: true, value: (await res.json()) as AddOutcomeWire };
}

export async function disableSystemAgentApi(
  headers: Headers,
  orgId: string,
  agentId: string,
): Promise<ApiResult<DisableOutcomeWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/orgs/${encodeURIComponent(orgId)}/system-agents/${encodeURIComponent(agentId)}/disable`,
    {
      method: "POST",
      headers: { "content-type": "application/json", ...headers },
      body: JSON.stringify({ confirm: true }),
      cache: "no-store",
    },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as DisableOutcomeWire };
}

export async function archiveSystemAgentApi(
  headers: Headers,
  orgId: string,
  agentId: string,
): Promise<ApiResult<ArchiveOutcomeWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/orgs/${encodeURIComponent(orgId)}/system-agents/${encodeURIComponent(agentId)}/archive`,
    {
      method: "POST",
      headers: { "content-type": "application/json", ...headers },
      body: "{}",
      cache: "no-store",
    },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as ArchiveOutcomeWire };
}
