// Server Actions for the System Agents page (M5/P7 — admin/13).

"use server";

import { revalidatePath } from "next/cache";

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import {
  addSystemAgentApi,
  archiveSystemAgentApi,
  disableSystemAgentApi,
  listSystemAgentsApi,
  tuneSystemAgentApi,
  type AddBodyWire,
  type AddOutcomeWire,
  type ApiResult,
  type ArchiveOutcomeWire,
  type DisableOutcomeWire,
  type SystemAgentsListingWire,
  type TuneOutcomeWire,
} from "@/lib/api/system-agents";

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

export async function listSystemAgentsAction(
  orgId: string,
): Promise<ActionOk<SystemAgentsListingWire> | ActionError> {
  return toResult(await listSystemAgentsApi(await headers(), orgId));
}

export async function tuneSystemAgentAction(
  orgId: string,
  agentId: string,
  parallelize: number,
): Promise<ActionOk<TuneOutcomeWire> | ActionError> {
  const r = toResult(
    await tuneSystemAgentApi(await headers(), orgId, agentId, parallelize),
  );
  if (r.ok) revalidatePath(`/organizations/${orgId}/system-agents`);
  return r;
}

export async function addSystemAgentAction(
  orgId: string,
  body: AddBodyWire,
): Promise<ActionOk<AddOutcomeWire> | ActionError> {
  const r = toResult(await addSystemAgentApi(await headers(), orgId, body));
  if (r.ok) revalidatePath(`/organizations/${orgId}/system-agents`);
  return r;
}

export async function disableSystemAgentAction(
  orgId: string,
  agentId: string,
): Promise<ActionOk<DisableOutcomeWire> | ActionError> {
  const r = toResult(
    await disableSystemAgentApi(await headers(), orgId, agentId),
  );
  if (r.ok) revalidatePath(`/organizations/${orgId}/system-agents`);
  return r;
}

export async function archiveSystemAgentAction(
  orgId: string,
  agentId: string,
): Promise<ActionOk<ArchiveOutcomeWire> | ActionError> {
  const r = toResult(
    await archiveSystemAgentApi(await headers(), orgId, agentId),
  );
  if (r.ok) revalidatePath(`/organizations/${orgId}/system-agents`);
  return r;
}
