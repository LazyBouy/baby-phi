// Agents roster — M4/P4 admin page 08.
//
// SSR entry-point fetches the current snapshot (optionally filtered
// by `role` + `search` query params) and renders the table. Filter
// chips submit via plain `<form method="get">` to keep the page
// streamable without client-side JavaScript — page 09's edit form
// (M4/P5) will add client-side state where it matters.

import Link from "next/link";

import { listAgentsAction } from "./actions";
import type { AgentRoleWire } from "@/lib/api/agents";

const ALL_ROLES: AgentRoleWire[] = [
  "executive",
  "admin",
  "member",
  "intern",
  "contract",
  "system",
];

function isRole(x: string | undefined): x is AgentRoleWire {
  return !!x && (ALL_ROLES as string[]).includes(x);
}

export default async function AgentsRosterPage({
  params,
  searchParams,
}: {
  params: Promise<{ id: string }>;
  searchParams: Promise<{ role?: string; search?: string }>;
}) {
  const { id } = await params;
  const sp = await searchParams;

  const role = isRole(sp.role) ? sp.role : null;
  const search = sp.search && sp.search.trim().length ? sp.search : null;

  const result = await listAgentsAction(id, { role, search });

  if (!result.ok) {
    return (
      <section className="space-y-4 p-6">
        <header className="flex items-center justify-between">
          <h1 className="text-xl font-semibold">Agents</h1>
          <Link
            href={`/organizations/${id}/dashboard`}
            className="text-sm underline"
          >
            ← Back to dashboard
          </Link>
        </header>
        <div className="rounded border border-red-500/40 bg-red-500/10 p-3 text-sm">
          <div className="font-semibold">{result.code}</div>
          <div>{result.message}</div>
        </div>
      </section>
    );
  }

  const { agents } = result.value;

  return (
    <section className="space-y-4 p-6">
      <header className="flex items-center justify-between">
        <h1 className="text-xl font-semibold">Agents</h1>
        <Link
          href={`/organizations/${id}/dashboard`}
          className="text-sm underline"
        >
          ← Back to dashboard
        </Link>
      </header>

      <form
        method="GET"
        className="flex flex-wrap items-center gap-3"
        aria-label="Agent roster filters"
      >
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-xs opacity-70">Role:</span>
          <FilterChip
            href={`/organizations/${id}/agents${search ? `?search=${encodeURIComponent(search)}` : ""}`}
            label="all"
            selected={role === null}
          />
          {ALL_ROLES.map((r) => {
            const params = new URLSearchParams();
            params.set("role", r);
            if (search) params.set("search", search);
            return (
              <FilterChip
                key={r}
                href={`/organizations/${id}/agents?${params.toString()}`}
                label={r}
                selected={role === r}
              />
            );
          })}
        </div>
        <label className="ml-auto flex items-center gap-2 text-xs">
          <span className="opacity-70">Search:</span>
          <input
            type="text"
            name="search"
            defaultValue={search ?? ""}
            placeholder="display name…"
            className="rounded border border-white/10 bg-black/20 px-2 py-1 text-sm"
          />
          {role && <input type="hidden" name="role" value={role} />}
          <button
            type="submit"
            className="rounded bg-blue-500 px-3 py-1 text-sm font-medium text-white hover:bg-blue-400"
          >
            Apply
          </button>
        </label>
      </form>

      {agents.length === 0 ? (
        <div className="rounded border border-white/10 bg-black/20 p-6 text-sm opacity-75">
          No agents match the current filter.
        </div>
      ) : (
        <table className="w-full text-sm">
          <thead className="text-left opacity-70">
            <tr>
              <th className="py-2">Display name</th>
              <th>Kind</th>
              <th>Role</th>
              <th>Created</th>
            </tr>
          </thead>
          <tbody>
            {agents.map((a) => (
              <tr key={a.id} className="border-t border-white/5">
                <td className="py-2">
                  <Link
                    className="underline"
                    href={`/organizations/${id}/agents/${a.id}`}
                  >
                    {a.display_name}
                  </Link>
                </td>
                <td>{a.kind}</td>
                <td>{a.role ?? "—"}</td>
                <td className="opacity-70">
                  {new Date(a.created_at).toLocaleString()}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

function FilterChip({
  href,
  label,
  selected,
}: {
  href: string;
  label: string;
  selected: boolean;
}) {
  return (
    <Link
      href={href}
      className={
        selected
          ? "rounded border border-blue-400 bg-blue-500/20 px-2 py-0.5 text-xs text-blue-100"
          : "rounded border border-white/10 bg-black/20 px-2 py-0.5 text-xs opacity-80 hover:opacity-100"
      }
      prefetch={false}
    >
      {label}
    </Link>
  );
}
