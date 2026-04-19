import { getHealth } from "@/lib/api";

export const dynamic = "force-dynamic";

export default async function Home() {
  const health = await getHealth();

  return (
    <main className="mx-auto max-w-2xl p-8">
      <h1 className="text-3xl font-semibold">baby-phi</h1>
      <p className="mt-2 text-sm opacity-70">
        Fresh install. Bootstrap flow lands in M1 (Phase 1 — claim platform
        admin).
      </p>

      <section className="mt-8 rounded border border-white/10 p-4">
        <h2 className="text-lg font-medium">API health</h2>
        <pre className="mt-2 text-xs opacity-80">
          {JSON.stringify(health, null, 2)}
        </pre>
      </section>
    </main>
  );
}
