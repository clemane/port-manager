# Mission Control Redesign — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Redesign the Port Manager UI/UX with a "Mission Control" cockpit aesthetic — collapsible sidebar, metric cards, enriched status bar, 3-column K8s browser, and polished animations.

**Architecture:** Incremental bottom-up approach. Start with design foundations (fonts, tokens, global CSS), then new atomic components, then upgrade existing components, then rework layout, and finally redesign each view. Each task produces a working commit.

**Tech Stack:** Vue 3 + TypeScript + CSS custom properties + Sora/DM Sans/JetBrains Mono (Google Fonts)

---

### Task 1: Fonts & Global CSS foundations

**Files:**
- Modify: `index.html`
- Modify: `src/style.css`
- Modify: `src/App.vue`

**Step 1: Add Google Fonts to index.html**

Add preconnect and font imports for Sora (400,500,600,700) and DM Sans (400,500,600) before the closing `</head>`:

```html
<link rel="preconnect" href="https://fonts.googleapis.com" />
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
<link href="https://fonts.googleapis.com/css2?family=DM+Sans:wght@400;500;600&family=Sora:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet" />
```

**Step 2: Replace style.css with global reset, font stacks, and keyframes**

Replace entire `src/style.css` content with:
- CSS reset (box-sizing, margin, padding)
- Font variables: `--pm-font-display: 'Sora'`, `--pm-font-body: 'DM Sans'`, `--pm-font-mono: 'JetBrains Mono'`
- Global keyframes: `pm-spin`, `pm-pulse`, `pm-blink`, `pm-shimmer`, `pm-fade-in`, `pm-slide-up`
- `prefers-reduced-motion` media query that disables all animations
- Scrollbar styling

**Step 3: Update App.vue body font to use DM Sans**

In App.vue `<style>` block, change `font-family` on `body` to `var(--pm-font-body), sans-serif`.

**Step 4: Commit**

```
feat: add typography foundations and global keyframes
```

---

### Task 2: Design tokens — colors & theme variables

**Files:**
- Modify: `src/assets/tokens/colors.css`
- Modify: `src/assets/tokens/themes/dark.css`
- Modify: `src/assets/tokens/themes/light.css`
- Modify: `src/assets/tokens/themes/cyberpunk.css`
- Modify: `src/assets/tokens/themes/matrix.css`

**Step 1: Enrich colors.css**

Keep existing vars, add new ones for the Mission Control palette (these are raw colors, not semantic):
- `--mc-bg: #06080f`, `--mc-surface: #0d1117`, `--mc-surface-elevated: #161b22`
- `--mc-border: #21262d`, `--mc-accent: #58a6ff`, `--mc-success: #3fb950`
- `--mc-danger: #f85149`, `--mc-warning: #d29922`
- `--mc-text-1: #e6edf3`, `--mc-text-2: #8b949e`, `--mc-text-3: #484f58`

**Step 2: Update dark.css**

Replace all `--pm-*` values with new Mission Control dark palette. Add new variables:
- `--pm-surface-elevated: var(--mc-surface-elevated)`
- `--pm-accent-glow: rgba(88, 166, 255, 0.15)`
- `--pm-gradient-ambiance: radial-gradient(ellipse at 50% 0%, rgba(88,166,255,0.03) 0%, transparent 70%)`
- `--pm-status-pulse-color: var(--mc-success)`

**Step 3: Update light.css**

New light palette:
- bg `#f6f8fa`, surface `#ffffff`, surface-elevated `#f0f3f6`
- border `#d0d7de`, accent `#0969da`
- text-primary `#1f2328`, secondary `#656d76`, muted `#8c959f`
- Add `--pm-surface-elevated`, `--pm-accent-glow`, `--pm-gradient-ambiance`, `--pm-status-pulse-color`

**Step 4: Update cyberpunk.css**

Add new vars (`--pm-surface-elevated`, `--pm-accent-glow` with cyan glow, `--pm-gradient-ambiance` with double cyan/magenta, `--pm-status-pulse-color`).

**Step 5: Update matrix.css**

Add new vars (`--pm-surface-elevated`, `--pm-accent-glow` with green glow, `--pm-gradient-ambiance` with green halo, `--pm-status-pulse-color`).

**Step 6: Commit**

```
feat: update design tokens with Mission Control palette
```

---

### Task 3: New component — PmStatusDot

**Files:**
- Create: `src/components/ui/PmStatusDot.vue`
- Modify: `src/components/ui/index.ts` (add export)

**Step 1: Create PmStatusDot.vue**

Props: `status: 'running' | 'error' | 'stopped' | 'idle'`, `size?: number` (default 8).

Template: single `<span>` with dynamic class. Styles use the keyframes from style.css (`pm-pulse` for running, `pm-blink` for error, static for stopped/idle). Colors from `--pm-success`, `--pm-danger`, `--pm-text-muted`.

**Step 2: Export from index.ts**

**Step 3: Commit**

```
feat: add PmStatusDot component with animated states
```

---

### Task 4: New component — PmSkeletonLoader

**Files:**
- Create: `src/components/ui/PmSkeletonLoader.vue`
- Modify: `src/components/ui/index.ts` (add export)

**Step 1: Create PmSkeletonLoader.vue**

Props: `variant: 'table' | 'card' | 'text'`, `lines?: number` (default 5).

Template: For `table` variant, renders N rows of skeleton bars with varied widths (70%, 40%, 90%, 55%, 80% repeating). Uses `pm-shimmer` keyframe animation.

**Step 2: Export from index.ts**

**Step 3: Commit**

```
feat: add PmSkeletonLoader component
```

---

### Task 5: New component — PmMetricCard

**Files:**
- Create: `src/components/ui/PmMetricCard.vue`
- Modify: `src/components/ui/index.ts` (add export)

**Step 1: Create PmMetricCard.vue**

Props: `label: string`, `value: number`, `icon?: string` (SVG path or slot), `color?: 'accent' | 'success' | 'danger' | 'warning' | 'muted'`, `animate?: boolean`.

Template: Card with left border-color (3px) based on `color` prop. Value in Sora 28px bold with count-up animation (CSS counter or JS requestAnimationFrame from 0 to value over 400ms). Label in DM Sans 12px muted below.

**Step 2: Export from index.ts**

**Step 3: Commit**

```
feat: add PmMetricCard component with count-up animation
```

---

### Task 6: Redesign PmButton

**Files:**
- Modify: `src/components/ui/PmButton.vue`

**Step 1: Update styles**

- border-radius: 6px
- Primary: add `background: linear-gradient(...)` subtle gradient + `box-shadow: 0 1px 3px var(--pm-accent-glow)`
- Ghost: add `transform: scale(0.97)` default, `scale(1)` on hover (pop effect)
- Danger: add subtle pulse keyframe on hover
- All: `transition: all 0.15s ease`, `font-family: var(--pm-font-body)`
- Add `:focus-visible` outline ring `0 0 0 2px var(--pm-accent-glow)`

**Step 2: Commit**

```
style: redesign PmButton with gradient, pop effect, and focus ring
```

---

### Task 7: Redesign PmInput

**Files:**
- Modify: `src/components/ui/PmInput.vue`

**Step 1: Update template and styles**

- Add wrapper `<div>` for search variant with SVG loupe icon positioned absolutely left
- Height: 36px, background: `var(--pm-surface-elevated)`
- Focus: `border-color: var(--pm-accent)` + `box-shadow: 0 0 0 3px var(--pm-accent-glow)`
- font-family: `var(--pm-font-body)`

**Step 2: Commit**

```
style: redesign PmInput with elevated surface and focus glow
```

---

### Task 8: Redesign PmBadge

**Files:**
- Modify: `src/components/ui/PmBadge.vue`

**Step 1: Update template and styles**

- Add a colored dot `<span>` (6px) before slot content using `::before` pseudo-element
- Compact padding: `2px 8px`, font-size: 11px
- Add subtle 1px border with badge color at 20% opacity
- font-family: `var(--pm-font-body)`

**Step 2: Commit**

```
style: redesign PmBadge with status dot and subtle border
```

---

### Task 9: Redesign PmTable

**Files:**
- Modify: `src/components/ui/PmTable.vue`

**Step 1: Update header styles**

- Background: `var(--pm-surface-elevated)`
- Text: uppercase, 11px, `letter-spacing: 0.05em`, `font-family: var(--pm-font-body)`
- Sticky position: `position: sticky; top: 0; z-index: 1`

**Step 2: Update row styles**

- Height: 44px (padding: 10px 12px)
- Hover: `background` + `transform: translateX(2px)` with transition
- font-family: `var(--pm-font-body)`, mono data via slot override in views

**Step 3: Add staggered reveal**

Add `animation: pm-slide-up 0.3s ease forwards` on rows with `animation-delay` computed from index (capped at 15). Use `opacity: 0` initial, animated to 1.

**Step 4: Add skeleton loading slot**

Add a `loading` prop (boolean). When true, show `PmSkeletonLoader variant="table"` instead of tbody.

**Step 5: Update pagination**

Compact style: "1-50 of 142" + chevron SVG buttons.

**Step 6: Commit**

```
style: redesign PmTable with sticky header, stagger reveal, skeleton loading
```

---

### Task 10: Redesign PmModal

**Files:**
- Modify: `src/components/ui/PmModal.vue`

**Step 1: Update overlay**

- `backdrop-filter: blur(4px)`, `background: rgba(0,0,0,0.6)`

**Step 2: Add enter/leave transition**

Wrap modal in `<Transition name="modal">`. CSS:
- `.modal-enter-from`: `opacity: 0; transform: scale(0.95)`
- `.modal-enter-to`: `opacity: 1; transform: scale(1)`
- `.modal-leave-to`: `opacity: 0; transform: scale(0.95)`
- Duration: 200ms

**Step 3: Update spacing**

- Remove header `border-bottom`
- Title: `font-family: var(--pm-font-display)`, 16px
- Increase body padding to 24px
- Footer gap: 8px, aligned right

**Step 4: Commit**

```
style: redesign PmModal with backdrop blur and scale transition
```

---

### Task 11: Redesign PmToast and PmToastContainer

**Files:**
- Modify: `src/components/ui/PmToast.vue`
- Modify: `src/components/layout/PmToastContainer.vue`

**Step 1: Move container to bottom-right**

Change `top: 16px` to `bottom: 16px` in PmToastContainer.

**Step 2: Add progress bar**

In toast item, add a `<div class="toast-progress">` that animates width from 100% to 0% over the toast duration. Use `animation: pm-toast-countdown linear forwards`.

**Step 3: Add type icons**

Add SVG icons (checkmark for success, X for error, info circle for info, warning triangle for warning) as inline SVGs before the message text.

**Step 4: Update transitions**

Slide from bottom-right: `translateY(16px)` enter, `translateX(30px)` leave.

**Step 5: Commit**

```
style: redesign toasts with progress bar, icons, and bottom-right position
```

---

### Task 12: Redesign PmSelect

**Files:**
- Modify: `src/components/ui/PmSelect.vue`

**Step 1: Update trigger styles**

- Height: 36px, background: `var(--pm-surface-elevated)`
- Focus/open: border accent + glow
- font-family: `var(--pm-font-body)`

**Step 2: Update dropdown**

- Background: `var(--pm-surface-elevated)`
- Add `<Transition name="dropdown">` with scale-Y from 0.95

**Step 3: Commit**

```
style: redesign PmSelect with elevated surface and dropdown transition
```

---

### Task 13: Redesign PmThemeSwitcher

**Files:**
- Modify: `src/components/ui/PmThemeSwitcher.vue`

**Step 1: Add color preview dots**

Add a 10px colored circle before each theme label showing the theme's accent color. Dark=blue, Light=blue darker, Cyber=magenta, Matrix=green.

**Step 2: Support collapsed mode**

Add `collapsed` prop (boolean). When true, render a single icon button that cycles through themes on click. Icon: sun/moon SVG.

**Step 3: Commit**

```
style: redesign PmThemeSwitcher with color previews and collapsed mode
```

---

### Task 14: Redesign PmSidebar — collapsible

**Files:**
- Modify: `src/components/layout/PmSidebar.vue`

**Step 1: Add collapsed state**

Add `const collapsed = ref(true)`. Toggle with hamburger button at top. Add mouseenter/mouseleave with 300ms delay for hover expand.

**Step 2: Replace Unicode icons with SVG**

Replace Unicode nav icons with inline SVG icons (simple line icons for Dashboard, Kubernetes, Forwards, Ngrok, Settings).

**Step 3: Update layout**

- Collapsed: `width: 56px`, icons centered, labels hidden (`opacity: 0`, `width: 0`, `overflow: hidden`)
- Expanded: `width: 200px`, labels visible
- `transition: width 200ms ease-out`
- Labels: `transition: opacity 100ms ease`

**Step 4: Active indicator**

Replace background highlight with left border bar: `border-left: 3px solid var(--pm-accent)` on active item. Remove background.

**Step 5: Logo**

"PM" in Sora bold, accent color. In collapsed mode: just "P". Subtle glow on hover.

**Step 6: Footer theme switcher**

Pass `collapsed` prop to PmThemeSwitcher.

**Step 7: Commit**

```
feat: redesign sidebar with collapse/expand and SVG icons
```

---

### Task 15: Add page header & transitions to AppLayout

**Files:**
- Modify: `src/components/layout/AppLayout.vue`
- Modify: `src/router/index.ts`

**Step 1: Add contextual header bar**

New `<header>` element between sidebar and main content, 56px height. Display page title from route meta. Add slot for right-side actions.

**Step 2: Add route meta for page titles**

In router, add `meta: { title: 'Dashboard' }` etc. to each route.

**Step 3: Add page transitions**

Wrap `<router-view>` in `<router-view v-slot="{ Component }"><Transition name="page"><component :is="Component" /></Transition></router-view>`. CSS transitions: fade + translateY.

**Step 4: Add ambient gradient**

On `.app-layout__main`, add `background: var(--pm-gradient-ambiance)`.

**Step 5: Commit**

```
feat: add contextual header, page transitions, and ambient gradient
```

---

### Task 16: Redesign PmStatusBar

**Files:**
- Modify: `src/components/layout/PmStatusBar.vue`

**Step 1: Update height and layout**

Height: 32px. Add `PmStatusDot` for forwards indicator (pulse when > 0).

**Step 2: Add clickable metrics**

Wrap each metric in `<router-link>` to its corresponding view. Style as subtle clickable text.

**Step 3: Add tunnel count**

Accept new `tunnelCount` prop. Display "N tunnels" with ngrok link.

**Step 4: Add connection indicator**

Right side: dot green/red for backend connectivity status.

**Step 5: Commit**

```
feat: redesign status bar with live indicators and clickable metrics
```

---

### Task 17: Redesign DashboardView

**Files:**
- Modify: `src/views/DashboardView.vue`

**Step 1: Add metric cards row**

Import `PmMetricCard`. Compute 4 metrics from ports data: Listening count (green), Established count (blue), Conflicts count (red), Total count (muted). Render in CSS grid `repeat(4, 1fr)`.

**Step 2: Redesign filter bar**

Wrap filters in `<div>` with `background: var(--pm-surface)`, padding, border-radius. Add protocol filter select. Search input with loupe icon.

**Step 3: Style conflict rows**

Add conditional class on conflict rows: `background: rgba(var(--pm-danger-rgb), 0.05)`.

**Step 4: Use Sora for title, DM Sans for body, JetBrains Mono for data**

Apply font-family vars to `.view-title`, `.pid`, `.process-name`.

**Step 5: Commit**

```
feat: redesign Dashboard with metric cards and enhanced filters
```

---

### Task 18: Redesign K8sBrowserView — 3 columns

**Files:**
- Modify: `src/views/K8sBrowserView.vue`

**Step 1: Replace dropdown selectors with 3-column layout**

Layout: `display: grid; grid-template-columns: 200px 250px 1fr; height: calc(100vh - 160px)`.

**Step 2: Column 1 — Clusters & Namespaces**

List clusters as expandable items (click to show namespaces). Active namespace highlighted with accent. No more dropdown selects.

**Step 3: Column 2 — Resources tree**

Existing TreeView but with updated styling. Add resource type icons (circle for Service, hexagon for Pod) via CSS/SVG. Port count badge next to each resource.

**Step 4: Column 3 — Detail panel**

Resource details with port list. Forward button inline on each port row. Empty state when no resource selected: "Select a resource to view ports" centered with muted icon.

**Step 5: Add fade transitions**

Each column content wrapped in `<Transition name="fade">` keyed on selection state.

**Step 6: Commit**

```
feat: redesign K8s Browser with 3-column layout
```

---

### Task 19: Redesign ForwardsView

**Files:**
- Modify: `src/views/ForwardsView.vue`

**Step 1: Add metric cards**

3 cards: Active (green), Errored (red), Favorites (accent). Computed from forwards and favorites data.

**Step 2: Redesign favorites section**

Grid layout `repeat(auto-fill, minmax(280px, 1fr))`. Cards more compact with group badge. Star icon. Launch button prominent.

**Step 3: Update table**

Use PmStatusDot instead of PmBadge for status column. Star icon button for save-as-favorite. URL as monospace clickable with copy icon that shows checkmark for 1.5s after click.

**Step 4: Commit**

```
feat: redesign Forwards with metric cards and grid favorites
```

---

### Task 20: Redesign NgrokView

**Files:**
- Modify: `src/views/NgrokView.vue`

**Step 1: Redesign launch tunnel section**

Wrap in card with `background: var(--pm-surface-elevated)`, `border: 1px solid var(--pm-accent-glow)`. Horizontal layout: domain select + port input + launch button inline.

**Step 2: Update active tunnels table**

Use PmStatusDot. URL in monospace with copy icon.

**Step 3: Redesign domains as chips**

Replace domain list with chips/tags layout. Each domain = inline pill with domain text + X delete button. Add domain input inline at the end.

**Step 4: Commit**

```
feat: redesign Ngrok with launch card and domain chips
```

---

### Task 21: Redesign SettingsView

**Files:**
- Modify: `src/views/SettingsView.vue`

**Step 1: Wrap each section in a card**

Each settings section gets: `background: var(--pm-surface)`, `border: 1px solid var(--pm-border)`, `border-radius: var(--pm-radius)`, `padding: 20px`. Section title in Sora, description in DM Sans muted.

**Step 2: Theme section**

Add color preview squares next to each theme button.

**Step 3: Credentials toggle visibility**

Replace password inputs with toggle-able visibility. Add eye/eye-off icon button to show/hide value.

**Step 4: Commit**

```
feat: redesign Settings with card sections and credential toggle
```

---

### Task 22: Final polish — wire up App.vue and status bar

**Files:**
- Modify: `src/App.vue`
- Modify: `src/components/layout/AppLayout.vue`

**Step 1: Pass tunnel count to status bar**

Add `tunnelCount` ref in App.vue, fetch from `list_tunnels` in the existing polling. Pass down through AppLayout to PmStatusBar.

**Step 2: Verify all themes render correctly**

Manually test each theme switch. Ensure all new variables (`--pm-surface-elevated`, `--pm-accent-glow`, etc.) are defined in every theme file.

**Step 3: Commit**

```
feat: wire up tunnel count in status bar and finalize polish
```
