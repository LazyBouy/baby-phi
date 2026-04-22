// Admin sidebar — lists the 4 M2 admin surfaces. Each link is marked
// `pending` until its P<n> lands:
//   - /secrets          (P4 — credentials vault)
//   - /model-providers  (P5)
//   - /mcp-servers      (P6)
//   - /platform-defaults (P7)
//
// Client component because the active-link highlight reads
// `usePathname()`. The surrounding layout is still a Server Component
// (auth gate + SSR probe).

"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

type NavItem = {
  href: string;
  label: string;
  /** True once the target page has landed; false for pending stubs. */
  ready: boolean;
};

const NAV: NavItem[] = [
  { href: "/secrets", label: "Credentials Vault", ready: true },
  { href: "/model-providers", label: "Model Providers", ready: true },
  { href: "/mcp-servers", label: "MCP Servers", ready: true },
  { href: "/platform-defaults", label: "Platform Defaults", ready: true },
  { href: "/organizations", label: "Organizations", ready: true },
];

export function AdminSidebar({ currentUserId }: { currentUserId: string }) {
  const pathname = usePathname();
  return (
    <aside className="w-64 shrink-0 border-r border-white/10 bg-black/20 p-4">
      <div className="mb-6 text-sm opacity-70">
        Signed in as
        <div
          className="mt-1 truncate font-mono text-xs"
          title={currentUserId}
        >
          {currentUserId}
        </div>
      </div>
      <nav className="flex flex-col gap-1">
        {NAV.map((item) => {
          const isActive = pathname === item.href;
          const baseClasses = "rounded px-3 py-2 text-sm";
          if (!item.ready) {
            return (
              <span
                key={item.href}
                className={`${baseClasses} cursor-not-allowed opacity-40`}
                title="Pending — lands in a later M2 phase."
              >
                {item.label}
              </span>
            );
          }
          return (
            <Link
              key={item.href}
              href={item.href}
              className={`${baseClasses} ${
                isActive
                  ? "bg-white/10 font-medium"
                  : "opacity-80 hover:bg-white/5"
              }`}
            >
              {item.label}
            </Link>
          );
        })}
      </nav>
    </aside>
  );
}
