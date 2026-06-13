# Agent Maker Flow — UI/UX Design Specification

> A redesign spec for the existing React SPA, derived from `PRD.md` and the `docs/F01–F10` feature specs, and mapped to the current components under `frontend/src/`.
>
> **Status of today's UI:** a functional MVP — all features wired, but styled entirely with scattered inline `style={}` objects, no design tokens, no reusable component library, no hover/focus states, and no responsive layout. This spec defines the target design system and per-screen layouts to close that gap without changing behavior.

---

## 1. Design Principles for this product

The PRD describes a **developer tool** for technically fluent, iteration-driven users who run the same flow repeatedly and need observable feedback. Design decisions follow from that:

1. **Observability is the hero.** The product's core value is making opaque multi-agent runs visible. Status, streaming, and per-node state get the strongest visual treatment (color, motion, contrast).
2. **Dense but legible.** This is a power-user tool, not a marketing site. Favor compact spacing and high information density over generous whitespace — but never at the cost of readability or touch targets.
3. **Calm chrome, loud signal.** Navigation, forms, and panels stay neutral and quiet so that running/complete/error states and streamed output stand out.
4. **Dark-mode-first is appropriate.** Developer audiences expect it, and the canvas + streaming feed read well on dark. Spec includes both themes via tokens; ship dark as default if choosing one.
5. **Every state is designed.** Loading, empty, error, and success are first-class — not afterthoughts. The PRD's error-handling sections are explicit acceptance criteria.

---

## 2. Design Tokens

Replace all hardcoded hex/spacing values with a single token layer. Recommended implementation: **CSS custom properties** in a global `index.css` (the app currently has none), optionally consumed via Tailwind v4 `@theme` or plain CSS. Tokens below are the source of truth.

### 2.1 Color — semantic tokens

Defined as `--color-*`, with light/dark values. The existing ad-hoc colors map onto these:

| Token | Light | Dark | Replaces / Usage |
|---|---|---|---|
| `--bg-base` | `#FFFFFF` | `#0E1116` | App background |
| `--bg-surface` | `#F7F8FA` | `#161B22` | Cards, panels, node bodies |
| `--bg-surface-raised` | `#FFFFFF` | `#1C2330` | Modals, popovers, raised nodes |
| `--bg-inset` | `#EEF0F3` | `#0A0D12` | Code/output blocks, canvas backdrop |
| `--border-subtle` | `#E3E6EA` | `#2A313C` | replaces `#ddd` `#ccc` `#888` borders |
| `--border-strong` | `#C7CCD3` | `#3A434F` | input borders, dividers |
| `--text-primary` | `#1A1F26` | `#E6EAF0` | body text |
| `--text-secondary` | `#5A6472` | `#9AA4B2` | replaces `#666` secondary/meta text |
| `--text-muted` | `#8A93A0` | `#6B7480` | replaces `#9aa0a6` hints, disabled |
| `--accent` | `#1A73E8` | `#4D9CFF` | primary brand / CTA (reuses existing blue) |
| `--accent-hover` | `#155CBA` | `#6BB0FF` | CTA hover |
| `--accent-subtle` | `#E8F0FE` | `#16314F` | selected rows, active nav, focus tints |
| `--success` | `#188038` | `#3FB950` | complete (reuses existing green) |
| `--success-subtle` | `#E6F4EA` | `#10301C` | complete badge bg |
| `--warning` | `#B26A00` | `#D29922` | caution, "agent missing" non-blocking warns |
| `--warning-subtle` | `#FEF3E2` | `#3A2D10` | warning banner bg |
| `--danger` | `#B00020` | `#F85149` | error (reuses existing red) |
| `--danger-subtle` | `#FCE8EC` | `#3D1418` | error badge/banner bg |
| `--running` | `#1A73E8` | `#4D9CFF` | node running state (aliases accent) |
| `--focus-ring` | `#1A73E8` | `#4D9CFF` | keyboard focus outline |

**Status palette** (used by `AgentNode.tsx` and `NodeBlock.tsx` dots/badges) maps directly:
`idle/skipped → --text-muted`, `running → --running`, `complete → --success`, `error → --danger`.

### 2.2 Typography

One UI typeface + one mono typeface. Use the system UI stack (no web-font download cost) and a mono stack for output/code/model names.

```
--font-ui:   ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
--font-mono: ui-monospace, "SF Mono", "JetBrains Mono", "Fira Code", Menlo, monospace;
```

| Role | Size / Line | Weight | Usage |
|---|---|---|---|
| `--text-display` | 28 / 34 | 700 | reserved (rarely used in this tool) |
| `--text-h1` | 22 / 28 | 700 | page titles (`<h2>` today: "Agents", "Flows") |
| `--text-h2` | 17 / 24 | 600 | section headers (`<h3>`: "Monitor", "Embedding") |
| `--text-h3` | 14 / 20 | 600 | card/dialog titles, table-region labels |
| `--text-body` | 14 / 21 | 400 | default body, inputs, table cells |
| `--text-sm` | 13 / 18 | 400 | secondary meta (model names, timestamps) |
| `--text-caption` | 12 / 16 | 500 | badges, field hints, char counters |
| `--text-mono` | 13 / 20 | 400 | streamed output, prompts, IDs |

Body text is 14px (appropriate density for a dev tool) — never below 12px. Use `--font-mono` for streamed node output in `NodeBlock`/`ConversationTurns` so multi-line model output stays readable.

### 2.3 Spacing — 4px base scale

| Token | px | Usage |
|---|---|---|
| `--space-1` | 4 | tight inline gaps (icon↔label) |
| `--space-2` | 8 | related elements (current `gap:8`) |
| `--space-3` | 12 | intra-card, panel gaps (current `gap:12`) |
| `--space-4` | 16 | default padding, form field spacing |
| `--space-5` | 24 | section spacing |
| `--space-6` | 32 | page padding, major sections |

### 2.4 Radius, elevation, motion

```
--radius-sm: 4px;   /* inputs, badges, palette items */
--radius-md: 8px;   /* cards, nodes, buttons, dialogs */
--radius-lg: 12px;  /* panels, large surfaces */
--radius-full: 999px; /* status dots, pills */

--shadow-sm:  0 1px 2px rgba(16,22,30,.06), 0 1px 1px rgba(16,22,30,.04);
--shadow-md:  0 4px 12px rgba(16,22,30,.10);
--shadow-lg:  0 12px 32px rgba(16,22,30,.18);   /* modals */

--ease: cubic-bezier(.2,.0,.2,1);
--dur-fast: 120ms;   /* hover, focus */
--dur-base: 200ms;   /* dialogs, badges */
--motion-pulse: 1.4s; /* running indicator */
```

Respect `prefers-reduced-motion`: disable the streaming pulse and dialog transitions when set.

---

## 3. Core Component Library

These replace today's bare HTML elements + inline styles. Build them as small typed React components under `frontend/src/components/ui/` so every screen reuses them. Each lists states the current UI is missing.

### 3.1 Button
- **Variants:** `primary` (accent fill — Run Flow, Save, Create), `secondary` (surface + border — Cancel, Edit), `ghost` (text only — row actions like Duplicate/Detach), `danger` (Delete confirms).
- **Sizes:** `sm` (28px, row actions), `md` (36px, default), `lg` (40px, primary CTAs).
- **Required states (all missing today):** default, hover (`--accent-hover` / surface darken), active (translateY 1px), focus-visible (`2px --focus-ring` outline, offset 2px), disabled (60% opacity, `cursor:not-allowed`), **loading** (spinner + label, e.g. "Running…", "Embedding…", "Saving…").
- Min touch target 44×44 honored via padding even at `sm`.
- Replaces every plain `<button>` in `FlowToolbar`, `PromptBar`, `AgentForm`, dialogs, list rows.

### 3.2 Input / Textarea / Select
- Label **above** the field (already the pattern in `AgentForm`); make `htmlFor`/`id` pairing mandatory (missing in `SaveFlowDialog`).
- Border `--border-strong`; focus → `--accent` border + focus ring; error → `--danger` border + `--danger-subtle` tint.
- Error message slot below the field (`role="alert"`) — keep existing `aria-invalid` wiring.
- Field hint / char counter slot (Memory form's 8000 counter, AgentForm preamble 2000 / system 32000 limits) right-aligned in `--text-caption`, turns `--danger` when over limit.
- `Select` (provider/model) styled consistently; **disabled model select until provider chosen** keeps existing logic but adds a visible disabled style + helper text "Select a provider first".
- Textareas use `--font-mono` for prompt/system-prompt fields; `resize: vertical` only.

### 3.3 Card / Panel
- `--bg-surface`, `--border-subtle`, `--radius-md`, `--shadow-sm`, padding `--space-4`.
- Optional header row (title `--text-h3` + actions). Used to wrap the three Flows-page regions, Settings sections, and the agent/flow list containers.

### 3.4 Modal / Dialog
- **Add a real overlay** (today dialogs render inline with no backdrop): scrim `rgba(8,11,16,.55)`, centered panel `--bg-surface-raised` + `--shadow-lg` + `--radius-md`, max-width 480 (forms) / 400 (confirms).
- **Focus trap + return focus** on close; `Esc` closes; click-scrim closes (except while submitting).
- Header (`--text-h2`) · body · footer with right-aligned actions (primary last).
- Applies to `SaveFlowDialog`, `DeleteFlowDialog`, `DeleteAgentDialog`, `AgentForm` (currently a modal-ish inline form), and the unsaved-changes confirm.

### 3.5 Badge / Pill
- Small pill: `--radius-full`, `--text-caption`, `--space-1`/`--space-2` padding, colored dot + label.
- **Status badge** (node/run state): subtle bg + matching text/dot per status color in §2.1.
- **Root badge** on `AgentNode`: accent-subtle pill "★ Root" (upgrade from plain text).
- **Provider/model tags** in `AgentList` rows and palette items.

### 3.6 StatusDot
- Reusable 8px `--radius-full` dot driven by `NodeStatus`. When `running`, add a pulsing ring (`@keyframes` scale+fade, `--motion-pulse`). Used by `AgentNode`, `NodeBlock`, and the canvas run badge. This is the single most important micro-component for the "observability" principle.

### 3.7 Toast / inline feedback
- **New:** ephemeral toast (top-right) for success ("Agent saved", "Flow saved") — the PRD calls for save success states the UI lacks.
- Variants success/error/info; auto-dismiss 4s; `aria-live="polite"`.

### 3.8 Skeleton & Spinner
- **Skeleton rows** for list/table loading (replace "Loading agents…" / "Loading flows…" plain text).
- **Spinner** (sm/md) for button-loading and inline "Embedding…" / "Reconnecting…".

### 3.9 EmptyState
- Centered icon + heading + one-line guidance + primary action.
- Replaces today's bare `<p>` empties: "No agents yet" → illustration + "Create your first agent" button; "No saved flows yet"; "No memory records yet"; monitor "No messages yet".

### 3.10 Alert / Banner
- Inline contextual banner (not toast) for persistent conditions: `MissingAgentsBanner` (warning), save/connection errors (danger), embedding-model-changed warning in `EmbeddingModelSelect` (warning). Icon + message + optional list/action, colored subtle bg + left accent border.

---

## 4. Application Shell & Navigation

**Files:** `components/AppShell.tsx`, `components/NavBar.tsx`

Current: a plain `<header>` with title text + inline-styled `NavLink`s + Clerk `UserButton`, then `<main>` with the route outlet.

### Target layout
```
┌──────────────────────────────────────────────────────────────┐
│ TopBar 56px:  ◈ Agent Maker Flow   [Agents][Flows][Settings]   ◐  ◌ user │
│                                      └ segmented nav ┘  theme  avatar     │
├──────────────────────────────────────────────────────────────┤
│  <main>  — page content, max-width 1440 centered,             │
│           padding --space-6 (X) / --space-5 (Y)               │
└──────────────────────────────────────────────────────────────┘
```

- **TopBar:** `--bg-surface`, bottom `--border-subtle`, `--shadow-sm`, sticky. Brand mark (simple node-graph glyph) + wordmark left.
- **Nav as segmented control:** active item = `--accent-subtle` bg + `--accent` text + `aria-current="page"` (replaces bold-only active state). Hover = surface tint. Three items only — well within the 7±2 rule.
- **Theme toggle** (light/dark) — persists to `localStorage`, defaults dark.
- **Clerk `UserButton`** stays, right-aligned, vertically centered.
- **Skip link** ("Skip to content") as first focusable element → `#main`.
- **Mobile (<768px):** nav collapses to a bottom tab bar or hamburger; Flows canvas shows a "best on desktop" notice (canvas DnD is desktop-first per §7).

---

## 5. Screen Specs

### 5.1 Agents Dashboard — `/agents`
**Files:** `pages/AgentsPage.tsx`, `components/agents/AgentList.tsx`, `AgentForm.tsx`, `DeleteAgentDialog.tsx`
**PRD:** F04.

```
┌ Agents ───────────────────────────────────  [ + New agent ] ┐
│ (page title H1)                              (primary btn)   │
│                                                              │
│ ┌ Card: registry table ──────────────────────────────────┐ │
│ │ Name        Provider   Model        Recent-N  Top-K   ⋯ │ │  ← sticky header row
│ │ ─────────────────────────────────────────────────────── │ │
│ │ Summarizer  ⬡ openai   gpt-4o          10       5    ⋮  │ │  ← row hover tint
│ │ Router      ⬡ anthropic claude-3.5…    20       8    ⋮  │ │     actions in kebab/inline ghost btns
│ └──────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```
- **Table → styled Card-wrapped table:** zebra-free, row hover `--accent-subtle`, sticky header, `--text-sm` cells, provider shown as a tag, model in `--font-mono`. Right-align numeric Recent-N/Top-K.
- **Row actions:** ghost icon-buttons (Edit ✎, Duplicate ⧉, Delete 🗑) revealed on row hover/focus; always reachable via keyboard. On narrow widths collapse into a kebab menu.
- **Loading:** 5 skeleton rows. **Empty:** EmptyState (§3.9) with "Create your first agent". **Error:** Alert banner with Retry button (keeps `role="alert"`).
- **AgentForm (modal):** two-column field grouping on ≥640px — left: Name, Provider, Model, Recent-N, Top-K; right: Preamble, System Prompt (taller). Live char counters on Preamble (2000) / System Prompt (32000). Provider→Model cascade keeps existing logic with disabled-state styling + helper text. Inline field errors per §3.2. Footer: Cancel (secondary) + Save/Create (primary, loading state). Duplicate mode prefills "(copy)" — show a small "Duplicating from {name}" caption.
- **DeleteAgentDialog:** danger confirm; when the agent is referenced by flows, show a warning Alert listing those flow names (PRD F04 requirement) before the Delete button.
- **Success:** toast "Agent saved" / "Agent duplicated"; optimistic row insert.

### 5.2 Flows Workspace — `/flows`
**Files:** `pages/FlowsPage.tsx`, `components/flow/*`, `components/monitor/*`
**PRD:** F07, F08, F09, F10. This is the product's primary screen and needs the most layout work.

Current: a flat flex row of `SavedFlowsList (260)` + `[toolbar over palette+canvas600]` + `Monitor (340)`, with hardcoded sizes and 1px borders.

#### Target: three-pane workspace
```
┌ Flows ──────────────────────────────────────────────────────────────────────┐
│ Flow name [Untitled •]      [New] [Save] [Save as]   status: saved 2m ago     │  ← toolbar row, dirty dot
├──────────┬───────────────────────────────────────────────┬───────────────────┤
│ LEFT     │  CANVAS (flex, fills height)                   │  RIGHT: Monitor   │
│ 260px    │  ┌─────────────────────────────────────────┐  │  360px            │
│          │  │  ⟳ Run badge   ⌧ fit  + zoom −           │  │  ┌ Conversation ┐ │
│ Palette  │  │                                          │  │  │ turns (scroll)│ │
│ (agents) │  │     ⬡ Root★ ──▶ ⬡ ──▶ ⬡                 │  │  │  user / asst  │ │
│  ─ drag  │  │      running    idle  idle               │  │  └───────────────┘ │
│          │  │                                          │  │  ┌ Live nodes ──┐ │
│ ───────  │  │   (dotted grid --bg-inset)               │  │  │ ●run Summ…   │ │
│ Saved    │  └─────────────────────────────────────────┘  │  │ ○idle Router │ │
│ flows    │  [FlowToolbar: ▶ Run Flow  · reason text]     │  └───────────────┘ │
│ (list)   │                                                │  ┌ PromptBar ───┐ │
│          │                                                │  │ textarea + ▶ │ │
└──────────┴───────────────────────────────────────────────┴───────────────────┘
```

**Layout mechanics**
- Replace hardcoded `height:600` canvas with a **full-height flex workspace**: `calc(100vh - topbar - toolbar)`. Canvas grows; left/right panes are fixed-width Cards with internal scroll.
- **Left pane = two stacked Cards:** Agent Palette (top, grows) + Saved Flows (bottom, collapsible). Both currently separate top-level regions — consolidating reclaims horizontal space.
- **Resizable splitters** (optional, nice-to-have) between panes; otherwise fixed 260 / flex / 360.

**Toolbar (top row)** — `FlowToolbar` + Flows-page controls merged:
- Left: editable flow name with a **dirty indicator** (• dot + "Unsaved" when `dirty`). Right: `New` / `Save` / `Save as` (secondary) and the prominent **Run Flow** primary button (move it here or keep in canvas overlay — pick one consistent home; recommend canvas-overlay top-left as the run affordance sits with the graph).
- Save status as `--text-sm --text-secondary` ("Saved 2m ago" / "Saving…" spinner / error link).

**Agent Palette** (`AgentPalette.tsx`)
- Section header "Agents" + count. Each item = compact Card: drag-handle cursor, agent name (`--text-body`), model tag (`--font-mono --text-sm`), provider dot. Hover raise (`--shadow-sm`), `cursor:grab` → `grabbing`. Loading skeletons / empty / error states.
- Add a search filter input when >8 agents.

**FlowCanvas** (`FlowCanvas.tsx`) + **AgentNode** (`AgentNode.tsx`)
- Canvas backdrop `--bg-inset` with React Flow dotted `Background`; style `Controls` to match tokens (the default RF CSS clashes with the new palette — override via CSS vars).
- **AgentNode redesign** (most-seen component):
  ```
  ┌─────────────────────────────┐
  │ ● Summarizer        ★ Root  │  ← StatusDot + name (H3) + Root badge
  │ ⬡ gpt-4o                    │  ← provider glyph + model (mono, secondary)
  │ ───────────────────────────  │
  │ status: Running ▮▮▯  ⧉ ⤺ 🗑 │  ← status text + ghost actions (set-root/dup/detach/delete)
  └─────────────────────────────┘
  ```
  - Card surface, `--radius-md`, `--shadow-sm`; border by state: idle `--border-strong`, running `--running` (animated), complete `--success`, error `--danger`, **missing-agent** `--danger` + "⚠ Agent missing" replacing the model line.
  - Input handle (left) / output handle (right) styled as visible accent dots, larger hit area, labeled for SR.
  - Actions become ghost icon-buttons with tooltips; `aria-pressed` on Set-root retained.
  - Running node shows the pulsing StatusDot + a thin indeterminate bar — the live "lighting up" the PRD F10 demands, within ~1s of the event.
- **Invalid connection feedback:** on a rejected cycle/self-loop, flash the target handle `--danger` and show an inline toast "Connection rejected: flows must be acyclic." (PRD F07).

**Conversation Monitor** (`components/monitor/*`)
- **ConversationMonitor:** a right Card with three stacked regions — Conversation (scroll, flex-grow), Live Nodes, PromptBar (pinned bottom). Header "Monitor" + a run-state chip (Idle / Running / Done / Failed).
- **Reconnecting state:** an Alert strip "Reconnecting…" with spinner (PRD F10) above the turns.
- **ConversationTurns:** chat bubbles — user (right, accent-subtle), assistant (left, surface), system/error (full-width Alert style, keep `role="alert"`). Role label as `--text-caption` above bubble; content in `--font-ui`, code/output in `--font-mono`. Auto-scroll to latest; "jump to latest" pill if scrolled up. Empty → EmptyState "No messages yet — enter a prompt and Run Flow".
- **NodeBlock (live per-node):** Card per executing node with StatusDot + name + model, a "streaming…" pulse while running, output body in `--font-mono` with a subtle typing/caret effect as partial chunks arrive, error text in `--danger` (`role="alert"`). Collapse completed blocks to a one-line summary the user can expand.
- **PromptBar:** Card-framed textarea (mono, 3 rows, vertical resize) + primary Run Flow button with loading state; `Cmd/Ctrl+Enter` submits. Disabled-reason text (no root / missing agent / run in progress) shown as `--text-caption --warning` inline, mirroring `FlowToolbar` logic so the user sees *why* run is blocked in both places.

**Persistence dialogs** (F08): `SaveFlowDialog` (name input, 80-char limit + uniqueness error), `DeleteFlowDialog` (danger confirm), unsaved-changes confirm ("Discard unsaved changes to this flow?") — all use the Modal component (§3.4) with real overlay + focus trap. `SavedFlowsList` rows: name + relative "last updated" + Open/Rename/Delete ghost actions; active flow row = `--accent-subtle` + `aria-current`.

**MissingAgentsBanner** (F08): warning Alert above the workspace listing missing agent names; affected nodes flagged in-canvas; Run disabled with matching reason.

### 5.3 Settings — `/settings`
**Files:** `pages/SettingsPage.tsx`, `components/settings|memory/*`
**PRD:** F05.

```
┌ Settings ────────────────────────────────────────────────┐
│ ┌ Embedding ─────────────────────────────────────────────┐│
│ │ Global embedding model                                  ││
│ │ Provider [openai ▾]   Model [text-embedding-3-small ▾]  ││
│ │ ⚠ Changing the model leaves existing records in the old ││  ← warning Alert when records exist
│ │   vector space; re-embed manually.                      ││
│ └─────────────────────────────────────────────────────────┘│
│ ┌ Memory ────────────────────────────────────────────────┐│
│ │ [ Add memory record ]                                   ││
│ │ ┌ textarea (mono) ──────────────────────  1,204 / 8,000┐││  ← live counter, danger when over
│ │ └──────────────────────────────  [Cancel] [Save record]┘││
│ │ ─ Records ──────────────────────────────────────────────││
│ │ ● Ready · text-embedding-3-small · 512 chars   ✎  🗑    ││  ← StatusDot Ready/Embedding/Error
│ │ ◌ Embedding… · …                                        ││
│ └─────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────┘
```
- Two Cards (Embedding, Memory). `EmbeddingModelSelect` → styled Select with the model-mismatch warning as an Alert (§3.10).
- `MemoryRecordForm`: Textarea + live counter (turns `--danger` >8000, blocks submit per F05), button shows "Embedding…" spinner, success toast "Record stored".
- `MemoryRecordList`: each record a compact row with a **StatusDot** for embedding state (Embedding… → Ready / Error), source-text preview (truncated, expandable), model tag, char count, Edit/Delete ghost actions. Empty → EmptyState.

---

## 6. Key Flow: Run Flow streaming (F09 → F10)

The signature interaction. Choreography target so status reflects events within ~1s:

1. **Dispatch:** user types in PromptBar → Run Flow. Button → loading; user turn appears immediately in ConversationTurns; run-state chip → "Running"; SSE opens.
2. **node started:** corresponding `AgentNode` border + StatusDot animate to `--running` (pulse); a `NodeBlock` appears in the monitor with streaming indicator.
3. **partial output:** chunks append into the NodeBlock body (mono, caret); node stays pulsing.
4. **node completed:** node → `--success`, dot stops pulsing; NodeBlock caret removed, block collapsible.
5. **node failed:** node + block → `--danger`, error text `role="alert"`; downstream nodes shown `skipped` (muted).
6. **run finished:** aggregated result rendered as assistant turn (left bubble); run-state chip → "Done"/"Failed"; Run button restored.
7. **Reconnect:** stream drop → "Reconnecting…" Alert; on resume continue rendering, or if the run already finished, fetch + render terminal result (PRD F10 recoverability).

Reduced-motion users get color/text transitions without pulse/caret animation.

---

## 7. Accessibility (WCAG 2.1 AA target)

The current app has decent ARIA roles but gaps. Required:
- **Contrast:** all token pairs meet 4.5:1 (text) / 3:1 (large + UI components). Status colors verified against their subtle backgrounds, not just white.
- **Focus-visible** outlines on every interactive element (`2px --focus-ring`, offset 2px) — entirely missing today.
- **Modal focus management:** trap focus, restore on close, `Esc` to dismiss (missing today).
- **Live regions:** run status / streamed output announced via `aria-live="polite"`; errors `assertive`. Keep existing `role="alert"`/`status`.
- **Forms:** every input has an associated `<label htmlFor>` (fix `SaveFlowDialog`); errors linked via `aria-describedby`.
- **Touch targets** ≥44×44 (row/icon actions get padding even at `sm`).
- **Color independence:** node/run state conveyed by icon/text + dot, never color alone.
- **Canvas a11y:** React Flow DnD is mouse-first — provide a keyboard fallback to add a palette agent to the canvas (e.g. "Add to canvas" button on each palette item) and document the desktop-first nature of graph editing. Add a skip link past the canvas to the monitor.

---

## 8. Responsive behavior

Power-user desktop tool, but degrade gracefully:
- **≥1280px:** full three-pane Flows workspace as specced.
- **768–1279px:** Flows monitor becomes a collapsible drawer; palette collapses to an icon rail with flyout.
- **<768px:** Agents/Settings reflow to single column; Flows canvas shows an EmptyState-style notice "Flow editing works best on a larger screen" with read-only view + the monitor accessible as a full-screen sheet. Nav → bottom tab bar.

---

## 9. Implementation approach (recommended)

Since there is **no global CSS and no styling framework today**, the highest-leverage path:

1. **Add `frontend/src/index.css`** with all §2 tokens as CSS custom properties (`:root` + `[data-theme="dark"]`), imported once in the app entry. This alone removes every hardcoded hex.
2. **Adopt a lightweight styling layer.** Two viable options — pick per team taste:
   - **Tailwind v4** with `@theme` mapping the tokens (fast, utility-driven, good for a dense tool). Aligns with `vite-react-best-practices`.
   - **CSS Modules** per component (zero new deps, scoped, explicit). Good if avoiding utility classes.
3. **Build the `components/ui/` primitives** (§3) first — Button, Input/Select/Textarea, Card, Modal, Badge, StatusDot, Toast, Skeleton, EmptyState, Alert. Add colocated `*.test.tsx` matching the existing vitest+RTL convention.
4. **Migrate screen-by-screen**, replacing inline styles with primitives — order by visibility: AppShell/NavBar → AgentNode + Monitor (the observability surface) → Flows layout → Agents table → Settings → dialogs.
5. **Theme toggle + `prefers-color-scheme`** default; persist to `localStorage`.
6. **No backend changes** — this is purely presentational; the success/error envelopes, SSE events, and route contracts are untouched.

**Suggested phasing** (one PR/commit each, matching the repo's `feat(Fxx): stage N` history convention — e.g. a new `F11` "Design System & UI Polish"):
- Stage 1 — tokens + global CSS + theme toggle.
- Stage 2 — `components/ui/` primitives + tests.
- Stage 3 — Shell, NavBar, AgentNode, Monitor (observability).
- Stage 4 — Flows three-pane layout + palette + dialogs.
- Stage 5 — Agents table + AgentForm + Settings.
- Stage 6 — responsive + a11y pass (focus management, live regions, contrast audit).

---

## 10. Acceptance criteria for the redesign

- [ ] Zero hardcoded color/spacing hex values remain in component files; all reference tokens.
- [ ] Every interactive element has visible hover + focus-visible states.
- [ ] All four states (loading/empty/error/success) are designed for Agents, Flows, Settings, and Monitor.
- [ ] Modals have an overlay, focus trap, focus restore, and `Esc`-to-close.
- [ ] Node/run status is reflected with a pulsing StatusDot + border within ~1s of the SSE event, conveyed by shape/text as well as color.
- [ ] Save/create/delete actions produce a toast + the correct optimistic/error state per the PRD error-handling sections.
- [ ] Light and dark themes both pass WCAG AA contrast for text and UI components.
- [ ] Layout holds at 1280 / 1024 / 768 / 375 widths per §8.
- [ ] No regression in existing vitest suites; new primitives covered by tests.
