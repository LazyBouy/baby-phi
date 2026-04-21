// Server Actions for the platform-defaults page.

"use server";

import { revalidatePath } from "next/cache";

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import {
  putDefaultsApi,
  type ApiResult,
  type PlatformDefaults,
  type PutDefaultsWire,
} from "@/lib/api/platform-defaults";

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

export async function putDefaultsAction(input: {
  ifVersion: number;
  defaults: PlatformDefaults;
}): Promise<ActionOk<PutDefaultsWire> | ActionError> {
  const r = await putDefaultsApi(
    await headers(),
    input.ifVersion,
    input.defaults,
  );
  const result = toResult(r);
  if (result.ok) revalidatePath("/platform-defaults");
  return result;
}
