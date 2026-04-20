"use client";

// Client-side wrapper around the bootstrap-claim form. Uses
// `useFormState` + `useFormStatus` from `react-dom` (React 18 /
// Next 14.2 idiom) so the Server Action result rerenders inline
// without a page refresh.

import { useFormState, useFormStatus } from "react-dom";

import { submitClaim, ClaimActionState } from "./actions";

const initialState: ClaimActionState = { kind: "idle" };

export default function ClaimForm() {
  const [state, formAction] = useFormState<ClaimActionState, FormData>(
    submitClaim,
    initialState,
  );

  if (state.kind === "success") {
    return (
      <section className="rounded border border-emerald-500/40 bg-emerald-500/5 p-6">
        <h2 className="text-lg font-medium">Claim succeeded</h2>
        <p className="mt-2 text-sm opacity-80">
          You are now the platform admin. Save these identifiers — they anchor
          the audit trail.
        </p>
        <dl className="mt-4 space-y-1 text-xs font-mono">
          <div>
            <dt className="inline opacity-60">human_agent_id: </dt>
            <dd className="inline">{state.humanAgentId}</dd>
          </div>
          <div>
            <dt className="inline opacity-60">grant_id: </dt>
            <dd className="inline">{state.grantId}</dd>
          </div>
          <div>
            <dt className="inline opacity-60">audit_event_id: </dt>
            <dd className="inline">{state.auditEventId}</dd>
          </div>
        </dl>
        <p className="mt-4 text-sm opacity-70">
          Next step: the M2 platform-admin journey (model-provider
          registration) — lands in a subsequent milestone.
        </p>
      </section>
    );
  }

  return (
    <form
      action={formAction}
      className="space-y-4 rounded border border-white/10 p-6"
    >
      <FormField
        label="Bootstrap credential"
        name="credential"
        placeholder="bphi-bootstrap-…"
        required
        autoComplete="off"
        monospace
      />
      <FormField
        label="Display name"
        name="displayName"
        placeholder="Alex Chen"
        required
      />
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <label className="block sm:col-span-1">
          <span className="block text-sm opacity-70">Channel</span>
          <select
            name="channelKind"
            className="mt-1 w-full rounded border border-white/20 bg-transparent px-2 py-2"
            defaultValue="slack"
          >
            <option value="slack">Slack</option>
            <option value="email">Email</option>
            <option value="web">Web</option>
          </select>
        </label>
        <div className="sm:col-span-2">
          <FormField
            label="Channel handle"
            name="channelHandle"
            placeholder="@alex"
            required
          />
        </div>
      </div>

      {state.kind === "validation" && (
        <Alert tone="warn">Invalid input: {state.message}</Alert>
      )}
      {state.kind === "rejected" && (
        <Alert tone="error">
          {state.code} ({state.httpStatus}): {state.message}
        </Alert>
      )}
      {state.kind === "error" && (
        <Alert tone="error">Request failed: {state.message}</Alert>
      )}

      <SubmitButton />
    </form>
  );
}

function SubmitButton() {
  const { pending } = useFormStatus();
  return (
    <button
      type="submit"
      disabled={pending}
      className="rounded bg-emerald-500 px-4 py-2 font-medium text-black transition disabled:opacity-50"
    >
      {pending ? "Claiming…" : "Claim platform admin"}
    </button>
  );
}

type FormFieldProps = {
  label: string;
  name: string;
  placeholder?: string;
  required?: boolean;
  autoComplete?: string;
  monospace?: boolean;
};

function FormField(props: FormFieldProps) {
  return (
    <label className="block">
      <span className="block text-sm opacity-70">{props.label}</span>
      <input
        name={props.name}
        type="text"
        placeholder={props.placeholder}
        required={props.required}
        autoComplete={props.autoComplete}
        className={`mt-1 w-full rounded border border-white/20 bg-transparent px-2 py-2 ${
          props.monospace ? "font-mono text-sm" : ""
        }`}
      />
    </label>
  );
}

type AlertProps = { tone: "warn" | "error"; children: React.ReactNode };

function Alert({ tone, children }: AlertProps) {
  const color =
    tone === "error"
      ? "border-red-500/40 bg-red-500/5"
      : "border-amber-500/40 bg-amber-500/5";
  return <div className={`rounded border ${color} p-3 text-sm`}>{children}</div>;
}
