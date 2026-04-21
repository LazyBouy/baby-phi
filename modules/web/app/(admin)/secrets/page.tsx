// /secrets — Credentials Vault (M2/P4).
//
// Server Component. On each request we:
//   1. Forward the admin's session cookie to `GET /api/v0/platform/secrets`.
//   2. Render the metadata list (plaintext never reaches this component).
//   3. Mount the Add / Reveal forms as client subtrees.
//
// Route-group layout (`app/(admin)/layout.tsx`) already enforced the
// auth gate before this page renders.

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import { listSecretsApi, type SecretSummary } from "@/lib/api/secrets";

import { AddSecretForm } from "./AddSecretForm";
import { RevealDialog } from "./RevealDialog";
import { SecretsTable } from "./SecretsTable";

export const dynamic = "force-dynamic";

export default async function SecretsPage() {
  const headers = await forwardSessionCookieHeader();
  const res = await listSecretsApi(headers);
  let secrets: SecretSummary[] = [];
  let errorLine: string | null = null;
  if (res.ok) {
    secrets = res.value;
  } else {
    errorLine = `${res.code}: ${res.message}`;
  }

  return (
    <div className="space-y-8">
      <header className="space-y-1">
        <h1 className="text-xl font-semibold">Credentials Vault</h1>
        <p className="text-sm opacity-70">
          Platform-wide sealed secrets. Material is stored AES-GCM-sealed;
          reveal requires <span className="font-mono">purpose=reveal</span> and
          is Alerted-audited.
        </p>
      </header>

      {errorLine ? (
        <div
          role="alert"
          className="rounded border border-red-500/40 bg-red-500/10 p-3 text-sm"
        >
          {errorLine}
        </div>
      ) : (
        <SecretsTable secrets={secrets} />
      )}

      <section className="space-y-3">
        <h2 className="text-lg font-medium">Add a secret</h2>
        <AddSecretForm />
      </section>

      {secrets.length > 0 ? (
        <section className="space-y-3">
          <h2 className="text-lg font-medium">Reveal plaintext</h2>
          <RevealDialog secrets={secrets} />
        </section>
      ) : null}
    </div>
  );
}
