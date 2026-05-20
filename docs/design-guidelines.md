# Design Guidelines — Unwrap

**Vibe:** Premium developer tool. Linear × VS Code × JetBrains Rider, applied to reverse engineering.
**NOT:** 90s gray, hacker neon, modal-dialog soup, generic SaaS purple.

**Skill profile:** VARIANCE=8 (asymmetric), MOTION=6 (fluid CSS + selective spring), DENSITY=4 (daily-app, not cockpit).

---

## 1. Brand Personality

| Attribute | Value |
|-----------|-------|
| Tone | Calm, precise, confident |
| Energy | Focused (low-key, not bouncy) |
| Authority | High — looks like pro tool |
| Playfulness | 1/10 — serious utility |
| Density | Moderate — leave breath |

**Tagline candidates** (to validate with user):
- "Decode anything you own."
- "From bundle to source."
- "Unwrap the build."

**Name candidates** (working: Unwrap):
- **Unwrap** — direct, verb-as-name (à la Linear)
- **Strata** — layers of a build
- **Decipher** — clean meaning, two-syllable
- **Bedrock** — foundational reveal
- (User chooses; codename Unwrap used in repo)

---

## 2. Color System

### Base (dark first, dark-only for v1)

| Token | Hex | Use |
|-------|-----|-----|
| `bg-base` | `#09090B` (zinc-950) | App background |
| `bg-surface` | `#111114` | Panel surface |
| `bg-elevated` | `#18181B` (zinc-900) | Cards, dropdowns |
| `bg-overlay` | `#27272A` (zinc-800) | Hover, active row |
| `border-subtle` | `rgba(63,63,70,0.35)` | Hairlines |
| `border-default` | `#27272A` | Dividers |
| `border-strong` | `#3F3F46` | Inputs, focus rings |

### Text

| Token | Hex | Use |
|-------|-----|-----|
| `text-primary` | `#F4F4F5` (zinc-100) | Headlines, body |
| `text-secondary` | `#A1A1AA` (zinc-400) | Labels, meta |
| `text-tertiary` | `#71717A` (zinc-500) | Hint, placeholder |
| `text-disabled` | `#52525B` (zinc-600) | Disabled |

### Accent (single — Emerald)

**Why Emerald** (#10B981 family): Reads as "extract/uncover", differentiates from blue/red dev tools, refined version of the matrix-green RE trope. Saturation tuned below 80% per skill rules.

| Token | Hex | Use |
|-------|-----|-----|
| `accent` | `#10B981` (emerald-500) | Primary action, brand |
| `accent-strong` | `#059669` (emerald-600) | Hover/pressed |
| `accent-muted` | `rgba(16,185,129,0.12)` | Tinted bg (selected row) |
| `accent-glow` | `rgba(16,185,129,0.20)` | Focus ring (subtle) |

### Semantic

| Token | Hex | Use |
|-------|-----|-----|
| `success` | `#22C55E` | Success status |
| `warning` | `#F59E0B` | Warning status |
| `danger` | `#EF4444` | Error status |
| `info` | `#0EA5E9` | Info status |

### File-type accent palette (asset tree icons)

| Kind | Color |
|------|-------|
| Texture/Image | `#F472B6` (rose-400, desaturated to 70%) |
| Audio | `#A78BFA` (violet-400 — sparingly, **only as file-type tint**) |
| Mesh/3D | `#FBBF24` (amber-400) |
| Script/Code | `#10B981` (emerald, same as accent) |
| Text/Localization | `#60A5FA` (blue-400) |
| Binary/Unknown | `#71717A` (zinc-500) |

*Note:* The violet tint is exempt from the "no lila" rule because it is **functional file-type categorization**, not aesthetic.

---

## 3. Typography

### Fonts

| Stack | Use | Source |
|-------|-----|--------|
| **Satoshi** (450/500/700) | UI text, headlines, labels | Fontshare |
| **JetBrains Mono** (400/500/700) | Code, hex, numeric data, file paths | Google Fonts |

**Banned:** Inter, Poppins, any serif, system-ui fallback as primary.

### Scale (Satoshi unless noted)

| Token | Size / Line | Use |
|-------|-------------|-----|
| `display` | 48/52, weight 700, tracking -0.02em | Welcome hero only |
| `h1` | 28/32, weight 600, tracking -0.015em | Project name, modal title |
| `h2` | 20/26, weight 600, tracking -0.01em | Section header |
| `h3` | 16/22, weight 600 | Sub-section |
| `body` | 14/22, weight 450 | Default text |
| `body-sm` | 13/20, weight 450 | Compact tables, sidebar |
| `caption` | 12/16, weight 500 | Meta, status bar |
| `code-md` | 14/22, JetBrains Mono | Code editor default |
| `code-sm` | 13/20, JetBrains Mono | Hex view, file paths |
| `mono-tab` | 12/16, JetBrains Mono, weight 500 | Tab labels |

**Rules:**
- All numerics (sizes, counts, hashes, hex) use JetBrains Mono.
- All file paths use JetBrains Mono.
- All editable text uses JetBrains Mono.
- Tracking tight on display + h1/h2 only.

---

## 4. Spacing & Sizing

### Scale (4px base)

`0, 2, 4, 6, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96, 128`

**Tailwind shorthand:** `0, 0.5, 1, 1.5, 2, 3, 4, 5, 6, 8, 10, 12, 16, 20, 24, 32`.

### Component sizing

| Element | Height |
|---------|--------|
| Title bar | 36px |
| Top toolbar | 44px |
| Sidebar item | 28px |
| Tab | 32px |
| Status bar | 24px |
| Button (md) | 32px |
| Input (md) | 32px |
| Tree row | 24px |

### Border radius

| Token | Value | Use |
|-------|-------|-----|
| `r-sm` | 4px | Inputs, small chips |
| `r-md` | 6px | Buttons, list items |
| `r-lg` | 8px | Cards, panels |
| `r-xl` | 12px | Modals, popovers |
| `r-2xl` | 16px | Hero / featured surfaces |
| `r-full` | 9999px | Pills, avatars |

---

## 5. Layout — App Shell

```
┌────────────────────────────────────────────────────────────────────┐
│ TITLE BAR  Unwrap  ⌄ <project>           [⊟] [▢] [✕]                │ 36
├────────────────────────────────────────────────────────────────────┤
│ TOOLBAR    ←  →   Recents ⌄    [⌘K Search…]            🔔 ⚙       │ 44
├──────┬─────────────────────────────────────────────────┬───────────┤
│      │ Tab 1 │ Tab 2 │ Tab 3                       + ⊕ │ INSPECTOR │ 32 (tabs)
│ FILES│ ┌─────────────────────────────────────────────┐ │           │
│ SRCH │ │                                             │ │  Props    │
│ XREF │ │                                             │ │           │
│ ──── │ │           PREVIEW / EDITOR AREA             │ │  Refs     │
│      │ │                                             │ │           │
│ ▸    │ │                                             │ │  Meta     │
│ ▸    │ │                                             │ │           │
│ ▾    │ │                                             │ │           │
│  ▸   │ │                                             │ │           │
│  ▾   │ │                                             │ │           │
│   •  │ │                                             │ │           │
│   •  │ │                                             │ │           │
│      │ └─────────────────────────────────────────────┘ │           │
├──────┴─────────────────────────────────────────────────┴───────────┤
│ STATUS  ● Mono • Unity 2022.3.18  •  4,217 assets  • ✓ ready       │ 24
└────────────────────────────────────────────────────────────────────┘
  240px               flex                                  320px (collapsible)
```

- Title bar: custom Tauri chrome
- Sidebar default 240px, resizable 200–400px
- Inspector default 320px, collapsible to 0
- Center area always flex-1

---

## 6. Components

### Buttons

- **Primary:** `bg-accent text-zinc-950 hover:bg-accent-strong active:translate-y-[1px]`
- **Secondary:** `bg-elevated text-primary border border-default hover:bg-overlay`
- **Ghost:** `text-secondary hover:text-primary hover:bg-overlay`
- **Danger:** `text-danger hover:bg-danger/10`
- Heights: `h-7` (sm), `h-8` (md), `h-9` (lg)
- Always `active:translate-y-[1px]` for tactile feedback

### Inputs

- 32px height
- `bg-base border border-default focus:border-accent focus:ring-2 focus:ring-accent-glow`
- Label above, helper/error below — per skill rules
- Search input: leading icon `MagnifyingGlass`, ⌘K kbd hint trailing

### Tree (asset tree)

- Row height 24px
- Indent: 16px per level
- Icon: 14px, file-type tinted
- Hover: `bg-overlay`, selected: `bg-accent-muted text-primary`
- Disclosure chevron rotates 90° on expand
- Drag handle on hover

### Tabs

- Underline indicator (2px) animated with `layoutId` (Framer)
- Tab text: monospace, 12/16, weight 500
- Modified dot: 6px circle in accent
- Close button on hover only

### Cards (use sparingly per anti-card-overuse rule)

Only used for:
- Recent project tiles on welcome screen
- Settings group cards
- Inspector property groups

Style: `bg-surface border border-subtle rounded-lg p-4` (NO drop shadow — borders only for depth).

### Status indicators

- 6px dot + label
- Mono backend: `bg-info`
- IL2CPP: `bg-warning`
- Loading: `bg-accent animate-pulse`

---

## 7. Iconography

- Library: **@phosphor-icons/react** (per skill rule)
- Stroke weight: **1.5** globally
- Sizes: 12 (status), 14 (tree/inline), 16 (toolbar/menu), 20 (action), 24 (empty state)
- Color: inherits `text-secondary` by default; `text-primary` on hover

**Banned:** Lucide user-icons as avatars, emoji anywhere.

### Asset-type icon mapping

| Kind | Phosphor icon |
|------|---------------|
| Folder | `Folder` (filled when open) |
| Texture | `Image` |
| Sprite | `Image` |
| Mesh | `Cube` |
| Audio | `Waveform` |
| Script | `FileCode` |
| TextAsset | `FileText` |
| Scene | `MapTrifold` |
| Material | `PaintBrush` |
| Shader | `Lightning` |
| Animation | `FilmStrip` |
| Binary | `File` |
| Unknown | `FileDashed` |

---

## 8. Motion (MOTION_INTENSITY = 6)

### Approved patterns

- Tab underline transition via Framer `layoutId` (200ms spring 100/20)
- Tree disclosure: 150ms ease-out
- Panel resize: live, no transition
- Modal: scale 0.96→1 + opacity 0→1, 180ms spring 200/25
- Tooltip: 80ms fade-in delay 200ms
- Drag-drop overlay: `border-accent-muted` pulsing border (CSS only, 1.5s)
- Loading: skeleton shimmer (no spinners except for atomic actions)
- Status dot pulse: 2s, accent only when actively decompiling
- Toolbar action `active:translate-y-[1px] scale-[0.98]` 80ms

### Banned

- Bouncy hero animations
- Background mesh gradients on app surfaces (allowed only on welcome hero)
- Custom mouse cursors
- Page transitions for in-app navigation

---

## 9. Welcome / Empty Screen

The welcome screen is the **only** "marketing-style" surface. Allowed:
- Subtle mesh gradient background (emerald @ 6% opacity) — top-right corner only
- Display-sized headline
- Two-column asymmetric (variance 8): left = headline + drop zone, right = recent projects grid 2×2

After project open, all UI is in dense pro-tool mode.

---

## 10. Accessibility

- Min contrast WCAG AA — verified for all text/bg combos above
- Focus ring: 2px accent-glow + 1px offset (visible on dark)
- Keyboard: every action reachable; ⌘K palette mirrors menu
- Reduced motion: respects `prefers-reduced-motion` — disables shimmer + pulse, keeps positional transitions

---

## 11. Tauri-specific

- Custom title bar with `data-tauri-drag-region` zones
- Window controls (min/max/close) Windows-native order on right
- WebView2 default; no transparency v1 (defer mica for v2)
- Cursor: native pointer; no custom

---

## 12. Don't List

- ❌ Inter, Poppins, any serif font
- ❌ Purple/blue gradients ("AI-purple" aesthetic)
- ❌ Pure black `#000`
- ❌ Mesh gradients on dashboard panels
- ❌ 3-equal-card feature rows
- ❌ Generic Lucide avatars
- ❌ Emoji (per skill anti-emoji rule)
- ❌ Inline modals — use pinned panels/popovers
- ❌ `h-screen` (mobile bug) — use `min-h-[100dvh]`
- ❌ Animated `width`/`height` — only `transform`/`opacity`
- ❌ More than 1 accent color (file-type tints don't count — those are functional)

---

## 13. Open Design Questions

- Final app name? Codename Unwrap — user picks: Unwrap / Strata / Decipher / Bedrock / other
- Logo direction: monogram, glyph (cube unfolding?), or wordmark — picks during logo gen
- Light theme: v2 priority?
- Mica/acrylic backgrounds: opt-in setting v2?
