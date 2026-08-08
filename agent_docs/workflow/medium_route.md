# Medium Route

Use this workflow only after the Medium route has been selected under `AGENTS.md`.

## Main-Agent Role

You are the main agent and perform implementation, verification, and documentation directly without spawning or delegating to worker subagents.

The one exception is the persistent `explorer` companion initialized when the session enters the `deployment state`. It is a read-only secretary and second brain, not a worker subagent. Keep the same explorer thread for the entire session and use it to absorb bounded supplementary context as needs arise, returning compact briefs while you retain ownership of decisions and the work itself.

Use the full project workflow at a proportionate level: understand the relevant context, plan when needed, implement the requested changes, verify the result, and keep the work within scope. Inspect only what is useful for the current task and avoid unnecessary process overhead.

In this route, for common queries, it's not necessary to implement complex workflow for simple tasks.

## Stage Execution and Tool Batching

Divide work into bounded stages such as context loading, targeted inspection, implementation, verification, and final review.

At any stage, send focused investigation of peripheral, unfamiliar, or newly discovered context to the existing explorer thread. The assigned focus is a starting point, and the explorer may follow related read-only context when useful. Do not initialize another explorer. Foundational project documents, central implementation surfaces, and decision-critical evidence remain the main agent's direct responsibility.

Before each stage, collect all independent, already-known, non-conflicting tool operations and apply the shared batching rules from `AGENTS.md`. Evaluate the returned results together before deciding the next stage.

Typical Medium-route batches include:

- Reading several already-identified source, header, test, or configuration files.
- Searching several known symbols or call sites.
- Collecting independent repository metadata.
- Running isolated validation commands after a coherent implementation increment.

Keep implementation edits sequential when one edit depends on another, files overlap, or intermediate results determine subsequent changes.

Run validation concurrently only when commands do not share mutable build output, generated files, fixtures, databases, ports, devices, or other state. Required checks remain required regardless of whether they were batched.

Do not manufacture stages or extra commands merely to create a batch. Small tasks may remain a single inspection, edit, and validation sequence.

## Plans and Status Writes

When the user says **"plan the implementation for..."** or explicitly requests a detailed implementation plan, persist and begin it unless they request planning only. 

Record:

* Goal and scope.
* Constraints and protected areas.
* Acceptance criteria.
* Major implementation steps when useful.
* Dependencies, verification approach, known blockers, and next action.

For durable or multi-session work packages, update `agent_docs/project_progress.md` at most twice:

1. Mark the package active and record its bounded plan.
2. Reconcile final status, verification evidence, blockers, and next action.

Do not update it after every checkpoint. Keep significant changes to the plan understandable and traceable.

## Documentation

Update durable documentation only when architecture, structure, workflow, public behavior, significant decisions, or module usage changes.

Use verified implementation and test results as the source of truth. Update `agent_docs/project_diary.md` only for decisions, discarded approaches, or lessons with lasting architectural value.

## Working Rules

Keep changes focused and preserve unrelated user work. Perform verification appropriate to the risk and scope of the task. Do not claim unrun checks passed, hide blockers, or broaden the task without a clear need.

Prefer one targeted inspection batch over a sequence of independent single-file or single-search outer calls. After implementation begins, repeat inspection only when a changed state, failure, or newly discovered dependency provides a concrete reason.

## Blockers

When blocked, record:

- The failed step and exact evidence.
- The suspected cause.
- Completed changes and current repository state.
- The affected acceptance criterion.
- The decision, dependency, or external input required.

Do not disguise partial work as completion. Adjust the plan and preserve a clear continuation point.

## End-of-Session Handoff

Run this section only when the user directly commands the exact phrase `end this session`, ignoring capitalization and surrounding punctuation.

1. Confirm verification occurred after the last relevant code or test change; do not rerun solely because the session is ending.
2. If meaningful project files changed, reuse the session-long explorer for bounded final read-only closure checks—concise status, diff statistics, whitespace or error checks, and unexpected changed surfaces. Do not spawn a separate closure explorer. The main agent still owns targeted critical review, status-document writes, Git staging, and the commit.
3. Empty `project_progress.md` content if the plan is complete, otherwise , reconcile it with the final status, verification evidence, blockers, and next action if its recorded state has changed.
4. Replace `latest_session_work.md` once with changes, verification, pending work, blockers, and the next entry point.
5. Update durable docs only when warranted and `project_diary.md` only for significant decisions or lessons.
6. If meaningful project files changed, run `git add .` and commit with `git commit -m "[auto commit] <summary>"`.
7. Include the persistent explorer in the final agent-usage table as a `companion` with its call count, even though no worker subagents were used.

If no meaningful project files changed, do not request a closure audit from the explorer and there is no need to refresh `latest_session_work.md`.

Every completed session must leave honest status, bounded changes, current verification, preserved user work, and a clear continuation point.
