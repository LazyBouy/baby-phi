"use server";

// Server Action for the bootstrap-claim form.
//
// Called by `<form action={submitClaim}>`. Validates the form,
// invokes the M1/P6 claim API, and — on success — forwards the
// `Set-Cookie` header the server returned so the browser carries
// the `phi_kernel_session` cookie on future requests.

import { cookies } from "next/headers";

import {
  ChannelKind,
  ClaimPayload,
  extractSessionJwt,
  postBootstrapClaim,
} from "@/lib/api";

export type ClaimActionState =
  | { kind: "idle" }
  | {
      kind: "success";
      humanAgentId: string;
      auditEventId: string;
      grantId: string;
    }
  | { kind: "validation"; message: string }
  | { kind: "rejected"; code: string; message: string; httpStatus: number }
  | { kind: "error"; message: string };

const COOKIE_NAME =
  process.env.PHI_SESSION_COOKIE_NAME ?? "phi_kernel_session";

function parseChannelKind(raw: FormDataEntryValue | null): ChannelKind | null {
  if (raw === "slack" || raw === "email" || raw === "web") {
    return raw;
  }
  return null;
}

function requireNonEmpty(
  value: FormDataEntryValue | null,
  field: string,
): string | { kind: "validation"; message: string } {
  if (typeof value !== "string" || value.trim().length === 0) {
    return { kind: "validation", message: `${field} must not be empty` };
  }
  return value.trim();
}

export async function submitClaim(
  _prev: ClaimActionState,
  formData: FormData,
): Promise<ClaimActionState> {
  const credential = requireNonEmpty(
    formData.get("credential"),
    "credential",
  );
  if (typeof credential !== "string") return credential;
  const displayName = requireNonEmpty(
    formData.get("displayName"),
    "display name",
  );
  if (typeof displayName !== "string") return displayName;
  const channelHandle = requireNonEmpty(
    formData.get("channelHandle"),
    "channel handle",
  );
  if (typeof channelHandle !== "string") return channelHandle;
  const channelKind = parseChannelKind(formData.get("channelKind"));
  if (!channelKind) {
    return {
      kind: "validation",
      message: "channel kind must be slack | email | web",
    };
  }

  const payload: ClaimPayload = {
    bootstrapCredential: credential,
    displayName,
    channelKind,
    channelHandle,
  };

  try {
    const result = await postBootstrapClaim(payload);
    if (result.ok) {
      // Forward the session cookie to the browser by extracting the
      // JWT value from the server's Set-Cookie header. `cookies().set`
      // reissues it with Next.js' own cookie-writing path.
      const jwt = extractSessionJwt(result.setCookie, COOKIE_NAME);
      if (jwt) {
        const jar = await cookies();
        jar.set(COOKIE_NAME, jwt, {
          httpOnly: true,
          sameSite: "lax",
          path: "/",
        });
      }
      return {
        kind: "success",
        humanAgentId: result.success.humanAgentId,
        auditEventId: result.success.auditEventId,
        grantId: result.success.grantId,
      };
    }
    return {
      kind: "rejected",
      code: result.code,
      message: result.message,
      httpStatus: result.httpStatus,
    };
  } catch (err) {
    return {
      kind: "error",
      message: err instanceof Error ? err.message : String(err),
    };
  }
}
