// Admin Route Group layout. Every page under app/(admin)/ is gated
// by `requireAdminSession()` — unauthenticated requests redirect to
// `/bootstrap`. M2 adds the page files as each vertical slice lands
// (P4 secrets, P5 model-providers, P6 mcp-servers, P7 platform-defaults).
//
// The route group name `(admin)` does NOT appear in URLs (Next.js
// convention) — pages here are reached at `/secrets`, `/model-providers`,
// etc.

import { requireAdminSession } from "@/lib/session";

import { AdminSidebar } from "./components/AdminSidebar";

export const dynamic = "force-dynamic";

export default async function AdminLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const session = await requireAdminSession();

  return (
    <div className="flex min-h-screen">
      <AdminSidebar currentUserId={session.user.id} />
      <main className="flex-1 p-8">{children}</main>
    </div>
  );
}
