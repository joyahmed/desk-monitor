# DESIGN.md — desk-monitor

Direction: B. The Lit Screen (chosen 2026-10; replaces A. Night Ops Map).
Story: a dark room at night, lit only by the screen your hand is on.

## Brief
- **Soul notes:** the light of screens at night; the Mac's breathing sleep LED; waking a sleeping machine; the server waiting behind its login screen.
- **Signature idea:** only ONE screen on the desk map is lit - a sky-to-blue glowing rectangle that casts a pool of light onto the desk edge below it; every other screen is dark. Switching moves the light (900ms crossfade + the pool slides). The asleep Mac shows only its tiny white breathing LED. The machine's name is a huge condensed gradient headline.
  - **The lit screen = the SELECTED screen.** The app has no API for where the cursor is, so the light follows the user's selection, not the mouse.
- **Ground:** bluish black `#00040F` (dark, default) / cool paper `#F3F7FD` (light). No cards: the room is one surface, split into bands by 1px rules.
- **Display font:** Big Shoulders Display 800/900 (condensed, uppercase machine names, hotkeys). Body: Onest 400/500/600. No mono: IPs and times are Onest with `tabular-nums`. Self-hosted in `static/fonts/` (`src/lib/fonts.css`), the app runs offline.
- **Image source:** light only - glow, bloom, the reflection pool. No photos, no illustrations. Logo `static/logo.svg` (B1: three screens, the middle one lit, light falling on the desk).
- **Motion presets:** `--motion-light` 900ms `cubic-bezier(.65,0,.35,1)` (the light moves) / `--motion-breathe` 4s ease-in-out (sleep LED) / `--motion-pulse` 2.5s ease-out (ring pulse) / `--motion-hover` 160ms ease / `--motion-enter` 240ms `cubic-bezier(.2,.8,.2,1)`.

## Avoid (do not drift back)
- The old electric `#2D68FA` as a button fill; filled buttons at all (except the ring).
- Cards, boxes, pills, drop shadows as structure. Radius on panels.
- IBM Plex (retired), any mono font, JetBrains Mono, Inter-only.
- Loops other than the sleep LED breathe and the ONE ring pulse.
- `--deep` or `--glow` or `--off` as small text. Hex colours outside `src/lib/tokens.css`.

## Colour (src/lib/tokens.css is the only file with colour literals)
| Token | Dark (default) | Light | Share | Use |
|---|---|---|---|---|
| `--ground` | #00040F | #F3F7FD | 75% | the room |
| `--surface` (= `--panel`) | #060B1C | #E6EEFA | 10% | a band that needs a floor (raised Block, rail) |
| `--raised` (= `--panel-2`) | #0B1430 | #FFFFFF | 3% | hover rows, raw buttons |
| `--ink` | #EAF2FF (18.2) | #0A1630 (16.7) | text | body text |
| `--ink-dim` | #8A99B8 (7.16) | #4D5B78 (6.34) | text | muted text, kickers, chips |
| `--placeholder` | #7F8DAD (6.17) | #5E6C8A (4.51) | text | input placeholder; never below 13px |
| `--line` | #141D36 | #D3DDEE | 1px | band rules |
| `--sky` (= `--accent`, `--focus`, `--glow`) | #7CD3FF (12.3) | #1F56D0 (5.92) | ~2% | ring, link hover, focus |
| `--sky-2` | #BFE9FF | #2A5BD7 | - | top of the light |
| `--deep` | #3A6FE0 (4.42) | #0B2E86 | - | bottom of the light; fills/large headline only |
| `--accent-ink` | #00040F | #FFFFFF | text | text on an accent fill |
| `--ok` | #62E3A6 (12.75) | #0B7A4B (5.01) | <1% | connected / online |
| `--warn` | #FFC266 (12.85) | #8F5300 (5.74) | <1% | at login screen / disconnected |
| `--bad` | #FF6B7F (7.48) | #C4213F (5.37) | <1% | hung / error / danger links |
| `--info` | #B9C2D9 (11.5) | #4A5675 (6.78) | <1% | asleep (white LED) |
| `--off` | #3A4562 | #A3AFC7 | dots | offline / unknown; dots and outlines only |
| `--grad-light` | #BFE9FF -> #7CD3FF -> #5088EC | #3A6FE0 -> #2A5BD7 -> #0B2E86 | ~5% | the lit screen |
| `--on-lit` | #00040F | #FFFFFF | text | text on `--grad-light` |
| `--grad-headline` | #BFE9FF -> #7CD3FF -> #3A6FE0 | #2A5BD7 -> #1F49B8 -> #0B2E86 | - | machine headline (`background-clip: text`) |
| `--pool` | sky radial, 26% -> 0 | blue radial, 18% -> 0 | - | light pool on the desk under the lit screen |
| `--screen-off` / `--screen-dim` | #050A19 / #0C1631 | #DFE7F4 / #CFDCF1 | - | dark screen face / connected-but-not-selected |
| `--bezel` | #1C2848 | #B4C2DC | 1px | screen bezel, input underline |
| `--link-line` | #2A3D6B | #A9BCE3 | 1px | resting underline under text links |
| `--lit-glow` | sky halo | blue halo | - | box-shadow round the lit screen |

Contrast ratios (in brackets) are on ground; all text pairs >= 4.5:1 on ground, surface and raised (checked with `scratchpad/contrast.cjs`). The dark lit gradient ends at #5088EC, not #3A6FE0, so `--on-lit` text passes on it.
State mapping (`src/lib/ui/state-colour.ts`): connected/online -> ok, disconnected -> warn, asleep -> info, hung/error -> bad, offline/unknown -> off.
Theme: dark is the default; the toggle sets `data-theme` on `<html>` and saves `desk-monitor-theme` in localStorage.

## Type (px)
| Role | Token | Size / weight / tracking |
|---|---|---|
| Machine headline | `--fs-headline` (`--fs-headline-sm` below 640) | 64 (48) / display 900 / line-height `--lh-headline` .85, uppercase, `--grad-headline` |
| Map screen label | `--fs-screen` | 20 |
| h1 | `--fs-h1` | 22 / display 800 / uppercase |
| Hotkey ("WIN 1") | `--fs-hotkey` | 15 / display 800 |
| Body | `--fs-body` | 15 / 400, line-height 1.5 |
| Small / meta | `--fs-small`, `--fs-mono` | 13 |
| Controls sm, meta | `--fs-xs` | 12 |
| Kicker (section label, h2) | `--fs-kicker` | 11 / 600 / caps / `--ls-kicker` .16em |
| Rail labels | `--fs-tiny` | 10 |
Casing: links and labels are Sentence case ("Edit keys", "Log in"). Uppercase only for the kicker and the display font (headline, hotkeys).

## Shape
- No cards. Bands separated by a 1px `--line` rule; a raised band gets the `--surface` floor, no border, no radius.
- Screens on the map: 1px `--bezel`, `--radius-screen` 2.
- Controls are lines: Input/Select are an underline, buttons are underlined text links. The only round thing is the ring (84px).
- Radius tokens (rarely used now): `--radius-sm` 2 / `--radius-md` 4 / `--radius-lg` 8 / `--radius-pill` 999.

## Layout
- Cinematic full-bleed: the "room" band on top (screens on the wall, desk edge with the light pool), the focused machine below as headline + one sentence + actions, hotkeys as a footer line. Medium-airy.
- Page gutter `--gutter`: 14 (base, 420), 22 (>=640), 28 (>=880). Bands bleed edge to edge and pad their content by `--gutter` (`<Block bleed>`).
- Breakpoints (literal in CSS): 420, 640 (`639.98` for max-width), 880. Below 640 the rail becomes a 56px bottom bar.
- Window 880x620, min 420x480. Check every screen at 420, 640, 880 (and 1400).
- Rail 64px: logo (`static/logo.svg`) on top, then tabs; controls at the bottom.

## Motion
| Preset | Value | Use |
|---|---|---|
| `--motion-light` | 900ms `--ease-light` cubic-bezier(.65,0,.35,1) | the light moves to another screen (crossfade); the pool slides |
| `--motion-breathe` | 4s ease-in-out, infinite | asleep Mac's white LED |
| `--motion-pulse` | 2.5s ease-out, infinite | ring pulse, only on the one action that needs you |
| `--motion-hover` | 160ms ease | underline colour, ring halo, input underline |
| `--motion-enter` | 240ms `--ease-lift` | a result line or panel appears |
Use the whole preset: `transition: opacity var(--motion-light)`, `animation: breathe var(--motion-breathe) infinite`. Old `--t-*` names still exist. Nothing else loops. No bounce, no lift. `prefers-reduced-motion`: the global rule in `app.css` removes all animation and transition (static light, no pulse, no breathe).

## Component rules
- **Ring = the ONE primary action per machine** (`<Button variant="ring">`): Log in (Linux login screen), Log in/Wake (asleep Mac). At most one ring per machine view. Lists (the RDP tab) get NO ring: Connect is a primary link, or every band would carry one. `pulse` only when the machine needs the user now (disconnected, asleep).
- **Everything else is a text link** (`variant="ghost"` default; `primary` = bolder with a sky underline for the main non-ring action of a band, e.g. Save, Add monitor).
- **Danger is a link in `--bad`** (`variant="danger"`), e.g. Restart Deskflow server, Delete, Log off. Never a ring, never filled.
- Status is a dot + a muted word (`Chip`), never a pill.
- Section headings are kickers (`SectionLabel`), not titles.
- Results (ok / error) are a line of text with a 2px (`--bar-w`) left bar in the state colour.

## Shared UI (src/lib/ui)
Layout: Block, Grid, Stack, Cluster (types in `layout.ts`). Controls: Button, Chip, StatusDot, Input, Select, Field, ConfirmInline, PasswordInline. Other: SectionLabel (kicker), OsGlyph, Icon, `state-colour.ts`, `select-options.ts`.

- `Block`: `{ title?; count?; variant?: 'panel'|'raised'|'plain' = 'panel'; span?: Span = 12; state?; label?; bleed?: boolean; header?: Snippet; actions?: Snippet; children: Snippet; footer?: Snippet }`. panel = a band: 1px rule on top, padding 24 top/bottom, none inline. raised = `--surface` floor, padding 24 (16/gutter at <=420). plain = nothing. `bleed` pads inline by `--gutter`. `state` colours the top rule.
- `Grid`: `{ gap?: SpaceStep = 3; stack?: 'sm'|'xs'|'never' = 'sm'; as?; label?; children }`. 12 columns.
- `Stack`: `{ direction?; gap?; align?; justify?; wrap?; as?; label?; children }`. `Cluster`: a wrapping row `{ gap?; justify?; align?; as?; label?; children }`.
- `Button`: `{ label; onclick; variant?: 'ghost'|'primary'|'danger'|'ring' = 'ghost'; size?: Size = 'md'; busy?; disabled?; title?; pulse?: boolean = false; type?: 'button'|'submit' = 'button' }`. Links: no padding, `size` sets hit height and font. Ring: 84px, 1.5px `--sky` ring + faint 6px halo, ignores `size`; `pulse` adds the 2.5s pulse ring (not while busy/disabled).
- `Chip`: `{ label; state?; title?; size?: Size = 'sm'; strong?: boolean = false }`: dot + muted word; `strong` uses `--ink`.
- `StatusDot`: `{ state; size?: Size = 'md' }` (6 / 8 / 10).
- `Input`: `{ value; oninput?; onchange?; size?; type?; placeholder?; readonly?; disabled?; label?; id?; mono?; autocomplete?; spellcheck?; focus? }`: underline (`--bezel`, `--sky` on focus).
- `Select`: `{ value; options; onchange; size?; disabled?; label?; id? }`: same underline.
- `Field`: `{ label; hint?; error?; children }`. `ConfirmInline`: `{ text; onyes; onno; busy?; size? }`.
- `PasswordInline`: `{ label; value; oninput; onyes; onno; placeholder?; action?: string = 'Log in'; pulse?: boolean = false; busy?: boolean = false }`: kicker + underline password field, ring button, Cancel link. Enter submits, the field takes focus, the row scrolls into view, the ring is disabled while the field is empty.
- `SectionLabel`: `{ text; count? }`: 11px caps kicker, count in `--ink`.

## Block system
Every spacing / size / radius literal lives in `src/lib/tokens.css`. `bun run check:blocks` (`scripts/check-blocks.mjs`, first step of `bun run gate`) fails a raw px/rem in padding*, margin*, gap, height, min-height, font-size, border-*radius anywhere else (0 and 1px allowed).

| Group | Tokens | Values (px) |
|---|---|---|
| Spacing | `--space-0` .. `--space-8`, `--space-hair` | 0 4 8 12 16 24 32 48 64, 2 |
| Gutter | `--gutter` | 14 / 22 (>=640) / 28 (>=880) |
| Control height | `--size-sm/md/lg` | 24 / 32 / 40 |
| Control font | `--size-sm-fs/md-fs/lg-fs` | 12 / 13 / 15 |
| Dot | `--dot-sm/md/lg` | 6 / 8 / 10 |
| Big targets | `--size-touch` / `--size-xl` | 44 / 48 |
| Ring | `--ring-size` / `--ring-w` / `--ring-halo` | 84 / 1.5 / 6 |
| Sleep LED | `--led-size` | 4 |
| Link underline gap | `--link-offset` | 6 |
| Result bar | `--bar-w` | 2 |
| Radius | `--radius-screen --radius-sm --radius-md --radius-lg --radius-pill` | 2 2 4 8 999 |
| Border | `--border-w`, `--border` | 1, `1px solid var(--line)` |
| z-index | `--z-base --z-raised --z-sticky --z-rail --z-overlay --z-toast` | 0 1 5 10 20 30 |
| Shell | `--rail-w`, `--rail-bar-h` | 64, 56 |
Old names still work: `--radius` = `--radius-md`, `--shadow-panel` = `--shadow-2`, `--panel` = `--surface`, `--panel-2` = `--raised`, `--font-mono` = Onest.

## References harvested
- A dark roofing-company landing page: #00040F ground, gradient headline, cyan ring CTA.
- apple.com/mac-studio: the product in the dark.
- lusion.co: light as the only image.

## Build with
Pick the direction first -> hooks hold reactivity, markup is mapped from data, check narrow widths first -> the light move and the ring -> review the rendered screen last.
