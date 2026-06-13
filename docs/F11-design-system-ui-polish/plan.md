# Implementation Plan: Design System & UI Polish

**Status:** implemented and merged to `master` (PR #1 stages 1–5, PR #2 stage 6).

**Prerequisites:**
- Frontend: React + Vite + TypeScript, TanStack Query, `@xyflow/react`, Clerk, Vitest + React Testing Library — all already configured.
- F01–F10 complete: every screen is wired and functional but styled with scattered inline `style={}` objects — no design tokens, component library, hover/focus states, or responsive layout (the gap this feature closes).
- No backend changes and no new environment variables — F11 is purely presentational; the success/error envelopes, SSE event contract, and route contracts are untouched.

**Stack decision (see spec §9):** CSS custom properties in `src/index.css` as the single token source of truth, consumed via **CSS Modules** per component (zero new deps — no Tailwind). Dark ships as the default theme, applied before first paint.

## Stage 1: Tokens + global CSS + theme toggle

**1. Token layer & base styles** - Add `src/index.css` with the full §2 token set (`:root` light values, `[data-theme="dark"]` overrides), base reset, `:focus-visible` rings, `prefers-reduced-motion` handling, and a `.skip-link`.

**2. Theme module** - `lib/theme.ts` (resolve order stored > OS > dark, applied pre-paint) + `hooks/useTheme.tsx` `ThemeProvider` + a `ThemeToggle` control.

## Stage 2: Core UI primitive library

**3. `components/ui/` primitives** - Button, Input/Textarea/Select (+ shared `Field` render-prop), Card, Modal (scrim + focus-trap + focus-restore + `Esc`, via portal), Badge, StatusDot (pulsing ring while running; `StatusKind` mirrors `NodeStatus`), Alert, Toast (`ToastProvider` + `useToast`, auto-dismiss), Skeleton, Spinner, EmptyState — with a barrel `index.ts` and colocated `*.test.tsx`.

## Stage 3: Shell, nav, AgentNode, monitor (observability surface)

**4. Shell & nav** - `AppShell` sticky topbar + skip-link + `ToastProvider`; `NavBar` segmented control with `aria-current`.

**5. Observability components** - `AgentNode` redesign (status border + StatusDot + Root badge); monitor `ConversationMonitor` / `ConversationTurns` / `NodeBlock` / `PromptBar` migrated to primitives.

## Stage 4: Flows three-pane layout + palette + dialogs

**6. Flows workspace** - `FlowsPage` full-height three-pane layout; `FlowCanvas` React-Flow chrome token overrides; palette / saved-flows / toolbar / banner restyle; Save/Delete/unsaved dialogs → `Modal`; invalid-connection + save toasts.

## Stage 5: Agents table + AgentForm + Settings

**7. Agents & settings** - Agents Card-wrapped table with row actions; `AgentForm` → modal two-column layout + live char counters; `DeleteAgentDialog` → `Modal`; Settings as two Cards.

## Stage 6: Responsive + a11y pass

**8. Responsive layout** - Mobile nav → fixed bottom tab bar (`--bottomnav-height` token, 0 on desktop); Flows monitor → off-canvas drawer below the three-pane width; phones stack panes with a desktop-first canvas notice; agents table scrolls horizontally.

**9. Accessibility** - Keyboard "Add to canvas" fallback for the mouse-only palette drag-and-drop; skip-to-monitor link; `aria-live="polite"` live-nodes region; `aria-expanded`/`aria-controls` on the monitor toggle; ≥44px touch targets on coarse pointers.

## Verification (each stage)

- Frontend stages: `npm run typecheck` + `npm test` + `npm run build`.
- Critical constraint: every existing component test pins accessible names (status text, button names, field labels) — all preserved, so the full vitest suite stays green at each stage (zero hardcoded hex in component files).
- One commit per stage, following the `feat(F11): stage N — …` convention.

## Post-spec follow-ups (after the staged redesign, on `master`)

- **Clerk auth pages** centered + themed via a shared `AuthLayout` + a `clerkAppearance` mapped to the design tokens.
- **Bundle code-splitting** (spec acceptance had no perf criterion, but the single 579 kB chunk tripped Vite's 500 kB warning): route-level `React.lazy` per page behind Suspense + a `manualChunks` vendor split — entry chunk 579 kB → 13 kB, React Flow lazy on `/flows` only.
