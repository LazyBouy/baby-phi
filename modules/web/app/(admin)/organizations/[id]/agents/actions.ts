// Server Actions for the agents roster + edit pages (M4/P4 + M4/P5).

"use server";

import { revalidatePath } from "next/cache";

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import {
  createAgentApi,
  listAgentsApi,
  revertExecutionLimitsOverrideApi,
  updateAgentProfileApi,
  type ApiResult,
  type CreateAgentBody,
  type CreateAgentResponseWire,
  type ListAgentsQuery,
  type ListAgentsWire,
  type RevertLimitsResponseWire,
  type UpdateAgentProfileBody,
  type UpdateAgentProfileResponseWire,
} from "@/lib/api/agents";

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

export async function listAgentsAction(
  orgId: string,
  q?: ListAgentsQuery,
): Promise<ActionOk<ListAgentsWire> | ActionError> {
  return toResult(await listAgentsApi(await headers(), orgId, q));
}

export async function createAgentAction(
  orgId: string,
  body: CreateAgentBody,
): Promise<ActionOk<CreateAgentResponseWire> | ActionError> {
  const r = await createAgentApi(await headers(), orgId, body);
  const result = toResult(r);
  if (result.ok) revalidatePath(`/organizations/${orgId}/agents`);
  return result;
}

export async function updateAgentProfileAction(
  orgId: string,
  agentId: string,
  body: UpdateAgentProfileBody,
): Promise<ActionOk<UpdateAgentProfileResponseWire> | ActionError> {
  const r = await updateAgentProfileApi(await headers(), agentId, body);
  const result = toResult(r);
  if (result.ok) {
    revalidatePath(`/organizations/${orgId}/agents`);
    revalidatePath(`/organizations/${orgId}/agents/${agentId}`);
  }
  return result;
}

export async function revertExecutionLimitsOverrideAction(
  orgId: string,
  agentId: string,
): Promise<ActionOk<RevertLimitsResponseWire> | ActionError> {
  const r = await revertExecutionLimitsOverrideApi(await headers(), agentId);
  const result = toResult(r);
  if (result.ok) {
    revalidatePath(`/organizations/${orgId}/agents/${agentId}`);
  }
  return result;
}
