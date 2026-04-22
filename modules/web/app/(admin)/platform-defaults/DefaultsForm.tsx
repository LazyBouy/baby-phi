// Edit + PUT form. Exposes the phi-native fields as first-class
// controls (retention days, alert channels) and the embedded phi-core
// sections as JSON textareas — the form never assumes a particular
// phi-core field layout, so phi-core evolution doesn't require a
// web-tier change.

"use client";

import { useState } from "react";

import { ApiErrorAlert } from "@/app/components/ApiErrorAlert";
import type { PlatformDefaults } from "@/lib/api/platform-defaults";

import { putDefaultsAction, type ActionError } from "./actions";

type Status =
  | { kind: "idle" }
  | { kind: "success"; newVersion: number; auditEventId: string }
  | { kind: "error"; err: ActionError };

/** Format a JSON value for a textarea; returns `{}` on empty. */
function fmt(value: Record<string, unknown>): string {
  return JSON.stringify(value, null, 2);
}

function parseSection(raw: string): Record<string, unknown> | string {
  try {
    const v = JSON.parse(raw);
    if (typeof v !== "object" || v === null || Array.isArray(v)) {
      return "must be a JSON object";
    }
    return v as Record<string, unknown>;
  } catch (err) {
    return err instanceof Error ? err.message : String(err);
  }
}

export function DefaultsForm({
  initialDefaults,
}: {
  initialDefaults: PlatformDefaults;
}) {
  const [retentionDays, setRetentionDays] = useState(
    String(initialDefaults.defaultRetentionDays),
  );
  const [alertChannels, setAlertChannels] = useState(
    initialDefaults.defaultAlertChannels.join(", "),
  );
  const [executionLimits, setExecutionLimits] = useState(
    fmt(initialDefaults.executionLimits),
  );
  const [agentProfile, setAgentProfile] = useState(
    fmt(initialDefaults.defaultAgentProfile),
  );
  const [contextConfig, setContextConfig] = useState(
    fmt(initialDefaults.contextConfig),
  );
  const [retryConfig, setRetryConfig] = useState(
    fmt(initialDefaults.retryConfig),
  );
  const [pending, setPending] = useState(false);
  const [status, setStatus] = useState<Status>({ kind: "idle" });

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setStatus({ kind: "idle" });

    // Parse all four phi-core sections.
    const el = parseSection(executionLimits);
    const ap = parseSection(agentProfile);
    const cc = parseSection(contextConfig);
    const rc = parseSection(retryConfig);
    for (const [name, v] of [
      ["execution_limits", el],
      ["default_agent_profile", ap],
      ["context_config", cc],
      ["retry_config", rc],
    ] as const) {
      if (typeof v === "string") {
        setStatus({
          kind: "error",
          err: {
            ok: false,
            httpStatus: 400,
            code: "VALIDATION_FAILED",
            message: `${name}: ${v}`,
          },
        });
        return;
      }
    }

    const retention = Number.parseInt(retentionDays, 10);
    if (!Number.isFinite(retention) || retention < 0) {
      setStatus({
        kind: "error",
        err: {
          ok: false,
          httpStatus: 400,
          code: "VALIDATION_FAILED",
          message: "Default retention days must be a non-negative integer.",
        },
      });
      return;
    }

    const channels = alertChannels
      .split(",")
      .map((s) => s.trim())
      .filter((s) => s.length > 0);

    setPending(true);
    const r = await putDefaultsAction({
      ifVersion: initialDefaults.version,
      defaults: {
        version: initialDefaults.version,
        updatedAt: initialDefaults.updatedAt,
        defaultRetentionDays: retention,
        defaultAlertChannels: channels,
        executionLimits: el as Record<string, unknown>,
        defaultAgentProfile: ap as Record<string, unknown>,
        contextConfig: cc as Record<string, unknown>,
        retryConfig: rc as Record<string, unknown>,
      },
    });
    setPending(false);
    if (r.ok) {
      setStatus({
        kind: "success",
        newVersion: r.value.new_version,
        auditEventId: r.value.audit_event_id,
      });
    } else {
      setStatus({ kind: "error", err: r });
    }
  }

  return (
    <form onSubmit={onSubmit} className="space-y-5">
      <div className="rounded border border-white/10 bg-black/20 p-3 text-xs">
        <span className="opacity-70">
          Current version:{" "}
          <span className="font-mono">{initialDefaults.version}</span>
        </span>
        <span className="mx-2 opacity-40">·</span>
        <span className="opacity-70">
          Last updated:{" "}
          <span className="font-mono">{initialDefaults.updatedAt}</span>
        </span>
      </div>

      <SectionInput
        label="Default retention (days)"
        value={retentionDays}
        onChange={setRetentionDays}
        type="number"
        hint="Baseline Silent-tier audit-log retention for new orgs."
      />

      <SectionInput
        label="Default alert channels"
        value={alertChannels}
        onChange={setAlertChannels}
        type="text"
        hint="Comma-separated handles (emails / webhook URLs) used when an org has no override."
      />

      <SectionTextarea
        label="ExecutionLimits (phi-core)"
        value={executionLimits}
        onChange={setExecutionLimits}
        hint="Max turns / tokens / duration / cost. Shape: phi_core::context::execution::ExecutionLimits."
      />

      <SectionTextarea
        label="AgentProfile (phi-core)"
        value={agentProfile}
        onChange={setAgentProfile}
        hint="Default system prompt / thinking level / skills. Shape: phi_core::agents::profile::AgentProfile."
      />

      <SectionTextarea
        label="ContextConfig (phi-core)"
        value={contextConfig}
        onChange={setContextConfig}
        hint="Token budget + compaction strategy. Shape: phi_core::context::config::ContextConfig."
      />

      <SectionTextarea
        label="RetryConfig (phi-core)"
        value={retryConfig}
        onChange={setRetryConfig}
        hint="Exponential-backoff retry tuning. Shape: phi_core::provider::retry::RetryConfig."
      />

      <div className="flex items-center gap-3">
        <button
          type="submit"
          disabled={pending}
          className="rounded bg-white/10 px-4 py-2 text-sm font-medium hover:bg-white/15 disabled:opacity-40"
        >
          {pending
            ? "Saving…"
            : `Save (if_version=${initialDefaults.version})`}
        </button>
        <span className="text-xs opacity-60">
          Stale-write safe — server returns 409 on a version mismatch; refresh
          to pull the latest.
        </span>
      </div>

      {status.kind === "error" ? <ApiErrorAlert error={status.err} /> : null}
      {status.kind === "success" ? (
        <div
          role="status"
          className="rounded border border-emerald-500/40 bg-emerald-500/10 p-3 text-sm"
        >
          Saved — new version <span className="font-mono">{status.newVersion}</span>{" "}
          · audit{" "}
          <span className="font-mono text-xs">{status.auditEventId}</span>
        </div>
      ) : null}
    </form>
  );
}

function SectionInput({
  label,
  value,
  onChange,
  type,
  hint,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  type: "text" | "number";
  hint: string;
}) {
  return (
    <div>
      <label className="block text-sm opacity-80">{label}</label>
      <input
        className="mt-1 w-full rounded border border-white/10 bg-black/20 px-3 py-2 text-sm"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        type={type}
      />
      <div className="mt-1 text-xs opacity-60">{hint}</div>
    </div>
  );
}

function SectionTextarea({
  label,
  value,
  onChange,
  hint,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  hint: string;
}) {
  return (
    <div>
      <label className="block text-sm opacity-80">{label}</label>
      <textarea
        className="mt-1 w-full rounded border border-white/10 bg-black/20 px-3 py-2 font-mono text-xs"
        rows={8}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
      <div className="mt-1 text-xs opacity-60">{hint}</div>
    </div>
  );
}
