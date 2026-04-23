// Project creation wizard — M4/P6 admin page 10.
//
// Six logical sections, submitted in one Server Action round-trip:
//
//  1. Identity      — name, description, goal
//  2. Shape         — Shape A (single-org, immediate) vs Shape B
//                     (co-owned, two-approver). Co-owner org picker
//                     appears only when Shape B selected.
//  3. Leads + members — lead_agent_id, member_ids (csv), sponsor_ids
//  4. OKRs          — inline editor; validated server-side.
//  5. Governance    — token_budget (optional, per-project cap).
//  6. Review & submit — shape-sensitive footer. Shape A shows
//                       "will be created immediately"; Shape B shows
//                       the pending-approval notice with the two
//                       approver handles once submit completes.
//
// The form follows the same "one big Server Action" pattern M3/P4's
// org-creation page uses — no client-side stepper component (that's
// an M6+ refinement). The 6 sections are visual grouping, not a
// stepper.

import { redirect } from "next/navigation";
import Link from "next/link";

import { createProjectAction } from "./actions";
import type {
  CreateProjectBody,
  ObjectiveWire,
  KeyResultWire,
  MeasurementTypeWire,
  OkrValueWire,
  ProjectShapeWire,
} from "@/lib/api/projects";

function readStr(fd: FormData, key: string): string | null {
  const v = fd.get(key);
  if (typeof v !== "string") return null;
  const t = v.trim();
  return t.length ? t : null;
}

function readNum(fd: FormData, key: string): number | null {
  const s = readStr(fd, key);
  if (s === null) return null;
  const n = Number(s);
  return Number.isFinite(n) ? n : null;
}

function splitCsv(s: string | null): string[] {
  return (s ?? "")
    .split(",")
    .map((x) => x.trim())
    .filter((x) => x.length > 0);
}

function parseOkrPayload(
  raw: string | null,
): { objectives: ObjectiveWire[]; key_results: KeyResultWire[] } {
  if (!raw) return { objectives: [], key_results: [] };
  try {
    const v = JSON.parse(raw);
    const objectives = Array.isArray(v.objectives) ? v.objectives : [];
    const key_results = Array.isArray(v.key_results) ? v.key_results : [];
    return { objectives, key_results };
  } catch {
    return { objectives: [], key_results: [] };
  }
}

// Ensure unused wire-type imports still typecheck — web bundler tree-shakes the void binding.
const _wireTypeAnchor: {
  m?: MeasurementTypeWire;
  o?: OkrValueWire;
} = {};
void _wireTypeAnchor;

export default async function CreateProjectPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id: orgId } = await params;

  async function onSubmit(fd: FormData): Promise<void> {
    "use server";
    const shape = (readStr(fd, "shape") ?? "shape_a") as ProjectShapeWire;
    const coOwner =
      shape === "shape_b" ? readStr(fd, "co_owner_org_id") : null;

    const okrJson = readStr(fd, "okrs_json");
    const { objectives, key_results } = parseOkrPayload(okrJson);

    const body: CreateProjectBody = {
      project_id:
        readStr(fd, "project_id") ?? crypto.randomUUID(),
      name: readStr(fd, "name") ?? "",
      description: readStr(fd, "description") ?? "",
      goal: readStr(fd, "goal"),
      shape,
      co_owner_org_id: coOwner,
      lead_agent_id: readStr(fd, "lead_agent_id") ?? "",
      member_agent_ids: splitCsv(readStr(fd, "member_ids")),
      sponsor_agent_ids: splitCsv(readStr(fd, "sponsor_ids")),
      token_budget: readNum(fd, "token_budget"),
      objectives,
      key_results,
    };

    const result = await createProjectAction(orgId, body);
    if (result.ok) {
      if (result.value.outcome === "materialised") {
        // Shape A → project page (page 11 lands at M4/P7; for now the
        // dashboard is the nearest populated target).
        redirect(`/organizations/${orgId}/dashboard`);
      } else {
        // Shape B → we stay on a confirmation route. For now we redirect
        // to the dashboard with a notice; a dedicated "pending projects"
        // panel is M4/P8 dashboard-rewrite work.
        redirect(`/organizations/${orgId}/dashboard?pending_project=${result.value.pending_ar_id}`);
      }
    }
    // On failure the form re-renders untouched; a `useFormState`-based
    // error banner is a future refinement.
  }

  return (
    <section className="space-y-4 p-6">
      <header className="flex items-center justify-between">
        <h1 className="text-xl font-semibold">Create project</h1>
        <Link
          href={`/organizations/${orgId}/dashboard`}
          className="text-sm underline"
        >
          ← Dashboard
        </Link>
      </header>
      <p className="text-sm opacity-70">
        Page 10 (M4/P6). <code>Shape A</code> creates the project
        immediately. <code>Shape B</code> opens a pending
        2-approver Auth Request; both co-owner admins must approve before
        the project materialises (per ADR-0025).
      </p>

      <form action={onSubmit} className="space-y-4">
        {/* Section 1 — Identity */}
        <fieldset className="space-y-2 rounded border border-white/10 p-4">
          <legend className="px-2 text-sm opacity-80">1. Identity</legend>
          <label className="block text-sm">
            <span className="block opacity-70">Name *</span>
            <input
              name="name"
              required
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
            />
          </label>
          <label className="block text-sm">
            <span className="block opacity-70">Description</span>
            <textarea
              name="description"
              rows={2}
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
            />
          </label>
          <label className="block text-sm">
            <span className="block opacity-70">Goal (one-line)</span>
            <input
              name="goal"
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
            />
          </label>
        </fieldset>

        {/* Section 2 — Shape */}
        <fieldset className="space-y-2 rounded border border-white/10 p-4">
          <legend className="px-2 text-sm opacity-80">2. Shape</legend>
          <div className="flex flex-col gap-2 text-sm">
            <label className="flex items-center gap-2">
              <input
                type="radio"
                name="shape"
                value="shape_a"
                defaultChecked
              />
              Shape A — single-org, immediate materialisation
            </label>
            <label className="flex items-center gap-2">
              <input type="radio" name="shape" value="shape_b" />
              Shape B — co-owned, two-approver flow (ADR-0025)
            </label>
          </div>
          <label className="block text-sm">
            <span className="block opacity-70">
              Co-owner org id (Shape B only)
            </span>
            <input
              name="co_owner_org_id"
              placeholder="leave blank for Shape A"
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1 font-mono text-xs"
            />
          </label>
        </fieldset>

        {/* Section 3 — Leads + members */}
        <fieldset className="space-y-2 rounded border border-white/10 p-4">
          <legend className="px-2 text-sm opacity-80">
            3. Leads &amp; members
          </legend>
          <label className="block text-sm">
            <span className="block opacity-70">Lead agent id *</span>
            <input
              name="lead_agent_id"
              required
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1 font-mono text-xs"
            />
          </label>
          <label className="block text-sm">
            <span className="block opacity-70">
              Members (csv of agent ids)
            </span>
            <input
              name="member_ids"
              placeholder="uuid1, uuid2"
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1 text-xs"
            />
          </label>
          <label className="block text-sm">
            <span className="block opacity-70">
              Sponsors (csv of agent ids — typically an org&apos;s CEO)
            </span>
            <input
              name="sponsor_ids"
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1 text-xs"
            />
          </label>
        </fieldset>

        {/* Section 4 — OKRs */}
        <fieldset className="space-y-2 rounded border border-white/10 p-4">
          <legend className="px-2 text-sm opacity-80">
            4. OKRs (optional at creation)
          </legend>
          <p className="text-xs opacity-70">
            Paste a JSON object{" "}
            <code>{`{"objectives": [...], "key_results": [...]}`}</code>{" "}
            matching the domain Objective + KeyResult shape. Values are
            validated server-side (measurement-type vs value shape
            checks). In-place OKR editing lands on page 11 (M4/P7).
          </p>
          <textarea
            name="okrs_json"
            rows={4}
            className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1 font-mono text-xs"
            placeholder='{"objectives":[{"objective_id":"obj-1","name":"Ship feature","description":"","status":"draft","owner":"<uuid>"}],"key_results":[]}'
          />
        </fieldset>

        {/* Section 5 — Governance */}
        <fieldset className="space-y-2 rounded border border-white/10 p-4">
          <legend className="px-2 text-sm opacity-80">
            5. Governance (optional)
          </legend>
          <label className="block text-sm">
            <span className="block opacity-70">
              Token budget (per-project cap)
            </span>
            <input
              name="token_budget"
              type="number"
              min="0"
              className="mt-1 w-full rounded border border-white/20 bg-black/10 px-2 py-1"
            />
          </label>
        </fieldset>

        {/* Section 6 — Review + submit */}
        <fieldset className="space-y-2 rounded border border-white/10 p-4">
          <legend className="px-2 text-sm opacity-80">
            6. Review &amp; submit
          </legend>
          <p className="text-xs opacity-70">
            Shape A submits create the project immediately. Shape B
            submits create a pending Auth Request with two approver
            slots (one per co-owner); both co-owner admins must drive
            their slot via{" "}
            <code>phi project approve-pending</code> before the project
            materialises. The 4-outcome decision matrix (both-approve /
            both-deny / mixed A-D / mixed D-A) is pinned by the
            <code>shape_b_approval_matrix_props</code> proptest at the
            domain tier.
          </p>
        </fieldset>

        <button
          type="submit"
          className="rounded bg-blue-500 px-3 py-2 text-sm font-medium text-white hover:bg-blue-400"
        >
          Create project
        </button>
      </form>
    </section>
  );
}
