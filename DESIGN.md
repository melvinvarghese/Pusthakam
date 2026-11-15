---
name: Pusthakam
description: A Better Way to Manage Libraries. An offline library management desk tool in the "Spine Label" world.
colors:
  shelf-ink: "#16202e"
  shelf-ink-raised: "#222f42"
  shelf-ink-edge: "#2f3f56"
  rail-text: "#c9d2de"
  rail-mute: "#8e9cb1"
  rail-field-text: "#a4b1c4"
  rail-line-strong: "#44566f"
  work-white: "#ffffff"
  band-grey: "#f6f8fb"
  hairline: "#dfe4ea"
  hairline-strong: "#c3cbd6"
  ink-text: "#16202e"
  muted-text: "#5b6676"
  focus-blue: "#3a6fb5"
  overdue-red: "#c2171d"
  overdue-red-wash: "#fdf2f1"
  fine-amber: "#8a5a00"
  fine-amber-wash: "#fff6dc"
  label-stock: "#ffffff"
  label-ink: "#111111"
  label-edge: "#c5ccd6"
  night-frame: "#090f19"
  night-frame-raised: "#1d2a42"
  night-frame-edge: "#2c3d58"
  night-work: "#131d2e"
  night-surface: "#192539"
  night-band: "#152033"
  night-hairline: "#2a3b58"
  night-hairline-strong: "#3a5070"
  night-text: "#e6ebf2"
  night-muted: "#98a6ba"
  night-focus: "#7fb0ee"
  night-overdue: "#ff8a80"
  night-overdue-wash: "#3a1a1d"
  night-fine: "#f2c14e"
  night-fine-wash: "#3a2d0c"
  night-label-stock: "#f4f6f9"
  night-label-edge: "#9aa6b6"
  class-unclassed: "#8e9cb1"
  class-000-general: "#6b7a90"
  class-100-philosophy: "#c0507a"
  class-200-religion: "#8a6a3f"
  class-300-social: "#f0b400"
  class-400-language: "#8a9a2b"
  class-500-science: "#3a6fb5"
  class-600-technology: "#1f8f83"
  class-700-arts: "#7b5ea7"
  class-800-literature: "#d9482b"
  class-900-history: "#3f7d3a"
typography:
  ui-body:
    fontFamily: "Hanken Grotesk Variable, Segoe UI, system-ui, -apple-system, Nirmala UI, Noto Sans Malayalam, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: "20px"
  page-title:
    fontFamily: "Hanken Grotesk Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "20px"
    fontWeight: 700
    lineHeight: "28px"
    letterSpacing: "-0.025em"
  empty-title:
    fontFamily: "Hanken Grotesk Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "16px"
    fontWeight: 600
    lineHeight: "24px"
  panel-title:
    fontFamily: "Hanken Grotesk Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 600
    lineHeight: "20px"
  brand-wordmark:
    fontFamily: "Hanken Grotesk Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 700
    lineHeight: "18px"
  small:
    fontFamily: "Hanken Grotesk Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: "16px"
  call-label:
    fontFamily: "Courier Prime, Courier New, ui-monospace, monospace"
    fontSize: "10.5px"
    fontWeight: 700
    lineHeight: "12px"
    letterSpacing: "0.2px"
  call-label-lg:
    fontFamily: "Courier Prime, Courier New, ui-monospace, monospace"
    fontSize: "13px"
    fontWeight: 700
    lineHeight: "15px"
    letterSpacing: "0.2px"
rounded:
  label: "3px"
  sm: "4px"
  md: "6px"
  lg: "8px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "20px"
  xxl: "24px"
  rail-width: "232px"
  top-bar-height: "56px"
components:
  button-primary:
    backgroundColor: "{colors.shelf-ink}"
    textColor: "{colors.work-white}"
    typography: "{typography.ui-body}"
    rounded: "{rounded.md}"
    height: "32px"
    padding: "0 12px"
  button-secondary:
    backgroundColor: "{colors.work-white}"
    textColor: "{colors.ink-text}"
    rounded: "{rounded.md}"
    height: "32px"
    padding: "0 12px"
  button-secondary-hover:
    backgroundColor: "{colors.band-grey}"
  button-ghost:
    backgroundColor: "{colors.work-white}"
    textColor: "{colors.muted-text}"
    rounded: "{rounded.md}"
    height: "32px"
    padding: "0 12px"
  panel:
    backgroundColor: "{colors.work-white}"
    textColor: "{colors.ink-text}"
    rounded: "{rounded.lg}"
    padding: "12px 16px"
  chip-neutral:
    backgroundColor: "{colors.work-white}"
    textColor: "{colors.ink-text}"
    rounded: "{rounded.sm}"
    padding: "0 8px"
  chip-overdue:
    backgroundColor: "{colors.work-white}"
    textColor: "{colors.overdue-red}"
    rounded: "{rounded.sm}"
    padding: "0 8px"
  chip-fine:
    backgroundColor: "{colors.fine-amber-wash}"
    textColor: "{colors.fine-amber}"
    rounded: "{rounded.sm}"
    padding: "0 8px"
  nav-item:
    backgroundColor: "{colors.shelf-ink}"
    textColor: "{colors.rail-text}"
    rounded: "{rounded.md}"
    height: "32px"
    padding: "0 12px"
  nav-item-active:
    backgroundColor: "{colors.shelf-ink-raised}"
    textColor: "{colors.work-white}"
  search-trigger:
    backgroundColor: "{colors.shelf-ink-raised}"
    textColor: "{colors.rail-field-text}"
    rounded: "{rounded.md}"
    height: "36px"
    padding: "0 10px"
  call-number-label:
    backgroundColor: "{colors.label-stock}"
    textColor: "{colors.label-ink}"
    typography: "{typography.call-label}"
    rounded: "{rounded.label}"
    padding: "8px 6px 4px 5px"
---

# Design System: Pusthakam

## Overview

**Creative North Star: "The Spine Label"**

A library tool should look like the shelf it serves. Every book in a library is identified by a stacked call-number label that a librarian already reads at a glance, so that label is the signature of the whole system: a white sticker with a coloured Dewey class band across its top and typed lines beneath. Everything else is built to stay out of its way. A deep shelf-ink frame (rail and top bar) surrounds a calm work area, hairline cool-grey rules do the separating, and one grotesque face carries the interface while a typed monospace is kept for call numbers and accession codes.

The mood is quiet, exact and desk-ready: dense enough for eight hours of scanning, issuing and cataloguing, restrained enough that the only strong colour on screen belongs to the books. It refuses the pale admin-dashboard default (icon-and-card grid, sage-on-cream, one sans everywhere) and is explicitly not the first Pusthakam shell, which the owner rejected as generic.

The product is offline and honest, and the design says so plainly: an offline footer in the rail, unbuilt modules that state "not available yet", no invented numbers. Recognisable with all content removed: the dark ink frame, and a white label with a coloured top band.

**Key Characteristics:**
- Dark ink frame (rail 232px, top bar 56px) around a flat work area; both themes keep the frame darker than the work area.
- The stacked call-number label is the one signature component, wherever a book appears.
- Ten Dewey class colours exist only on label bands; red and amber exist only for overdue and fines.
- Flat by default: hairline rules and tonal fills, no card shadows.
- Text-only navigation; icons are used sparingly (search, theme, empty-state tile).
- Light and dark themes are both first-class; labels stay light stock in dark theme.

## Colors

A cool navy-grey system: one deep ink, a family of hairline greys, two reserved state colours, and a ten-colour class palette that never leaves the label band.

### Primary
- **Shelf Ink** (#16202e): The rail and top bar background, the primary button fill, and body text on the white work area. The one dark neutral of the light theme. In dark theme the frame deepens to **Night Frame** (#090f19).
- **Shelf Ink Raised** (#222f42) and **Shelf Ink Edge** (#2f3f56): Hover and active fill for rail items, the search trigger fill, and the rail's hairline dividers. Dark theme equivalents are #1d2a42 and #2c3d58.

### Secondary
- **Overdue Red** (#c2171d): Overdue chips and error text only. Wash #fdf2f1. In dark theme it lifts to #ff8a80 on #3a1a1d.
- **Fine Amber** (#8a5a00): Fine chips only, on wash #fff6dc. In dark theme #f2c14e on #3a2d0c.
- **Focus Blue** (#3a6fb5): Focus ring, text caret and selection tint. In dark theme #7fb0ee.

### Tertiary: Dewey class colours (label bands only)
- **000 General** #6b7a90, **100 Philosophy** #c0507a, **200 Religion** #8a6a3f, **300 Social sciences** #f0b400, **400 Language** #8a9a2b, **500 Science** #3a6fb5, **600 Technology** #1f8f83, **700 Arts** #7b5ea7, **800 Literature** #d9482b, **900 History** #3f7d3a. **Unclassed** #8e9cb1 is the fallback when a call number has no class. The mapping is provisional; it is intended to become configurable per library.

### Neutral
- **Work White** (#ffffff): The light-theme work area and panel surface. Dark theme: **Night Work** (#131d2e) for the work area, **Night Surface** (#192539) for panels.
- **Band Grey** (#f6f8fb): Hover fill, ghost-button hover, empty-state tile. Dark theme #152033.
- **Hairline** (#dfe4ea) and **Hairline Strong** (#c3cbd6): Panel borders, row dividers, secondary button borders. Dark theme #2a3b58 and #3a5070.
- **Muted Text** (#5b6676): Descriptions, labels, secondary copy. Dark theme #98a6ba. Body text is Shelf Ink on white and #e6ebf2 in dark.
- **Rail Text** (#c9d2de), **Rail Mute** (#8e9cb1), **Rail Field Text** (#a4b1c4), **Rail Line Strong** (#44566f): Text and borders on the ink frame.
- **Label Stock** (#ffffff, #f4f6f9 in dark), **Label Ink** (#111111), **Label Edge** (#c5ccd6, #9aa6b6 in dark): The sticker itself.

### Named Rules
**The Band-Only Rule.** The ten class colours appear only as the top band of a call-number label (and on the logo mark, which is a miniature label). They are never used for buttons, chips, charts, row accents or decoration.

**The Reserved Alarm Rule.** Red and amber mean overdue and fines, nothing else. If a screen has neither, it has no red or amber.

**The Stock Stays Light Rule.** A call-number label is light stock in both themes, like a real sticker. Never invert it for dark mode.

**The Frame Is Darker Rule.** In both themes the ink frame is darker than the work area (light: #16202e over white; dark: #090f19 over #131d2e).

## Typography

**UI Font:** Hanken Grotesk Variable (with Segoe UI, system-ui, Nirmala UI, Noto Sans Malayalam)
**Label/Mono Font:** Courier Prime (with Courier New, ui-monospace)

**Character:** A friendly but exact grotesque carries all interface text; a typed monospace appears only where the library itself types: call numbers and accession codes. Both are bundled from npm, with no remote fonts. Tabular numerals are on globally.

### Hierarchy
- **Page Title** (700, 20px, 28px line, -0.025em): One h1 per screen, top left of the work area.
- **Panel Title** (600, 14px, 20px): The h2 of a panel header row.
- **Empty Title** (600, 16px, 24px): Heading of an empty or not-available state.
- **Body** (400, 13px, 20px): All interface text; secondary copy in Muted Text. Button text is 13px (12px at size sm).
- **Small** (400, 12px, 16px): Chips, panel asides, rail footer, helper text.
- **Brand Wordmark** (700, 15px, 18px): "Pusthakam" in the rail header, white on ink.
- **Call Label** (Courier Prime 700, 10.5px, 12px, 0.2px; first line bold, following lines 400): The typed lines of a label. Large size is 13px / 15px.

### Named Rules
**The Typed-Only Rule.** Courier Prime is for call numbers and accession codes. Never set headings, buttons or running text in it.

**The One Grotesque Rule.** Hanken Grotesk is the only interface face; hierarchy comes from weight and size, not extra families.

## Layout

A fixed application frame: a two-column grid of a 232px ink rail and a flexible work area, with a 56px ink top bar spanning only the work-area column (the rail runs full height and holds the brand). The work area scrolls independently. Desktop only, minimum window 1024x640.

Inside the work area, content sits in a centred column up to 1100px with 24px padding and a 20px rhythm between panels (spacing steps 4, 8, 12, 16, 20, 24). Page headers align title left and actions right on the baseline. Dashboards use a main column plus a 320px side column at large widths. Row density is high: 32px nav rows and buttons, 36px search trigger, 62px record rows in the gallery to hold a label.

Navigation sits in four labelled groups (Main, Collections, Operations, Administration). The search-first trigger lives in the top bar and opens the quick-navigation palette with Ctrl K.

## Elevation & Depth

Flat by default. Depth is tonal and linear: the ink frame against the work area, panels separated by 1px hairlines, hover and selection shown by fill. The only shadows are a 1px soft drop under a call-number label (`box-shadow: 0 1px 1px rgb(22 32 46 / 0.12)`, so it reads as a sticker) and one overlay shadow on the command palette (`0 16px 40px -8px rgb(13 20 32 / 0.45)`) over a 60% ink scrim.

### Named Rules
**The Flat Panel Rule.** Panels, buttons, chips and rows carry no shadow. Only the label sticker and the floating palette may.

## Shapes

Small, tidy corners. Controls and nav rows use 6px, panels 8px, chips 4px, and the call-number label has square top corners (the band) and 3px bottom corners, like a cut sticker. The label has no top border so the band forms its top edge. Empty ("ghost") labels use a dashed hairline border and a hairline band to mean "nothing catalogued here yet". Borders are 1px; the overdue chip uses 1.5px.

## Components

### Buttons
- **Shape:** 6px radius, 1px border, 32px high at default (26px sm, 36px lg), 13px text.
- **Primary:** Shelf Ink fill with white text, semibold; hover 90% opacity, active 80%. In dark theme it inverts to #e6ebf2 fill with #101826 text.
- **Secondary (default):** Work-surface fill, Hairline Strong border, ink text; hover Band Grey fill.
- **Ghost:** Transparent, Muted Text; hover Band Grey fill and ink text.
- **Disabled:** 50% opacity, no pointer events. Focus: 2px Focus Blue outline, 2px offset.

### Chips
- **Neutral:** Hairline Strong border, ink text, 4px radius, 20px line.
- **Overdue:** 1.5px Overdue Red border, bold red text on the surface.
- **Fine:** Amber wash fill, 40%-amber border, semibold amber text.

### Panels
- **Corner Style:** 8px. **Background:** surface with 1px Hairline border. **Shadow:** none.
- **Header:** 14px semibold h2 left, 12px muted aside right, 1px bottom hairline, padding 12px 16px.

### Navigation
- **Rail:** Ink background; items are 32px tall, 6px radius, text-only, Rail Text. Unbuilt modules are dimmed to Rail Mute with a "Not available in this version yet" tooltip.
- **Hover:** Shelf Ink Raised fill, white text. **Active:** Shelf Ink Raised fill and bold white text, with no side marker or bar.
- **Group labels:** 11px semibold, Rail Mute. **Footer:** a white dot and "Offline · your data stays on this computer" over a hairline.
- **Top bar:** Search trigger (up to 540px, 36px high, raised-ink fill, Ctrl K keycap) and a three-way theme segmented control (light, system, dark) at the right.

### Command Palette
Centred modal, 32rem wide, 14vh from top, surface fill, Hairline Strong border, 8px radius, 140ms ease-out entrance (disabled by reduced motion). Rows are 6px radius with Band Grey on the active row; unbuilt modules are muted and show their phase.

### Empty State
Centred, max 28rem. An optional 40px icon tile on Band Grey with a hairline border, a 16px semibold title, muted description, and an optional action. Not-available pages instead use a ghost call-number label beside the message.

### Call-Number Label (signature)
A stacked white label: a 5px (6px large) full-width band in the Dewey class colour across the top, then typed Courier Prime lines (class number bold, author/cutter and following lines regular) on Label Stock, with a 1px Label Edge border and 3px bottom corners. Minimum 50px wide and 48px tall (64 and 60 large); it never shrinks inside flex rows and text never wraps. A label with no class uses the Unclassed colour. The ghost variant is transparent with a dashed border. It is announced as one image with the full call number as its name. A miniature of it (26x30px, "PUS / THK", literature band) is the logo mark.

## Do's and Don'ts

### Do:
- **Do** show every book with its stacked call-number label, and let the label be the loudest colour on the row.
- **Do** keep red (#c2171d) and amber (#8a5a00) for overdue and fines only.
- **Do** build with the CSS variable tokens in `src/styles/index.css`; give every colour a light and a dark value.
- **Do** keep the frame darker than the work area in both themes and keep labels light stock in dark theme.
- **Do** show unbuilt modules honestly with a "not available yet" page and a dimmed rail item.
- **Do** keep the offline reassurance visible in the rail footer.
- **Do** mark active navigation with fill and bold white text.
- **Do** set call numbers and accession codes in Courier Prime and nothing else.
- **Do** keep the UI fully offline: bundled fonts, no remote assets.

### Don't:
- **Don't** use Dewey class colours anywhere except a label band.
- **Don't** add coloured side-borders or left accent stripes to rows, cards or nav items.
- **Don't** put small uppercase eyebrow labels above headings.
- **Don't** add shadows to panels, buttons or chips.
- **Don't** show the Malayalam word in the interface (owner decision, 2026-11-15).
- **Don't** return to the rejected ivory, sage and Inter sidebar look, or to an icon-and-card dashboard grid.
- **Don't** display placeholder figures or fake records in production screens.
- **Don't** invert a call-number label for dark theme.
