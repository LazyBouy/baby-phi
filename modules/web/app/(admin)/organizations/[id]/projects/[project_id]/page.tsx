// Project detail — M4/P7 admin page 11.
//
// Server-rendered detail view. Three panels:
//
//  1. Header           — project identity + owning orgs + lead.
//  2. OKRs             — Objective cards with nested KR rows. The
//                        inline editor is a static JSON textarea
//                        (operator-grade at M4; a richer editor
//                        lands at a UX refinement milestone).
//  3. Roster           — M4 ships lead only (full roster read-back
//                        scoped for a follow-up).
//  4. Recent sessions  — M4 placeholder ("No sessions yet — ships
//                        at M5 / C-M5-3"). Always empty.
//
// ## phi-core leverage
//
// Q1: zero `phi_core` references at the web tier (same as every
// other baby-phi web page). Q2: the `ProjectDetail` wire shape is
// phi-core-stripped by design (no `blueprint` / `execution_limits` /
// `defaults_snapshot`). Q3: `phi_core::Session` → baby-phi's
// governance `Session` node, scoped for M5.

import Link from "next/link";
import { notFound } from "next/navigation";

import { getProjectDetailAction, patchOkrsAction } from "./actions";
import type {
  KeyResultWire,
  ObjectiveWire,
  ProjectDetailWire,
} from "@/lib/api/projects";

export const dynamic = "force-dynamic";

type Params = { params: { id: string; project_id: string } };

function titleForShape(shape: ProjectDetailWire["project"]["shape"]): string {
  switch (shape) {
    case "shape_a":
      return "Shape A · single-org, immediate";
    case "shape_b":
      return "Shape B · co-owned, two-approver";
  }
}

function fmtValue(v: KeyResultWire["target_value"] | null | undefined): string {
  if (!v) return "—";
  switch (v.kind) {
    case "integer":
      return String(v.value);
    case "bool":
      return v.value ? "true" : "false";
    case "percentage":
      return `${Math.round((v.value as number) * 100)}%`;
    case "custom":
      return JSON.stringify(v.value);
  }
}

export default async function ProjectDetailPage({ params }: Params) {
  const r = await getProjectDetailAction(params.project_id);
  if (!r.ok) {
    if (r.httpStatus === 404) notFound();
    return (
      <section style={{ padding: "2rem" }}>
        <h1>Project unavailable</h1>
        <p>
          <strong>{r.code}</strong> · {r.message}
        </p>
        <p>
          <Link href={`/organizations/${params.id}/dashboard`}>
            ← Back to dashboard
          </Link>
        </p>
      </section>
    );
  }

  const detail = r.value;
  const project = detail.project;
  const objectives = project.objectives;
  const keyResults = project.key_results;
  const lead = detail.roster.find((m) => m.project_role === "lead");

  return (
    <section style={{ padding: "2rem", maxWidth: "56rem" }}>
      <p style={{ fontSize: "0.85rem", opacity: 0.7 }}>
        <Link href={`/organizations/${params.id}/dashboard`}>
          ← Back to dashboard
        </Link>
      </p>
      <header style={{ marginBottom: "2rem" }}>
        <h1 style={{ margin: 0 }}>{project.name}</h1>
        <p style={{ margin: "0.25rem 0", opacity: 0.8 }}>
          {titleForShape(project.shape)} · status {project.status}
        </p>
        {project.goal ? (
          <p style={{ margin: "0.75rem 0" }}>
            <strong>Goal:</strong> {project.goal}
          </p>
        ) : null}
        <dl style={{ display: "grid", gridTemplateColumns: "12rem 1fr", gap: "0.25rem", marginTop: "1rem" }}>
          <dt>Project id</dt>
          <dd>
            <code>{project.id}</code>
          </dd>
          <dt>Owning orgs</dt>
          <dd>
            {detail.owning_org_ids.map((o) => (
              <code key={o} style={{ marginRight: "0.5rem" }}>
                {o}
              </code>
            ))}
          </dd>
          <dt>Lead</dt>
          <dd>
            {lead ? (
              <>
                {lead.display_name} · <code>{lead.agent_id}</code>
                {lead.role ? ` · ${lead.role}` : ""}
              </>
            ) : (
              <em>— no lead recorded —</em>
            )}
          </dd>
          <dt>Token budget</dt>
          <dd>
            {project.token_budget ? `${project.tokens_spent} / ${project.token_budget}` : "unset"}
          </dd>
          <dt>Created</dt>
          <dd>{project.created_at}</dd>
        </dl>
      </header>

      <h2>OKRs</h2>
      {objectives.length === 0 ? (
        <p>
          <em>
            No objectives yet. Use <code>phi project update-okrs</code> or
            POST a patch to <code>/api/v0/projects/{project.id}/okrs</code>.
          </em>
        </p>
      ) : (
        <ul style={{ listStyle: "none", padding: 0 }}>
          {objectives.map((obj) => (
            <ObjectiveCard
              key={obj.objective_id}
              objective={obj}
              keyResults={keyResults.filter(
                (k) => k.objective_id === obj.objective_id,
              )}
            />
          ))}
        </ul>
      )}

      <h2>OKR patch (operator-grade)</h2>
      <p style={{ opacity: 0.75 }}>
        Paste a JSON array of patch entries and submit — see{" "}
        <Link
          href="/docs/specs/v0/implementation/m4/architecture/project-detail.md"
        >
          architecture doc
        </Link>{" "}
        for the grammar.
      </p>
      <OkrPatchForm orgId={params.id} projectId={project.id} />

      <h2>Roster</h2>
      <ul>
        {detail.roster.map((m) => (
          <li key={m.agent_id}>
            <strong>{m.display_name}</strong> · {m.kind}
            {m.role ? ` · ${m.role}` : ""} · <em>{m.project_role}</em>
          </li>
        ))}
        {detail.roster.length === 0 ? (
          <li>
            <em>No roster members (the lead surfaces via a dedicated edge)</em>
          </li>
        ) : null}
      </ul>
      <p style={{ fontSize: "0.85rem", opacity: 0.7 }}>
        Full roster read-back (members + sponsors) lands at M5; the edges
        exist today but the dedicated repo method is scoped as a follow-up.
      </p>

      <h2>Recent sessions</h2>
      {detail.recent_sessions.length === 0 ? (
        <p>
          <em>
            No sessions yet. The first session launched via{" "}
            <code>phi session launch</code> will appear here at M5 (see
            C-M5-3 in the base build plan).
          </em>
        </p>
      ) : (
        <ul>
          {detail.recent_sessions.map((s) => (
            <li key={s.session_id}>{s.summary}</li>
          ))}
        </ul>
      )}
    </section>
  );
}

function ObjectiveCard({
  objective,
  keyResults,
}: {
  objective: ObjectiveWire;
  keyResults: KeyResultWire[];
}) {
  return (
    <li
      style={{
        border: "1px solid #ddd",
        borderRadius: "0.5rem",
        padding: "1rem",
        margin: "0.5rem 0",
      }}
    >
      <h3 style={{ margin: 0 }}>{objective.name}</h3>
      <p style={{ margin: "0.25rem 0", opacity: 0.7 }}>
        <code>{objective.objective_id}</code> · status {objective.status}
      </p>
      {objective.description ? <p>{objective.description}</p> : null}
      {keyResults.length === 0 ? (
        <p style={{ fontStyle: "italic", opacity: 0.6 }}>
          No key results yet.
        </p>
      ) : (
        <table style={{ width: "100%", fontSize: "0.9rem" }}>
          <thead>
            <tr>
              <th style={{ textAlign: "left" }}>KR</th>
              <th style={{ textAlign: "left" }}>Measurement</th>
              <th style={{ textAlign: "left" }}>Current</th>
              <th style={{ textAlign: "left" }}>Target</th>
              <th style={{ textAlign: "left" }}>Status</th>
            </tr>
          </thead>
          <tbody>
            {keyResults.map((kr) => (
              <tr key={kr.kr_id}>
                <td>{kr.name}</td>
                <td>{kr.measurement_type}</td>
                <td>{fmtValue(kr.current_value)}</td>
                <td>{fmtValue(kr.target_value)}</td>
                <td>{kr.status}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </li>
  );
}

async function submitPatch(formData: FormData) {
  "use server";
  const orgId = String(formData.get("org_id") ?? "");
  const projectId = String(formData.get("project_id") ?? "");
  const raw = String(formData.get("patch_json") ?? "").trim();
  if (!raw) return;
  let patches: unknown;
  try {
    patches = JSON.parse(raw);
  } catch (_e) {
    return;
  }
  if (!Array.isArray(patches)) return;
  await patchOkrsAction(orgId, projectId, {
    patches: patches as ReturnType<typeof JSON.parse>,
  });
}

function OkrPatchForm({ orgId, projectId }: { orgId: string; projectId: string }) {
  return (
    <form action={submitPatch}>
      <input type="hidden" name="org_id" value={orgId} />
      <input type="hidden" name="project_id" value={projectId} />
      <textarea
        name="patch_json"
        rows={10}
        cols={80}
        defaultValue='[{"kind":"objective","op":"create","payload":{"objective_id":"obj-1","name":"…","description":"","status":"active","owner":"<agent_id>","key_result_ids":[]}}]'
        style={{ display: "block", width: "100%", fontFamily: "monospace" }}
      />
      <button type="submit" style={{ marginTop: "0.5rem" }}>
        Apply patch
      </button>
    </form>
  );
}
