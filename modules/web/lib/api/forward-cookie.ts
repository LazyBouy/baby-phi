// Cookie-forwarding helper for M2 Server Actions that need to send
// the admin's session cookie along with an upstream baby-phi API call.
//
// Usage:
//   const res = await fetch(`${API_BASE}/api/v0/platform/secrets`, {
//     method: "POST",
//     headers: {
//       "content-type": "application/json",
//       ...(await forwardSessionCookieHeader()),
//     },
//     body: JSON.stringify(payload),
//   });
//
// Only the `baby_phi_session` cookie is forwarded — other cookies
// (analytics, preferences) are left in the browser.

import { cookies } from "next/headers";

const COOKIE_NAME =
  process.env.BABY_PHI_SESSION_COOKIE_NAME ?? "baby_phi_session";

export async function forwardSessionCookieHeader(): Promise<
  Record<string, string>
> {
  const jar = await cookies();
  const val = jar.get(COOKIE_NAME)?.value;
  if (!val) return {};
  return { cookie: `${COOKIE_NAME}=${val}` };
}
