// Organization dashboard — M3/P5 admin page 07.
//
// SSR entry-point fetches the first snapshot; the client-side
// `DashboardClient` takes over to poll every 30s (R-ADMIN-07-N1,
// plan D4). Keeps the SSR HTML useful for back-button + non-JS
// fallback while the polling loop handles steady-state refresh.

import Link from "next/link";

import { dashboardOrgAction } from "../../actions";

import { DashboardClient } from "./DashboardClient";

export default async function OrgDashboardPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const initial = await dashboardOrgAction(id);

  if (!initial.ok) {
    return (
      <section className="space-y-4 p-6">
        <h1 className="text-xl font-semibold">Organization dashboard</h1>
        <div className="rounded border border-red-500/40 bg-red-500/10 p-3 text-sm">
          <div className="font-semibold">{initial.code}</div>
          <div>{initial.message}</div>
        </div>
        <Link href="/organizations" className="text-sm underline">
          ← Back to organizations
        </Link>
      </section>
    );
  }

  return <DashboardClient orgId={id} initial={initial.value} />;
}
