// Wizard navigation buttons — Back / Next / Save draft / Submit.
//
// Owns the disabled-state logic so per-step components don't
// re-implement it: Back disabled on step 1; Next disabled when
// `canProceed` is false; Submit shown only on the final step;
// Save-draft always enabled (drafts are cheap).
//
// Does not own draft persistence — that's `DraftContext`'s job
// (this component calls the onSaveDraft callback, and the caller
// wires it to the Context). Clean separation so tests can assert
// each concern in isolation.

"use client";

export type StepNavProps = {
  stepIndex: number;
  stepCount: number;
  /// Step-specific validation: false disables Next / Submit.
  canProceed: boolean;
  /// Pending indicator. Disables every button while a network call
  /// is in flight so double-clicks don't double-submit.
  pending?: boolean;
  onBack?: () => void;
  onNext?: () => void;
  /// Final-step submit. Replaces Next on the last step. Caller is
  /// responsible for showing a success/error state after the Promise
  /// resolves.
  onSubmit?: () => void | Promise<void>;
  /// Called when the operator clicks Save draft. Caller (typically
  /// the wizard page) forwards to `DraftContext.setDraft(...)`.
  onSaveDraft?: () => void;
};

export function StepNav({
  stepIndex,
  stepCount,
  canProceed,
  pending = false,
  onBack,
  onNext,
  onSubmit,
  onSaveDraft,
}: StepNavProps) {
  const isFirst = stepIndex <= 1;
  const isLast = stepIndex >= stepCount;
  return (
    <nav
      aria-label="Wizard navigation"
      className="flex items-center justify-between gap-2 border-t border-white/10 pt-4"
    >
      <div className="flex gap-2">
        <button
          type="button"
          onClick={onBack}
          disabled={isFirst || pending}
          className="rounded border border-white/20 px-3 py-1.5 text-sm hover:bg-white/5 disabled:opacity-40"
        >
          Back
        </button>
        <button
          type="button"
          onClick={onSaveDraft}
          disabled={pending}
          className="rounded border border-white/20 px-3 py-1.5 text-sm hover:bg-white/5 disabled:opacity-40"
        >
          Save draft
        </button>
      </div>
      {isLast ? (
        <button
          type="button"
          onClick={() => {
            if (onSubmit) void onSubmit();
          }}
          disabled={!canProceed || pending}
          className="rounded bg-white/15 px-4 py-1.5 text-sm font-medium hover:bg-white/20 disabled:opacity-40"
        >
          {pending ? "Submitting…" : "Submit"}
        </button>
      ) : (
        <button
          type="button"
          onClick={onNext}
          disabled={!canProceed || pending}
          className="rounded bg-white/10 px-4 py-1.5 text-sm hover:bg-white/15 disabled:opacity-40"
        >
          Next
        </button>
      )}
    </nav>
  );
}
