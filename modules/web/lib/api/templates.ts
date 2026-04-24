// Wire translators for the Authority Templates endpoints (M5/P7 D5.1).
//
// Mirrors `server/src/handlers/templates.rs`. Bucketed list +
// approve / deny / adopt / revoke actions. All payloads are pure
// phi-governance — zero phi-core types in transit (see Part 1.5 Q1:
// 0 imports expected on the templates surface).

const API_BASE = process.env.PHI_API_URL ?? "http://127.0.0.1:8080";

export type TemplateKindWire = "a" | "b" | "c" | "d" | "e";

export type TemplateRowWire = {
  kind: TemplateKindWire | string;
  summary?: string;
  adoption_auth_request_id?: string | null;
  state?: string | null;
  active_grant_count?: number;
  revoked_grant_count?: number;
};

export type TemplatesListingWire = {
  pending: TemplateRowWire[];
  active: TemplateRowWire[];
  revoked: TemplateRowWire[];
  available: TemplateRowWire[];
};

export type ApproveOutcomeWire = {
  adoption_auth_request_id: string;
  new_state: string;
  audit_event_id: string;
};

export type DenyOutcomeWire = ApproveOutcomeWire;

export type AdoptOutcomeWire = {
  adoption_auth_request_id: string;
  state: string;
  audit_event_id: string;
};

export type RevokeOutcomeWire = {
  adoption_auth_request_id: string;
  grants_revoked: string[];
  grant_count_revoked: number;
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

export async function listTemplatesApi(
  headers: Headers,
  orgId: string,
): Promise<ApiResult<TemplatesListingWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/orgs/${encodeURIComponent(orgId)}/authority-templates`,
    { method: "GET", headers, cache: "no-store" },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as TemplatesListingWire };
}

async function postAction<T>(
  headers: Headers,
  orgId: string,
  kind: string,
  action: "approve" | "deny" | "adopt" | "revoke",
  body: Record<string, unknown>,
): Promise<ApiResult<T>> {
  const res = await fetch(
    `${API_BASE}/api/v0/orgs/${encodeURIComponent(orgId)}/authority-templates/${encodeURIComponent(kind)}/${action}`,
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
  return { ok: true, value: (await res.json()) as T };
}

export function approveTemplateApi(
  headers: Headers,
  orgId: string,
  kind: string,
): Promise<ApiResult<ApproveOutcomeWire>> {
  return postAction(headers, orgId, kind, "approve", {});
}

export function denyTemplateApi(
  headers: Headers,
  orgId: string,
  kind: string,
  reason: string,
): Promise<ApiResult<DenyOutcomeWire>> {
  return postAction(headers, orgId, kind, "deny", { reason });
}

export function adoptTemplateApi(
  headers: Headers,
  orgId: string,
  kind: string,
): Promise<ApiResult<AdoptOutcomeWire>> {
  return postAction(headers, orgId, kind, "adopt", {});
}

export function revokeTemplateApi(
  headers: Headers,
  orgId: string,
  kind: string,
  reason: string,
): Promise<ApiResult<RevokeOutcomeWire>> {
  return postAction(headers, orgId, kind, "revoke", { reason });
}
