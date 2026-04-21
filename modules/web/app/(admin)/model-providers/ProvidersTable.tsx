// Read-only model-providers table. phi-core's `ModelConfig.api`
// string surfaces as the "kind" column; the platform governance
// status (Ok / Degraded / Archived) is a separate column.

"use client";

import { useState } from "react";

import { ApiErrorAlert } from "@/app/components/ApiErrorAlert";
import type { ProviderSummary } from "@/lib/api/model-providers";

import { archiveProviderAction, type ActionError } from "./actions";

export function ProvidersTable({
  providers,
}: {
  providers: ProviderSummary[];
}) {
  const [pending, setPending] = useState<string | null>(null);
  const [error, setError] = useState<ActionError | null>(null);

  if (providers.length === 0) {
    return (
      <div className="rounded border border-dashed border-white/15 p-6 text-sm opacity-70">
        No model providers registered. Use <span className="font-mono">Register
        provider</span> below to bind your first LLM runtime.
      </div>
    );
  }

  async function onArchive(id: string) {
    setPending(id);
    setError(null);
    const r = await archiveProviderAction({ providerId: id });
    setPending(null);
    if (!r.ok) setError(r);
  }

  return (
    <div className="space-y-3">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-white/10 text-left opacity-70">
            <th className="py-2 pr-4">Provider</th>
            <th className="py-2 pr-4">Model</th>
            <th className="py-2 pr-4">API kind</th>
            <th className="py-2 pr-4">Secret ref</th>
            <th className="py-2 pr-4">Status</th>
            <th className="py-2 pr-4">Actions</th>
          </tr>
        </thead>
        <tbody>
          {providers.map((p) => {
            const isArchived = p.archivedAt !== null;
            return (
              <tr
                key={p.id}
                className={`border-b border-white/5 ${
                  isArchived ? "opacity-50" : ""
                }`}
              >
                <td className="py-2 pr-4 font-mono text-xs">{p.providerLabel}</td>
                <td className="py-2 pr-4">
                  <div className="font-medium">{p.modelName}</div>
                  <div className="font-mono text-xs opacity-70">
                    {p.modelId}
                  </div>
                </td>
                <td className="py-2 pr-4 font-mono text-xs">{p.providerKind}</td>
                <td className="py-2 pr-4 font-mono text-xs">{p.secretRef}</td>
                <td className="py-2 pr-4">
                  {isArchived ? "archived" : p.status}
                </td>
                <td className="py-2 pr-4">
                  {isArchived ? (
                    <span className="text-xs opacity-50">—</span>
                  ) : (
                    <button
                      type="button"
                      onClick={() => onArchive(p.id)}
                      disabled={pending === p.id}
                      className="rounded border border-white/20 px-2 py-1 text-xs hover:bg-white/10 disabled:opacity-40"
                    >
                      {pending === p.id ? "Archiving…" : "Archive"}
                    </button>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {error ? <ApiErrorAlert error={error} /> : null}
    </div>
  );
}
