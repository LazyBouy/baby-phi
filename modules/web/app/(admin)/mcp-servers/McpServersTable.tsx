// MCP-servers table. Each row exposes a PatchTenants dialog
// (narrowing-aware) and an Archive button.

"use client";

import { useState } from "react";

import { ApiErrorAlert } from "@/app/components/ApiErrorAlert";
import type { ServerSummary } from "@/lib/api/mcp-servers";

import { PatchTenantsDialog } from "./PatchTenantsDialog";
import { archiveServerAction, type ActionError } from "./actions";

export function McpServersTable({ servers }: { servers: ServerSummary[] }) {
  const [pending, setPending] = useState<string | null>(null);
  const [error, setError] = useState<ActionError | null>(null);
  const [dialogFor, setDialogFor] = useState<ServerSummary | null>(null);

  if (servers.length === 0) {
    return (
      <div className="rounded border border-dashed border-white/15 p-6 text-sm opacity-70">
        No MCP servers registered. Use <span className="font-mono">Register
        an MCP server</span> below to bind your first external service.
      </div>
    );
  }

  async function onArchive(id: string) {
    setPending(id);
    setError(null);
    const r = await archiveServerAction({ mcpServerId: id });
    setPending(null);
    if (!r.ok) setError(r);
  }

  return (
    <div className="space-y-3">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-white/10 text-left opacity-70">
            <th className="py-2 pr-4">Display name</th>
            <th className="py-2 pr-4">Kind</th>
            <th className="py-2 pr-4">Endpoint</th>
            <th className="py-2 pr-4">Tenants</th>
            <th className="py-2 pr-4">Secret ref</th>
            <th className="py-2 pr-4">Status</th>
            <th className="py-2 pr-4">Actions</th>
          </tr>
        </thead>
        <tbody>
          {servers.map((s) => {
            const isArchived = s.archivedAt !== null;
            const tenantsLabel =
              s.tenantsAllowed.mode === "all"
                ? "all"
                : `only: ${s.tenantsAllowed.orgs.length}`;
            return (
              <tr
                key={s.id}
                className={`border-b border-white/5 ${
                  isArchived ? "opacity-50" : ""
                }`}
              >
                <td className="py-2 pr-4">{s.displayName}</td>
                <td className="py-2 pr-4 font-mono text-xs">{s.kind}</td>
                <td className="py-2 pr-4 font-mono text-xs">{s.endpoint}</td>
                <td className="py-2 pr-4 font-mono text-xs">{tenantsLabel}</td>
                <td className="py-2 pr-4 font-mono text-xs">
                  {s.secretRef ?? "—"}
                </td>
                <td className="py-2 pr-4">
                  {isArchived ? "archived" : s.status}
                </td>
                <td className="py-2 pr-4 space-x-2">
                  {isArchived ? (
                    <span className="text-xs opacity-50">—</span>
                  ) : (
                    <>
                      <button
                        type="button"
                        onClick={() => setDialogFor(s)}
                        className="rounded border border-white/20 px-2 py-1 text-xs hover:bg-white/10"
                      >
                        Patch tenants
                      </button>
                      <button
                        type="button"
                        onClick={() => onArchive(s.id)}
                        disabled={pending === s.id}
                        className="rounded border border-white/20 px-2 py-1 text-xs hover:bg-white/10 disabled:opacity-40"
                      >
                        {pending === s.id ? "Archiving…" : "Archive"}
                      </button>
                    </>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {error ? <ApiErrorAlert error={error} /> : null}
      {dialogFor ? (
        <PatchTenantsDialog
          server={dialogFor}
          onClose={() => setDialogFor(null)}
        />
      ) : null}
    </div>
  );
}
