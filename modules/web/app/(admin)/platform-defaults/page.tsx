// /platform-defaults — Platform Defaults admin page (M2/P7).
//
// Server Component. Fetches GET /api/v0/platform/defaults and hands
// the result to the client form.

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import { getDefaultsApi } from "@/lib/api/platform-defaults";

import { DefaultsForm } from "./DefaultsForm";
import { FactoryDefaultsPanel } from "./FactoryDefaultsPanel";

export const dynamic = "force-dynamic";

export default async function PlatformDefaultsPage() {
  const headers = await forwardSessionCookieHeader();
  const res = await getDefaultsApi(headers);

  if (!res.ok) {
    return (
      <div className="space-y-4">
        <header>
          <h1 className="text-xl font-semibold">Platform Defaults</h1>
        </header>
        <div
          role="alert"
          className="rounded border border-red-500/40 bg-red-500/10 p-3 text-sm"
        >
          {res.code}: {res.message}
        </div>
      </div>
    );
  }

  const { defaults, persisted, factory } = res.value;

  return (
    <div className="space-y-8">
      <header className="space-y-1">
        <h1 className="text-xl font-semibold">Platform Defaults</h1>
        <p className="text-sm opacity-70">
          Platform-wide baselines for the agent loop (phi-core{" "}
          <span className="font-mono">ExecutionLimits</span>,{" "}
          <span className="font-mono">AgentProfile</span>,{" "}
          <span className="font-mono">ContextConfig</span>,{" "}
          <span className="font-mono">RetryConfig</span>), audit retention,
          and alert channels.{" "}
          <strong>Non-retroactive</strong> — edits never mutate existing orgs;
          they apply only to orgs created after the write.
        </p>
        {!persisted ? (
          <div className="rounded border border-amber-500/40 bg-amber-500/10 p-2 text-xs">
            No <span className="font-mono">platform_defaults</span> row
            persisted yet — the form below is pre-populated with phi-core&apos;s
            factory baseline. Submitting at{" "}
            <span className="font-mono">if_version=0</span> creates the first
            row.
          </div>
        ) : null}
      </header>

      <div className="grid gap-8 lg:grid-cols-3">
        <div className="lg:col-span-2">
          <DefaultsForm initialDefaults={defaults} />
        </div>
        <div>
          <FactoryDefaultsPanel factory={factory} />
        </div>
      </div>
    </div>
  );
}
