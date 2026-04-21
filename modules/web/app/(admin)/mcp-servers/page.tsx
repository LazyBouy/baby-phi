// /mcp-servers — MCP servers admin page (M2/P6).
//
// Server Component. Fetches list + mounts client forms for
// register / patch-tenants / archive. Auth gate already applied by
// `app/(admin)/layout.tsx`.

import { forwardSessionCookieHeader } from "@/lib/api/forward-cookie";
import { listServersApi, type ServerSummary } from "@/lib/api/mcp-servers";

import { AddServerForm } from "./AddServerForm";
import { McpServersTable } from "./McpServersTable";

export const dynamic = "force-dynamic";

export default async function McpServersPage() {
  const headers = await forwardSessionCookieHeader();
  const res = await listServersApi(headers, true);
  let servers: ServerSummary[] = [];
  let errorLine: string | null = null;
  if (res.ok) {
    servers = res.value;
  } else {
    errorLine = `${res.code}: ${res.message}`;
  }

  return (
    <div className="space-y-8">
      <header className="space-y-1">
        <h1 className="text-xl font-semibold">MCP Servers</h1>
        <p className="text-sm opacity-70">
          Bound external services (MCP, OpenAPI, webhook). The endpoint string
          is phi-core&apos;s{" "}
          <span className="font-mono">McpClient</span> transport argument
          verbatim — a live client is constructed on-demand at probe /
          invocation time. Narrowing <span className="font-mono">tenants_allowed</span>{" "}
          cascades a grant revocation across every affected org.
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
        <McpServersTable servers={servers} />
      )}

      <section className="space-y-3">
        <h2 className="text-lg font-medium">Register an MCP server</h2>
        <AddServerForm />
      </section>
    </div>
  );
}
