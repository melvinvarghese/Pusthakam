# Pusthakam contributor instructions

Follow the project architecture, conventions, and release rules in `CLAUDE.md`. The local development monitor lives in `tools/progress-monitor/` and is independent of the desktop app.

## Maintain progress in every Codex session

1. Before starting work, read `PROGRESS.json` and `PROGRESS.md`. Identify the current phase, the next milestone, and the roadmap task that the requested work advances.
2. Record the current task and its status in both files. Set `currentPhase` to the relevant development phase. For work outside the v1 product scope, describe it in `currentTask` without a roadmap task ID or a new product weight.
3. After meaningful work, check the task's acceptance criteria against the implementation and run the relevant tests. Record the actual command or walkthrough, outcome, timestamp, and result in `verificationRuns`. Include failures and incomplete checks. Add evidence references to the relevant tasks and summarize the results in `PROGRESS.md`.
4. Update the status only when the evidence supports it. A placeholder or partial implementation earns no implementation credit for that task. Use `implemented` once the implementation criteria are met, and `verified` once all required checks have passed and the evidence is recorded. Keep outstanding checks in `verificationNeeded`.
5. Record blockers and the next actionable milestone in both files. Add meaningful completions and changes to `recentActivity`, and set `lastUpdated` to the actual UTC time of the update.
6. Recalculate totals with `npm run progress:validate -- --write`, then run `npm run progress:validate`. Update the weighted percentages and status summary in `PROGRESS.md` in the same change as `PROGRESS.json`.

The v1 score combines weighted implementation (60%), verification (30%), and release gates (10%). Release gates pass when all their linked tasks are verified. A score of 100% requires the full v1 definition of done in `docs/PRODUCT_SPECIFICATION.md`. Change the task scope or weights only when the agreed product scope changes, or to correct and document an inaccurate baseline.
