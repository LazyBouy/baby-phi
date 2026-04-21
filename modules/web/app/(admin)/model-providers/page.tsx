// /model-providers — Model Providers (M2/P5).
//
// Server Component. Fetches list + mounts client forms for
// register / archive. Route-group layout (`app/(admin)/layout.tsx`)
// already applied the auth gate.

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import { listProvidersApi, type ProviderSummary } from "@/lib/api/model-providers";

import { AddProviderForm } from "./AddProviderForm";
import { ProvidersTable } from "./ProvidersTable";

export const dynamic = "force-dynamic";

export default async function ModelProvidersPage() {
  const headers = await forwardSessionCookieHeader();
  const res = await listProvidersApi(headers, true);
  let providers: ProviderSummary[] = [];
  let errorLine: string | null = null;
  if (res.ok) {
    providers = res.value;
  } else {
    errorLine = `${res.code}: ${res.message}`;
  }

  return (
    <div className="space-y-8">
      <header className="space-y-1">
        <h1 className="text-xl font-semibold">Model Providers</h1>
        <p className="text-sm opacity-70">
          Bound LLM runtimes. Each row wraps a{" "}
          <span className="font-mono">phi_core::ModelConfig</span>; API keys
          live in the Credentials Vault and are spliced in at invocation time.
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
        <ProvidersTable providers={providers} />
      )}

      <section className="space-y-3">
        <h2 className="text-lg font-medium">Register a provider</h2>
        <AddProviderForm />
      </section>
    </div>
  );
}
