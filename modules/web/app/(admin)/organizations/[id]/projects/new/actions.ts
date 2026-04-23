// Server Actions for the project-creation wizard (M4/P6 — page 10).

"use server";

import { revalidatePath } from "next/cache";

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import {
  approvePendingProjectApi,
  createProjectApi,
  type ApiResult,
  type ApprovePendingBody,
  type ApprovePendingResponseWire,
  type CreateProjectBody,
  type CreateProjectResponseWire,
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

export async function createProjectAction(
  orgId: string,
  body: CreateProjectBody,
): Promise<ActionOk<CreateProjectResponseWire> | ActionError> {
  const r = await createProjectApi(await headers(), orgId, body);
  const result = toResult(r);
  if (result.ok) {
    revalidatePath(`/organizations/${orgId}/dashboard`);
    if (result.value.outcome === "materialised") {
      revalidatePath(
        `/organizations/${orgId}/projects/${result.value.project_id}`,
      );
    }
  }
  return result;
}

export async function approvePendingProjectAction(
  orgId: string,
  arId: string,
  body: ApprovePendingBody,
): Promise<ActionOk<ApprovePendingResponseWire> | ActionError> {
  const r = await approvePendingProjectApi(await headers(), arId, body);
  const result = toResult(r);
  if (result.ok) {
    revalidatePath(`/organizations/${orgId}/dashboard`);
  }
  return result;
}
