// Session event stream renderer for M5 page 14 (First Session Launch).
//
// Renders a live list of `phi_core::types::event::AgentEvent` payloads
// as they arrive over SSE from `GET /api/v0/sessions/:id/events`.
// Ships as a presentation-only Client Component at M5/P1 — the SSE
// consumer + event-kind exhaustive rendering lands at M5/P4 + M5/P7
// when the HTTP handler + CLI body ship.
//
// Kept minimal at M5/P1 so the shell completion regression sees the
// `phi session launch --detach` flag today even before page 14's
// launch wizard wires the real renderer.
"use client";

export type SessionEventKind =
  | "agent_start"
  | "turn_start"
  | "message_start"
  | "message_update"
  | "message_end"
  | "tool_execution_start"
  | "tool_execution_update"
  | "tool_execution_end"
  | "progress_message"
  | "input_rejected"
  | "turn_end"
  | "agent_end";

export type SessionEventRow = {
  kind: SessionEventKind;
  timestamp: string;
  /// Freeform serialised payload; full exhaustive typing lands at
  /// M5/P4 when the HTTP handler materialises typed variants.
  summary: string;
};

export function SessionEventStreamRenderer({
  events,
  status,
}: {
  events: SessionEventRow[];
  status: "connecting" | "streaming" | "ended" | "aborted";
}) {
  return (
    <div
      role="log"
      aria-live="polite"
      className="rounded border border-gray-500/40 bg-black/40 p-4 font-mono text-xs"
    >
      <div className="mb-2 flex items-center justify-between">
        <span className="uppercase tracking-wider opacity-60">
          session events
        </span>
        <span
          className={
            status === "streaming"
              ? "text-green-400"
              : status === "ended"
                ? "text-gray-400"
                : status === "aborted"
                  ? "text-red-400"
                  : "text-yellow-400"
          }
        >
          {status}
        </span>
      </div>
      {events.length === 0 ? (
        <div className="opacity-50">
          {status === "connecting" ? "connecting…" : "no events yet"}
        </div>
      ) : (
        <ul className="space-y-1">
          {events.map((ev, i) => (
            <li key={`${ev.timestamp}-${i}`} className="flex gap-3">
              <span className="opacity-40">{ev.timestamp}</span>
              <span className="opacity-80">{ev.kind}</span>
              <span className="flex-1">{ev.summary}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
