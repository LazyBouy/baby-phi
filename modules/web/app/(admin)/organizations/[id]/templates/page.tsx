// Authority Templates admin page — M5/P7 (carries forward D5.1).
//
// SSR-renders four buckets (pending / active / revoked / available) +
// one `<form>` per row invoking the matching Server Action. Zero
// phi-core types on the wire; the `kind` string is the governance id.

import Link from "next/link";

import {
  adoptTemplateAction,
  approveTemplateAction,
  denyTemplateAction,
  listTemplatesAction,
  revokeTemplateAction,
} from "./actions";
import type { TemplateRowWire } from "@/lib/api/templates";

export const dynamic = "force-dynamic";

export default async function AuthorityTemplatesPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const result = await listTemplatesAction(id);

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

  const { pending, active, revoked, available } = result.value;

  return (
    <section className="space-y-6 p-6">
      <Header orgId={id} />

      <Bucket title="Pending" rows={pending} emptyHint="No pending adoptions.">
        {(row) => (
          <div className="flex gap-2">
            <ApproveForm orgId={id} kind={String(row.kind)} />
            <DenyForm orgId={id} kind={String(row.kind)} />
          </div>
        )}
      </Bucket>

      <Bucket title="Active" rows={active} emptyHint="No active adoptions.">
        {(row) => <RevokeForm orgId={id} kind={String(row.kind)} />}
      </Bucket>

      <Bucket title="Revoked" rows={revoked} emptyHint="No revoked adoptions.">
        {() => <span className="text-xs opacity-60">terminal</span>}
      </Bucket>

      <Bucket
        title="Available"
        rows={available}
        emptyHint="All templates already adopted or revoked."
      >
        {(row) => <AdoptForm orgId={id} kind={String(row.kind)} />}
      </Bucket>
    </section>
  );
}

function Header({ orgId }: { orgId: string }) {
  return (
    <header className="flex items-center justify-between">
      <h1 className="text-xl font-semibold">Authority Templates</h1>
      <Link
        href={`/organizations/${orgId}/dashboard`}
        className="text-sm underline"
      >
        ← Back to dashboard
      </Link>
    </header>
  );
}

function Bucket({
  title,
  rows,
  emptyHint,
  children,
}: {
  title: string;
  rows: TemplateRowWire[];
  emptyHint: string;
  children: (row: TemplateRowWire) => React.ReactNode;
}) {
  return (
    <div className="space-y-2">
      <h2 className="text-sm font-semibold uppercase tracking-wide opacity-70">
        {title} ({rows.length})
      </h2>
      {rows.length === 0 ? (
        <div className="rounded border border-white/10 bg-black/20 p-3 text-xs opacity-75">
          {emptyHint}
        </div>
      ) : (
        <table className="w-full text-sm">
          <thead className="text-left opacity-70">
            <tr>
              <th className="py-2">Kind</th>
              <th>Summary</th>
              <th className="text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr
                key={`${title}-${row.kind}`}
                className="border-t border-white/5"
              >
                <td className="py-2 font-mono text-xs">
                  {String(row.kind).toUpperCase()}
                </td>
                <td>{row.summary ?? ""}</td>
                <td className="text-right">{children(row)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function ApproveForm({ orgId, kind }: { orgId: string; kind: string }) {
  async function run() {
    "use server";
    await approveTemplateAction(orgId, kind);
  }
  return (
    <form action={run}>
      <button
        className="rounded bg-emerald-500/80 px-2 py-1 text-xs text-white hover:bg-emerald-400"
        type="submit"
      >
        Approve
      </button>
    </form>
  );
}

function DenyForm({ orgId, kind }: { orgId: string; kind: string }) {
  async function run(formData: FormData) {
    "use server";
    const reason = String(formData.get("reason") ?? "operator denied");
    await denyTemplateAction(orgId, kind, reason);
  }
  return (
    <form action={run} className="inline-flex items-center gap-1">
      <input
        name="reason"
        defaultValue="operator denied"
        className="rounded border border-white/10 bg-black/20 px-1 py-0.5 text-xs"
      />
      <button
        className="rounded bg-rose-500/80 px-2 py-1 text-xs text-white hover:bg-rose-400"
        type="submit"
      >
        Deny
      </button>
    </form>
  );
}

function AdoptForm({ orgId, kind }: { orgId: string; kind: string }) {
  async function run() {
    "use server";
    await adoptTemplateAction(orgId, kind);
  }
  return (
    <form action={run}>
      <button
        className="rounded bg-blue-500 px-2 py-1 text-xs text-white hover:bg-blue-400"
        type="submit"
      >
        Adopt
      </button>
    </form>
  );
}

function RevokeForm({ orgId, kind }: { orgId: string; kind: string }) {
  async function run(formData: FormData) {
    "use server";
    const reason = String(formData.get("reason") ?? "operator revoked");
    await revokeTemplateAction(orgId, kind, reason);
  }
  return (
    <form action={run} className="inline-flex items-center gap-1">
      <input
        name="reason"
        defaultValue="operator revoked"
        className="rounded border border-white/10 bg-black/20 px-1 py-0.5 text-xs"
      />
      <button
        className="rounded bg-amber-500/80 px-2 py-1 text-xs text-white hover:bg-amber-400"
        type="submit"
      >
        Revoke
      </button>
    </form>
  );
}
