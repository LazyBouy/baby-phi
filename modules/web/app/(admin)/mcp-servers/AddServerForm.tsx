// Register-MCP-server form. Operator supplies display_name, kind,
// endpoint (phi-core transport arg), optional secret_ref. Widening/
// narrowing tenants_allowed happens via PatchTenantsDialog on each
// existing row, not here — registration always starts with
// `{mode: "all"}` or the explicit list supplied below.

"use client";

import { useState } from "react";

import { ApiErrorAlert } from "@/app/components/ApiErrorAlert";

import { registerServerAction, type ActionError } from "./actions";

type Status =
  | { kind: "idle" }
  | {
      kind: "success";
      mcpServerId: string;
      auditEventId: string;
    }
  | { kind: "error"; err: ActionError };

const KIND_OPTIONS: ReadonlyArray<{ value: string; label: string }> = [
  { value: "mcp", label: "MCP (phi-core McpClient)" },
  { value: "open_api", label: "OpenAPI (reserved)" },
  { value: "webhook", label: "Webhook (reserved)" },
  { value: "other", label: "Other (reserved)" },
];

export function AddServerForm() {
  const [displayName, setDisplayName] = useState("");
  const [kind, setKind] = useState("mcp");
  const [endpoint, setEndpoint] = useState("");
  const [secretRef, setSecretRef] = useState("");
  const [pending, setPending] = useState(false);
  const [status, setStatus] = useState<Status>({ kind: "idle" });

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setPending(true);
    setStatus({ kind: "idle" });
    const r = await registerServerAction({
      displayName,
      kind,
      endpoint,
      secretRef: secretRef.trim().length === 0 ? null : secretRef.trim(),
      tenantsAllowed: { mode: "all" },
    });
    setPending(false);
    if (r.ok) {
      setStatus({
        kind: "success",
        mcpServerId: r.value.mcp_server_id,
        auditEventId: r.value.audit_event_id,
      });
      setDisplayName("");
      setEndpoint("");
      setSecretRef("");
    } else {
      setStatus({ kind: "error", err: r });
    }
  }

  return (
    <form onSubmit={onSubmit} className="space-y-4">
      <div>
        <label className="block text-sm opacity-80">Display name</label>
        <input
          className="mt-1 w-full rounded border border-white/10 bg-black/20 px-3 py-2 text-sm"
          value={displayName}
          onChange={(e) => setDisplayName(e.target.value)}
          placeholder="memory-mcp"
          required
        />
      </div>

      <div>
        <label className="block text-sm opacity-80">Kind</label>
        <select
          className="mt-1 w-full rounded border border-white/10 bg-black/20 px-3 py-2 text-sm"
          value={kind}
          onChange={(e) => setKind(e.target.value)}
        >
          {KIND_OPTIONS.map((k) => (
            <option key={k.value} value={k.value}>
              {k.label}
            </option>
          ))}
        </select>
        <div className="mt-1 text-xs opacity-60">
          Only <span className="font-mono">mcp</span> is wired in M2; the
          others are reserved for later milestones.
        </div>
      </div>

      <div>
        <label className="block text-sm opacity-80">Endpoint</label>
        <input
          className="mt-1 w-full rounded border border-white/10 bg-black/20 px-3 py-2 font-mono text-xs"
          value={endpoint}
          onChange={(e) => setEndpoint(e.target.value)}
          placeholder="stdio:///usr/local/bin/memory-mcp"
          required
        />
        <div className="mt-1 text-xs opacity-60">
          Passed verbatim to phi-core&apos;s{" "}
          <span className="font-mono">McpClient::connect_stdio</span> (for
          <span className="font-mono"> stdio:///cmd args…</span>) or
          <span className="font-mono"> connect_http</span> (for
          <span className="font-mono"> http[s]://…</span>).
        </div>
      </div>

      <div>
        <label className="block text-sm opacity-80">
          Vault secret_ref <span className="opacity-60">(optional)</span>
        </label>
        <input
          className="mt-1 w-full rounded border border-white/10 bg-black/20 px-3 py-2 font-mono text-sm"
          value={secretRef}
          onChange={(e) => setSecretRef(e.target.value)}
          placeholder="mcp-memory-key"
          pattern="[a-z0-9]+(-[a-z0-9]+)*"
          title="must already exist in the vault (see the Credentials Vault page)"
        />
        <div className="mt-1 text-xs opacity-60">
          Leave blank for services that require no authentication.
        </div>
      </div>

      <button
        type="submit"
        disabled={
          pending || displayName.length === 0 || endpoint.length === 0
        }
        className="rounded bg-white/10 px-4 py-2 text-sm font-medium hover:bg-white/15 disabled:opacity-40"
      >
        {pending ? "Registering…" : "Register MCP server"}
      </button>

      {status.kind === "error" ? <ApiErrorAlert error={status.err} /> : null}
      {status.kind === "success" ? (
        <div
          role="status"
          className="rounded border border-emerald-500/40 bg-emerald-500/10 p-3 text-sm"
        >
          Registered MCP server{" "}
          <span className="font-mono text-xs">{status.mcpServerId}</span> —
          audit <span className="font-mono text-xs">{status.auditEventId}</span>
        </div>
      ) : null}
    </form>
  );
}
