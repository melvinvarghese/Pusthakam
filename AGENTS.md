# Pusthakam contributor instructions

Follow the project architecture, conventions, and release rules in `CLAUDE.md`. The local development monitor lives in `tools/progress-monitor/` and is independent of the desktop app.

For work continued on another computer, also read `docs/SESSION_HANDOFF.md`. After every meaningful change, immediately update `PROGRESS.md`, `PROGRESS.json` and `docs/SESSION_HANDOFF.md`, plus affected feature documentation. Do not wait for the owner to announce a computer switch or for the end of a session. Record partial work, decisions, actual verification results (including failed/incomplete checks), blockers and the next exact step. Before ending a turn, check that these files agree with the working tree and each other. Source and documentation travel through Git commits/pushes/pulls; chat memory and temporary verification paths do not. Never commit library databases, backups or credentials.

## Maintain progress in every Codex session

1. Before starting work, read `PROGRESS.json` and `PROGRESS.md`. To choose what to work on next, use `docs/PENDING_WORK.md`; when an item is finished, tick it there with the date and evidence. Identify the current phase, the next milestone, and the roadmap task that the requested work advances.
2. Record the current task and its status in both files. Set `currentPhase` to the relevant development phase. For work outside the v1 product scope, describe it in `currentTask` without a roadmap task ID or a new product weight.
3. After meaningful work, check the task's acceptance criteria against the implementation and run the relevant tests. Record the actual command or walkthrough, outcome, timestamp, and result in `verificationRuns`. Include failures and incomplete checks. Add evidence references to the relevant tasks and summarize the results in `PROGRESS.md`.
4. Update the status only when the evidence supports it. A placeholder or partial implementation earns no implementation credit for that task. Use `implemented` once the implementation criteria are met, and `verified` once all required checks have passed and the evidence is recorded. Keep outstanding checks in `verificationNeeded`.
5. Record blockers and the next actionable milestone in both files. Add meaningful completions and changes to `recentActivity`, and set `lastUpdated` to the actual UTC time of the update.
6. Recalculate totals with `npm run progress:validate -- --write`, then run `npm run progress:validate`. Update the weighted percentages and status summary in `PROGRESS.md` in the same change as `PROGRESS.json`.

The v1 score combines weighted implementation (60%), verification (30%), and release gates (10%). Release gates pass when all their linked tasks are verified. A score of 100% requires the full v1 definition of done in `docs/PRODUCT_SPECIFICATION.md`. Change the task scope or weights only when the agreed product scope changes, or to correct and document an inaccurate baseline.

## Skill use and pauses

Proactively identify, read and apply relevant available skills before applicable work; the owner should not need to invoke them repeatedly. Report actual skill use accurately. An explicit owner pause stops implementation and design work until explicit resume. Immediately record pauses and owner decisions in PROGRESS.md, PROGRESS.json and docs/SESSION_HANDOFF.md without requiring a reminder. A request to update these records does not resume development.
