// Permission Check preview panel for M5 page 14 (First Session Launch,
// R3). Renders the server-side 6-step Permission Check trace that
// `POST /api/v0/orgs/:org_id/projects/:pid/sessions/preview` returns.
//
// Server Component — the trace is fully materialised at render time,
// no client-side state. At M5/P1 this ships as a presentation-only
// primitive; the HTTP handler that populates the trace lands at
// M5/P4 (decision D5 — server-side preview).
//
// The 7-step (0-6) shape matches `domain::permissions::check::run`
// from M1; the exhaustive step enumeration guards against a future
// Permission Check refactor silently dropping a step.

export type PermissionCheckStep = {
  step: 0 | 1 | 2 | 3 | 4 | 5 | 6;
  label: string;
  outcome: "pass" | "fail" | "skipped";
  detail?: string;
};

export type PermissionCheckPreview = {
  steps: PermissionCheckStep[];
  /// Overall verdict — true iff every non-`skipped` step passed.
  granted: boolean;
  /// When `granted = false`, the first failing step's zero-based
  /// index — surfaces as `PERMISSION_CHECK_FAILED_AT_STEP_<N>` on
  /// the server stable-code table.
  failed_at_step?: number;
};

const STEP_LABELS: Record<number, string> = {
  0: "owner identity resolved",
  1: "resource exists",
  2: "grant chain traversable",
  3: "action permitted",
  4: "tenant set match",
  5: "consent satisfied",
  6: "authority template active",
};

export function PermissionCheckPreviewPanel({
  preview,
}: {
  preview: PermissionCheckPreview;
}) {
  return (
    <section
      aria-labelledby="permission-check-preview-title"
      className="rounded border border-gray-500/40 p-4 text-sm"
    >
      <header className="mb-3 flex items-center justify-between">
        <h3
          id="permission-check-preview-title"
          className="font-semibold uppercase tracking-wider opacity-60"
        >
          permission check
        </h3>
        <span
          className={
            preview.granted
              ? "text-green-400"
              : "text-red-400"
          }
        >
          {preview.granted ? "grant" : "deny"}
          {!preview.granted && preview.failed_at_step !== undefined
            ? ` · step ${preview.failed_at_step}`
            : null}
        </span>
      </header>
      <ol className="space-y-1 font-mono text-xs">
        {preview.steps.map((s) => (
          <li key={s.step} className="flex gap-3">
            <span className="opacity-40">step {s.step}</span>
            <span className="flex-1">
              {s.label || STEP_LABELS[s.step]}
            </span>
            <span
              className={
                s.outcome === "pass"
                  ? "text-green-400"
                  : s.outcome === "fail"
                    ? "text-red-400"
                    : "opacity-40"
              }
            >
              {s.outcome}
            </span>
          </li>
        ))}
      </ol>
    </section>
  );
}
