# Themes

Ten standalone aesthetic directions for the argumentation learning site.
Each file is self-contained — it overrides Infima CSS custom properties,
loads its own Google Fonts, and ships all decorative CSS it needs.

## Switching themes

Edit `../custom.css`. Exactly one `@import` should be uncommented:

```css
@import './themes/01-scholastica.css';
/* @import './themes/02-brutalist.css'; */
/* ... */
```

Restart the Docusaurus dev server (`npm run start`) to pick up new
font imports. Plain CSS edits hot-reload without a restart.

All themes support both light and dark mode via Docusaurus' built-in
toggle (`data-theme='dark'` on `<html>`).

## The ten directions

| # | Theme | Palette | Display font | Mood |
|---|-------|---------|--------------|------|
| 01 | **Scholastica** | Burgundy, gold, parchment | Cormorant Garamond | Illuminated manuscript. Reading as ritual. |
| 02 | **Brutalist** | Paper, ink, oxide orange | IBM Plex Mono | Exposed concrete. No radii. |
| 03 | **Editorial** | Ivory, charcoal, saffron | Playfair Display | Longform magazine. Hairlines and pull quotes. |
| 04 | **Phosphor** | CRT green / olive on paper | VT323, JetBrains Mono | Terminal. Scanlines and cursor blink. |
| 05 | **Atelier Deco** | Navy, gold, ivory | Cinzel | Art deco scholarly journal. |
| 06 | **Bauhaus** | Red / blue / yellow / cream | Archivo Black | Primary colors and geometric blocks. |
| 07 | **Helvetica Suisse** | Black, white, hot red | Archivo | Swiss grid. Numbered sections. |
| 08 | **Herbarium** | Sage, terracotta, cream | Libre Caslon Display italic | Naturalist's field notebook. |
| 09 | **Nocturne** | Magenta, cyan, deep purple | Orbitron, Chivo Mono | Cyberpunk neon. Grid background, glow. |
| 10 | **Washi** | Sumi ink, vermilion, indigo | Shippori Mincho | Japanese paper. Asymmetry and breathing room. |

## Anatomy of a theme

Every theme file provides:

1. **Google Fonts import** — display, body, and often monospace.
2. **`:root` block** — Infima primary (7 shades), background, surface,
   font families, heading/content/link colors, radii, borders.
3. **`[data-theme='dark']` block** — same variables, recomputed for dark.
4. **Decorative CSS** — treatments unique to the theme (drop caps,
   scanlines, gradient text, geometric rules, etc.).

The variables that matter most for visual identity:

- `--ifm-color-primary-*` (7 shades — primary through lightest)
- `--ifm-background-color`, `--ifm-background-surface-color`
- `--ifm-font-family-base`, `--ifm-heading-font-family`, `--ifm-font-family-monospace`
- `--ifm-heading-color`, `--ifm-color-content`
- `--ifm-link-color`, `--ifm-link-hover-color`
- `--ifm-global-radius` (0 for brutalist/bauhaus/phosphor/nocturne)
- `--ifm-hr-border-color`, `--ifm-hr-border-width`
- `--docusaurus-highlighted-code-line-bg`

## Adding a new theme

1. Create `themes/NN-my-theme.css` — start by copying an existing theme
   whose structure is closest to what you want.
2. Import your Google Fonts at the top.
3. Fill in `:root` (light) and `[data-theme='dark']` (dark) variable blocks.
4. Add decorative CSS — keep it scoped to the theme so it can't leak.
5. Add a commented `@import` line to `../custom.css`.
6. Document it in this README's table.

## Notes

- Font imports use `display=swap` so text renders while fonts load.
- Themes use `!important` on `.hero--primary` and button styles because
  Docusaurus' `.hero--primary` is assertive. Use it sparingly elsewhere.
- Some themes add body-level decoration (parchment gradients, CRT
  scanlines, neon grid). These are theme-specific and disappear cleanly
  when you swap themes.
- Light/dark mode is always supported. If you're designing a dark-first
  theme (Nocturne, Phosphor), the light mode is a deliberate paper-ish
  inversion — not an afterthought.
