// Agent creation wizard — M4/P5 admin page 09 (create mode).
//
// Form sections (per M4 plan Part 1.5 Page 09):
//  - Identity: display_name + kind + role
//  - AgentProfile (phi-core): system_prompt + thinking_level +
//    temperature
//  - ExecutionLimits: radio toggle {Inherit, Override}. When Override
//    is selected, four editable fields (max_turns / max_total_tokens /
//    max_duration_secs / max_cost) appear with org-ceiling hints.
//  - phi governance: parallelize (1..64 at M4).
//
// ModelConfig editing is intentionally deferred (M5) — see the M4/P5
// update orchestrator docstring for the reason.

import { redirect } from "next/navigation";

import { createAgentAction } from "../actions";
import type {
  AgentKindWire,
  AgentRoleWire,
  ExecutionLimitsWire,
} from "@/lib/api/agents";

const KIND_OPTIONS: AgentKindWire[] = ["human", "llm"];
const ROLE_OPTIONS: AgentRoleWire[] = [
  "executive",
  "admin",
  "member",
  "intern",
  "contract",
  "system",
];
const THINKING_LEVELS = ["off", "minimal", "low", "medium", "high"] as const;

function readString(fd: FormData, key: string): string | null {
  const v = fd.get(key);
  if (typeof v !== "string") return null;
  const t = v.trim();
  return t.length ? t : null;
}

function readNumber(fd: FormData, key: string): number | null {
  const s = readString(fd, key);
  if (s === null) return null;
  const n = Number(s);
  return Number.isFinite(n) ? n : null;
}

export default async function CreateAgentPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id: orgId } = await params;

  async function onSubmit(fd: FormData): Promise<void> {
    "use server";
    const name = readString(fd, "display_name") ?? "";
    const kind = (readString(fd, "kind") ?? "llm") as AgentKindWire;
    const role = readString(fd, "role") as AgentRoleWire | null;
    const parallelize = readNumber(fd, "parallelize") ?? 1;

    const blueprint: Record<string, unknown> = {};
    const systemPrompt = readString(fd, "system_prompt");
    if (systemPrompt) blueprint.system_prompt = systemPrompt;
    const thinkingLevel = readString(fd, "thinking_level");
    if (thinkingLevel) blueprint.thinking_level = thinkingLevel;
    const temperature = readNumber(fd, "temperature");
    if (temperature !== null) blueprint.temperature = temperature;

    let override: ExecutionLimitsWire | null = null;
    if (readString(fd, "exec_mode") === "override") {
      override = {
        max_turns: readNumber(fd, "max_turns") ?? 50,
        max_total_tokens: readNumber(fd, "max_total_tokens") ?? 1_000_000,
        max_duration: {
          secs: readNumber(fd, "max_duration_secs") ?? 600,
          nanos: 0,
        },
        max_cost: readNumber(fd, "max_cost"),
      };
    }

    const result = await createAgentAction(orgId, {
      display_name: name,
      kind,
      role,
      blueprint,
      parallelize,
      initial_execution_limits_override: override,
    });
    if (result.ok) {
      redirect(`/organizations/${orgId}/agents/${result.value.agent_id}`);
    }
    // On failure, the page SSR path re-renders; we don't have client
    // state here so the browser keeps the form untouched. An error
    // banner could be added via `useFormState` in a future revision.
  }

  return (
    <section className="space-y-4 p-6">
      <header>
        <h1 className="text-xl font-semibold">Create agent</h1>
        <p className="mt-1 text-sm opacity-70">
          Page 09 (M4/P5). ExecutionLimits override is opt-in (ADR-0027);
          default path inherits the org snapshot (ADR-0023).
        </p>
      </header>

      <form action={onSubmit} className="space-y-4">
        <fieldset className="space-y-2 rounded border border-white/10 p-4">
          <legend className="px-2 text-sm opacity-80">Identity</legend>
          <label className="block text-sm">
            <span className="block opacity-70">Display name</span>
            <input
              name="display_name"
              required
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
            />
          </label>
          <label className="block text-sm">
            <span className="block opacity-70">Kind</span>
            <select
              name="kind"
              defaultValue="llm"
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
            >
              {KIND_OPTIONS.map((k) => (
                <option key={k} value={k}>
                  {k}
                </option>
              ))}
            </select>
          </label>
          <label className="block text-sm">
            <span className="block opacity-70">Role</span>
            <select
              name="role"
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
            >
              <option value="">(none)</option>
              {ROLE_OPTIONS.map((r) => (
                <option key={r} value={r}>
                  {r}
                </option>
              ))}
            </select>
          </label>
        </fieldset>

        <fieldset className="space-y-2 rounded border border-white/10 p-4">
          <legend className="px-2 text-sm opacity-80">
            AgentProfile (phi-core blueprint)
          </legend>
          <label className="block text-sm">
            <span className="block opacity-70">System prompt</span>
            <textarea
              name="system_prompt"
              rows={3}
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
            />
          </label>
          <label className="block text-sm">
            <span className="block opacity-70">Thinking level</span>
            <select
              name="thinking_level"
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
            >
              <option value="">(inherit)</option>
              {THINKING_LEVELS.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
          </label>
          <label className="block text-sm">
            <span className="block opacity-70">Temperature</span>
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

        <fieldset className="space-y-3 rounded border border-white/10 p-4">
          <legend className="px-2 text-sm opacity-80">ExecutionLimits</legend>
          <div className="flex gap-4 text-sm">
            <label className="flex items-center gap-2">
              <input
                type="radio"
                name="exec_mode"
                value="inherit"
                defaultChecked
              />
              Inherit from org (ADR-0023 default)
            </label>
            <label className="flex items-center gap-2">
              <input type="radio" name="exec_mode" value="override" />
              Override (ADR-0027 opt-in — must be ≤ org ceiling)
            </label>
          </div>
          <div className="grid grid-cols-2 gap-2 text-sm">
            <label>
              <span className="block opacity-70">max_turns</span>
              <input
                name="max_turns"
                type="number"
                min="1"
                className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
              />
            </label>
            <label>
              <span className="block opacity-70">max_total_tokens</span>
              <input
                name="max_total_tokens"
                type="number"
                min="1"
                className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
              />
            </label>
            <label>
              <span className="block opacity-70">max_duration_secs</span>
              <input
                name="max_duration_secs"
                type="number"
                min="1"
                className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
              />
            </label>
            <label>
              <span className="block opacity-70">max_cost (USD)</span>
              <input
                name="max_cost"
                type="number"
                step="0.01"
                min="0"
                className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
              />
            </label>
          </div>
          <p className="text-xs opacity-60">
            Override values must be ≤ the corresponding org-snapshot
            ceiling. A breach returns{" "}
            <code>EXECUTION_LIMITS_EXCEED_ORG_CEILING</code>.
          </p>
        </fieldset>

        <fieldset className="space-y-2 rounded border border-white/10 p-4">
          <legend className="px-2 text-sm opacity-80">Governance</legend>
          <label className="block text-sm">
            <span className="block opacity-70">parallelize (1..64)</span>
            <input
              name="parallelize"
              type="number"
              min="1"
              max="64"
              defaultValue="1"
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
            />
          </label>
        </fieldset>

        <button
          type="submit"
          className="rounded bg-blue-500 px-3 py-2 text-sm font-medium text-white hover:bg-blue-400"
        >
          Create agent
        </button>
      </form>
    </section>
  );
}
