// Wire translators for the Orgs endpoints (M3/P4).
//
// Mirrors `server/src/handlers/orgs.rs`. The persisted
// `organization.defaults_snapshot` wraps 4 phi-core types
// (`ExecutionLimits` / `ContextConfig` / `RetryConfig` /
// `AgentProfile`) — we keep those as opaque `Record<string, unknown>`
// here, exactly as the platform-defaults translator does. The web
// tier never re-specifies phi-core field layouts (leverage-checklist
// Q3: "no phi-core imports on the web tier").

const API_BASE = process.env.BABY_PHI_API_URL ?? "http://127.0.0.1:8080";
const PATH = "/api/v0/orgs";

// ---- Wire shapes -----------------------------------------------------------

export type ConsentPolicyWire = "implicit" | "one_time" | "per_session";
export type AuditClassWire = "silent" | "logged" | "alerted";
export type ChannelKindWire = "slack" | "email" | "web";
export type TemplateKindWire = "a" | "b" | "c" | "d";

/** Matches `Organization` in `domain/src/model/nodes.rs`. */
export type OrganizationWire = {
  id: string;
  display_name: string;
  vision?: string | null;
  mission?: string | null;
  consent_policy: ConsentPolicyWire;
  audit_class_default: AuditClassWire;
  authority_templates_enabled: TemplateKindWire[];
  /** Opaque phi-core wraps — `ExecutionLimits` + 3 more. */
  defaults_snapshot?: Record<string, unknown> | null;
  default_model_provider?: string | null;
  system_agents: string[];
  created_at: string;
};

export type CreateOrgBody = {
  display_name: string;
  vision?: string | null;
  mission?: string | null;
  consent_policy: ConsentPolicyWire;
  audit_class_default: AuditClassWire;
  authority_templates_enabled: TemplateKindWire[];
  /** Optional override — opaque phi-core fields. */
  defaults_snapshot_override?: Record<string, unknown> | null;
  default_model_provider?: string | null;
  ceo_display_name: string;
  ceo_channel_kind: ChannelKindWire;
  ceo_channel_handle: string;
  token_budget: number;
};

export type CreateOrgResponseWire = {
  org_id: string;
  ceo_agent_id: string;
  ceo_channel_id: string;
  ceo_inbox_id: string;
  ceo_outbox_id: string;
  ceo_grant_id: string;
  system_agent_ids: [string, string];
  token_budget_pool_id: string;
  adoption_auth_request_ids: string[];
  audit_event_ids: string[];
};

export type OrgListItemWire = {
  id: string;
  display_name: string;
  consent_policy: ConsentPolicyWire;
  authority_templates_enabled: TemplateKindWire[];
  member_count: number;
};

export type ListOrgsWire = { orgs: OrgListItemWire[] };

export type ShowOrgWire = {
  organization: OrganizationWire;
  member_count: number;
  project_count: number;
  adopted_template_count: number;
};

export type ApiErrorWire = { code: string; message: string };

export type ApiResult<T> =
  | { ok: true; value: T }
  | { ok: false; httpStatus: number; code: string; message: string };

// ---- HTTP (server-side only) ---------------------------------------------

type Headers = Record<string, string>;

async function readError(res: Response): Promise<ApiErrorWire> {
  try {
    return (await res.json()) as ApiErrorWire;
  } catch {
    return { code: "UNKNOWN", message: `HTTP ${res.status}` };
  }
}

export async function createOrgApi(
  headers: Headers,
  body: CreateOrgBody,
): Promise<ApiResult<CreateOrgResponseWire>> {
  const res = await fetch(`${API_BASE}${PATH}`, {
    method: "POST",
    headers: { "content-type": "application/json", ...headers },
    body: JSON.stringify(body),
    cache: "no-store",
  });
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as CreateOrgResponseWire };
}

export async function listOrgsApi(
  headers: Headers,
): Promise<ApiResult<ListOrgsWire>> {
  const res = await fetch(`${API_BASE}${PATH}`, {
    method: "GET",
    headers,
    cache: "no-store",
  });
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as ListOrgsWire };
}

export async function showOrgApi(
  headers: Headers,
  id: string,
): Promise<ApiResult<ShowOrgWire>> {
  const res = await fetch(`${API_BASE}${PATH}/${encodeURIComponent(id)}`, {
    method: "GET",
    headers,
    cache: "no-store",
  });
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as ShowOrgWire };
}
