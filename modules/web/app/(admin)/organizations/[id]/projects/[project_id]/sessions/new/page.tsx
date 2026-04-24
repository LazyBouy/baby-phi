// First-session launch admin page — M5/P7 (admin/14).
//
// Minimal SSR shell:
//   1. Picker form (agent id, prompt).
//   2. "Preview" submit runs the Permission-Check trace server-side
//      and redirects back with query params showing the decision.
//   3. "Launch" submit actually launches the session and redirects
//      to the project detail page so "Recent sessions" renders.
//
// Live SSE tail is intentionally deferred to M7 (plan drift D4.2 —
// `agent_loop` call is synthetic feeder today). Terminal state
// surfaces via `GET /api/v0/sessions/:id`.

import Link from "next/link";
import { redirect } from "next/navigation";

import {
  launchSessionAction,
  previewSessionAction,
} from "./actions";

export const dynamic = "force-dynamic";

export default async function NewSessionPage({
  params,
  searchParams,
}: {
  params: Promise<{ id: string; project_id: string }>;
  searchParams: Promise<{
    preview_outcome?: string;
    preview_error?: string;
    launch_error?: string;
  }>;
}) {
  const { id, project_id } = await params;
  const sp = await searchParams;

  async function previewSubmit(formData: FormData) {
    "use server";
    const agentId = String(formData.get("agent_id") ?? "").trim();
    if (!agentId) {
      redirect(
        `/organizations/${id}/projects/${project_id}/sessions/new?preview_error=${encodeURIComponent("agent_id required")}`,
      );
    }
    const res = await previewSessionAction(id, project_id, agentId);
    if (!res.ok) {
      redirect(
        `/organizations/${id}/projects/${project_id}/sessions/new?preview_error=${encodeURIComponent(`${res.code}: ${res.message}`)}`,
      );
    }
    const decision = res.value.decision ?? {};
    const outcome =
      (decision as Record<string, unknown>).decision ??
      (decision as Record<string, unknown>).outcome ??
      JSON.stringify(decision);
    redirect(
      `/organizations/${id}/projects/${project_id}/sessions/new?preview_outcome=${encodeURIComponent(String(outcome))}`,
    );
  }

  async function launchSubmit(formData: FormData) {
    "use server";
    const agentId = String(formData.get("agent_id") ?? "").trim();
    const prompt = String(formData.get("prompt") ?? "").trim();
    if (!agentId || !prompt) {
      redirect(
        `/organizations/${id}/projects/${project_id}/sessions/new?launch_error=${encodeURIComponent("agent_id + prompt required")}`,
      );
    }
    const res = await launchSessionAction(id, project_id, agentId, prompt);
    if (!res.ok) {
      redirect(
        `/organizations/${id}/projects/${project_id}/sessions/new?launch_error=${encodeURIComponent(`${res.code}: ${res.message}`)}`,
      );
    }
    redirect(`/organizations/${id}/projects/${project_id}`);
  }

  return (
    <section className="space-y-4 p-6">
      <header className="flex items-center justify-between">
        <h1 className="text-xl font-semibold">Launch a session</h1>
        <Link
          href={`/organizations/${id}/projects/${project_id}`}
          className="text-sm underline"
        >
          ← Back to project
        </Link>
      </header>

      {sp.preview_outcome ? (
        <div className="rounded border border-emerald-500/40 bg-emerald-500/10 p-3 text-sm">
          Permission-check preview:{" "}
          <strong>{decodeURIComponent(sp.preview_outcome)}</strong>
        </div>
      ) : null}
      {sp.preview_error ? (
        <div className="rounded border border-red-500/40 bg-red-500/10 p-3 text-sm">
          {sp.preview_error}
        </div>
      ) : null}
      {sp.launch_error ? (
        <div className="rounded border border-red-500/40 bg-red-500/10 p-3 text-sm">
          {sp.launch_error}
        </div>
      ) : null}

      <form
        action={previewSubmit}
        className="space-y-2 rounded border border-white/10 bg-black/20 p-4"
      >
        <h2 className="text-sm font-semibold uppercase tracking-wide opacity-70">
          Preview permission check
        </h2>
        <label className="block text-xs">
          <span className="opacity-70">Agent id</span>
          <input
            name="agent_id"
            required
            className="mt-1 block w-full rounded border border-white/10 bg-black/20 px-2 py-1 text-sm"
          />
        </label>
        <button
          type="submit"
          className="rounded bg-blue-500/80 px-3 py-1 text-sm font-medium text-white hover:bg-blue-400"
        >
          Preview
        </button>
      </form>

      <form
        action={launchSubmit}
        className="space-y-2 rounded border border-white/10 bg-black/20 p-4"
      >
        <h2 className="text-sm font-semibold uppercase tracking-wide opacity-70">
          Launch session
        </h2>
        <label className="block text-xs">
          <span className="opacity-70">Agent id</span>
          <input
            name="agent_id"
            required
            className="mt-1 block w-full rounded border border-white/10 bg-black/20 px-2 py-1 text-sm"
          />
        </label>
        <label className="block text-xs">
          <span className="opacity-70">Prompt</span>
          <textarea
            name="prompt"
            required
            rows={5}
            className="mt-1 block w-full rounded border border-white/10 bg-black/20 px-2 py-1 font-mono text-xs"
          />
        </label>
        <p className="text-xs opacity-70">
          Live tail is deferred to M7 (plan drift D4.2). After launch, the
          project detail page shows the new session row; drill via the{" "}
          <code>GET /api/v0/sessions/:id</code> endpoint for terminal
          state.
        </p>
        <button
          type="submit"
          className="rounded bg-blue-500 px-3 py-1 text-sm font-medium text-white hover:bg-blue-400"
        >
          Launch
        </button>
      </form>
    </section>
  );
}
