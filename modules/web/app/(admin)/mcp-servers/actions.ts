// Server Actions for the MCP-servers page.
//
// Every action forwards the admin's session cookie + calls the
// matching baby-phi API endpoint. PATCH-tenants carries the full
// cascade summary back to the dialog so the client can show
// "revoked N grants across M orgs".

"use server";

import { revalidatePath } from "next/cache";

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import {
  archiveServerApi,
  patchTenantsApi,
  registerServerApi,
  type ApiResult,
  type ArchiveWire,
  type PatchWire,
  type RegisterWire,
  type TenantSetWire,
} from "@/lib/api/mcp-servers";

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

export async function registerServerAction(input: {
  displayName: string;
  kind: string;
  endpoint: string;
  secretRef: string | null;
  tenantsAllowed: TenantSetWire;
}): Promise<ActionOk<RegisterWire> | ActionError> {
  if (input.displayName.trim().length === 0) {
    return {
      ok: false,
      httpStatus: 400,
      code: "VALIDATION_FAILED",
      message: "Display name must not be empty.",
    };
  }
  if (input.endpoint.trim().length === 0) {
    return {
      ok: false,
      httpStatus: 400,
      code: "VALIDATION_FAILED",
      message: "Endpoint must not be empty.",
    };
  }
  const r = await registerServerApi(await headers(), {
    display_name: input.displayName,
    kind: input.kind,
    endpoint: input.endpoint,
    secret_ref: input.secretRef,
    tenants_allowed: input.tenantsAllowed,
  });
  const result = toResult(r);
  if (result.ok) revalidatePath("/mcp-servers");
  return result;
}

export async function patchTenantsAction(input: {
  mcpServerId: string;
  tenantsAllowed: TenantSetWire;
}): Promise<ActionOk<PatchWire> | ActionError> {
  const r = await patchTenantsApi(
    await headers(),
    input.mcpServerId,
    input.tenantsAllowed,
  );
  const result = toResult(r);
  if (result.ok) revalidatePath("/mcp-servers");
  return result;
}

export async function archiveServerAction(input: {
  mcpServerId: string;
}): Promise<ActionOk<ArchiveWire> | ActionError> {
  const r = await archiveServerApi(await headers(), input.mcpServerId);
  const result = toResult(r);
  if (result.ok) revalidatePath("/mcp-servers");
  return result;
}
