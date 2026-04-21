// PatchTenantsDialog — the contract-dense piece of the P6 web surface.
//
// Operator workflow:
//   1. Dialog opens pre-populated with the server's current
//      `tenants_allowed` set.
//   2. Operator chooses `all` or a comma-separated list of org UUIDs.
//   3. The dialog computes a **client-side is-narrowing diff** (mirrors
//      `is_narrowing` in `server/src/platform/mcp_servers/patch_tenants.rs`)
//      and renders a prominent warning when the PATCH will trigger the
//      cascade: "this narrows the set; the server will revoke every
//      grant descending from ARs requested by a now-excluded org".
//   4. Submit → server runs the cascade + returns the blast-radius
//      summary → dialog shows "revoked N grants across M orgs" on
//      success.

"use client";

import { useState } from "react";

import { ApiErrorAlert } from "@/app/components/ApiErrorAlert";
import {
  cascadeSummary,
  isNarrowing,
  type PatchWire,
  type ServerSummary,
  type TenantSetWire,
} from "@/lib/api/mcp-servers";

import { patchTenantsAction, type ActionError } from "./actions";

type Status =
  | { kind: "idle" }
  | { kind: "success"; summary: ReturnType<typeof cascadeSummary>; raw: PatchWire }
  | { kind: "error"; err: ActionError };

function parseTenantsSpec(spec: string): TenantSetWire | string {
  const trimmed = spec.trim();
  if (trimmed.toLowerCase() === "all") {
    return { mode: "all" };
  }
  const orgs: string[] = [];
  for (const token of trimmed.split(",")) {
    const t = token.trim();
    if (t.length === 0) continue;
    // Shallow UUID shape check (hex + dashes); server re-validates.
    if (!/^[0-9a-fA-F-]{36}$/.test(t)) {
      return `"${t}" is not a valid UUID`;
    }
    orgs.push(t);
  }
  return { mode: "only", orgs };
}

function currentSpec(t: TenantSetWire): string {
  return t.mode === "all" ? "all" : t.orgs.join(", ");
}

export function PatchTenantsDialog({
  server,
  onClose,
}: {
  server: ServerSummary;
  onClose: () => void;
}) {
  const [spec, setSpec] = useState(currentSpec(server.tenantsAllowed));
  const [pending, setPending] = useState(false);
  const [status, setStatus] = useState<Status>({ kind: "idle" });

  const parsed = parseTenantsSpec(spec);
  const specError = typeof parsed === "string" ? parsed : null;
  const nextTenants = typeof parsed === "string" ? null : parsed;
  const willNarrow = nextTenants
    ? isNarrowing(server.tenantsAllowed, nextTenants)
    : false;
  const isNoop =
    nextTenants !== null &&
    JSON.stringify(nextTenants) === JSON.stringify(server.tenantsAllowed);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!nextTenants) return;
    setPending(true);
    setStatus({ kind: "idle" });
    const r = await patchTenantsAction({
      mcpServerId: server.id,
      tenantsAllowed: nextTenants,
    });
    setPending(false);
    if (r.ok) {
      setStatus({
        kind: "success",
        summary: cascadeSummary(r.value),
        raw: r.value,
      });
    } else {
      setStatus({ kind: "error", err: r });
    }
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
    >
      <div className="w-full max-w-lg rounded border border-white/10 bg-neutral-900 p-4">
        <header className="mb-3 flex items-center justify-between">
          <h2 className="text-base font-medium">
            Patch tenants on{" "}
            <span className="font-mono text-sm">{server.displayName}</span>
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="rounded px-2 py-1 text-sm opacity-70 hover:bg-white/10"
          >
            Close
          </button>
        </header>

        {status.kind === "success" ? (
          <SuccessPanel
            summary={status.summary}
            mcpServerId={status.raw.mcp_server_id}
            onClose={onClose}
          />
        ) : (
          <form onSubmit={onSubmit} className="space-y-3">
            <div>
              <label className="block text-sm opacity-80">
                New <span className="font-mono">tenants_allowed</span>
              </label>
              <input
                className="mt-1 w-full rounded border border-white/10 bg-black/20 px-3 py-2 font-mono text-xs"
                value={spec}
                onChange={(e) => setSpec(e.target.value)}
                placeholder="all or uuid1,uuid2,..."
                required
              />
              <div className="mt-1 text-xs opacity-60">
                <span className="font-mono">all</span> or a comma-separated
                list of org UUIDs. Current:{" "}
                <span className="font-mono">
                  {currentSpec(server.tenantsAllowed)}
                </span>
                .
              </div>
              {specError ? (
                <div className="mt-1 text-xs text-amber-400">{specError}</div>
              ) : null}
            </div>

            {willNarrow ? (
              <div className="rounded border border-amber-500/40 bg-amber-500/10 p-3 text-xs">
                <strong>Narrowing detected.</strong> Submitting will revoke
                every grant descending from an Auth Request requested by an
                org now outside the allowed set. This is cascade-irreversible
                (grants carry <span className="font-mono">revoked_at</span>;
                replays are M7b work).
              </div>
            ) : null}

            {isNoop && !specError ? (
              <div className="rounded border border-white/10 p-2 text-xs opacity-70">
                No change — this PATCH would be a no-op.
              </div>
            ) : null}

            <div className="flex gap-2">
              <button
                type="submit"
                disabled={
                  pending || specError !== null || nextTenants === null || isNoop
                }
                className="rounded bg-white/10 px-3 py-1.5 text-sm hover:bg-white/15 disabled:opacity-40"
              >
                {pending ? "Patching…" : willNarrow ? "Confirm narrow" : "Apply"}
              </button>
              <button
                type="button"
                onClick={onClose}
                className="rounded border border-white/20 px-3 py-1.5 text-sm hover:bg-white/10"
              >
                Cancel
              </button>
            </div>

            {status.kind === "error" ? (
              <ApiErrorAlert error={status.err} />
            ) : null}
          </form>
        )}
      </div>
    </div>
  );
}

function SuccessPanel({
  summary,
  mcpServerId,
  onClose,
}: {
  summary: ReturnType<typeof cascadeSummary>;
  mcpServerId: string;
  onClose: () => void;
}) {
  return (
    <div className="space-y-3">
      <div
        role="status"
        className="rounded border border-emerald-500/40 bg-emerald-500/10 p-3 text-sm"
      >
        Patched{" "}
        <span className="font-mono text-xs">{mcpServerId}</span>.
      </div>
      <div className="rounded border border-white/10 p-2 text-xs">
        <div>
          Cascade: revoked <strong>{summary.grantCount}</strong> grant(s) across{" "}
          <strong>{summary.arCount}</strong> Auth Request(s) covering{" "}
          <strong>{summary.orgCount}</strong> org(s).
        </div>
        {!summary.isNarrowing ? (
          <div className="mt-1 opacity-60">
            (PATCH did not narrow the set — no revocations.)
          </div>
        ) : null}
      </div>
      <div className="flex justify-end">
        <button
          type="button"
          onClick={onClose}
          className="rounded bg-white/10 px-3 py-1.5 text-sm hover:bg-white/15"
        >
          Close
        </button>
      </div>
    </div>
  );
}
