// Server Actions for the Organizations pages (M3/P4).

"use server";

import { revalidatePath } from "next/cache";

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import {
  createOrgApi,
  dashboardOrgApi,
  listOrgsApi,
  showOrgApi,
  type ApiResult,
  type CreateOrgBody,
  type CreateOrgResponseWire,
  type DashboardSummaryWire,
  type ListOrgsWire,
  type ShowOrgWire,
} from "@/lib/api/orgs";

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

export async function createOrgAction(
  body: CreateOrgBody,
): Promise<ActionOk<CreateOrgResponseWire> | ActionError> {
  const r = await createOrgApi(await headers(), body);
  const result = toResult(r);
  if (result.ok) revalidatePath("/organizations");
  return result;
}

export async function listOrgsAction(): Promise<
  ActionOk<ListOrgsWire> | ActionError
> {
  return toResult(await listOrgsApi(await headers()));
}

export async function showOrgAction(
  id: string,
): Promise<ActionOk<ShowOrgWire> | ActionError> {
  return toResult(await showOrgApi(await headers(), id));
}

export async function dashboardOrgAction(
  id: string,
): Promise<ActionOk<DashboardSummaryWire> | ActionError> {
  return toResult(await dashboardOrgApi(await headers(), id));
}
