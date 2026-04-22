// Wizard step container — heading + content slot + error-alert slot.
//
// Establishes the multi-step-wizard pattern M3/P4's org-creation flow
// consumes (and M4+'s project-creation wizard reuses). A step renders
// its own form inside `children`; the shell owns the visual framing,
// progress indicator, and error-alert placement so per-step code stays
// focused on the domain payload.
//
// This is baby-phi-native web UX — phi-core has no web tier.

"use client";

import { ApiErrorAlert } from "@/app/components/ApiErrorAlert";

/// Minimal error shape the wizard surfaces inline. Shares the wire
/// contract the Server Actions return via `ApiErrorAlert` (see
/// `app/components/ApiErrorAlert.tsx`), so any step can pass a
/// Rust-side `ApiError` through unchanged.
export type StepError = {
  ok: false;
  httpStatus: number;
  code: string;
  message: string;
};

export type StepShellProps = {
  /// Human-readable step heading ("Step 3 of 8 — Consent policy").
  title: string;
  /// Short hint below the heading describing what this step decides.
  subtitle?: string;
  /// 1-indexed step number (for screen readers + the progress bar).
  stepIndex: number;
  /// Total steps in this wizard (8 for org creation).
  stepCount: number;
  /// Inline error to render above the content. Re-uses
  /// `ApiErrorAlert` so error shapes stay consistent across M2+M3.
  error?: StepError | null;
  children: React.ReactNode;
};

export function StepShell({
  title,
  subtitle,
  stepIndex,
  stepCount,
  error,
  children,
}: StepShellProps) {
  const pct = Math.round((stepIndex / stepCount) * 100);
  return (
    <section
      aria-labelledby="wizard-step-title"
      className="space-y-4"
    >
      <header className="space-y-2">
        <div
          className="h-1 w-full overflow-hidden rounded bg-white/5"
          aria-hidden="true"
        >
          <div
            className="h-full bg-white/30 transition-all"
            style={{ width: `${pct}%` }}
          />
        </div>
        <div className="flex items-baseline justify-between">
          <h2
            id="wizard-step-title"
            className="text-lg font-semibold"
          >
            {title}
          </h2>
          <span className="text-xs opacity-60">
            Step {stepIndex} of {stepCount}
          </span>
        </div>
        {subtitle ? (
          <p className="text-sm opacity-70">{subtitle}</p>
        ) : null}
      </header>

      {error ? <ApiErrorAlert error={error} /> : null}

      <div className="space-y-3">{children}</div>
    </section>
  );
}
