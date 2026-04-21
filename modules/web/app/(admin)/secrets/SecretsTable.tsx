// Read-only table of vault entries. Plaintext is NEVER rendered —
// use <RevealDialog /> for the opt-in reveal flow.

"use client";

import type { SecretSummary } from "@/lib/api/secrets";

export function SecretsTable({ secrets }: { secrets: SecretSummary[] }) {
  if (secrets.length === 0) {
    return (
      <div className="rounded border border-dashed border-white/15 p-6 text-sm opacity-70">
        Vault is empty. Use <span className="font-mono">Add secret</span> below
        to register the first entry.
      </div>
    );
  }
  return (
    <table className="w-full text-sm">
      <thead>
        <tr className="border-b border-white/10 text-left opacity-70">
          <th className="py-2 pr-4">Slug</th>
          <th className="py-2 pr-4">Sensitive</th>
          <th className="py-2 pr-4">Last rotated</th>
          <th className="py-2 pr-4">Custodian</th>
          <th className="py-2 pr-4">Created</th>
        </tr>
      </thead>
      <tbody>
        {secrets.map((s) => (
          <tr key={s.id} className="border-b border-white/5">
            <td className="py-2 pr-4 font-mono">{s.slug}</td>
            <td className="py-2 pr-4">{s.sensitive ? "yes" : "no"}</td>
            <td className="py-2 pr-4">{s.lastRotatedAt ?? "(never)"}</td>
            <td
              className="py-2 pr-4 font-mono text-xs opacity-70"
              title={s.custodianId}
            >
              {s.custodianId.slice(0, 8)}…
            </td>
            <td className="py-2 pr-4 text-xs opacity-70">{s.createdAt}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
