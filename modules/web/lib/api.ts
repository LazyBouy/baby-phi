// Server-side API client for the phi HTTP surface.
//
// `API_BASE` is read from `PHI_API_URL` (same env var the CLI uses)
// or falls back to localhost:8080. All functions here run server-side
// (in Next.js Server Components or Server Actions); they never ship to
// the browser.

const API_BASE = process.env.PHI_API_URL ?? "http://127.0.0.1:8080";

// ---- Health probes (M0) ----------------------------------------------------

export type HealthProbe =
  | { reachable: true; live: unknown; ready: unknown }
  | { reachable: false; error: string };

async function probe(path: string): Promise<unknown> {
  const res = await fetch(`${API_BASE}${path}`, { cache: "no-store" });
  return res.json();
}

export async function getHealth(): Promise<HealthProbe> {
  try {
    const [live, ready] = await Promise.all([
      probe("/healthz/live"),
      probe("/healthz/ready"),
    ]);
    return { reachable: true, live, ready };
  } catch (err) {
    return {
      reachable: false,
      error: err instanceof Error ? err.message : String(err),
    };
  }
}

// ---- Bootstrap endpoints (M1/P6) ------------------------------------------

export type BootstrapStatus =
  | { claimed: true; adminAgentId: string }
  | { claimed: false; awaitingCredential: boolean };

export type BootstrapStatusResult =
  | { ok: true; status: BootstrapStatus }
  | { ok: false; error: string };

type StatusWire = {
  claimed: boolean;
  admin_agent_id?: string;
  awaiting_credential?: boolean;
};

/**
 * Pure wire → domain translator for `GET /api/v0/bootstrap/status`.
 * Exported so unit tests can exercise the parse without standing up a
 * mock HTTP server.
 */
export function parseStatusBody(body: StatusWire): BootstrapStatus {
  if (body.claimed && typeof body.admin_agent_id === "string") {
    return { claimed: true, adminAgentId: body.admin_agent_id };
  }
  return {
    claimed: false,
    awaitingCredential: body.awaiting_credential ?? true,
  };
}

export async function getBootstrapStatus(): Promise<BootstrapStatusResult> {
  try {
    const res = await fetch(`${API_BASE}/api/v0/bootstrap/status`, {
      cache: "no-store",
    });
    if (!res.ok) {
      return { ok: false, error: `status HTTP ${res.status}` };
    }
    return { ok: true, status: parseStatusBody(await res.json()) };
  } catch (err) {
    return {
      ok: false,
      error: err instanceof Error ? err.message : String(err),
    };
  }
}

export type ChannelKind = "slack" | "email" | "web";

export type ClaimPayload = {
  bootstrapCredential: string;
  displayName: string;
  channelKind: ChannelKind;
  channelHandle: string;
};

export type ClaimSuccess = {
  humanAgentId: string;
  inboxId: string;
  outboxId: string;
  grantId: string;
  bootstrapAuthRequestId: string;
  auditEventId: string;
};

export type ClaimResult =
  | { ok: true; success: ClaimSuccess; setCookie: string | null }
  | {
      ok: false;
      code: string;
      message: string;
      httpStatus: number;
    };

type ClaimSuccessWire = {
  human_agent_id: string;
  inbox_id: string;
  outbox_id: string;
  grant_id: string;
  bootstrap_auth_request_id: string;
  audit_event_id: string;
};

type ApiErrorWire = {
  code: string;
  message: string;
};

/** Pure wire → domain translator for the 201 claim payload. */
export function parseClaimSuccess(body: ClaimSuccessWire): ClaimSuccess {
  return {
    humanAgentId: body.human_agent_id,
    inboxId: body.inbox_id,
    outboxId: body.outbox_id,
    grantId: body.grant_id,
    bootstrapAuthRequestId: body.bootstrap_auth_request_id,
    auditEventId: body.audit_event_id,
  };
}

/**
 * Extract the JWT value from a `Set-Cookie` header for the
 * `phi_kernel_session` cookie. Returns `null` when the header is absent
 * or doesn't carry that cookie.
 */
export function extractSessionJwt(
  setCookie: string | null,
  cookieName: string = "phi_kernel_session",
): string | null {
  if (!setCookie) return null;
  const escaped = cookieName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = setCookie.match(new RegExp(`${escaped}=([^;]+)`));
  return match ? match[1] : null;
}

/**
 * POST /api/v0/bootstrap/claim.
 *
 * Returns the parsed success payload + the raw `Set-Cookie` header value
 * so the caller (a Server Action) can forward it to the browser response.
 * On error, returns the `{code, message}` envelope exactly as the server
 * produced it.
 */
export async function postBootstrapClaim(
  payload: ClaimPayload,
): Promise<ClaimResult> {
  const res = await fetch(`${API_BASE}/api/v0/bootstrap/claim`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      bootstrap_credential: payload.bootstrapCredential,
      display_name: payload.displayName,
      channel: { kind: payload.channelKind, handle: payload.channelHandle },
    }),
    cache: "no-store",
  });

  if (res.status === 201) {
    const body = (await res.json()) as ClaimSuccessWire;
    return {
      ok: true,
      setCookie: res.headers.get("set-cookie"),
      success: parseClaimSuccess(body),
    };
  }

  // Try to decode the stable error envelope.
  try {
    const err = (await res.json()) as ApiErrorWire;
    return {
      ok: false,
      httpStatus: res.status,
      code: err.code ?? "UNKNOWN",
      message: err.message ?? `HTTP ${res.status}`,
    };
  } catch {
    return {
      ok: false,
      httpStatus: res.status,
      code: "UNKNOWN",
      message: `HTTP ${res.status}`,
    };
  }
}
