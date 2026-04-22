// Wire translators for the Platform Defaults endpoints (M2/P7).
//
// Mirrors `server/src/handlers/platform_defaults.rs`. The persisted
// struct wraps four phi-core types directly
// (`ExecutionLimits` / `AgentProfile` / `ContextConfig` / `RetryConfig`),
// so we keep the embedded sections as opaque `Record<string, unknown>`
// on the wire — the web tier never re-specifies phi-core field layouts.
// A handful of top-level fields surface as first-class domain properties
// (`version`, `updated_at`, `default_retention_days`, alert channels) so
// the form can render a dedicated control for each.

const API_BASE = process.env.PHI_API_URL ?? "http://127.0.0.1:8080";
const PATH = "/api/v0/platform/defaults";

// ---- Wire shapes -----------------------------------------------------------

export type PlatformDefaultsWire = {
  singleton: number;
  execution_limits: Record<string, unknown>;
  default_agent_profile: Record<string, unknown>;
  context_config: Record<string, unknown>;
  retry_config: Record<string, unknown>;
  default_retention_days: number;
  default_alert_channels: string[];
  updated_at: string;
  version: number;
};

export type GetDefaultsWire = {
  defaults: PlatformDefaultsWire;
  persisted: boolean;
  factory: PlatformDefaultsWire;
};

export type PutDefaultsBody = {
  if_version: number;
  defaults: PlatformDefaultsWire;
};

export type PutDefaultsWire = {
  new_version: number;
  auth_request_id: string;
  audit_event_id: string;
};

export type ApiErrorWire = { code: string; message: string };

// ---- Domain shapes ---------------------------------------------------------

export type PlatformDefaults = {
  version: number;
  updatedAt: string;
  defaultRetentionDays: number;
  defaultAlertChannels: string[];
  /** Embedded phi-core `ExecutionLimits` — kept opaque. */
  executionLimits: Record<string, unknown>;
  /** Embedded phi-core `AgentProfile` — kept opaque. */
  defaultAgentProfile: Record<string, unknown>;
  /** Embedded phi-core `ContextConfig` — kept opaque. */
  contextConfig: Record<string, unknown>;
  /** Embedded phi-core `RetryConfig` — kept opaque. */
  retryConfig: Record<string, unknown>;
};

export type ApiResult<T> =
  | { ok: true; value: T }
  | { ok: false; httpStatus: number; code: string; message: string };

// ---- Pure translators ------------------------------------------------------

export function parseDefaults(w: PlatformDefaultsWire): PlatformDefaults {
  return {
    version: w.version,
    updatedAt: w.updated_at,
    defaultRetentionDays: w.default_retention_days,
    defaultAlertChannels: w.default_alert_channels,
    executionLimits: w.execution_limits,
    defaultAgentProfile: w.default_agent_profile,
    contextConfig: w.context_config,
    retryConfig: w.retry_config,
  };
}

export function toWire(d: PlatformDefaults): PlatformDefaultsWire {
  return {
    singleton: 1,
    execution_limits: d.executionLimits,
    default_agent_profile: d.defaultAgentProfile,
    context_config: d.contextConfig,
    retry_config: d.retryConfig,
    default_retention_days: d.defaultRetentionDays,
    default_alert_channels: d.defaultAlertChannels,
    updated_at: d.updatedAt,
    version: d.version,
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

export async function getDefaultsApi(
  headers: Headers,
): Promise<
  ApiResult<{
    defaults: PlatformDefaults;
    persisted: boolean;
    factory: PlatformDefaults;
  }>
> {
  const res = await fetch(`${API_BASE}${PATH}`, {
    headers,
    cache: "no-store",
  });
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  const body = (await res.json()) as GetDefaultsWire;
  return {
    ok: true,
    value: {
      defaults: parseDefaults(body.defaults),
      persisted: body.persisted,
      factory: parseDefaults(body.factory),
    },
  };
}

export async function putDefaultsApi(
  headers: Headers,
  ifVersion: number,
  defaults: PlatformDefaults,
): Promise<ApiResult<PutDefaultsWire>> {
  const body: PutDefaultsBody = {
    if_version: ifVersion,
    defaults: toWire(defaults),
  };
  const res = await fetch(`${API_BASE}${PATH}`, {
    method: "PUT",
    headers: { ...headers, "content-type": "application/json" },
    body: JSON.stringify(body),
    cache: "no-store",
  });
  if (!res.ok) {
    const err = await readError(res);
    return { ok: false, httpStatus: res.status, ...err };
  }
  return { ok: true, value: (await res.json()) as PutDefaultsWire };
}
