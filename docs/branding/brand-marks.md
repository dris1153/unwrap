# Unwrap — Brand Marks

**Status:** v0.1 locked · pick made 2026-05-20

---

## The Mark — "Stack"

Three `{` curly brackets stacked with a `>` chevron echo on the right. Reads as **code layers being peeled / unwrapped** — the literal action of the product.

### Files

| File | Use |
|------|-----|
| `unwrap-mark.svg` | Icon-only, 256×256 viewBox. Title bars, dock, taskbar. |
| `unwrap-wordmark.svg` | Mark + "Unwrap" wordmark, horizontal. README, splash, marketing. |
| `favicon.svg` | Simplified single-bracket + chevron + rounded base. 16×16 → 64×64. |

### Colors

- **Mark fill/stroke:** `#10B981` (emerald-500). Three depth layers at opacity 1.0 / 0.5 / 0.22.
- **Background:** transparent (preferred) or `#09090B` (zinc-950, app base).
- **On light:** flip opacity reading — verify contrast separately at v0.2.

### Wordmark typography

- Font: **Satoshi 900** (fallback: system-ui sans-serif)
- Tracking: -1.4 (tight, marketing only — UI uses tighter tracking on smaller sizes)
- Color: `#F4F4F5` (zinc-100) on dark; revisit for light theme

### Spacing & clear-area

- Mark padding: at least 1/4 mark-width on every side
- Wordmark padding: mark-height on left + right
- Never crop, recolor, or rotate the mark

### Bad uses

- Don't use the mark on red, orange, or purple backgrounds (clash)
- Don't apply drop shadows or glows
- Don't add taglines below the wordmark in v0.1
- Don't combine with other dev-tool logos in close proximity

---

## Open items (v0.2+)

- Light-mode variant (currently dark-only)
- Animated intro mark (frames separating + chevron snap-in)
- Print-friendly mono version (single-tone)
- App-icon platform variants: Windows ICO, .icns macOS, Android adaptive
