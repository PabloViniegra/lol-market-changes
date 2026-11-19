---
version: alpha
name: Transfer Board
description: A split-flap departures board for the League of Legends transfer market. Deep petrol surfaces, signal-yellow for what changed, condensed signage type for team codes.
colors:
  primary: "#F2C230"
  primary-hover: "#F7D35E"
  on-primary: "#0E2A2B"
  background: "#0E2A2B"
  surface: "#133536"
  surface-raised: "#1A4243"
  line: "#2A5556"
  text: "#EEF3EF"
  text-muted: "#A9BDB8"
  league-lck: "#7FB7E6"
  league-lpl: "#F07A63"
  league-lec: "#B7A3F0"
  league-lcs: "#6FCF97"
typography:
  board-code:
    fontFamily: Big Shoulders Display
    fontSize: 31px
    fontWeight: 800
    lineHeight: 1
    letterSpacing: 0.02em
  board-player:
    fontFamily: Big Shoulders Display
    fontSize: 25px
    fontWeight: 600
    lineHeight: 1.1
    letterSpacing: 0.01em
  heading-lg:
    fontFamily: Big Shoulders Display
    fontSize: 39px
    fontWeight: 700
    lineHeight: 1.05
    letterSpacing: 0em
  heading-md:
    fontFamily: Big Shoulders Display
    fontSize: 25px
    fontWeight: 700
    lineHeight: 1.15
    letterSpacing: 0.01em
  body-md:
    fontFamily: Atkinson Hyperlegible Next
    fontSize: 16px
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: 0em
  body-sm:
    fontFamily: Atkinson Hyperlegible Next
    fontSize: 13px
    fontWeight: 400
    lineHeight: 1.45
    letterSpacing: 0.01em
  label-md:
    fontFamily: Atkinson Hyperlegible Next
    fontSize: 13px
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: 0.01em
  data-tabular:
    fontFamily: Atkinson Hyperlegible Next
    fontSize: 16px
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: 0em
    fontFeature: tnum
rounded:
  flap: 3px
  control: 6px
  panel: 10px
  pill: 9999px
spacing:
  xs: 4px
  sm: 8px
  md: 12px
  lg: 16px
  xl: 24px
  2xl: 32px
  3xl: 48px
components:
  page:
    backgroundColor: "{colors.background}"
    textColor: "{colors.text}"
    typography: "{typography.body-md}"
  panel:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
    rounded: "{rounded.panel}"
    padding: "{spacing.xl}"
  board-row:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
    typography: "{typography.board-player}"
    height: 64px
    padding: "{spacing.md}"
  board-flap:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text}"
    typography: "{typography.board-code}"
    rounded: "{rounded.flap}"
    width: 72px
    height: 44px
  board-meta:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-muted}"
    typography: "{typography.body-sm}"
  divider:
    backgroundColor: "{colors.line}"
    height: 1px
  status-official:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
    typography: "{typography.label-md}"
    rounded: "{rounded.pill}"
    padding: 4px 10px
  status-confirmed:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.pill}"
    padding: 4px 10px
  status-rumor:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.primary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.pill}"
    padding: 4px 10px
  status-denied:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-muted}"
    typography: "{typography.label-md}"
    rounded: "{rounded.pill}"
    padding: 4px 10px
  status-faded:
    backgroundColor: "{colors.background}"
    textColor: "{colors.text-muted}"
    typography: "{typography.label-md}"
    rounded: "{rounded.pill}"
    padding: 4px 10px
  league-tag-lck:
    backgroundColor: "{colors.league-lck}"
    textColor: "{colors.on-primary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.flap}"
    padding: 2px 6px
  league-tag-lpl:
    backgroundColor: "{colors.league-lpl}"
    textColor: "{colors.on-primary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.flap}"
    padding: 2px 6px
  league-tag-lec:
    backgroundColor: "{colors.league-lec}"
    textColor: "{colors.on-primary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.flap}"
    padding: 2px 6px
  league-tag-lcs:
    backgroundColor: "{colors.league-lcs}"
    textColor: "{colors.on-primary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.flap}"
    padding: 2px 6px
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.control}"
    height: 40px
    padding: 0 16px
  button-primary-hover:
    backgroundColor: "{colors.primary-hover}"
    textColor: "{colors.on-primary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.control}"
    height: 40px
    padding: 0 16px
  button-secondary:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text}"
    typography: "{typography.label-md}"
    rounded: "{rounded.control}"
    height: 40px
    padding: 0 16px
  filter-chip:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-muted}"
    typography: "{typography.label-md}"
    rounded: "{rounded.control}"
    height: 32px
    padding: 0 12px
  filter-chip-active:
    backgroundColor: "{colors.text}"
    textColor: "{colors.background}"
    typography: "{typography.label-md}"
    rounded: "{rounded.control}"
    height: 32px
    padding: 0 12px
  input-field:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text}"
    typography: "{typography.body-md}"
    rounded: "{rounded.control}"
    height: 40px
    padding: 0 12px
  stat-figure:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
    typography: "{typography.data-tabular}"
---

## Overview

Transfer Board treats the League of Legends offseason like a terminal departures hall. Players leave one team and arrive at another, and the interface shows each move the way a split-flap board shows a flight: origin code, destination code, a name, a status.

That metaphor is the one bold idea, and it lives in a single place: **the board**, the feed of moves that opens the dashboard and the feed page. Team codes sit on raised "flaps" set in condensed signage type, with a hairline seam across the middle of each flap like a real Solari board. Everything around the board stays quiet: plain panels, small type, flat surfaces.

The palette comes from the same place. Departure boards are dark with bright yellow characters, so the base here is a deep petrol green (a real colour, not a tinted black) and the single signal colour is a sodium-lamp yellow. Yellow means **this changed**: a rumor that came true is filled with it, rumors are outlined in it, and plain official moves, the bulk of the board, stay neutral so the changes stand out. Uncertainty is shown by taking the fill away, never by adding a new colour.

The audience is one person who follows competitive LoL closely and checks in a few times a day during the transfer window. Density beats spectacle. Someone glancing at the board should be able to read "who, from where, to where, how sure" in under two seconds per row.

## Colors

The palette is dark-first. A light theme is out of scope for v1.

- **Primary — Signal yellow (#F2C230):** The only accent. Fills `Confirmed` status pills and primary buttons, outlines `Rumor` pills, and marks the focus ring. It never decorates and never marks the default state: `Official` is neutral because nearly every row is official. If yellow is on screen, it is telling the user something changed or is actionable.
- **Primary hover (#F7D35E):** Hover state for yellow buttons only.
- **On-primary — Petrol ink (#0E2A2B):** Text on yellow and on league tags. It is the background colour reused, which keeps the palette closed.
- **Background — Petrol (#0E2A2B):** The page. A blue-green deep enough to read as dark, coloured enough not to read as generic black.
- **Surface (#133536):** Panels and board rows. One step up from the page.
- **Surface raised (#1A4243):** Flaps, inputs and secondary buttons: things that sit on a panel.
- **Line (#2A5556):** Dividers between board rows and the seam across each flap. Used as a 1px fill, never as a decorative border around cards.
- **Text (#EEF3EF):** Primary text, slightly green-tinted white so it belongs to the petrol family.
- **Text muted (#A9BDB8):** Metadata, dates, sources, denied and faded rumors. Passes AA on every surface.

League colours are **categorical data colours**, used only to identify a league in tags, chart series and the flows diagram. They are never used for status, buttons or emphasis.

- **LCK (#7FB7E6):** sky blue.
- **LPL (#F07A63):** coral.
- **LEC (#B7A3F0):** lavender.
- **LCS (#6FCF97):** mint.

All four pass 4.5:1 against petrol ink as text-on-fill, and all four are distinct from signal yellow so a league tag is never mistaken for a status. In charts, pair colour with a direct text label: a league is never identified by colour alone.

## Typography

Two families with clearly different jobs.

- **Big Shoulders Display** (Patric King, derived from Chicago municipal signage) is the board's voice: team codes, player names, page headings. It is condensed, so a code like `FNC` or `HLE` reads large without taking width, which is exactly what a departures board needs. Use it only at 25px and above.
- **Atkinson Hyperlegible Next** carries everything else: body copy, metadata, controls, numbers. It was designed for legibility, with distinct `I/l/1` and `0/O` shapes. That matters when names like `Ill1ya` or `0din` are the content.

The scale is a 1.25 ratio from 16px: 13, 16, 20, 25, 31, 39. `board-code` (31px, weight 800) is the largest repeated element on the page, larger than section headings. That inversion is deliberate: on this product, the data is the headline.

Numbers in stats and tables use `data-tabular` with tabular figures (`tnum`) so columns of counts align.

Rules:

- Team codes appear in capitals because that is how the scene writes them (`T1`, `G2`, `BLG`). That is content, not styling. Labels, headings and buttons are sentence case.
- Body line length stays under 75 characters. Prose is rare here (rumor notes, empty states), and when it appears it sits in a narrow column.
- No monospace anywhere. Tabular figures do the alignment work.

## Layout

A 12-column grid, 1280px max content width, 24px gutters, left-aligned throughout. The board is always full content width. Secondary information sits in a right-hand column on wide screens and drops below on narrow ones.

Dashboard (`/`), desktop:

```
┌──────────────────────────────────────────────────────────────────────┐
│ Transfer Board                       LCK LPL LEC LCS  · updated 4m   │
├──────────────────────────────────────────────────────┬───────────────┤
│ Latest moves                                         │ This window   │
│ ┌────┐   ┌────┐                                      │  LCK  ████ 23 │
│ │ GEN│ ▸ │ T1 │  Chovy        Mid     Official  2d   │  LPL  ██████31│
│ └────┘   └────┘                                      │  LEC  ███  17 │
│ ┌────┐   ┌────┐                                      │  LCS  ██   11 │
│ │ G2 │ ▸ │ FNC│  Caps         Mid     Rumor     5h   │               │
│ └────┘   └────┘                                      │ Free agents   │
│  ...                     (12 rows, then "See all")   │  38 players   │
├──────────────────────────────────────────────────────┴───────────────┤
│ Where players went                     [ by league | by team ]       │
│   LCK ═══════╗                                                       │
│   LPL ═══════╬═══ flows diagram (Sankey), labelled ends              │
│   LEC ═══════╝                                                       │
├──────────────────────────────────────────────────────────────────────┤
│ Rumors to watch            (compact board rows, rumor status only)   │
└──────────────────────────────────────────────────────────────────────┘
```

Board row anatomy, left to right: origin flap, a direction glyph, destination flap, player portrait (48px, hidden below 768px), player name, role, league tag(s), status pill, relative date, source icon (an external-link outline icon whose accessible name and tooltip give the outlet). A missing origin shows an empty flap with `FA` (free agent). A missing destination shows `OUT`.

Spacing comes from a 4px base (`xs` 4 → `3xl` 48). Board rows are 64px tall with `md` (12px) internal padding. Panels use `xl` (24px). Sections are separated by `3xl` (48px) and a heading, never by stacking cards.

Below 768px the board row reflows to two lines: flaps and name on the first, role, tag, status and date on the second. Flaps shrink to at least 80×36 and `board-code` drops to 25px. The right column moves under the board.

## Elevation & Depth

Flat. There are no drop shadows anywhere in the system.

Depth is expressed with three surface tones: background → surface → surface-raised. A flap is "raised" because it is one tone lighter than its row, the way a physical flap catches light. The only line-based depth cue is the 1px `line` seam across the horizontal middle of each flap.

Overlays (dialogs, the filter sheet on mobile) sit on `surface-raised` over a 60% petrol scrim. Popovers and tooltips use `surface-raised` with a 1px `line` outline.

## Shapes

Radius signals what kind of object something is, so it changes with hierarchy rather than being one value everywhere.

- **flap (3px):** flaps and league tags. Nearly square, like the physical tiles.
- **control (6px):** buttons, inputs, filter chips.
- **panel (10px):** panels and dialogs.
- **pill (9999px):** status pills only. The round shape is reserved for "how sure is this", so status is recognisable by shape before it is read.

The direction glyph between flaps is a solid right-pointing triangle (`▸`), not an arrow, matching departure-board iconography. Icons elsewhere are 1.5px-stroke outline icons at 20px. Role icons are the exception: filled 20px glyphs in the minimap language players already read (a lane corner for Top and Bot, a diagonal for Mid, a fang for Jungle, a shield for Support).

## Components

**Board row.** The core component. A `surface` row 64px tall, separated from its neighbours by a 1px `divider`, not by gaps or cards. Rows are clickable: the player name is a stretched link, so the whole row opens the player page while team codes, the source icon and other links inside stay separately clickable. Hover lifts the row to `surface-raised`. Keyboard focus draws a 2px signal-yellow outline inset by 2px.

**Flap.** `surface-raised`, at least 104×44, `board-code` type, centred text, with a 1px `line` seam across the middle. Team codes are at most 4 characters. When the team has a Leaguepedia square logo, a 24px logo (20px below 768px) sits before the code; the flap grows with its content. Logos drawn mostly in near-black, which vanish on petrol, sit on a `text` plate with `flap` radius and 2px padding; the tone is measured once per logo in the browser. Empty states: `FA` for free agent origin, `OUT` for release or retirement, both in `text-muted`.

**Team crest.** On the team page, the logo sits in an 80px tile (96px from 768px) with `flap` radius and the flap seam, on `surface-raised`, or on `text` for near-black logos. It is omitted when the logo fails to load.

**Player portrait.** The Leaguepedia tournament photo, cropped to the top of a square tile with `flap` radius on `surface-raised`: 48px in board rows, 32px in rosters (hidden below 768px), 112px in the player header. Departed and muted players render in greyscale. Without a photo, rows and rosters show the role icon in `line` so columns stay aligned; the player header omits the tile.

**Role icon.** Shown before the role in board rows, roster role cells and the player header, in the colour of its text.

**Status pills.** Five states, distinguished by fill, outline and decoration, not only by colour:

| State     | Label       | Treatment                                                                                                      |
| --------- | ----------- | -------------------------------------------------------------------------------------------------------------- |
| Official  | `Official`  | Surface fill, text colour, 1px solid `text-muted` outline. The quiet default: most rows are official.          |
| Confirmed | `Confirmed` | Solid yellow fill, petrol text, plus a small dashed-outline dot before the label meaning "started as a rumor". |
| Rumor     | `Rumor`     | Surface fill, yellow text, 1.5px dashed yellow outline. Followed by 1–5 small filled dots showing reliability. |
| Denied    | `Denied`    | Surface fill, muted text, label struck through.                                                                |
| Faded     | `Faded`     | No outline, muted text, on background. Used when a rumor went 60 days without confirmation.                    |

Rows with `Denied` or `Faded` status render the whole row's text in `text-muted`, and both are hidden from the feed by default (a toggle shows them).

**League tag.** League colour fill, petrol text, `flap` radius, `label-md`. Shows the league short name (`LCK`). A cross-league transfer shows two tags in origin → destination order.

**Buttons.** Primary is yellow and is used at most once per view (for example "See all moves"). Secondary is `surface-raised`. Button text says what happens: "See all moves", "Show faded rumors", "Clear filters".

**Filter chips.** Leagues, roles, move types and statuses. Inactive chips are surface with muted text. Active chips invert to light fill with petrol text, so the active state never competes with signal yellow. Chips are 44px tall below 768px and 32px above, so every chip is a comfortable tap target. Each group (League, Role, Move, Status) has a visible label. Every filter is mirrored in the URL.

**Dropdowns and comboboxes.** No native selects or datalists, because their option lists can't carry the system. A dropdown trigger is styled like an input field, with a 20px outline chevron. Its list is a popover (`surface-raised`, 1px `line` outline, `control` radius) of 40px rows: the highlighted row is `line` with `text`, the others are `text-muted`, and the chosen option ends with a yellow check. Team comboboxes list each match as code, full name and league tag, and show the matched code and tag inside the field. Reliability is picked on a row of five 40px `surface-raised` tiles, each holding a dot that fills yellow up to the chosen level, next to its label ("3 · Plausible").

**Board legend.** A collapsed "How to read the board" disclosure next to every board heading. Opened, it shows a `surface` panel explaining origin ▸ destination, `FA` and `OUT`, and each status pill with one line of meaning. It never opens on its own.

**Freshness indicator.** In the header: "Updated 4 min ago" in `body-sm`, muted. When the last sync is older than 1 hour, the header text is hidden and a full-width `surface-raised` bar appears under the header with a signal-yellow warning icon: "**Data hasn't refreshed for 2 hours.** Moves from Leaguepedia may be missing."

**Charts (flows, per-league bars).** Series use league colours only. Every series and every Sankey node carries a direct text label in `label-md` ("LCK 3 out", "LEC 2 in"); no legends that require matching colours. The Sankey's height follows its node count and a caption states how many of the window's moves it shows. Gridlines use `line`. Tooltips follow popover styling.

**Empty and error states.** Plain sentences that say what to do. Empty filtered feed: "No moves match these filters. Try removing a role or widening the dates." API error: "Couldn't load moves. Retrying in 30 seconds." with a "Retry now" secondary button.

**Motion.** When a refetch brings in a new move, each flap flips once around its seam. The upper half folds in for 140ms and the lower half unfolds for 140ms using `--ease-out` (`cubic-bezier(0.23, 1, 0.32, 1)`), with a 30ms delay on the destination flap to follow the direction of the move. The mobile filters sheet slides from and back to the bottom edge over 240ms using `--ease-drawer` (`cubic-bezier(0.32, 0.72, 0, 1)`). Dropdown and combobox lists open over 140ms (fade plus a 4px drop) using `--ease-out` and close with a 90ms fade. Hover, focus and inline disclosures update instantly. With `prefers-reduced-motion`, the flip is replaced by a 1-second yellow left edge, the sheet uses a 240ms opacity transition and lists fade without moving.

## Do's and Don'ts

**Do**

- Keep yellow for changes (confirmed rumors, open rumors) and the single primary action per view. If you need emphasis anywhere else, use size or weight.
- Show uncertainty by removing fill (dashed outline, muted text), never by introducing new colours.
- Give the board the full content width and the largest type on the page.
- Label every chart series and node directly; pair league colours with text.
- Write status and UI copy in sentence case with plain verbs: "Show faded rumors", not "Toggle stale".
- Credit Leaguepedia in the footer: "Official moves from Leaguepedia, licensed CC BY-SA."
- Keep the Riot disclaimer under the Leaguepedia credit: the site is not endorsed by Riot Games, and League of Legends is a Riot Games trademark.
- Hit AA contrast on every text pair, visible focus on every interactive element, and full keyboard reachability of rows, chips and charts.

**Don't**

- Don't wrap content in identical rounded cards with shadows. Panels are flat, rows are separated by 1px dividers.
- Don't add all-caps eyebrow labels above headings or tracked-out micro labels. Team codes are the only capitals.
- Don't use monospace for data, gradients for decoration, or glows of any kind.
- Don't use league colours for status, buttons or highlights.
- Don't use team logos or player photos as decoration. They identify a team or player next to its code or name, never alone, and never in backgrounds, banners or patterns. The only Riot mark is the League of Legends wordmark at the right end of the header, 24px tall in `text-muted`, so visitors place the site in competitive LoL. League marks are not used.
- Don't animate section entrances, card hovers or chart loading.
- Don't use numbered markers or step indicators: nothing on this site is a sequence except time, and time is shown as dates.
- Don't append arrows to button or link text. The only directional glyph is the `▸` between flaps.
