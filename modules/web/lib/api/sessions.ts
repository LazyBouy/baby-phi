// Wire translators for the Sessions endpoints (M5/P7 — page 14).
//
// Mirrors `server/src/handlers/sessions.rs`. Launch + list are the
// surfaces used by the web UI; preview exposes the server-side
// Permission-Check trace. Full `SessionDetail` (show / terminate)
// carries nested phi-core types via `Record<string, unknown>` — the
// web tier never re-declares phi-core layouts.

const API_BASE = process.env.PHI_API_URL ?? "http://127.0.0.1:8080";

export type LaunchBodyWire = {
  agent_id: string;
  prompt: string;
};

export type LaunchResponseWire = {
  session_id: string;
  first_loop_id: string;
  session_started_event_id: string;
  /** Opaque Decision shape — `ok: bool, reasons: [...]`. */
  permission_check: Record<string, unknown>;
};

export type PreviewResponseWire = {
  agent_id: string;
  project_id: string;
  decision: Record<string, unknown>;
};

export type SessionHeaderWire = {
  id: string;
  started_by: string;
  governance_state: "running" | "completed" | "aborted" | "failed_launch" | string;
  started_at: string;
  ended_at: string | null;
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

export async function launchSessionApi(
  headers: Headers,
  orgId: string,
  projectId: string,
  body: LaunchBodyWire,
): Promise<ApiResult<LaunchResponseWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/orgs/${encodeURIComponent(orgId)}/projects/${encodeURIComponent(projectId)}/sessions`,
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
  return { ok: true, value: (await res.json()) as LaunchResponseWire };
}

export async function previewSessionApi(
  headers: Headers,
  orgId: string,
  projectId: string,
  agentId: string,
): Promise<ApiResult<PreviewResponseWire>> {
  const res = await fetch(
    `${API_BASE}/api/v0/orgs/${encodeURIComponent(orgId)}/projects/${encodeURIComponent(projectId)}/sessions/preview`,
    {
      method: "POST",
      headers: { "content-type": "application/json", ...headers },
      body: JSON.stringify({ agent_id: agentId }),
      cache: "no-store",
    },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as PreviewResponseWire };
}

export async function listSessionsInProjectApi(
  headers: Headers,
  projectId: string,
): Promise<ApiResult<SessionHeaderWire[]>> {
  const res = await fetch(
    `${API_BASE}/api/v0/projects/${encodeURIComponent(projectId)}/sessions`,
    { method: "GET", headers, cache: "no-store" },
  );
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  const payload = (await res.json()) as
    | SessionHeaderWire[]
    | { sessions?: SessionHeaderWire[] };
  if (Array.isArray(payload)) return { ok: true, value: payload };
  return { ok: true, value: payload.sessions ?? [] };
}
