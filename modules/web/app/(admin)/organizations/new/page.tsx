// 8-step Organization-Creation Wizard (M3/P4 page 06).
//
// Each step is a minimal form slice that reads + writes a shared
// `DraftState` via `useDraft`. Step 8 (Review) assembles the full
// wire payload and calls the server action.
//
// ## phi-core leverage (web tier)
//
// Q1 **none**: no `import phi_core` anywhere (it's a Rust crate).
// Q2 **yes, via opaque serialisation**: `defaults_snapshot_override`
// on the wire body carries phi-core-wrapped fields as
// `Record<string, unknown>`; the wizard never re-specifies the
// phi-core schema. For M3 the default path leaves the override
// undefined (server snapshot-copies platform defaults per ADR-0019);
// advanced-mode expansion happens M3.5+.
// Q3: phi-core has no web tier; nothing to reuse by import.

"use client";

import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";

import { DraftProvider, useDraft } from "@/app/components/wizard/DraftContext";
import { ReviewDiff } from "@/app/components/wizard/ReviewDiff";
import { StepNav } from "@/app/components/wizard/StepNav";
import { StepShell } from "@/app/components/wizard/StepShell";
import type {
  AuditClassWire,
  ChannelKindWire,
  ConsentPolicyWire,
  CreateOrgBody,
  TemplateKindWire,
} from "@/lib/api/orgs";

import { createOrgAction } from "../actions";

// ---------------------------------------------------------------------------
// Shared draft type — the wizard's single source of truth.
// ---------------------------------------------------------------------------

const DRAFT_KEY = "m3.org-create.draft";
const STEP_COUNT = 8;

type DraftState = {
  // Step 1
  display_name: string;
  vision: string;
  mission: string;
  // Step 2
  consent_policy: ConsentPolicyWire;
  // Step 3
  audit_class_default: AuditClassWire;
  // Step 4
  templates: TemplateKindWire[];
  // Step 5
  default_model_provider: string; // empty string = null on wire
  // Step 6
  token_budget: number;
  // Step 7
  ceo_display_name: string;
  ceo_channel_kind: ChannelKindWire;
  ceo_channel_handle: string;
};

const INITIAL: DraftState = {
  display_name: "",
  vision: "",
  mission: "",
  consent_policy: "implicit",
  audit_class_default: "logged",
  templates: [],
  default_model_provider: "",
  token_budget: 1_000_000,
  ceo_display_name: "",
  ceo_channel_kind: "email",
  ceo_channel_handle: "",
};

function toWireBody(d: DraftState): CreateOrgBody {
  return {
    display_name: d.display_name.trim(),
    vision: d.vision.trim() || null,
    mission: d.mission.trim() || null,
    consent_policy: d.consent_policy,
    audit_class_default: d.audit_class_default,
    authority_templates_enabled: d.templates,
    default_model_provider:
      d.default_model_provider.trim() === "" ? null : d.default_model_provider,
    ceo_display_name: d.ceo_display_name.trim(),
    ceo_channel_kind: d.ceo_channel_kind,
    ceo_channel_handle: d.ceo_channel_handle.trim(),
    token_budget: d.token_budget,
  };
}

// Which fields are required to proceed from each step.
function canProceed(step: number, d: DraftState): boolean {
  switch (step) {
    case 1:
      return d.display_name.trim().length > 0;
    case 6:
      return d.token_budget > 0;
    case 7:
      return (
        d.ceo_display_name.trim().length > 0 &&
        d.ceo_channel_handle.trim().length > 0
      );
    default:
      return true;
  }
}

// ---------------------------------------------------------------------------
// Outer orchestrator
// ---------------------------------------------------------------------------

export default function OrgCreationWizardPage() {
  return (
    <DraftProvider>
      <WizardInner />
    </DraftProvider>
  );
}

function WizardInner() {
  const router = useRouter();
  const [draft, setDraft, clearDraft] = useDraft<DraftState>(DRAFT_KEY, INITIAL);
  const [step, setStep] = useState(1);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<{
    ok: false;
    httpStatus: number;
    code: string;
    message: string;
  } | null>(null);

  const wireBody = useMemo(() => toWireBody(draft), [draft]);

  const onSubmit = async () => {
    setPending(true);
    setError(null);
    const result = await createOrgAction(wireBody);
    setPending(false);
    if (!result.ok) {
      setError(result);
      return;
    }
    clearDraft();
    router.push(`/organizations/${result.value.org_id}`);
  };

  return (
    <section className="mx-auto max-w-2xl space-y-6 p-6">
      {step === 1 && (
        <Step1Identity draft={draft} setDraft={setDraft} error={error} />
      )}
      {step === 2 && (
        <Step2Consent draft={draft} setDraft={setDraft} error={error} />
      )}
      {step === 3 && (
        <Step3AuditClass draft={draft} setDraft={setDraft} error={error} />
      )}
      {step === 4 && (
        <Step4Templates draft={draft} setDraft={setDraft} error={error} />
      )}
      {step === 5 && (
        <Step5Catalogue draft={draft} setDraft={setDraft} error={error} />
      )}
      {step === 6 && (
        <Step6Budget draft={draft} setDraft={setDraft} error={error} />
      )}
      {step === 7 && (
        <Step7Ceo draft={draft} setDraft={setDraft} error={error} />
      )}
      {step === 8 && (
        <Step8Review draft={draft} wire={wireBody} error={error} />
      )}

      <StepNav
        stepIndex={step}
        stepCount={STEP_COUNT}
        canProceed={canProceed(step, draft)}
        pending={pending}
        onBack={() => setStep((s) => Math.max(1, s - 1))}
        onNext={() => setStep((s) => Math.min(STEP_COUNT, s + 1))}
        onSubmit={onSubmit}
        onSaveDraft={() => setDraft(draft)}
      />
    </section>
  );
}

// ---------------------------------------------------------------------------
// Per-step components — compact field groups; each reads a slice of
// the shared draft. Kept small so the wizard page stays readable.
// ---------------------------------------------------------------------------

type StepProps = {
  draft: DraftState;
  setDraft: (next: DraftState) => void;
  error:
    | { ok: false; httpStatus: number; code: string; message: string }
    | null;
};

function Step1Identity({ draft, setDraft, error }: StepProps) {
  return (
    <StepShell
      title="Identity"
      subtitle="Operator-facing name + (optional) vision/mission."
      stepIndex={1}
      stepCount={STEP_COUNT}
      error={error}
    >
      <LabeledInput
        label="Display name"
        required
        value={draft.display_name}
        onChange={(v) => setDraft({ ...draft, display_name: v })}
      />
      <LabeledInput
        label="Vision"
        value={draft.vision}
        onChange={(v) => setDraft({ ...draft, vision: v })}
      />
      <LabeledInput
        label="Mission"
        value={draft.mission}
        onChange={(v) => setDraft({ ...draft, mission: v })}
      />
    </StepShell>
  );
}

function Step2Consent({ draft, setDraft, error }: StepProps) {
  return (
    <StepShell
      title="Consent policy"
      subtitle="How cross-session data access works for members."
      stepIndex={2}
      stepCount={STEP_COUNT}
      error={error}
    >
      <LabeledSelect
        label="Consent policy"
        value={draft.consent_policy}
        options={[
          ["implicit", "Implicit — inherited from org membership"],
          ["one_time", "One-time — explicit consent once per member"],
          ["per_session", "Per-session — explicit consent each session"],
        ]}
        onChange={(v) =>
          setDraft({ ...draft, consent_policy: v as ConsentPolicyWire })
        }
      />
    </StepShell>
  );
}

function Step3AuditClass({ draft, setDraft, error }: StepProps) {
  return (
    <StepShell
      title="Default audit class"
      subtitle="Tier applied when an event has no explicit class."
      stepIndex={3}
      stepCount={STEP_COUNT}
      error={error}
    >
      <LabeledSelect
        label="Audit class default"
        value={draft.audit_class_default}
        options={[
          ["silent", "Silent — write-only, no alert"],
          ["logged", "Logged — durable + queryable"],
          ["alerted", "Alerted — delivered to org alert channels"],
        ]}
        onChange={(v) =>
          setDraft({ ...draft, audit_class_default: v as AuditClassWire })
        }
      />
    </StepShell>
  );
}

function Step4Templates({ draft, setDraft, error }: StepProps) {
  const toggle = (k: TemplateKindWire) => {
    const has = draft.templates.includes(k);
    setDraft({
      ...draft,
      templates: has
        ? draft.templates.filter((x) => x !== k)
        : [...draft.templates, k],
    });
  };
  return (
    <StepShell
      title="Authority templates"
      subtitle="Which A/B/C/D lifecycle patterns to adopt at creation."
      stepIndex={4}
      stepCount={STEP_COUNT}
      error={error}
    >
      {(["a", "b", "c", "d"] as const).map((k) => (
        <label key={k} className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={draft.templates.includes(k)}
            onChange={() => toggle(k)}
          />
          Template {k.toUpperCase()}
        </label>
      ))}
    </StepShell>
  );
}

function Step5Catalogue({ draft, setDraft, error }: StepProps) {
  return (
    <StepShell
      title="Default model provider"
      subtitle="Optional. Leave blank to inherit platform default at invoke time."
      stepIndex={5}
      stepCount={STEP_COUNT}
      error={error}
    >
      <LabeledInput
        label="Model provider id (UUID, or blank)"
        value={draft.default_model_provider}
        onChange={(v) => setDraft({ ...draft, default_model_provider: v })}
      />
    </StepShell>
  );
}

function Step6Budget({ draft, setDraft, error }: StepProps) {
  return (
    <StepShell
      title="Token budget"
      subtitle="Initial allocation for the org's token budget pool."
      stepIndex={6}
      stepCount={STEP_COUNT}
      error={error}
    >
      <LabeledInput
        label="Initial allocation (tokens)"
        type="number"
        value={String(draft.token_budget)}
        onChange={(v) =>
          setDraft({ ...draft, token_budget: Math.max(0, Number(v) || 0) })
        }
      />
    </StepShell>
  );
}

function Step7Ceo({ draft, setDraft, error }: StepProps) {
  return (
    <StepShell
      title="CEO"
      subtitle="The first human member with root authority over the org."
      stepIndex={7}
      stepCount={STEP_COUNT}
      error={error}
    >
      <LabeledInput
        label="CEO display name"
        required
        value={draft.ceo_display_name}
        onChange={(v) => setDraft({ ...draft, ceo_display_name: v })}
      />
      <LabeledSelect
        label="CEO channel kind"
        value={draft.ceo_channel_kind}
        options={[
          ["email", "Email"],
          ["slack", "Slack"],
          ["web", "Web"],
        ]}
        onChange={(v) =>
          setDraft({ ...draft, ceo_channel_kind: v as ChannelKindWire })
        }
      />
      <LabeledInput
        label="CEO channel handle"
        required
        value={draft.ceo_channel_handle}
        onChange={(v) => setDraft({ ...draft, ceo_channel_handle: v })}
      />
    </StepShell>
  );
}

function Step8Review({
  draft,
  wire,
  error,
}: {
  draft: DraftState;
  wire: CreateOrgBody;
  error: StepProps["error"];
}) {
  return (
    <StepShell
      title="Review"
      subtitle="Everything you entered, in the exact shape the server will receive."
      stepIndex={8}
      stepCount={STEP_COUNT}
      error={error}
    >
      <ReviewDiff
        rows={(
          [
            ["display_name", "display_name"],
            ["vision", "vision"],
            ["mission", "mission"],
            ["consent_policy", "consent_policy"],
            ["audit_class_default", "audit_class_default"],
            ["templates", "templates"],
            ["default_model_provider", "default_model_provider"],
            ["token_budget", "token_budget"],
            ["ceo_display_name", "ceo_display_name"],
            ["ceo_channel_kind", "ceo_channel_kind"],
            ["ceo_channel_handle", "ceo_channel_handle"],
          ] as const
        ).map(([label, key]) => ({
          label,
          expected: INITIAL[key as keyof DraftState],
          current: draft[key as keyof DraftState],
        }))}
      />
      <details className="rounded border border-white/10 bg-black/20 p-3 text-sm">
        <summary className="cursor-pointer opacity-70">
          Wire payload (POST /api/v0/orgs)
        </summary>
        <pre className="mt-2 overflow-auto text-xs">
          {JSON.stringify(wire, null, 2)}
        </pre>
      </details>
    </StepShell>
  );
}

// ---------------------------------------------------------------------------
// Minimal field primitives
// ---------------------------------------------------------------------------

function LabeledInput({
  label,
  value,
  onChange,
  type = "text",
  required = false,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  type?: string;
  required?: boolean;
}) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="opacity-70">
        {label}
        {required ? " *" : ""}
      </span>
      <input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="rounded border border-white/20 bg-black/20 px-2 py-1 text-sm"
      />
    </label>
  );
}

function LabeledSelect({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: [string, string][];
  onChange: (v: string) => void;
}) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="opacity-70">{label}</span>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="rounded border border-white/20 bg-black/20 px-2 py-1 text-sm"
      >
        {options.map(([val, label]) => (
          <option key={val} value={val}>
            {label}
          </option>
        ))}
      </select>
    </label>
  );
}
