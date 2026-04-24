# argumentation.dev website

Docusaurus 3 (TypeScript preset) for the learning site — concepts, examples, guides, academic references.

## Local development

```bash
npm install
npm run start
```

Opens `http://localhost:3000`.

## Build

```bash
npm run build
```

Emits static files to `build/`. The production deploy combines this with workspace rustdoc (`cargo doc --workspace --no-deps` → `target/doc/`) under `/api/`.

## Regenerate scene traces

The β slider on `/examples/thermostat` and `/concepts/weighted-and-beta` pulls pre-computed JSON traces from `static/traces/`. When the underlying library semantics change, regenerate:

```bash
cd ..
cargo run -q -p scene-tracer -- thermostat 0.0 website/static/traces/thermostat-b00.json
cargo run -q -p scene-tracer -- thermostat 0.4 website/static/traces/thermostat-b04.json
cargo run -q -p scene-tracer -- thermostat 0.5 website/static/traces/thermostat-b05.json
cargo run -q -p scene-tracer -- thermostat 1.0 website/static/traces/thermostat-b10.json
```

## Deployment

Deploys via `.github/workflows/deploy-site.yml` on push to `main`. Manual runs via workflow_dispatch.

**One-time repo setup:** Enable GitHub Pages in repo Settings → Pages, with Source = "GitHub Actions".

**baseUrl:** If deploying to `https://<user>.github.io/argumentation/` (project pages), edit `docusaurus.config.ts`:

```typescript
baseUrl: '/argumentation/',
```

If using a custom domain (e.g. `argumentation.dev`), leave `baseUrl: '/'` and add a `CNAME` file to `website/static/`.

## File layout

- `docs/` — all markdown/MDX content, split by Diataxis type:
  - `getting-started/` — tutorial
  - `concepts/` — explanation
  - `examples/` — walkthroughs (+ 1 demo)
  - `guides/` — how-to
  - `reference/` — curated API synthesis
  - `academic/` — bibliography + reading order + history
- `src/components/` — React components (`AttackGraph`, `SchemeCard`, `SceneTrace`, `BetaSlider`)
- `src/pages/index.tsx` — custom landing page
- `static/traces/` — pre-computed scene-trace JSON fixtures

## Contributing

Follow the Diataxis type of the directory you're editing. Use the doc-writer skill patterns:
- Active voice
- Present tense
- One job per page
- Code examples complete + runnable
- `Next steps` at the end

See the [plan doc](../docs/superpowers/plans/2026-04-23-learning-site-docusaurus.md) for the full design rationale.
