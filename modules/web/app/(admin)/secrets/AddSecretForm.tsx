// Add-secret form. Client component because we want to keep the
// plaintext off the URL + clear the controlled input on success.

"use client";

import { useState } from "react";

import { ApiErrorAlert } from "@/app/components/ApiErrorAlert";

import { addSecretAction, type AddActionResult } from "./actions";

export function AddSecretForm() {
  const [slug, setSlug] = useState("");
  const [material, setMaterial] = useState("");
  const [sensitive, setSensitive] = useState(true);
  const [pending, setPending] = useState(false);
  const [result, setResult] = useState<AddActionResult | null>(null);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setPending(true);
    setResult(null);
    const r = await addSecretAction({ slug, material, sensitive });
    setResult(r);
    setPending(false);
    if (r.ok) {
      setSlug("");
      setMaterial("");
    }
  }

  return (
    <form onSubmit={onSubmit} className="space-y-4">
      <div>
        <label className="block text-sm opacity-80">Slug</label>
        <input
          className="mt-1 w-full rounded border border-white/10 bg-black/20 px-3 py-2 font-mono text-sm"
          value={slug}
          onChange={(e) => setSlug(e.target.value)}
          placeholder="anthropic-api-key"
          required
          pattern="[a-z0-9]+(-[a-z0-9]+)*"
          title="lowercase letters, digits, dashes (no leading/trailing dash)"
        />
      </div>
      <div>
        <label className="block text-sm opacity-80">Material</label>
        <textarea
          className="mt-1 w-full rounded border border-white/10 bg-black/20 px-3 py-2 font-mono text-sm"
          value={material}
          onChange={(e) => setMaterial(e.target.value)}
          placeholder="sk-ant-… (pasted here, never stored in this field)"
          required
          rows={3}
        />
      </div>
      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={sensitive}
          onChange={(e) => setSensitive(e.target.checked)}
        />
        Mask value in list views + audit diffs
      </label>
      <button
        type="submit"
        disabled={pending || slug.length === 0 || material.length === 0}
        className="rounded bg-white/10 px-4 py-2 text-sm font-medium hover:bg-white/15 disabled:opacity-40"
      >
        {pending ? "Adding…" : "Add secret"}
      </button>
      {result && !result.ok ? <ApiErrorAlert error={result} /> : null}
      {result && result.ok ? (
        <div
          role="status"
          className="rounded border border-emerald-500/40 bg-emerald-500/10 p-3 text-sm"
        >
          Added <span className="font-mono">{result.value.slug}</span> (audit:{" "}
          <span className="font-mono text-xs">{result.value.audit_event_id}</span>
          )
        </div>
      ) : null}
    </form>
  );
}
