// Inline Objective + KeyResult editor — used by page 10 wizard step 3
// (project creation) and by page 11 detail (in-place OKR edits).
//
// Minimal shape: lists existing objectives / KRs with add + remove +
// edit. No server round-trip — the parent passes current state +
// onChange. Validation (measurement-type-vs-value shape) is the
// server's job at submit; this component is purely presentational +
// state-mutating.
//
// Mirrors the domain value objects in
// `modules/crates/domain/src/model/composites_m4.rs`. Strings only at
// this tier (the web tier treats domain payloads as opaque records per
// the M3/P5 phi-core-strip discipline — nothing phi-core-typed crosses
// this surface).

"use client";

import { useCallback } from "react";

export type OKRObjective = {
  objective_id: string;
  name: string;
  description: string;
  /// Serde-matched status (Draft / Active / Achieved / Missed /
  /// Cancelled) — `as_str()` wire-form.
  status: string;
  owner: string; // AgentId (uuid string)
  deadline?: string | null;
  key_result_ids?: string[];
};

export type OKRKeyResult = {
  kr_id: string;
  objective_id: string;
  name: string;
  description: string;
  /// Count / Boolean / Percentage / Custom — serde snake_case.
  measurement_type: "count" | "boolean" | "percentage" | "custom";
  /// Opaque OkrValue payload — the exact shape is gated by
  /// measurement_type server-side.
  target_value: unknown;
  current_value?: unknown;
  owner: string;
  deadline?: string | null;
  /// NotStarted / InProgress / Achieved / Missed / Cancelled — serde snake_case.
  status: string;
};

export type OKREditorProps = {
  objectives: OKRObjective[];
  keyResults: OKRKeyResult[];
  onChange: (next: {
    objectives: OKRObjective[];
    keyResults: OKRKeyResult[];
  }) => void;
  /// Optional read-only flag — page 11 may render the editor in
  /// view-only mode for operators without [allocate] on the project.
  readOnly?: boolean;
};

function randomShortId(prefix: string): string {
  // Non-cryptographic; operators can override before submit. Good
  // enough to give each new objective / KR a unique handle in the
  // form.
  const suffix = Math.random().toString(36).slice(2, 8);
  return `${prefix}-${suffix}`;
}

export function OKREditor({
  objectives,
  keyResults,
  onChange,
  readOnly = false,
}: OKREditorProps) {
  const addObjective = useCallback(() => {
    const next: OKRObjective = {
      objective_id: randomShortId("o"),
      name: "",
      description: "",
      status: "draft",
      owner: "",
      key_result_ids: [],
    };
    onChange({ objectives: [...objectives, next], keyResults });
  }, [objectives, keyResults, onChange]);

  const removeObjective = useCallback(
    (id: string) => {
      onChange({
        objectives: objectives.filter((o) => o.objective_id !== id),
        keyResults: keyResults.filter((kr) => kr.objective_id !== id),
      });
    },
    [objectives, keyResults, onChange],
  );

  const addKeyResult = useCallback(
    (objectiveId: string) => {
      const next: OKRKeyResult = {
        kr_id: randomShortId("kr"),
        objective_id: objectiveId,
        name: "",
        description: "",
        measurement_type: "count",
        target_value: 0,
        owner: "",
        status: "not_started",
      };
      onChange({ objectives, keyResults: [...keyResults, next] });
    },
    [objectives, keyResults, onChange],
  );

  const removeKeyResult = useCallback(
    (krId: string) => {
      onChange({
        objectives,
        keyResults: keyResults.filter((kr) => kr.kr_id !== krId),
      });
    },
    [objectives, keyResults, onChange],
  );

  const updateObjectiveField = useCallback(
    <K extends keyof OKRObjective>(
      id: string,
      field: K,
      value: OKRObjective[K],
    ) => {
      onChange({
        objectives: objectives.map((o) =>
          o.objective_id === id ? { ...o, [field]: value } : o,
        ),
        keyResults,
      });
    },
    [objectives, keyResults, onChange],
  );

  return (
    <section
      className="rounded border border-white/10 p-4"
      data-testid="okr-editor"
    >
      <header className="mb-3 flex items-center justify-between">
        <h3 className="text-sm font-semibold">Objectives &amp; Key Results</h3>
        {!readOnly ? (
          <button
            type="button"
            onClick={addObjective}
            className="rounded border border-white/20 px-2 py-1 text-xs hover:bg-white/5"
            data-testid="okr-add-objective"
          >
            + Objective
          </button>
        ) : null}
      </header>

      {objectives.length === 0 ? (
        <p className="text-xs opacity-60">
          No objectives yet. Projects may ship without OKRs (the simpler{" "}
          <code>goal</code> field covers the one-line case).
        </p>
      ) : null}

      <ul className="space-y-4">
        {objectives.map((obj) => {
          const objKRs = keyResults.filter(
            (kr) => kr.objective_id === obj.objective_id,
          );
          return (
            <li
              key={obj.objective_id}
              className="rounded border border-white/10 p-3"
              data-testid={`okr-objective-${obj.objective_id}`}
            >
              <div className="mb-2 flex items-center justify-between">
                <input
                  type="text"
                  value={obj.name}
                  readOnly={readOnly}
                  onChange={(e) =>
                    updateObjectiveField(obj.objective_id, "name", e.target.value)
                  }
                  placeholder="Objective name"
                  className="flex-1 rounded border border-white/10 bg-transparent px-2 py-1 text-sm"
                />
                {!readOnly ? (
                  <button
                    type="button"
                    onClick={() => removeObjective(obj.objective_id)}
                    className="ml-2 rounded border border-red-500/40 px-2 py-1 text-xs text-red-300 hover:bg-red-500/10"
                    aria-label={`Remove objective ${obj.objective_id}`}
                  >
                    Remove
                  </button>
                ) : null}
              </div>
              <textarea
                value={obj.description}
                readOnly={readOnly}
                onChange={(e) =>
                  updateObjectiveField(
                    obj.objective_id,
                    "description",
                    e.target.value,
                  )
                }
                placeholder="Objective description"
                rows={2}
                className="mb-2 w-full rounded border border-white/10 bg-transparent px-2 py-1 text-sm"
              />
              <ul className="space-y-2 pl-3">
                {objKRs.map((kr) => (
                  <li
                    key={kr.kr_id}
                    className="flex items-center gap-2 text-xs"
                    data-testid={`okr-kr-${kr.kr_id}`}
                  >
                    <span className="font-mono opacity-60">{kr.kr_id}</span>
                    <span className="flex-1">
                      {kr.name || "(unnamed key result)"}
                    </span>
                    <span className="rounded border border-white/10 px-1 font-mono">
                      {kr.measurement_type}
                    </span>
                    {!readOnly ? (
                      <button
                        type="button"
                        onClick={() => removeKeyResult(kr.kr_id)}
                        className="rounded border border-red-500/30 px-1 text-red-300 hover:bg-red-500/10"
                        aria-label={`Remove key result ${kr.kr_id}`}
                      >
                        ✕
                      </button>
                    ) : null}
                  </li>
                ))}
              </ul>
              {!readOnly ? (
                <button
                  type="button"
                  onClick={() => addKeyResult(obj.objective_id)}
                  className="mt-2 rounded border border-white/20 px-2 py-1 text-xs hover:bg-white/5"
                  data-testid={`okr-add-kr-${obj.objective_id}`}
                >
                  + Key Result
                </button>
              ) : null}
            </li>
          );
        })}
      </ul>
    </section>
  );
}
