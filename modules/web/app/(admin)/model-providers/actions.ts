// Server Actions for the model-providers page.
//
// Every action forwards the admin's session cookie + calls the
// matching phi API endpoint. The register action accepts a
// `configJson` string (operator-pasted) so the web tier stays
// ignorant of phi-core's `ModelConfig` field layout — whatever JSON
// the operator supplies goes straight to the server, which
// deserialises via phi-core's own serde.

"use server";

import { revalidatePath } from "next/cache";

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import {
  archiveProviderApi,
  listProviderKindsApi,
  registerProviderApi,
  type ApiResult,
  type ArchiveWire,
  type RegisterWire,
  type TenantSetWire,
} from "@/lib/api/model-providers";

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

export async function registerProviderAction(input: {
  configJson: string;
  secretRef: string;
  tenantsAllowed: TenantSetWire;
}): Promise<ActionOk<RegisterWire> | ActionError> {
  let config: Record<string, unknown>;
  try {
    const parsed = JSON.parse(input.configJson);
    if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
      return {
        ok: false,
        httpStatus: 400,
        code: "VALIDATION_FAILED",
        message: "Config must be a JSON object.",
      };
    }
    config = parsed as Record<string, unknown>;
  } catch (err) {
    return {
      ok: false,
      httpStatus: 400,
      code: "VALIDATION_FAILED",
      message: `Config is not valid JSON: ${
        err instanceof Error ? err.message : String(err)
      }`,
    };
  }
  const r = await registerProviderApi(await headers(), {
    config,
    secret_ref: input.secretRef,
    tenants_allowed: input.tenantsAllowed,
  });
  const result = toResult(r);
  if (result.ok) revalidatePath("/model-providers");
  return result;
}

export async function archiveProviderAction(input: {
  providerId: string;
}): Promise<ActionOk<ArchiveWire> | ActionError> {
  const r = await archiveProviderApi(await headers(), input.providerId);
  const result = toResult(r);
  if (result.ok) revalidatePath("/model-providers");
  return result;
}

export async function fetchProviderKindsAction(): Promise<
  ActionOk<string[]> | ActionError
> {
  const r = await listProviderKindsApi(await headers());
  return toResult(r);
}
