// Template adoption table for M5 page 12 (Authority Template Adoption).
//
// Renders the 4-bucket list (pending / active / revoked / available)
// the `GET /api/v0/orgs/:org_id/authority-templates` endpoint returns
// at M5/P5. Server Component — no client state.
//
// At M5/P1 this ships as a presentation-only primitive; the HTTP
// handler + adoption actions (approve / deny / adopt / revoke) land
// at M5/P5.

export type TemplateKind = "A" | "B" | "C" | "D" | "E";

export type TemplateAdoptionRow = {
  kind: TemplateKind;
  name: string;
  state: "pending" | "active" | "revoked" | "available";
  /// Adoption AR id (pending / active / revoked rows); absent for
  /// `available` rows (no AR yet).
  ar_id?: string;
  /// Number of grants still in force that descend from this
  /// adoption's AR. Present for `active` rows; surfaces at the
  /// revoke-cascade confirm dialog to warn operators.
  active_grant_count?: number;
};

export function TemplateAdoptionTable({
  rows,
}: {
  rows: TemplateAdoptionRow[];
}) {
  const byState = {
    pending: rows.filter((r) => r.state === "pending"),
    active: rows.filter((r) => r.state === "active"),
    revoked: rows.filter((r) => r.state === "revoked"),
    available: rows.filter((r) => r.state === "available"),
  };
  return (
    <div className="space-y-6">
      {(["pending", "active", "revoked", "available"] as const).map(
        (bucket) => (
          <section key={bucket}>
            <header className="mb-2 text-xs uppercase tracking-wider opacity-60">
              {bucket} ({byState[bucket].length})
            </header>
            {byState[bucket].length === 0 ? (
              <div className="text-xs opacity-40">none</div>
            ) : (
              <ul className="divide-y divide-gray-500/20">
                {byState[bucket].map((row) => (
                  <li
                    key={`${row.kind}-${row.ar_id ?? "available"}`}
                    className="flex items-center justify-between py-2"
                  >
                    <div>
                      <span className="font-mono opacity-60">
                        Template {row.kind}
                      </span>
                      <span className="ml-2">{row.name}</span>
                    </div>
                    {row.state === "active" &&
                    row.active_grant_count !== undefined ? (
                      <span className="text-xs opacity-60">
                        {row.active_grant_count} active grant
                        {row.active_grant_count === 1 ? "" : "s"}
                      </span>
                    ) : null}
                  </li>
                ))}
              </ul>
            )}
          </section>
        ),
      )}
    </div>
  );
}
