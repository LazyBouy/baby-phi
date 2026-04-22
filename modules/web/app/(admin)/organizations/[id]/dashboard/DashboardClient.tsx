"use client";

// Client-side dashboard shell — panel layout + 30s polling.
//
// Why a client component: polling plus React state for the latest
// snapshot requires `useEffect`; Next.js Server Actions are used as
// the polling RPC so auth cookies ride along automatically (no
// client-side fetch/cookie wrangling required).

import Link from "next/link";
import { useEffect, useRef, useState } from "react";

import type { DashboardSummaryWire } from "@/lib/api/orgs";

import { dashboardOrgAction } from "../../actions";

const POLL_INTERVAL_MS = 30_000;

export function DashboardClient({
  orgId,
  initial,
}: {
  orgId: string;
  initial: DashboardSummaryWire;
}) {
  const [summary, setSummary] = useState<DashboardSummaryWire>(initial);
  const [lastFetched, setLastFetched] = useState<Date>(new Date());
  const [pollError, setPollError] = useState<string | null>(null);
  const alive = useRef(true);

  useEffect(() => {
    alive.current = true;
    const tick = async () => {
      const res = await dashboardOrgAction(orgId);
      if (!alive.current) return;
      if (res.ok) {
        setSummary(res.value);
        setLastFetched(new Date());
        setPollError(null);
      } else {
        setPollError(`${res.code}: ${res.message}`);
      }
    };
    const handle = setInterval(() => void tick(), POLL_INTERVAL_MS);
    return () => {
      alive.current = false;
      clearInterval(handle);
    };
  }, [orgId]);

  return (
    <section className="space-y-4 p-6" data-testid="dashboard-root">
      <OrgHeader summary={summary} />

      {summary.welcome_banner && (
        <div
          className="rounded border border-blue-500/40 bg-blue-500/10 p-3 text-sm"
          data-testid="welcome-banner"
        >
          {summary.welcome_banner}
        </div>
      )}

      <div className="grid gap-3 md:grid-cols-4">
        <AgentsSummary summary={summary} />
        <ProjectsSummary summary={summary} />
        {summary.viewer.can_admin_manage && <PendingAuthRequests summary={summary} />}
        {summary.viewer.can_admin_manage && <AlertedEventsCount summary={summary} />}
        <TokenBudget summary={summary} />
      </div>

      <AdoptedTemplates summary={summary} />

      <RecentAuditEvents summary={summary} />

      <EmptyStateCtaCards summary={summary} />

      <footer className="flex items-center justify-between text-xs opacity-60">
        <span data-testid="last-fetched">
          Last refreshed {lastFetched.toISOString()}
        </span>
        {pollError && (
          <span className="text-red-400" data-testid="poll-error">
            poll error — {pollError}
          </span>
        )}
        <Link href={`/organizations/${orgId}`} className="underline">
          Organization detail →
        </Link>
      </footer>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Panels
// ---------------------------------------------------------------------------

function OrgHeader({ summary }: { summary: DashboardSummaryWire }) {
  const { org, viewer } = summary;
  return (
    <header
      className="flex items-start justify-between"
      data-testid="panel-org-header"
    >
      <div>
        <h1 className="text-xl font-semibold">{org.display_name}</h1>
        {org.vision && (
          <div className="text-sm opacity-70" data-testid="org-vision">
            {org.vision}
          </div>
        )}
        {org.mission && (
          <div className="text-sm opacity-70" data-testid="org-mission">
            {org.mission}
          </div>
        )}
        <div className="mt-1 font-mono text-xs opacity-50">{org.id}</div>
      </div>
      <div className="text-right text-xs opacity-70">
        <div data-testid="viewer-role">
          role: <span className="font-mono">{viewer.role}</span>
        </div>
        <div>consent: {org.consent_policy}</div>
      </div>
    </header>
  );
}

function AgentsSummary({ summary }: { summary: DashboardSummaryWire }) {
  const a = summary.agents_summary;
  return (
    <Tile title="Agents" value={a.total} testid="panel-agents-summary">
      <div className="text-xs opacity-70">
        human {a.human} · llm {a.llm}
      </div>
    </Tile>
  );
}

function ProjectsSummary({ summary }: { summary: DashboardSummaryWire }) {
  const p = summary.projects_summary;
  return (
    <Tile title="Projects" value={p.active} testid="panel-projects-summary">
      {(p.shape_a > 0 || p.shape_b > 0) && (
        <div className="text-xs opacity-70">
          shape A {p.shape_a} · shape B {p.shape_b}
        </div>
      )}
    </Tile>
  );
}

function PendingAuthRequests({ summary }: { summary: DashboardSummaryWire }) {
  return (
    <Tile
      title="Pending approvals"
      value={summary.pending_auth_requests_count}
      testid="panel-pending-auth-requests"
    />
  );
}

function AlertedEventsCount({ summary }: { summary: DashboardSummaryWire }) {
  return (
    <Tile
      title="Alerted (24h)"
      value={summary.alerted_events_24h}
      testid="panel-alerted-events"
    />
  );
}

function TokenBudget({ summary }: { summary: DashboardSummaryWire }) {
  const { used, total } = summary.token_budget;
  const ratio = total > 0 ? Math.round((used / total) * 100) : 0;
  return (
    <Tile
      title="Token budget"
      value={`${used.toLocaleString()} / ${total.toLocaleString()}`}
      testid="panel-token-budget"
    >
      <div className="text-xs opacity-70">{ratio}% used</div>
    </Tile>
  );
}

function AdoptedTemplates({ summary }: { summary: DashboardSummaryWire }) {
  if (summary.templates_adopted.length === 0) return null;
  return (
    <div
      className="rounded border border-white/10 bg-black/20 p-3 text-sm"
      data-testid="panel-adopted-templates"
    >
      <div className="mb-1 text-xs opacity-70">Adopted templates</div>
      <div className="font-mono">
        {summary.templates_adopted
          .map((t) => t.toUpperCase())
          .join(" · ") || "—"}
      </div>
    </div>
  );
}

function RecentAuditEvents({ summary }: { summary: DashboardSummaryWire }) {
  if (summary.recent_events.length === 0) return null;
  return (
    <div
      className="rounded border border-white/10 bg-black/20 p-3 text-sm"
      data-testid="panel-recent-events"
    >
      <div className="mb-2 text-xs opacity-70">Recent activity</div>
      <ul className="space-y-1">
        {summary.recent_events.map((ev) => (
          <li
            key={ev.id}
            className="flex items-center justify-between text-xs"
          >
            <span className="font-mono opacity-60">{ev.timestamp}</span>
            <span className="flex-1 px-2">{ev.summary}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

function EmptyStateCtaCards({ summary }: { summary: DashboardSummaryWire }) {
  const c = summary.cta_cards;
  const cards: { label: string; href: string | null | undefined; hint: string }[] = [
    { label: "Add Agent", href: c.add_agent, hint: "Build your roster" },
    { label: "Create Project", href: c.create_project, hint: "Scope the first work" },
    { label: "Adopt Template", href: c.adopt_template, hint: "Review & approve" },
    {
      label: "System Agents",
      href: c.configure_system_agents,
      hint: "Review & tune",
    },
  ];
  const visible = cards.filter((card) => typeof card.href === "string");
  if (visible.length === 0) return null;
  return (
    <div className="grid gap-3 md:grid-cols-4" data-testid="panel-cta-cards">
      {visible.map((card) => (
        <Link
          key={card.label}
          href={card.href as string}
          className="rounded border border-white/10 bg-black/20 p-3 text-sm hover:border-white/30"
        >
          <div className="font-semibold">{card.label}</div>
          <div className="text-xs opacity-70">{card.hint}</div>
        </Link>
      ))}
    </div>
  );
}

function Tile({
  title,
  value,
  testid,
  children,
}: {
  title: string;
  value: number | string;
  testid: string;
  children?: React.ReactNode;
}) {
  return (
    <div
      className="rounded border border-white/10 bg-black/20 p-3"
      data-testid={testid}
    >
      <div className="text-xs opacity-70">{title}</div>
      <div className="text-2xl font-semibold" data-testid={`${testid}-value`}>
        {value}
      </div>
      {children}
    </div>
  );
}
