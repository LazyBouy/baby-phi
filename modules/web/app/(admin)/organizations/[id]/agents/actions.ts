// Server Actions for the agents roster page (M4/P4).

"use server";

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import {
  listAgentsApi,
  type ApiResult,
  type ListAgentsQuery,
  type ListAgentsWire,
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
