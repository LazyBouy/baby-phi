// Shape B review-step notice — shown on page 10 wizard step 6 when the
// operator selects a co-owned project shape. Replaces the immediate
// "will be created" language with "pending co-owner approval" messaging
// and surfaces the two approver-slot handles so the operator knows
// exactly whose approval they'll wait on.
//
// Shape A uses the sibling `ReviewDiff` primitive unchanged — there's
// no approval deadline on Shape A because the project materialises in
// one transaction. Shape B adds this notice instead of a ReviewDiff
// trailing line per M4 plan G15 (wizard primitive extension).

"use client";

export type ShapeBApprover = {
  /// Operator-facing display name (not the raw agent id).
  displayName: string;
  /// Org the approver belongs to — shown alongside so operators see
  /// at a glance which org each slot is for.
  orgName: string;
};

export type ShapeBPendingApprovalNoticeProps = {
  /// Two approver slots — one per co-owning org. Shape B is exactly
  /// 2-approver at M4 per ADR-0025 (3+ is deferred until a Shape C
  /// concept doc emerges).
  approvers: [ShapeBApprover, ShapeBApprover];
  /// Optional active window in days (defaults to 7) — shown as "This
  /// request expires in X days" so operators know when the AR lapses
  /// without action.
  activeWindowDays?: number;
};

export function ShapeBPendingApprovalNotice({
  approvers,
  activeWindowDays = 7,
}: ShapeBPendingApprovalNoticeProps) {
  return (
    <section
      role="status"
      aria-live="polite"
      className="rounded border border-amber-500/40 bg-amber-500/5 p-4"
      data-testid="shape-b-pending-approval-notice"
    >
      <h3 className="mb-2 text-sm font-semibold text-amber-300">
        Pending co-owner approval
      </h3>
      <p className="mb-3 text-sm opacity-80">
        This is a <strong>Shape B</strong> (co-owned) project. Submitting
        creates an <em>Auth Request</em> with two approver slots — one per
        co-owning org. The project materialises only once <strong>both</strong>{" "}
        approvers approve.
      </p>
      <ul className="mb-3 space-y-1 text-sm">
        {approvers.map((a, i) => (
          <li key={`${a.orgName}-${i}`} className="font-mono text-xs">
            Slot {i + 1}:{" "}
            <strong>{a.displayName}</strong>{" "}
            <span className="opacity-60">({a.orgName})</span>
          </li>
        ))}
      </ul>
      <p className="text-xs opacity-60">
        Approvers receive a message in their inbox on submit. This request
        expires in {activeWindowDays} days if unresolved. If one approver has
        departed, the operations runbook describes the{" "}
        <em>approval-deadlock playbook</em>.
      </p>
    </section>
  );
}
