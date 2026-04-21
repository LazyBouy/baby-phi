// Reveal dialog — 3-state flow per plan §P4:
//   idle        → operator picks a slug + writes a justification
//   confirming  → explicit "reveal" click triggers the HTTP call
//   revealed    → plaintext shown; a 30 s countdown then discards it
//
// The plaintext is held in React state only; discard-on-navigation +
// countdown-zero both clear it. Base64 material is decoded locally via
// `decodeB64NoPad`.

"use client";

import { useEffect, useState } from "react";

import { ApiErrorAlert } from "@/app/components/ApiErrorAlert";
import { decodeB64NoPad, type SecretSummary } from "@/lib/api/secrets";

import { revealSecretAction, type RevealActionResult } from "./actions";

type Mode =
  | { kind: "idle" }
  | { kind: "confirming"; slug: string; justification: string }
  | { kind: "revealed"; slug: string; material: string; auditEventId: string; remainingSec: number };

const COUNTDOWN_SECONDS = 30;

export function RevealDialog({ secrets }: { secrets: SecretSummary[] }) {
  const [slug, setSlug] = useState(secrets[0]?.slug ?? "");
  const [justification, setJustification] = useState("");
  const [mode, setMode] = useState<Mode>({ kind: "idle" });
  const [error, setError] = useState<RevealActionResult | null>(null);
  const [pending, setPending] = useState(false);

  useEffect(() => {
    if (mode.kind !== "revealed") return;
    if (mode.remainingSec <= 0) {
      setMode({ kind: "idle" });
      return;
    }
    const h = setTimeout(() => {
      setMode((m) =>
        m.kind === "revealed"
          ? { ...m, remainingSec: m.remainingSec - 1 }
          : m,
      );
    }, 1000);
    return () => clearTimeout(h);
  }, [mode]);

  async function doReveal() {
    setPending(true);
    setError(null);
    const r = await revealSecretAction({ slug, justification });
    setPending(false);
    if (!r.ok) {
      setError(r);
      setMode({ kind: "idle" });
      return;
    }
    const bytes = decodeB64NoPad(r.value.material_b64);
    const material = new TextDecoder("utf-8", { fatal: false }).decode(bytes);
    setMode({
      kind: "revealed",
      slug: r.value.slug,
      material,
      auditEventId: r.value.audit_event_id,
      remainingSec: COUNTDOWN_SECONDS,
    });
    setJustification("");
  }

  if (mode.kind === "revealed") {
    return (
      <div className="space-y-3 rounded border border-amber-500/40 bg-amber-500/10 p-4">
        <div className="text-sm opacity-80">
          Plaintext for <span className="font-mono">{mode.slug}</span> (audit:{" "}
          <span className="font-mono text-xs">{mode.auditEventId}</span>)
        </div>
        <pre className="max-h-48 overflow-auto rounded bg-black/40 p-3 text-xs">
          {mode.material}
        </pre>
        <div className="flex items-center justify-between text-xs opacity-70">
          <span>Auto-discards in {mode.remainingSec}s.</span>
          <button
            type="button"
            onClick={() => setMode({ kind: "idle" })}
            className="rounded border border-white/20 px-2 py-1 text-xs hover:bg-white/10"
          >
            Discard now
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <div>
        <label className="block text-sm opacity-80">Slug</label>
        <select
          className="mt-1 w-full rounded border border-white/10 bg-black/20 px-3 py-2 font-mono text-sm"
          value={slug}
          onChange={(e) => setSlug(e.target.value)}
        >
          {secrets.map((s) => (
            <option key={s.id} value={s.slug}>
              {s.slug}
            </option>
          ))}
        </select>
      </div>
      <div>
        <label className="block text-sm opacity-80">
          Justification (required — surfaced in the audit diff)
        </label>
        <input
          className="mt-1 w-full rounded border border-white/10 bg-black/20 px-3 py-2 text-sm"
          value={justification}
          onChange={(e) => setJustification(e.target.value)}
          placeholder="rotate-downstream"
          required
        />
      </div>
      <div className="rounded border border-amber-500/40 bg-amber-500/5 p-3 text-xs opacity-80">
        Reveal is always recorded to the alert channel (Alerted audit class).
        Continue only if the justification above accurately describes the need.
      </div>
      <button
        type="button"
        onClick={doReveal}
        disabled={pending || slug.length === 0 || justification.length === 0}
        className="rounded bg-amber-500/20 px-4 py-2 text-sm font-medium hover:bg-amber-500/30 disabled:opacity-40"
      >
        {pending ? "Revealing…" : "Reveal plaintext"}
      </button>
      {error && !error.ok ? <ApiErrorAlert error={error} /> : null}
    </div>
  );
}
