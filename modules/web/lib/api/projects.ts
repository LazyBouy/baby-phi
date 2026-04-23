// Wire translators for the Projects endpoints (M4/P6 — create +
// approve-pending). Mirrors `server/src/handlers/projects.rs`.
//
// The wire shape carries phi governance types only (Project + OKRs +
// ProjectShape). **Zero phi-core type references** at this tier — per
// M4 plan Part 1.5 Page 10 Q3 rejection.

const API_BASE = process.env.PHI_API_URL ?? "http://127.0.0.1:8080";

export type ProjectShapeWire = "shape_a" | "shape_b";
export type AuthRequestStateWire =
  | "draft"
  | "pending"
  | "in_progress"
  | "approved"
  | "denied"
  | "partial"
  | "expired"
  | "revoked"
  | "cancelled";

export type MeasurementTypeWire = "count" | "boolean" | "percentage" | "custom";
export type ObjectiveStatusWire =
  | "draft"
  | "active"
  | "achieved"
  | "missed"
  | "cancelled";
export type KeyResultStatusWire =
  | "not_started"
  | "in_progress"
  | "achieved"
  | "missed"
  | "cancelled";

export type OkrValueWire =
  | { kind: "integer"; value: number }
  | { kind: "bool"; value: boolean }
  | { kind: "percentage"; value: number }
  | { kind: "custom"; value: unknown };

export type ObjectiveWire = {
  objective_id: string;
  name: string;
  description: string;
  status: ObjectiveStatusWire;
  owner: string;
  deadline?: string | null;
  key_result_ids?: string[];
};

export type KeyResultWire = {
  kr_id: string;
  objective_id: string;
  name: string;
  description: string;
  measurement_type: MeasurementTypeWire;
  target_value: OkrValueWire;
  current_value?: OkrValueWire | null;
  owner: string;
  deadline?: string | null;
  status: KeyResultStatusWire;
};

export type CreateProjectBody = {
  project_id: string;
  name: string;
  description?: string;
  goal?: string | null;
  shape: ProjectShapeWire;
  co_owner_org_id?: string | null;
  lead_agent_id: string;
  member_agent_ids?: string[];
  sponsor_agent_ids?: string[];
  token_budget?: number | null;
  objectives?: ObjectiveWire[];
  key_results?: KeyResultWire[];
};

export type CreateProjectResponseWire =
  | {
      outcome: "materialised";
      project_id: string;
      lead_agent_id: string;
      has_lead_edge_id: string;
      owning_org_ids: string[];
      audit_event_id: string;
    }
  | {
      outcome: "pending";
      pending_ar_id: string;
      approver_ids: [string, string];
      audit_event_id: string;
    };

export type ApprovePendingBody = {
  approver_id: string;
  approve: boolean;
};

export type ApprovePendingResponseWire =
  | { outcome: "still_pending"; ar_id: string }
  | {
      outcome: "terminal";
      ar_id: string;
      state: AuthRequestStateWire;
      project_id: string | null;
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

export async function createProjectApi(
  headers: Headers,
  orgId: string,
  body: CreateProjectBody,
): Promise<ApiResult<CreateProjectResponseWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/orgs/${encodeURIComponent(orgId)}/projects`,
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
  return { ok: true, value: (await res.json()) as CreateProjectResponseWire };
}

// ---------------------------------------------------------------------------
// Project detail (M4/P7) — GET /api/v0/projects/:id
// ---------------------------------------------------------------------------

export type ProjectStatusWire =
  | "planned"
  | "in_progress"
  | "on_hold"
  | "finished";

export type ResourceBoundariesWire = {
  allowed_model_providers?: string[];
  allowed_mcp_servers?: string[];
  allowed_skills?: string[];
  token_budget?: number | null;
};

export type ProjectWire = {
  id: string;
  name: string;
  description: string;
  goal?: string | null;
  status: ProjectStatusWire;
  shape: ProjectShapeWire;
  token_budget?: number | null;
  tokens_spent: number;
  objectives: ObjectiveWire[];
  key_results: KeyResultWire[];
  resource_boundaries?: ResourceBoundariesWire | null;
  created_at: string;
};

export type ProjectMembershipRoleWire = "lead" | "member" | "sponsor";

export type AgentKindWire = "human" | "llm";
export type AgentRoleWire =
  | "executive"
  | "admin"
  | "member"
  | "intern"
  | "contract"
  | "system";

export type RosterMemberWire = {
  agent_id: string;
  kind: AgentKindWire;
  display_name: string;
  role?: AgentRoleWire | null;
  project_role: ProjectMembershipRoleWire;
};

export type RecentSessionStubWire = {
  session_id: string;
  started_at: string;
  summary: string;
};

export type ProjectDetailWire = {
  project: ProjectWire;
  owning_org_ids: string[];
  lead_agent_id?: string | null;
  roster: RosterMemberWire[];
  recent_sessions: RecentSessionStubWire[];
};

export async function getProjectApi(
  headers: Headers,
  projectId: string,
): Promise<ApiResult<ProjectDetailWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/projects/${encodeURIComponent(projectId)}`,
    {
      method: "GET",
      headers: { ...headers },
      cache: "no-store",
    },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as ProjectDetailWire };
}

// ---------------------------------------------------------------------------
// OKR patch (M4/P7) — PATCH /api/v0/projects/:id/okrs
// ---------------------------------------------------------------------------

export type OkrPatchEntryWire =
  | { kind: "objective"; op: "create"; payload: ObjectiveWire }
  | { kind: "objective"; op: "update"; payload: ObjectiveWire }
  | { kind: "objective"; op: "delete"; objective_id: string }
  | { kind: "key_result"; op: "create"; payload: KeyResultWire }
  | { kind: "key_result"; op: "update"; payload: KeyResultWire }
  | { kind: "key_result"; op: "delete"; kr_id: string };

export type OkrPatchBody = {
  patches: OkrPatchEntryWire[];
};

export type OkrPatchResponseWire = {
  project_id: string;
  audit_event_ids: string[];
  objectives: ObjectiveWire[];
  key_results: KeyResultWire[];
};

export async function patchProjectOkrsApi(
  headers: Headers,
  projectId: string,
  body: OkrPatchBody,
): Promise<ApiResult<OkrPatchResponseWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/projects/${encodeURIComponent(projectId)}/okrs`,
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
  return { ok: true, value: (await res.json()) as OkrPatchResponseWire };
}

export async function approvePendingProjectApi(
  headers: Headers,
  arId: string,
  body: ApprovePendingBody,
): Promise<ApiResult<ApprovePendingResponseWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/projects/_pending/${encodeURIComponent(arId)}/approve`,
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
  return {
    ok: true,
    value: (await res.json()) as ApprovePendingResponseWire,
  };
}
