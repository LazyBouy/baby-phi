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

// ---- Dashboard wire shapes (M3/P5) ---------------------------------------
//
// The dashboard endpoint **deliberately strips** phi-core-wrapping
// fields from the payload (see the Q1/Q2/Q3 pre-audit pinned in
// `server/src/platform/orgs/dashboard.rs`). Nothing in the shapes
// below carries phi-core types — keeping the polling contract
// decoupled from phi-core schema evolution.

export type ViewerRoleWire = "admin" | "project_lead" | "member" | "none";

export type OrganizationDashboardHeaderWire = {
  id: string;
  display_name: string;
  vision?: string | null;
  mission?: string | null;
  consent_policy: ConsentPolicyWire;
};

export type ViewerContextWire = {
  agent_id: string;
  role: ViewerRoleWire;
  can_admin_manage: boolean;
};

export type AgentsSummaryWire = {
  total: number;
  human: number;
  llm: number;
};

export type ProjectsSummaryWire = {
  active: number;
  shape_a: number;
  shape_b: number;
};

export type TokenBudgetViewWire = {
  used: number;
  total: number;
  pool_id: string;
};

export type RecentEventSummaryWire = {
  id: string;
  kind: string;
  actor: string | null;
  timestamp: string;
  summary: string;
};

export type EmptyStateCtaCardsWire = {
  add_agent?: string | null;
  create_project?: string | null;
  adopt_template?: string | null;
  configure_system_agents?: string | null;
};

export type DashboardSummaryWire = {
  org: OrganizationDashboardHeaderWire;
  viewer: ViewerContextWire;
  agents_summary: AgentsSummaryWire;
  projects_summary: ProjectsSummaryWire;
  pending_auth_requests_count: number;
  alerted_events_24h: number;
  token_budget: TokenBudgetViewWire;
  recent_events: RecentEventSummaryWire[];
  templates_adopted: TemplateKindWire[];
  cta_cards: EmptyStateCtaCardsWire;
  welcome_banner?: string | null;
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

export async function dashboardOrgApi(
  headers: Headers,
  id: string,
): Promise<ApiResult<DashboardSummaryWire>> {
  const res = await fetch(
    `${API_BASE}${PATH}/${encodeURIComponent(id)}/dashboard`,
    {
      method: "GET",
      headers,
      cache: "no-store",
    },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as DashboardSummaryWire };
}
