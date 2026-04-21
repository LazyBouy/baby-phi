// Server Actions for the credentials-vault page.
//
// Each action:
//   1. Reads the admin's `baby_phi_session` cookie off the current request
//      (via `forwardSessionCookieHeader`).
//   2. Calls the matching `listSecretsApi` / `addSecretApi` / etc.
//   3. Returns a discriminated union the client form can render into
//      the shared `<ApiErrorAlert />` on failure, or revalidate on success.
//
// Keeps zero business logic of its own — the Rust server is the
// source of truth. Pure HTTP pass-through with cookie forwarding.

"use server";

import { revalidatePath } from "next/cache";

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import {
  addSecretApi,
  encodeB64NoPad,
  reassignCustodyApi,
  revealSecretApi,
  rotateSecretApi,
  type AddWire,
  type RevealWire,
  type SecretOpResult,
  type WriteWire,
} from "@/lib/api/secrets";

export type ActionError = {
  ok: false;
  httpStatus: number;
  code: string;
  message: string;
};

export type ActionOk<T> = { ok: true; value: T };

export type AddActionResult = ActionOk<AddWire> | ActionError;
export type RotateActionResult = ActionOk<WriteWire> | ActionError;
export type RevealActionResult = ActionOk<RevealWire> | ActionError;
export type ReassignActionResult = ActionOk<WriteWire> | ActionError;

async function headers(): Promise<Record<string, string>> {
  return await forwardSessionCookieHeader();
}

function toActionResult<T>(r: SecretOpResult<T>): ActionOk<T> | ActionError {
  return r.ok ? { ok: true, value: r.value } : (r as ActionError);
}

export async function addSecretAction(input: {
  slug: string;
  material: string; // plaintext from the form (UTF-8 string)
  sensitive: boolean;
}): Promise<AddActionResult> {
  const encoded = encodeB64NoPad(new TextEncoder().encode(input.material));
  const r = await addSecretApi(await headers(), {
    slug: input.slug,
    material_b64: encoded,
    sensitive: input.sensitive,
  });
  const result = toActionResult(r);
  if (result.ok) revalidatePath("/secrets");
  return result;
}

export async function rotateSecretAction(input: {
  slug: string;
  material: string;
}): Promise<RotateActionResult> {
  const encoded = encodeB64NoPad(new TextEncoder().encode(input.material));
  const r = await rotateSecretApi(await headers(), input.slug, {
    material_b64: encoded,
  });
  const result = toActionResult(r);
  if (result.ok) revalidatePath("/secrets");
  return result;
}

export async function revealSecretAction(input: {
  slug: string;
  justification: string;
}): Promise<RevealActionResult> {
  const r = await revealSecretApi(await headers(), input.slug, {
    justification: input.justification,
  });
  return toActionResult(r);
}

export async function reassignCustodyAction(input: {
  slug: string;
  newCustodianAgentId: string;
}): Promise<ReassignActionResult> {
  const r = await reassignCustodyApi(await headers(), input.slug, {
    new_custodian_agent_id: input.newCustodianAgentId,
  });
  const result = toActionResult(r);
  if (result.ok) revalidatePath("/secrets");
  return result;
}
