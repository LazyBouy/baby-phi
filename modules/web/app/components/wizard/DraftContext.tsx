// Wizard draft persistence — React Context wrapping sessionStorage
// (primary) + localStorage (fallback) per ADR-0021.
//
// The `useDraft<T>(key)` hook gives each wizard step a typed
// get/set pair. Writes go to BOTH storage layers so:
//   - refresh inside the same tab picks up from sessionStorage
//     (strictly-correct same-session semantics);
//   - opening a fresh tab or restarting the browser picks up from
//     localStorage (cross-tab + cross-restart refresh tolerance).
//
// No server round-trip. No `organization_drafts` table (per
// ADR-0021). Drafts are cleared on wizard submit via
// `clearDraft(key)` — the caller invokes it after a successful
// `createOrgAction` response.

"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

// ---- Storage helpers ------------------------------------------------------

/// Storage key prefix keeps wizard drafts from colliding with other
/// local/session storage keys the app might use. The M3/P1 wizard
/// uses keys like `m3.org-create.step1`; M4+ wizards will prefix
/// differently.
const DRAFT_KEY_PREFIX = "baby-phi.wizard.";

/// SSR guard — `window` is undefined during Next.js server render.
/// Every storage read/write routes through these helpers so the
/// component renders without crashing on the first paint.
function readStorage(key: string): string | null {
  if (typeof window === "undefined") return null;
  const full = `${DRAFT_KEY_PREFIX}${key}`;
  // sessionStorage first — same-tab state wins.
  const fromSession = window.sessionStorage.getItem(full);
  if (fromSession !== null) return fromSession;
  return window.localStorage.getItem(full);
}

function writeStorage(key: string, value: string) {
  if (typeof window === "undefined") return;
  const full = `${DRAFT_KEY_PREFIX}${key}`;
  // Write to both layers. The quotas on both are ~5 MB and our
  // draft payloads are <50 KB, so we accept the 2x write cost in
  // exchange for cross-tab+refresh tolerance.
  try {
    window.sessionStorage.setItem(full, value);
  } catch {
    // Quota exceeded or storage disabled — fall through to
    // localStorage as a best-effort recovery.
  }
  try {
    window.localStorage.setItem(full, value);
  } catch {
    // Both layers full or disabled: caller's setDraft silently
    // becomes a no-op. Wizard submit still works; only refresh
    // tolerance is lost.
  }
}

function removeStorage(key: string) {
  if (typeof window === "undefined") return;
  const full = `${DRAFT_KEY_PREFIX}${key}`;
  window.sessionStorage.removeItem(full);
  window.localStorage.removeItem(full);
}

// ---- Context --------------------------------------------------------------

export type DraftContextValue = {
  /// Synchronously read the current value for `key`. Returns
  /// `undefined` if never set (or if the browser cleared storage).
  getDraft: <T>(key: string) => T | undefined;
  /// Write `value` under `key`. Serialises with `JSON.stringify`.
  setDraft: <T>(key: string, value: T) => void;
  /// Remove the draft entry entirely — call after a successful
  /// wizard submit so the next wizard run starts fresh.
  clearDraft: (key: string) => void;
};

const Context = createContext<DraftContextValue | null>(null);

export function DraftProvider({ children }: { children: React.ReactNode }) {
  // We don't keep state here — the hook reads/writes storage
  // directly. The provider exists so multiple wizards in different
  // parts of the tree can share the same draft-namespace without
  // wiring their own hooks independently.
  const api = useMemo<DraftContextValue>(
    () => ({
      getDraft<T>(key: string): T | undefined {
        const raw = readStorage(key);
        if (raw === null) return undefined;
        try {
          return JSON.parse(raw) as T;
        } catch {
          return undefined;
        }
      },
      setDraft<T>(key: string, value: T) {
        writeStorage(key, JSON.stringify(value));
      },
      clearDraft(key: string) {
        removeStorage(key);
      },
    }),
    [],
  );
  return <Context.Provider value={api}>{children}</Context.Provider>;
}

/// Hook consumed by wizard steps. Returns `[value, setValue,
/// clearValue]`. `value` is `undefined` until the step first reads
/// from storage (happens synchronously on mount via `useEffect` to
/// keep SSR hydration consistent — the first render is always
/// `initial`; the effect hydrates from storage on the client).
export function useDraft<T>(
  key: string,
  initial: T,
): [T, (next: T) => void, () => void] {
  const ctx = useContext(Context);
  const [value, setValueState] = useState<T>(initial);
  const loadedRef = useRef(false);

  useEffect(() => {
    if (loadedRef.current) return;
    loadedRef.current = true;
    const stored = ctx?.getDraft<T>(key);
    if (stored !== undefined) setValueState(stored);
  }, [ctx, key]);

  const setValue = useCallback(
    (next: T) => {
      setValueState(next);
      ctx?.setDraft(key, next);
    },
    [ctx, key],
  );

  const clearValue = useCallback(() => {
    ctx?.clearDraft(key);
    setValueState(initial);
  }, [ctx, key, initial]);

  return [value, setValue, clearValue];
}
