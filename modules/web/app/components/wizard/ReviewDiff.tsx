// Before/after review panel — used by wizard final step to preview
// the payload the operator is about to submit.
//
// Renders a two-column diff of JSON-serializable values: expected
// vs. current. Missing keys show as "—"; added keys (in current but
// not in expected) render with an "added" tag; changed values
// render both sides.
//
// Not a general-purpose diff library (mature ones exist in the npm
// ecosystem); this is a minimal shape focused on the shallow-object
// shape wizard steps produce.

"use client";

export type ReviewRow = {
  /// Field name (operator-facing).
  label: string;
  /// Expected value — typically the factory default or the previously
  /// saved draft.
  expected: unknown;
  /// Current value — what the operator typed in the wizard.
  current: unknown;
};

export type ReviewDiffProps = {
  /// Rows to render. Order is meaningful — typically matches the
  /// wizard step order so operators scan top-to-bottom.
  rows: ReviewRow[];
};

function render(value: unknown): string {
  if (value === undefined || value === null) return "—";
  if (typeof value === "string") return value.length === 0 ? "(empty)" : value;
  if (typeof value === "number" || typeof value === "boolean")
    return String(value);
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return "(unserialisable)";
  }
}

function isChanged(expected: unknown, current: unknown): boolean {
  // Cheap structural comparison via JSON normalisation. Good enough
  // for wizard payloads (shallow, JSON-serialisable); would not be
  // correct for cyclic graphs, but the wizard never passes those.
  try {
    return JSON.stringify(expected) !== JSON.stringify(current);
  } catch {
    return true;
  }
}

export function ReviewDiff({ rows }: ReviewDiffProps) {
  return (
    <div className="overflow-x-auto rounded border border-white/10">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-white/10 text-left opacity-70">
            <th className="py-2 pl-3 pr-4">Field</th>
            <th className="py-2 pr-4">Default / previous</th>
            <th className="py-2 pr-3">Your value</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row, i) => {
            const changed = isChanged(row.expected, row.current);
            return (
              <tr
                key={`${row.label}-${i}`}
                className={`border-b border-white/5 ${
                  changed ? "bg-amber-500/5" : ""
                }`}
              >
                <td className="py-2 pl-3 pr-4 font-mono text-xs">{row.label}</td>
                <td className="py-2 pr-4 font-mono text-xs opacity-60">
                  <pre className="whitespace-pre-wrap">{render(row.expected)}</pre>
                </td>
                <td className="py-2 pr-3 font-mono text-xs">
                  <pre className="whitespace-pre-wrap">{render(row.current)}</pre>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {rows.some((r) => isChanged(r.expected, r.current)) ? (
        <p className="border-t border-white/10 p-2 text-xs opacity-60">
          Highlighted rows differ from the default / previous value. Review
          before submitting — the org snapshot is <strong>non-retroactive</strong>
          (ADR-0020).
        </p>
      ) : null}
    </div>
  );
}
