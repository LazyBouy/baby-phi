// Server Actions for the session-launch page (M5/P7 — admin/14).

"use server";

import { revalidatePath } from "next/cache";

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import {
  launchSessionApi,
  previewSessionApi,
  type ApiResult,
  type LaunchResponseWire,
  type PreviewResponseWire,
} from "@/lib/api/sessions";

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

export async function previewSessionAction(
  orgId: string,
  projectId: string,
  agentId: string,
): Promise<ActionOk<PreviewResponseWire> | ActionError> {
  return toResult(
    await previewSessionApi(await headers(), orgId, projectId, agentId),
  );
}

export async function launchSessionAction(
  orgId: string,
  projectId: string,
  agentId: string,
  prompt: string,
): Promise<ActionOk<LaunchResponseWire> | ActionError> {
  const r = toResult(
    await launchSessionApi(await headers(), orgId, projectId, {
      agent_id: agentId,
      prompt,
    }),
  );
  if (r.ok) {
    revalidatePath(`/organizations/${orgId}/projects/${projectId}`);
  }
  return r;
}
