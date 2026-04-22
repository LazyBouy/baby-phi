// Single-org detail — dashboard-preview shape. The real per-panel
// dashboard lives at /organizations/[id]/dashboard in M3/P5.

import Link from "next/link";

import { showOrgAction } from "../actions";

export default async function OrgDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const result = await showOrgAction(id);

  if (!result.ok) {
    return (
      <section className="space-y-4 p-6">
        <h1 className="text-xl font-semibold">Organization</h1>
        <div className="rounded border border-red-500/40 bg-red-500/10 p-3 text-sm">
          {result.code}: {result.message}
        </div>
        <Link href="/organizations" className="text-sm underline">
          ← Back to organizations
        </Link>
      </section>
    );
  }

  const { organization, member_count, project_count, adopted_template_count } =
    result.value;

  return (
    <section className="space-y-4 p-6">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold">{organization.display_name}</h1>
          <div className="font-mono text-xs opacity-60">{organization.id}</div>
        </div>
        <Link href="/organizations" className="text-sm underline">
          ← Back
        </Link>
      </header>

      {(organization.vision || organization.mission) && (
        <div className="grid gap-3 md:grid-cols-2">
          {organization.vision && (
            <div className="rounded border border-white/10 bg-black/20 p-3">
              <div className="text-xs opacity-70">Vision</div>
              <div className="text-sm">{organization.vision}</div>
            </div>
          )}
          {organization.mission && (
            <div className="rounded border border-white/10 bg-black/20 p-3">
              <div className="text-xs opacity-70">Mission</div>
              <div className="text-sm">{organization.mission}</div>
            </div>
          )}
        </div>
      )}

      <div className="grid gap-3 md:grid-cols-3">
        <SummaryCard label="Members" value={member_count} />
        <SummaryCard label="Projects" value={project_count} />
        <SummaryCard
          label="Adopted Templates"
          value={adopted_template_count}
        />
      </div>

      <div className="rounded border border-white/10 bg-black/20 p-3 text-sm">
        <div className="mb-1 text-xs opacity-70">Policies</div>
        <div>Consent: {organization.consent_policy}</div>
        <div>Default audit class: {organization.audit_class_default}</div>
        <div>
          Templates:{" "}
          {organization.authority_templates_enabled.join(", ") || "—"}
        </div>
      </div>

      {organization.defaults_snapshot && (
        <details className="rounded border border-white/10 bg-black/20 p-3 text-sm">
          <summary className="cursor-pointer opacity-70">
            Frozen defaults snapshot (phi-core fields)
          </summary>
          <pre className="mt-2 overflow-auto text-xs">
            {JSON.stringify(organization.defaults_snapshot, null, 2)}
          </pre>
        </details>
      )}
    </section>
  );
}

function SummaryCard({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded border border-white/10 bg-black/20 p-3">
      <div className="text-xs opacity-70">{label}</div>
      <div className="text-2xl font-semibold">{value}</div>
    </div>
  );
}
