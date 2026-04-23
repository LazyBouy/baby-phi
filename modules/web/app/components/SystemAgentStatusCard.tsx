// System agent status card for M5 page 13 (System Agents Config, R2).
//
// Renders the per-system-agent live-status tile populated from the
// `SystemAgentRuntimeStatus` governance row (migration 0005 /
// composites_m5). Server Component — no client state.
//
// The 5-listener upsert helper (Template A / C / D + memory-extraction
// + agent-catalog) keeps the row fresh at M5/P3 close; this primitive
// just renders what the GET endpoint returns.

export type SystemAgentStatus = {
  agent_id: string;
  display_name: string;
  /// One of the governance-plane trigger enums from
  /// `concepts/system-agents.md` — NOT `phi_core::AgentEvent`.
  trigger: "session_end" | "edge_change" | "periodic" | "explicit" | "custom_event";
  queue_depth: number;
  last_fired_at: string | null;
  effective_parallelize: number;
  last_error: string | null;
  active: boolean;
};

export function SystemAgentStatusCard({
  status,
}: {
  status: SystemAgentStatus;
}) {
  const fired = status.last_fired_at
    ? new Date(status.last_fired_at).toISOString()
    : null;
  return (
    <article
      className={
        "rounded border p-4 text-sm " +
        (status.active
          ? "border-gray-500/40"
          : "border-gray-500/20 opacity-60")
      }
      aria-labelledby={`system-agent-${status.agent_id}-name`}
    >
      <header className="mb-2 flex items-center justify-between">
        <h4
          id={`system-agent-${status.agent_id}-name`}
          className="font-semibold"
        >
          {status.display_name}
        </h4>
        <span className="text-xs uppercase tracking-wider opacity-60">
          {status.trigger}
        </span>
      </header>
      <dl className="grid grid-cols-2 gap-1 font-mono text-xs">
        <dt className="opacity-60">queue</dt>
        <dd
          className={
            status.queue_depth > 0 ? "text-yellow-400" : ""
          }
        >
          {status.queue_depth}
        </dd>
        <dt className="opacity-60">parallelize</dt>
        <dd>{status.effective_parallelize}</dd>
        <dt className="opacity-60">last fired</dt>
        <dd>{fired ?? "never"}</dd>
      </dl>
      {status.last_error ? (
        <div className="mt-2 rounded border border-red-500/40 bg-red-500/10 p-2 text-xs">
          <div className="mb-1 font-mono uppercase tracking-wider opacity-60">
            last error
          </div>
          <div>{status.last_error}</div>
        </div>
      ) : null}
      {!status.active ? (
        <div className="mt-2 text-xs text-red-400">disabled</div>
      ) : null}
    </article>
  );
}
