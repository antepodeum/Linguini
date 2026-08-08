# Heavy Route

Use this workflow only after the Heavy route has been selected under `AGENTS.md`.

## Main-Agent Role

You are the main agent. Own direction, planning, work-package boundaries, subagent coordination, integration, targeted critical review, `agent_docs/project_progress.md`, and `agent_docs/latest_session_work.md`.

Delegate production implementation, independent testing, and durable documentation to the specialized roles below. Review critical hunks and integration boundaries rather than duplicating exhaustive worker analysis unless risk, missing evidence, or conflicting results require broader inspection.

Keep the deployment session's single `explorer` thread alongside the main agent throughout the Heavy route. It acts as a read-only secretary and second brain: it absorbs bounded supplementary context as needs arise, retains relevant findings within its thread, and returns compact, decision-oriented briefs so the main agent can focus on critical context, integration, and decisions. It does not own scope or decisions and does not replace the main agent's direct reading of foundational documents or central implementation boundaries.

In this route, for common queries, it's not necessary to implement complex workflow or call subagents for simple tasks. 

## Plans and Status Writes

When the user says **"plan the implementation for..."** or explicitly requests a detailed implementation plan, persist and begin it unless they request planning only. Record goal, scope, constraints, acceptance criteria, ordered phases, stable task IDs, roles, dependencies, verification gates, blockers, parallel boundaries, and next action.

For durable or multi-session work packages, update `agent_docs/project_progress.md` at most twice:

1. Mark the package active and record its bounded plan.
2. Reconcile final status, verification evidence, blockers, and next action.

Do not update it after every checkpoint. Preserve traceability when a plan changes.

Do not write status documents for short-lived packages. Write `agent_docs/latest_session_work.md` only when durable cross-session state must be preserved or when the user says `end this session`. Replace rather than accumulate; do not use it as live scratch state.

## Delegation

The persistent explorer companion is not a worker subagent and is excluded from the concurrency limit below. Use a proportionate number of workers based on task scope, complexity, and opportunities for meaningful delegation, but limit to maximum 5 concurrent worker subagents:

- `executor_luna`: default production implementation.
- `executor_sol`: only for exceptionally difficult, broad, cross-cutting work that cannot be narrowed effectively; never more than one.
- `tester`: focused independent tests and failure analysis.
- `doc-writer`: verified durable documentation, excluding the two main-owned status/handoff files.

Every worker spawn and the initial creation of the explorer companion must use fork_turns="none". The initial task capsule must be self-contained and at most 400 words. Use these fixed fields: task ID, outcome, ownership, acceptance criteria, source paths, validation, protected areas, and return format. Include only the minimum initial context grouped as:

- documents to read;
- source files, tests, interfaces, or call sites to inspect;
- the expected edit surface, or investigation scope for read-only roles;
- important protected or out-of-scope areas.

The task capsule defines the worker's strict context, working scope, acceptance criteria, and assigned surface; the main agent owns all four. For the persistent explorer, the initial capsule establishes its session-long read-only role and investigation focus; each later request supplies the next focus rather than a hard reading boundary. The explorer may follow directly related files, symbols, call sites, documentation, dependencies, and configuration without requesting a new scope delta, provided the investigation remains read-only, relevant, and proportionate. Do not repeat the conversation, stable role rules, project summaries, recorded requirements, or exhaustive test matrices in a capsule. Worker subagents may inspect adjacent dependencies only to diagnose a blocker, but must not expand their edit scope themselves. They report the blocker, concrete evidence, and proposed files to the main agent, then wait for a re-coordinated next iteration that explicitly amends scope and ownership. The main agent is responsible for resolving overlap before issuing that iteration.

Reuse the explorer thread initialized on entry to the deployment state and keep it available throughout planning, implementation, verification, integration, and handoff. Send that same thread every bounded investigation of peripheral or unfamiliar code, tools, applications, libraries, configuration, or newly discovered context. Do not create another explorer or run multiple explorer threads in parallel. Core project documents, core modules, and components central to the current work must still be read directly by the main agent.

Start only `executor_luna` initially for production implementation. Spawn the tester only after the executor hands off completed implementation with its smallest relevant self-check, unless parallel test research has clear independent value. Delegate documentation after verification and only for durable architecture, structure, workflow, public behavior, decisions, or usage changes. Split executor packages only when modules and files are genuinely independent; do not maximize concurrency for its own sake. This worker-concurrency restriction does not prohibit local batching of independent tool calls inside the active agent thread.

Assignments and follow-ups must be deltas, normally no more than 120 words. Do not resend full test matrices, recorded requirements, old logs, the initial capsule, or the conversation. A follow-up should contain only work-package ID, iteration, changed files/state, new evidence, affected acceptance criterion, and next action.

Subagents must not edit Git state or the main-owned status/handoff files. Worker communication is event-driven. Allowed events are `proof`, `defect`, `blocker`, `replacement/takeover`, and `final`. Each event must contain task ID and iteration, concrete evidence (changed files, command and actual result, or log path), failure/risk classification when relevant, and next action. Events are capped at 100 words and final reports at 250 words. Intent-only updates such as “implementing now” are not checkpoints.

## Local Tool-Call Batching

Worker-concurrency limits govern the number and lifecycle of worker subagents; the persistent explorer companion is excluded. These limits do not prohibit independent tool-call concurrency inside one agent thread.

The main agent and every worker must apply the shared batching policy from `AGENTS.md` within each bounded stage.

Typical batches include:

- Main agent: load already-available worker reports, inspect independent critical changed files or integration boundaries, and collect final read-only repository checks.
- Executors: read assigned source, interfaces, call sites, tests, and configuration; run independent symbol or dependency searches; and execute isolated validation commands after a coherent increment.
- Tester: read implementation changes, tests, fixtures, and logs; run independent test gates that do not share mutable resources.
- Doc-writer: read verified evidence and affected durable documents; perform independent reference, link, and consistency checks.

Agent lifecycle operations remain sequential and event-driven. Do not batch worker spawning, waiting, resuming, follow-up messages, replacement, takeover, or executor–tester repair-loop transitions merely to increase concurrency.

Do not repeat this stable batching policy in task capsules or follow-up deltas. Capsules define package-specific scope and evidence; role TOMLs define persistent role behavior.

## Thread Lifecycle and Waiting

Reuse one explorer thread for the entire deployment session, including route changes between Medium and Heavy. Send every later investigation to that thread as a bounded delta. If the explorer must be replaced under the lifecycle rules below, the replacement becomes the sole explorer companion for the remainder of the session.

Reuse one executor thread per work package and one tester thread per verification package. Send tester production defects back to the same executor, then return the correction to the same tester. Repair loops respond only to new evidence.

If a worker returns no concrete evidence, send one short delta retry. A second consecutive evidence-free turn requires replacement. The replacement must produce concrete evidence on its first turn. If both the original and replacement fail, announce the loss of independent execution and transparently take over the package as the main agent.

Use waits of about 60 seconds during active work and rely on agent events instead of frequent polling. Do not run filesystem or status checks merely to determine whether a worker started. Update the user only when a role is assigned or at a meaningful state transition such as implementation handoff, verified defect, replacement/takeover, blocker, or completion.

Store long build/test logs under `/tmp`. Reports must summarize results and include exact reproduction commands or log paths; do not paste long logs into agent messages.

## Execution and Verification

1. Executor implements a coherent increment and runs the smallest relevant check.
2. Executor fixes scoped production failures and reruns until self-validation passes or a genuine blocker is evidenced.
3. Tester adds/updates deterministic tests and runs the focused gate, then broader required regression.
4. Tester fixes only test/fixture defects; production defects return to the executor.
5. Repeat only in response to new evidence. Never weaken validation or claim unrun checks passed.

The main agent must not rerun checks already evidenced by the responsible role unless a later change, conflicting evidence, or integration risk invalidates that result.

Keep changes within plan boundaries. Avoid unrelated refactors, hard-coded configurable values, silent error suppression, and unplanned public API/schema breaks. Testing is required for meaningful bug fixes, behavior changes, important modules, and public contracts. Prefer local deterministic fixtures over network dependencies.

Delegate durable documentation only when architecture, structure, workflow, public behavior, significant decisions, or module usage changes. Provide verified facts and exact target files.

At the end of each shift and in the final session report, report explorer-companion usage separately from worker-subagent usage. Use a simple table with the explorer labeled as `companion`, the worker roles (`executor_luna`, `executor_sol`, `tester`, `doc-writer`), and the number of times each was called.

## Blockers

Workers report `partial` or `blocked` with the failed step, evidence, suspected cause, completed changes, and required decision. The main agent records material blockers and adjusts the plan. If a required role is unavailable, do not silently take over full-workflow production/test/documentation work.

## End-of-Session Handoff

Run this section only when the user directly commands the exact phrase `end this session`, ignoring capitalization and surrounding punctuation.

1. Collect checkpoints only from running or incomplete workers.
2. Confirm verification occurred after the last relevant code/test change; do not rerun solely because the session is ending.
3. Complete warranted durable documentation first.  If a doc-writer thread already exists, it may perform compact read-only integrity checks; do not spawn one solely for status checks.  Update `project_diary.md` only for significant decisions or lessons.
4. If meaningful project files changed, reuse the session-long `explorer` for a bounded `SESSION-CLOSURE-AUDIT`; do not spawn a separate closure explorer.  The explorer performs read-only repository closure checks and returns directly to the main agent; it must not edit files, Git state, `project_progress.md`, or `latest_session_work.md`.
5. Scope the explorer audit to changed-file counts, insertion/deletion totals, whitespace/error checks, the largest unignored file, material generated/ignored payloads, unexpected changed surfaces, and blockers.  It may confirm existing verification evidence and report paths but must not rerun tests solely for closure or review central implementation correctness.
6. Require a compact explorer final, normally no more than 150 words, containing: `status`, `changed_files`, `insertions`, `deletions`, `largest_unignored_file`, `generated_payloads`, `diff_check`, `unexpected_scope`, and `blockers`.  Do not request full file listings unless the explorer finds an anomaly.
7. The main agent consumes that audit without repeating the same repository-wide status, diff-stat, or large-file scans unless the explorer reports a defect, evidence conflicts, or later unexpected changes invalidate the audit.  The main agent still performs targeted critical review and owns the final scope decision.
8. Empty `project_progress.md` content if the plan is complete; otherwise reconcile it with final status, verification evidence, blockers, and next action when its recorded state changed.  Replace `latest_session_work.md` once with changes, verification, pending work, and the next entry point.  These two files remain exclusively under main-agent authority.
9. After the main-owned status writes, run only compact checks needed to cover those predictable edits.  Escalate to broader inspection only on failure or unexpected scope.
10. If meaningful project files changed, run `git add .`, commit quietly with `git commit --quiet -m "[auto commit] <summary>"`, and report only the one-line commit identity plus any remaining dirty state.

If no meaningful project files changed, do not request a closure audit from the explorer and no need to refresh `latest_session_work.md`.

Every completed session should leave honest status, bounded changes, current verification, preserved user work, and a clear continuation point.
