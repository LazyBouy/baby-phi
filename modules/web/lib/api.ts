const API_BASE = process.env.BABY_PHI_API_URL ?? "http://127.0.0.1:8080";

export type HealthProbe =
  | { reachable: true; live: unknown; ready: unknown }
  | { reachable: false; error: string };

async function probe(path: string): Promise<unknown> {
  const res = await fetch(`${API_BASE}${path}`, { cache: "no-store" });
  return res.json();
}

export async function getHealth(): Promise<HealthProbe> {
  try {
    const [live, ready] = await Promise.all([
      probe("/healthz/live"),
      probe("/healthz/ready"),
    ]);
    return { reachable: true, live, ready };
  } catch (err) {
    return {
      reachable: false,
      error: err instanceof Error ? err.message : String(err),
    };
  }
}
