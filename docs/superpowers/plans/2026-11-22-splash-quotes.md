# Splash Quotations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show one locally bundled, source-reviewed quotation about books, reading, or libraries at each Pusthakam launch without delaying or compromising offline startup.

**Architecture:** A dedicated quote catalogue provides 100 reviewed public-domain entries and their audit metadata. A pure selection helper chooses a quote while excluding the immediately prior selection when storage works; `StartupSplashScreen` receives one stable selection and renders the quote plus author throughout its lifetime.

**Tech Stack:** React, TypeScript, Vitest, Testing Library, browser localStorage; no network or third-party runtime dependency.

**Spec:** `docs/superpowers/specs/2026-11-22-splash-quotes-design.md`

## Global Constraints

- Exactly 100 authentic, source-reviewed public-domain quotations bundled in the application.
- No runtime network access, telemetry, remote font, quote service, or generated quotation.
- Keep the splash screen's existing three-second minimum, primary tagline, and accessible status region.
- Display only quote text and author in the splash; keep sources and verification notes in local documentation.
- Use existing design tokens and keep the product interface Malayalam-free.
- Preserve unrelated working-tree content and update progress records after meaningful work.

## Review Focus

- Corrupted or blocked local storage must never prevent splash rendering; test safe fallback selection in Task 2.
- A quote must stay unchanged during progress updates in one launch; test stable component rendering in Task 3.
- The prior quote must be excluded only when there is another valid quote; test single-entry fallback in Task 2.
- Every entry needs a stable source and attribution; test collection metadata and exact entry count in Task 1.
- A quotation must remain legible in the existing splash layout, including keyboard/screen-reader behavior; test accessible render and inspect in Task 3.

---

### Task 1: Curated local quotation catalogue

**Files:**
- Create: `src/features/boot/splashQuotes.ts`
- Create: `src/features/boot/splashQuotes.test.ts`
- Create: `docs/SPLASH_QUOTE_SOURCES.md`

**Interfaces:**
- Produces: `SplashQuote`, `SPLASH_QUOTES`, and `SPLASH_QUOTE_STORAGE_KEY` for Tasks 2 and 3.
- Consumes: reviewed source metadata gathered from primary texts or authoritative archives.

- [ ] **Step 1: Write failing collection-integrity tests**

```ts
expect(SPLASH_QUOTES).toHaveLength(100);
expect(new Set(SPLASH_QUOTES.map((quote) => quote.id)).size).toBe(100);
expect(SPLASH_QUOTES.every((quote) => quote.text && quote.author && quote.sourceUrl && quote.verificationNote)).toBe(true);
```

- [ ] **Step 2: Run the focused test to verify it fails**

Run: `npm.cmd test -- src/features/boot/splashQuotes.test.ts`

Expected: FAIL because the catalogue module does not exist.

- [ ] **Step 3: Implement the catalogue and source record**

Create `SplashQuote` with `id`, `text`, `author`, `work`, `sourceUrl`, and `verificationNote`. Add exactly 100 unique entries and document their full source/verification data in `docs/SPLASH_QUOTE_SOURCES.md`.

- [ ] **Step 4: Run the focused test to verify it passes**

Run: `npm.cmd test -- src/features/boot/splashQuotes.test.ts`

Expected: PASS.

### Task 2: Stable non-repeating quote selection

**Files:**
- Modify: `src/features/boot/splashQuotes.ts`
- Modify: `src/features/boot/splashQuotes.test.ts`

**Interfaces:**
- Consumes: `SPLASH_QUOTES`, `SplashQuote`, and `SPLASH_QUOTE_STORAGE_KEY` from Task 1.
- Produces: `selectSplashQuote(random?: () => number): SplashQuote` and `readStoredQuoteId(): string | null` for Task 3.

- [ ] **Step 1: Write failing selection tests**

```ts
expect(selectSplashQuote(() => 0).id).not.toBe(previousId);
expect(selectSplashQuote(() => 0)).toEqual(SPLASH_QUOTES[0]);
expect(() => selectSplashQuote(() => 0)).not.toThrow();
```

Cover a stored previous value, no available storage, malformed stored value, and a one-entry fallback list through an exported testable helper.

- [ ] **Step 2: Run the focused test to verify it fails**

Run: `npm.cmd test -- src/features/boot/splashQuotes.test.ts`

Expected: FAIL because selection is not implemented.

- [ ] **Step 3: Implement `selectSplashQuote` and safe storage handling**

Use the previous quote ID only to exclude one entry from a non-empty candidate list. Persist the newly selected ID after selection, catch storage errors, and never select a second quote during a component lifetime.

- [ ] **Step 4: Run the focused test to verify it passes**

Run: `npm.cmd test -- src/features/boot/splashQuotes.test.ts`

Expected: PASS.

### Task 3: Splash presentation

**Files:**
- Modify: `src/features/boot/StartupSplashScreen.tsx`
- Modify: `src/features/boot/StartupSplashScreen.test.tsx`

**Interfaces:**
- Consumes: `selectSplashQuote(): SplashQuote` from Task 2.
- Produces: a stable rendered quotation and author credit in the existing splash screen.

- [ ] **Step 1: Write a failing splash render test**

```tsx
expect(screen.getByTestId("startup-splash-quote")).toHaveTextContent(expectedQuote.text);
expect(screen.getByText(`— ${expectedQuote.author}`)).toBeInTheDocument();
```

Assert that advancing splash progress does not replace the quotation.

- [ ] **Step 2: Run the focused test to verify it fails**

Run: `npm.cmd test -- src/features/boot/StartupSplashScreen.test.tsx`

Expected: FAIL because no quotation is rendered.

- [ ] **Step 3: Render the stable quote and author in `StartupSplashScreen`**

Select the quote once via lazy React state and replace the existing closing line with a readable, token-based quotation block. Preserve the current accessibility status region and duration behavior.

- [ ] **Step 4: Run focused tests, then all frontend checks**

Run: `npm.cmd test -- src/features/boot/StartupSplashScreen.test.tsx src/features/boot/splashQuotes.test.ts; npm.cmd run typecheck; npm.cmd run lint; npm.cmd test; npm.cmd run build`

Expected: all pass, with no preview sample data included in the production bundle.

### Task 4: Project records and visual verification

**Files:**
- Modify: `PROGRESS.md`
- Modify: `PROGRESS.json`
- Modify: `docs/SESSION_HANDOFF.md`
- Modify: `CHANGELOG.md`

**Interfaces:**
- Consumes: test results and a fresh rendered splash screen from Tasks 1–3.
- Produces: accurate resume notes and evidence without claiming authenticity beyond reviewed sources.

- [ ] **Step 1: Run the project progress validator**

Run: `npm.cmd run progress:validate -- --write; npm.cmd run progress:validate`

Expected: valid progress data with a recorded non-roadmap branding enhancement.

- [ ] **Step 2: Check the splash in the running app**

Start the app, capture a fresh splash render, and verify it contains a readable quote, author credit, unchanged tagline, and no stale architecture badge.

- [ ] **Step 3: Record only actual evidence**

Update all progress records with test counts, visual outcome, known limits, and next release milestone.

## Self-review

- Spec coverage: Tasks 1–3 cover the local collection, audit trail, selection, no-immediate-repeat behavior, stable display, author credit, offline behavior, and unchanged duration. Task 4 covers evidence and records.
- Type consistency: `SplashQuote` is produced in Task 1; Task 2 returns it; Task 3 consumes it.
- Review Focus coverage: Task 1 covers metadata; Task 2 covers storage, malformed values, previous-choice exclusion and one-entry fallback; Task 3 covers stable accessible rendering.
- Proportion: Four independently testable tasks map directly to the approved design without including unrelated logo or release work.
