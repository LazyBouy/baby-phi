// Auth placeholder — real OAuth 2.0 (PKCE) + local-password wiring lands in
// M3/M7b per the build plan. For M0 we define the contract surface only so
// downstream code can be written against it.

export type SessionUser = {
  id: string;
  displayName: string;
  // Principal type — `PlatformAdmin | OrgAdmin | AgentLead | Human | LlmAgent`
  // (aligned with `docs/specs/v0/concepts/permissions/02-auth-request.md`).
  principal: string;
};

export type Session =
  | { authenticated: true; user: SessionUser; expiresAt: string }
  | { authenticated: false };

export async function getSession(): Promise<Session> {
  // TODO (M1): read the server-signed session cookie and validate.
  return { authenticated: false };
}
