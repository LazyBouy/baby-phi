// Server Actions for project detail + in-place OKR patch (M4/P7 — page 11).

"use server";

import { revalidatePath } from "next/cache";

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import {
  getProjectApi,
  patchProjectOkrsApi,
  type ApiResult,
  type OkrPatchBody,
  type OkrPatchResponseWire,
  type ProjectDetailWire,
} from "@/lib/api/projects";

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

export async function getProjectDetailAction(
  projectId: string,
): Promise<ActionOk<ProjectDetailWire> | ActionError> {
  const r = await getProjectApi(await headers(), projectId);
  return toResult(r);
}

export async function patchOkrsAction(
  orgId: string,
  projectId: string,
  body: OkrPatchBody,
): Promise<ActionOk<OkrPatchResponseWire> | ActionError> {
  const r = await patchProjectOkrsApi(await headers(), projectId, body);
  const result = toResult(r);
  if (result.ok) {
    revalidatePath(`/organizations/${orgId}/projects/${projectId}`);
  }
  return result;
}
