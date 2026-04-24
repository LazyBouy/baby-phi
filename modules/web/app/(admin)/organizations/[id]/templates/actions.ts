// Server Actions for the Authority Templates page (M5/P7 — admin/12).

"use server";

import { revalidatePath } from "next/cache";

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import {
  adoptTemplateApi,
  approveTemplateApi,
  denyTemplateApi,
  listTemplatesApi,
  revokeTemplateApi,
  type AdoptOutcomeWire,
  type ApiResult,
  type ApproveOutcomeWire,
  type DenyOutcomeWire,
  type RevokeOutcomeWire,
  type TemplatesListingWire,
} from "@/lib/api/templates";

export type ActionError = {
  ok: false;
  httpStatus: number;
  code: string;
  message: string;
};
export type ActionOk<T> = { ok: true; value: T };

async function headers(): Promise<Record<string, string>> {
  return await forwardSessionCookieHeader();
}

function toResult<T>(r: ApiResult<T>): ActionOk<T> | ActionError {
  return r.ok ? { ok: true, value: r.value } : (r as ActionError);
}

export async function listTemplatesAction(
  orgId: string,
): Promise<ActionOk<TemplatesListingWire> | ActionError> {
  return toResult(await listTemplatesApi(await headers(), orgId));
}

export async function approveTemplateAction(
  orgId: string,
  kind: string,
): Promise<ActionOk<ApproveOutcomeWire> | ActionError> {
  const r = toResult(await approveTemplateApi(await headers(), orgId, kind));
  if (r.ok) revalidatePath(`/organizations/${orgId}/templates`);
  return r;
}

export async function denyTemplateAction(
  orgId: string,
  kind: string,
  reason: string,
): Promise<ActionOk<DenyOutcomeWire> | ActionError> {
  const r = toResult(
    await denyTemplateApi(await headers(), orgId, kind, reason),
  );
  if (r.ok) revalidatePath(`/organizations/${orgId}/templates`);
  return r;
}

export async function adoptTemplateAction(
  orgId: string,
  kind: string,
): Promise<ActionOk<AdoptOutcomeWire> | ActionError> {
  const r = toResult(await adoptTemplateApi(await headers(), orgId, kind));
  if (r.ok) revalidatePath(`/organizations/${orgId}/templates`);
  return r;
}

export async function revokeTemplateAction(
  orgId: string,
  kind: string,
  reason: string,
): Promise<ActionOk<RevokeOutcomeWire> | ActionError> {
  const r = toResult(
    await revokeTemplateApi(await headers(), orgId, kind, reason),
  );
  if (r.ok) revalidatePath(`/organizations/${orgId}/templates`);
  return r;
}
