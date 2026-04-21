// Shared error alert for M2 admin pages. Renders the stable `{code,
// message}` envelope the Rust server emits from `handler_support::ApiError`,
// plus a user-friendly hint when the code is known.
//
// Kept as a Server Component so it can render inside error boundaries
// without needing `"use client"`. M2 page actions pass the error down
// via their props.

import { humanMessageForCode } from "@/lib/api/errors";

export type ApiErrorPayload = {
  code: string;
  message: string;
};

export function ApiErrorAlert({ error }: { error: ApiErrorPayload }) {
  const hint = humanMessageForCode(error.code);
  return (
    <div
      role="alert"
      className="rounded border border-red-500/40 bg-red-500/10 p-4 text-sm"
    >
      <div className="font-mono text-xs uppercase tracking-wider opacity-60">
        {error.code}
      </div>
      <div className="mt-1">{error.message}</div>
      {hint ? (
        <div className="mt-2 text-xs opacity-70">{hint}</div>
      ) : null}
    </div>
  );
}
