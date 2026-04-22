// Server-rendered `/bootstrap` page.
//
// SSR probes `GET /api/v0/bootstrap/status`; if the platform admin has
// already been claimed, renders a terminal "already assigned" view. If
// not, renders the claim form (which posts back through a Server
// Action — see `./actions.ts`).

import Link from "next/link";

import { getBootstrapStatus } from "@/lib/api";

import ClaimForm from "./ClaimForm";

export const dynamic = "force-dynamic";
export const revalidate = 0;

export const metadata = {
  title: "Bootstrap — phi",
};

export default async function BootstrapPage() {
  const result = await getBootstrapStatus();

  if (!result.ok) {
    return (
      <Shell>
        <section className="rounded border border-red-500/40 bg-red-500/5 p-6">
          <h2 className="text-lg font-medium">Cannot reach the server</h2>
          <p className="mt-2 text-sm opacity-80">
            The status probe to <code>/api/v0/bootstrap/status</code> failed:
          </p>
          <pre className="mt-2 whitespace-pre-wrap text-xs opacity-70">
            {result.error}
          </pre>
          <p className="mt-3 text-sm opacity-70">
            Make sure <code>phi-server</code> is running and that
            <code> PHI_API_URL</code> points at it.
          </p>
        </section>
      </Shell>
    );
  }

  if (result.status.claimed) {
    return (
      <Shell>
        <section className="rounded border border-white/10 p-6">
          <h2 className="text-lg font-medium">Platform admin already claimed</h2>
          <p className="mt-2 text-sm opacity-80">
            A platform admin was assigned for this install.
          </p>
          <p className="mt-4 font-mono text-xs">
            <span className="opacity-60">admin_agent_id: </span>
            {result.status.adminAgentId}
          </p>
          <p className="mt-4 text-sm opacity-70">
            Creating additional admins is out of scope for the first-install
            flow — it requires an Auth Request from the existing admin
            (lands in M3+).
          </p>
          <p className="mt-4 text-sm">
            <Link href="/" className="underline">
              ← Back to home
            </Link>
          </p>
        </section>
      </Shell>
    );
  }

  return (
    <Shell>
      <p className="text-sm opacity-70">
        No platform admin has been claimed yet. Paste the bootstrap credential
        printed by <code>phi-server bootstrap-init</code> to take the
        role.
      </p>
      <ClaimForm />
    </Shell>
  );
}

function Shell({ children }: { children: React.ReactNode }) {
  return (
    <main className="mx-auto max-w-2xl space-y-6 p-8">
      <header>
        <h1 className="text-3xl font-semibold">Claim platform admin</h1>
        <p className="mt-2 text-sm opacity-70">
          First-install flow — exchange your single-use bootstrap credential
          for the <code>[allocate]</code>-on-
          <code>system:root</code> grant that roots every subsequent
          authority chain in the platform.
        </p>
      </header>
      {children}
    </main>
  );
}
