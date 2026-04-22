// Register-provider form. The operator pastes a JSON
// `phi_core::ModelConfig` (same shape they'd put in a phi-core
// `AgentConfig.provider` section) + picks a vault slug for the API
// key. Pre-populated templates for the most common providers are
// served as textarea placeholders, not hard-coded defaults — phi-core
// evolves the ModelConfig shape independently and phi shouldn't
// pin a particular set of fields.

"use client";

import { useEffect, useState } from "react";

import { ApiErrorAlert } from "@/app/components/ApiErrorAlert";

import {
  fetchProviderKindsAction,
  registerProviderAction,
  type ActionError,
} from "./actions";

const PLACEHOLDER = `{
  "id": "claude-sonnet-4-20250514",
  "name": "Claude Sonnet 4",
  "api": "anthropic_messages",
  "provider": "anthropic",
  "base_url": "https://api.anthropic.com",
  "reasoning": false,
  "context_window": 200000,
  "max_tokens": 8192
}`;

type Status =
  | { kind: "idle" }
  | { kind: "success"; providerId: string; auditEventId: string }
  | { kind: "error"; err: ActionError };

export function AddProviderForm() {
  const [configJson, setConfigJson] = useState("");
  const [secretRef, setSecretRef] = useState("");
  const [kinds, setKinds] = useState<string[] | null>(null);
  const [pending, setPending] = useState(false);
  const [status, setStatus] = useState<Status>({ kind: "idle" });

  useEffect(() => {
    (async () => {
      const r = await fetchProviderKindsAction();
      if (r.ok) setKinds(r.value);
    })();
  }, []);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setPending(true);
    setStatus({ kind: "idle" });
    const r = await registerProviderAction({
      configJson,
      secretRef,
      tenantsAllowed: { mode: "all" },
    });
    setPending(false);
    if (r.ok) {
      setStatus({
        kind: "success",
        providerId: r.value.provider_id,
        auditEventId: r.value.audit_event_id,
      });
      setConfigJson("");
      setSecretRef("");
    } else {
      setStatus({ kind: "error", err: r });
    }
  }

  return (
    <form onSubmit={onSubmit} className="space-y-4">
      {kinds ? (
        <div className="rounded border border-white/10 bg-black/20 p-3 text-xs">
          <span className="opacity-70">Supported API kinds (phi-core):</span>
          <div className="mt-1 flex flex-wrap gap-2 font-mono">
            {kinds.map((k) => (
              <span
                key={k}
                className="rounded bg-white/5 px-2 py-0.5"
              >
                {k}
              </span>
            ))}
          </div>
        </div>
      ) : null}

      <div>
        <label className="block text-sm opacity-80">ModelConfig (JSON)</label>
        <textarea
          className="mt-1 w-full rounded border border-white/10 bg-black/20 px-3 py-2 font-mono text-xs"
          rows={10}
          value={configJson}
          onChange={(e) => setConfigJson(e.target.value)}
          placeholder={PLACEHOLDER}
          required
        />
        <div className="mt-1 text-xs opacity-60">
          Same shape as phi-core&apos;s <span className="font-mono">ModelConfig</span>.
          Leave <span className="font-mono">api_key</span> unset — the server
          always scrubs it; the real key lives in the vault.
        </div>
      </div>

      <div>
        <label className="block text-sm opacity-80">Vault secret_ref</label>
        <input
          className="mt-1 w-full rounded border border-white/10 bg-black/20 px-3 py-2 font-mono text-sm"
          value={secretRef}
          onChange={(e) => setSecretRef(e.target.value)}
          placeholder="anthropic-api-key"
          required
          pattern="[a-z0-9]+(-[a-z0-9]+)*"
          title="must already exist in the vault (see the Credentials Vault page)"
        />
      </div>

      <button
        type="submit"
        disabled={pending || configJson.length === 0 || secretRef.length === 0}
        className="rounded bg-white/10 px-4 py-2 text-sm font-medium hover:bg-white/15 disabled:opacity-40"
      >
        {pending ? "Registering…" : "Register provider"}
      </button>

      {status.kind === "error" ? <ApiErrorAlert error={status.err} /> : null}
      {status.kind === "success" ? (
        <div
          role="status"
          className="rounded border border-emerald-500/40 bg-emerald-500/10 p-3 text-sm"
        >
          Registered provider{" "}
          <span className="font-mono text-xs">{status.providerId}</span> — audit{" "}
          <span className="font-mono text-xs">{status.auditEventId}</span>
        </div>
      ) : null}
    </form>
  );
}
