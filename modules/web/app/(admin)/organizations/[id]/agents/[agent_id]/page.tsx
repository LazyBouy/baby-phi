// Agent profile editor — M4/P5 admin page 09 (edit mode).
//
// Shape at M4:
//  - Identity (read-only): id, kind, role, owning_org. These are
//    immutable post-creation per D3 — the server returns
//    AGENT_IMMUTABLE_FIELD_CHANGED if the caller tries to patch them.
//  - display_name: editable free-text.
//  - AgentProfile blueprint (phi-core, editable): system_prompt,
//    thinking_level, temperature. parallelize (phi governance,
//    editable; 1..64).
//  - ExecutionLimits: radio {Inherit / Override}, plus a "Revert to
//    org default" button that DELETEs the override row via the
//    dedicated endpoint.
//
// System-role agents: the server returns SYSTEM_AGENT_READ_ONLY on
// any edit attempt. The page still renders for inspection.

import Link from "next/link";

import { listAgentsAction, updateAgentProfileAction } from "../actions";

export default async function EditAgentPage({
  params,
}: {
  params: Promise<{ id: string; agent_id: string }>;
}) {
  const { id: orgId, agent_id: agentId } = await params;

  // Find the agent by listing the roster. A dedicated "show agent"
  // endpoint lands at M4/P5b; until then, the list is the only
  // read-surface and is cheap at M4 volume (tens of agents).
  const list = await listAgentsAction(orgId);
  if (!list.ok) {
    return (
      <section className="space-y-4 p-6">
        <h1 className="text-xl font-semibold">Edit agent</h1>
        <div className="rounded border border-red-500/40 bg-red-500/10 p-3 text-sm">
          <div className="font-semibold">{list.code}</div>
          <div>{list.message}</div>
        </div>
      </section>
    );
  }
  const agent = list.value.agents.find((a) => a.id === agentId);
  if (!agent) {
    return (
      <section className="space-y-4 p-6">
        <h1 className="text-xl font-semibold">Edit agent</h1>
        <div className="rounded border border-amber-500/40 bg-amber-500/10 p-3 text-sm">
          No agent with id {agentId} in this org.
        </div>
        <Link
          href={`/organizations/${orgId}/agents`}
          className="text-sm underline"
        >
          ← Back to roster
        </Link>
      </section>
    );
  }
  const isSystem = agent.role === "system";

  async function onPatchSubmit(fd: FormData): Promise<void> {
    "use server";
    const body: {
      display_name?: string | null;
      parallelize?: number | null;
      blueprint?: Record<string, unknown> | null;
    } = {};
    const displayName = fd.get("display_name");
    if (typeof displayName === "string" && displayName.trim().length) {
      body.display_name = displayName.trim();
    }
    const parallelize = fd.get("parallelize");
    if (typeof parallelize === "string" && parallelize.length) {
      const n = Number(parallelize);
      if (Number.isFinite(n)) body.parallelize = n;
    }
    const blueprint: Record<string, unknown> = {};
    const systemPrompt = fd.get("system_prompt");
    if (typeof systemPrompt === "string" && systemPrompt.length) {
      blueprint.system_prompt = systemPrompt;
    }
    const thinking = fd.get("thinking_level");
    if (typeof thinking === "string" && thinking.length) {
      blueprint.thinking_level = thinking;
    }
    const temp = fd.get("temperature");
    if (typeof temp === "string" && temp.length) {
      const n = Number(temp);
      if (Number.isFinite(n)) blueprint.temperature = n;
    }
    if (Object.keys(blueprint).length) body.blueprint = blueprint;
    await updateAgentProfileAction(orgId, agentId, body);
  }

  return (
    <section className="space-y-4 p-6">
      <header className="flex items-center justify-between">
        <h1 className="text-xl font-semibold">Edit agent</h1>
        <Link
          href={`/organizations/${orgId}/agents`}
          className="text-sm underline"
        >
          ← Roster
        </Link>
      </header>

      {isSystem && (
        <div className="rounded border border-amber-500/40 bg-amber-500/10 p-3 text-sm">
          This is a <code>system</code> agent. All edit attempts return{" "}
          <code>SYSTEM_AGENT_READ_ONLY</code> by design — blueprints
          are platform-managed.
        </div>
      )}

      <dl className="grid grid-cols-2 gap-2 rounded border border-white/10 p-4 text-sm">
        <dt className="opacity-70">id (immutable)</dt>
        <dd className="font-mono text-xs">{agent.id}</dd>
        <dt className="opacity-70">kind (immutable)</dt>
        <dd>{agent.kind}</dd>
        <dt className="opacity-70">role (immutable at M4)</dt>
        <dd>{agent.role ?? "—"}</dd>
        <dt className="opacity-70">owning_org (immutable)</dt>
        <dd className="font-mono text-xs">{agent.owning_org ?? "—"}</dd>
        <dt className="opacity-70">created_at</dt>
        <dd>{new Date(agent.created_at).toLocaleString()}</dd>
      </dl>

      <form action={onPatchSubmit} className="space-y-4">
        <fieldset
          disabled={isSystem}
          className="space-y-2 rounded border border-white/10 p-4"
        >
          <legend className="px-2 text-sm opacity-80">display_name</legend>
          <input
            name="display_name"
            defaultValue={agent.display_name}
            className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1 text-sm"
          />
        </fieldset>

        <fieldset
          disabled={isSystem}
          className="space-y-2 rounded border border-white/10 p-4"
        >
          <legend className="px-2 text-sm opacity-80">
            Blueprint (phi-core, patch — empty fields leave current
            value untouched)
          </legend>
          <label className="block text-sm">
            <span className="block opacity-70">system_prompt</span>
            <textarea
              name="system_prompt"
              rows={3}
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
            />
          </label>
          <label className="block text-sm">
            <span className="block opacity-70">thinking_level</span>
            <select
              name="thinking_level"
              defaultValue=""
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
            >
              <option value="">(unchanged)</option>
              {["off", "minimal", "low", "medium", "high"].map((v) => (
                <option key={v} value={v}>
                  {v}
                </option>
              ))}
            </select>
          </label>
          <label className="block text-sm">
            <span className="block opacity-70">temperature</span>
            <input
              name="temperature"
              type="number"
              step="0.05"
              min="0"
              max="2"
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
            />
          </label>
        </fieldset>

        <fieldset
          disabled={isSystem}
          className="space-y-2 rounded border border-white/10 p-4"
        >
          <legend className="px-2 text-sm opacity-80">
            Governance (phi-only)
          </legend>
          <label className="block text-sm">
            <span className="block opacity-70">parallelize (1..64)</span>
            <input
              name="parallelize"
              type="number"
              min="1"
              max="64"
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
            />
          </label>
        </fieldset>

        <button
          type="submit"
          disabled={isSystem}
          className="rounded bg-blue-500 px-3 py-2 text-sm font-medium text-white hover:bg-blue-400 disabled:bg-gray-500"
        >
          Save changes
        </button>
      </form>

      <div className="rounded border border-white/10 p-4">
        <h2 className="text-sm font-medium">ExecutionLimits override</h2>
        <p className="mt-1 text-xs opacity-70">
          Currently showing the roster-level view. The full override
          editor (including &quot;Revert to org default&quot;) ships with
          the richer show-agent endpoint at M4/P5b.
        </p>
      </div>
    </section>
  );
}
