// Read-only side panel showing the phi-core factory baseline.
//
// Operators use this as the "revert to factory" reference when
// editing the form. The server sends `factory` on every GET so the
// panel stays in sync even after a phi-core bump without baby-phi
// code changes (all four phi-core sections are opaque on the wire).

"use client";

import type { PlatformDefaults } from "@/lib/api/platform-defaults";

function section(
  label: string,
  value: Record<string, unknown>,
): React.ReactNode {
  return (
    <div key={label}>
      <div className="text-xs font-medium opacity-80">{label}</div>
      <pre className="mt-1 overflow-x-auto rounded bg-black/30 p-2 font-mono text-xs">
        {JSON.stringify(value, null, 2)}
      </pre>
    </div>
  );
}

export function FactoryDefaultsPanel({
  factory,
}: {
  factory: PlatformDefaults;
}) {
  return (
    <aside className="space-y-3 rounded border border-white/10 bg-black/20 p-3">
      <header>
        <h3 className="text-sm font-medium">phi-core factory baseline</h3>
        <p className="mt-1 text-xs opacity-60">
          Copy any section below into the form to reset that slice to
          phi-core&apos;s default. The baseline lives in{" "}
          <span className="font-mono">
            PlatformDefaults::factory()
          </span>
          .
        </p>
      </header>
      <div className="space-y-3">
        <div>
          <div className="text-xs font-medium opacity-80">retention</div>
          <div className="mt-1 text-xs font-mono">
            {factory.defaultRetentionDays} days
          </div>
        </div>
        <div>
          <div className="text-xs font-medium opacity-80">alert channels</div>
          <div className="mt-1 text-xs font-mono">
            {factory.defaultAlertChannels.length === 0
              ? "(none)"
              : factory.defaultAlertChannels.join(", ")}
          </div>
        </div>
        {section("ExecutionLimits", factory.executionLimits)}
        {section("AgentProfile", factory.defaultAgentProfile)}
        {section("ContextConfig", factory.contextConfig)}
        {section("RetryConfig", factory.retryConfig)}
      </div>
    </aside>
  );
}
