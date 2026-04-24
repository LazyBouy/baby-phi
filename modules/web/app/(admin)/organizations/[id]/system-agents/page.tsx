// System Agents admin page — M5/P7 (D6.2 carryover).
//
// SSR renders two buckets (standard / org_specific) + a "recent
// events" feed. Each row has inline `<form>` Server Actions for tune
// / disable / archive; a second section hosts the add-new form.
// Standard agents cannot be archived (enforced server-side) so the
// archive button is only rendered for org_specific rows.

import Link from "next/link";

import {
  addSystemAgentAction,
  archiveSystemAgentAction,
  disableSystemAgentAction,
  listSystemAgentsAction,
  tuneSystemAgentAction,
} from "./actions";
import type { SystemAgentRowWire } from "@/lib/api/system-agents";

export const dynamic = "force-dynamic";

export default async function SystemAgentsPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const result = await listSystemAgentsAction(id);

  if (!result.ok) {
    return (
      <section className="space-y-4 p-6">
        <Header orgId={id} />
        <div
          role="alert"
          className="rounded border border-red-500/40 bg-red-500/10 p-3 text-sm"
        >
          <div className="font-semibold">{result.code}</div>
          <div>{result.message}</div>
        </div>
      </section>
    );
  }

  const { standard, org_specific, recent_events } = result.value;

  return (
    <section className="space-y-6 p-6">
      <Header orgId={id} />

      <AgentTable
        title="Standard"
        rows={standard}
        orgId={id}
        canArchive={false}
      />
      <AgentTable
        title="Org-specific"
        rows={org_specific}
        orgId={id}
        canArchive={true}
      />

      <AddForm orgId={id} />

      <RecentEvents
        rows={recent_events}
        standardById={new Map(standard.map((r) => [r.agent_id, r]))}
        orgSpecificById={new Map(org_specific.map((r) => [r.agent_id, r]))}
      />
    </section>
  );
}

function Header({ orgId }: { orgId: string }) {
  return (
    <header className="flex items-center justify-between">
      <h1 className="text-xl font-semibold">System Agents</h1>
      <Link
        href={`/organizations/${orgId}/dashboard`}
        className="text-sm underline"
      >
        ← Back to dashboard
      </Link>
    </header>
  );
}

function AgentTable({
  title,
  rows,
  orgId,
  canArchive,
}: {
  title: string;
  rows: SystemAgentRowWire[];
  orgId: string;
  canArchive: boolean;
}) {
  return (
    <div className="space-y-2">
      <h2 className="text-sm font-semibold uppercase tracking-wide opacity-70">
        {title} ({rows.length})
      </h2>
      {rows.length === 0 ? (
        <div className="rounded border border-white/10 bg-black/20 p-3 text-xs opacity-75">
          (none)
        </div>
      ) : (
        <table className="w-full text-sm">
          <thead className="text-left opacity-70">
            <tr>
              <th className="py-2">Display name</th>
              <th>Profile</th>
              <th>Parallelize</th>
              <th>Queue</th>
              <th>Last fired</th>
              <th className="text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={row.agent_id} className="border-t border-white/5">
                <td className="py-2">
                  <div>{row.display_name}</div>
                  <div className="font-mono text-xs opacity-60">
                    {row.agent_id}
                  </div>
                </td>
                <td className="font-mono text-xs">{row.profile_ref ?? "—"}</td>
                <td>{row.parallelize}</td>
                <td>{row.queue_depth ?? 0}</td>
                <td className="text-xs opacity-70">
                  {row.last_fired_at
                    ? new Date(row.last_fired_at).toLocaleString()
                    : "—"}
                </td>
                <td className="text-right">
                  <div className="flex flex-wrap items-center justify-end gap-1">
                    <TuneForm orgId={orgId} agentId={row.agent_id} />
                    <DisableForm orgId={orgId} agentId={row.agent_id} />
                    {canArchive ? (
                      <ArchiveForm orgId={orgId} agentId={row.agent_id} />
                    ) : null}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function TuneForm({ orgId, agentId }: { orgId: string; agentId: string }) {
  async function run(formData: FormData) {
    "use server";
    const parsed = Number(formData.get("parallelize") ?? 1);
    const parallelize =
      Number.isFinite(parsed) && parsed > 0 ? Math.floor(parsed) : 1;
    await tuneSystemAgentAction(orgId, agentId, parallelize);
  }
  return (
    <form action={run} className="inline-flex items-center gap-1">
      <input
        type="number"
        name="parallelize"
        min={1}
        max={32}
        defaultValue={1}
        className="w-14 rounded border border-white/10 bg-black/20 px-1 py-0.5 text-xs"
      />
      <button
        className="rounded bg-blue-500/80 px-2 py-1 text-xs text-white hover:bg-blue-400"
        type="submit"
      >
        Tune
      </button>
    </form>
  );
}

function DisableForm({ orgId, agentId }: { orgId: string; agentId: string }) {
  async function run() {
    "use server";
    await disableSystemAgentAction(orgId, agentId);
  }
  return (
    <form action={run}>
      <button
        className="rounded bg-amber-500/80 px-2 py-1 text-xs text-white hover:bg-amber-400"
        type="submit"
      >
        Disable
      </button>
    </form>
  );
}

function ArchiveForm({ orgId, agentId }: { orgId: string; agentId: string }) {
  async function run() {
    "use server";
    await archiveSystemAgentAction(orgId, agentId);
  }
  return (
    <form action={run}>
      <button
        className="rounded bg-rose-500/80 px-2 py-1 text-xs text-white hover:bg-rose-400"
        type="submit"
      >
        Archive
      </button>
    </form>
  );
}

function AddForm({ orgId }: { orgId: string }) {
  async function run(formData: FormData) {
    "use server";
    await addSystemAgentAction(orgId, {
      display_name: String(formData.get("display_name") ?? "").trim(),
      profile_ref: String(formData.get("profile_ref") ?? "").trim(),
      parallelize: Math.max(1, Number(formData.get("parallelize") ?? 1)),
      trigger: String(formData.get("trigger") ?? "explicit"),
    });
  }
  return (
    <div className="space-y-2">
      <h2 className="text-sm font-semibold uppercase tracking-wide opacity-70">
        Add org-specific system agent
      </h2>
      <form
        action={run}
        className="flex flex-wrap items-end gap-2 rounded border border-white/10 bg-black/20 p-3"
      >
        <Field label="Display name" name="display_name" required />
        <Field label="Profile ref" name="profile_ref" required />
        <Field
          label="Parallelize"
          name="parallelize"
          type="number"
          defaultValue="1"
        />
        <label className="flex flex-col text-xs">
          <span className="opacity-70">Trigger</span>
          <select
            name="trigger"
            defaultValue="explicit"
            className="rounded border border-white/10 bg-black/20 px-2 py-1 text-sm"
          >
            <option value="session_end">session_end</option>
            <option value="edge_change">edge_change</option>
            <option value="periodic">periodic</option>
            <option value="explicit">explicit</option>
            <option value="custom_event">custom_event</option>
          </select>
        </label>
        <button
          type="submit"
          className="rounded bg-blue-500 px-3 py-1 text-sm font-medium text-white hover:bg-blue-400"
        >
          Add
        </button>
      </form>
    </div>
  );
}

function Field({
  label,
  name,
  type = "text",
  defaultValue,
  required,
}: {
  label: string;
  name: string;
  type?: string;
  defaultValue?: string;
  required?: boolean;
}) {
  return (
    <label className="flex flex-col text-xs">
      <span className="opacity-70">{label}</span>
      <input
        type={type}
        name={name}
        defaultValue={defaultValue}
        required={required}
        className="rounded border border-white/10 bg-black/20 px-2 py-1 text-sm"
      />
    </label>
  );
}

function RecentEvents({
  rows,
  standardById,
  orgSpecificById,
}: {
  rows: { agent_id: string; at: string }[];
  standardById: Map<string, SystemAgentRowWire>;
  orgSpecificById: Map<string, SystemAgentRowWire>;
}) {
  return (
    <div className="space-y-2">
      <h2 className="text-sm font-semibold uppercase tracking-wide opacity-70">
        Recent fires ({rows.length})
      </h2>
      {rows.length === 0 ? (
        <div className="rounded border border-white/10 bg-black/20 p-3 text-xs opacity-75">
          No recent listener fires.
        </div>
      ) : (
        <ul className="space-y-1 text-xs">
          {rows.map((e, i) => {
            const row =
              standardById.get(e.agent_id) ?? orgSpecificById.get(e.agent_id);
            const name = row?.display_name ?? e.agent_id;
            return (
              <li key={`${e.agent_id}-${i}`} className="font-mono">
                {new Date(e.at).toLocaleString()} — {name}
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
