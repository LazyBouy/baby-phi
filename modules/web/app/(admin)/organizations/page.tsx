// Organizations index — lists existing orgs + links to the wizard.

import Link from "next/link";

import { listOrgsAction } from "./actions";

export default async function OrganizationsPage() {
  const result = await listOrgsAction();

  if (!result.ok) {
    return (
      <section className="space-y-4 p-6">
        <h1 className="text-xl font-semibold">Organizations</h1>
        <div className="rounded border border-red-500/40 bg-red-500/10 p-3 text-sm">
          Failed to load organizations ({result.code}): {result.message}
        </div>
      </section>
    );
  }

  const { orgs } = result.value;

  return (
    <section className="space-y-4 p-6">
      <header className="flex items-center justify-between">
        <h1 className="text-xl font-semibold">Organizations</h1>
        <Link
          href="/organizations/new"
          className="rounded bg-blue-500 px-3 py-2 text-sm font-medium text-white hover:bg-blue-400"
        >
          Create new organization
        </Link>
      </header>

      {orgs.length === 0 ? (
        <div className="rounded border border-white/10 bg-black/20 p-6 text-sm opacity-75">
          No organizations yet — use the wizard to create one.
        </div>
      ) : (
        <table className="w-full text-sm">
          <thead className="text-left opacity-70">
            <tr>
              <th className="py-2">Name</th>
              <th>Members</th>
              <th>Templates</th>
              <th>Consent</th>
            </tr>
          </thead>
          <tbody>
            {orgs.map((o) => (
              <tr key={o.id} className="border-t border-white/5">
                <td className="py-2">
                  <Link
                    className="underline"
                    href={`/organizations/${o.id}`}
                  >
                    {o.display_name}
                  </Link>
                </td>
                <td>{o.member_count}</td>
                <td>{o.authority_templates_enabled.join(", ") || "—"}</td>
                <td>{o.consent_policy}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
