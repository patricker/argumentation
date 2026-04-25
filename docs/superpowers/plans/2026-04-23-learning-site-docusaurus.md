# Argumentation.dev — Learning Site Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a Docusaurus-powered learning site — THE canonical resource for people who want to understand formal argumentation *and* our scene-AI bridge. Static content first; interactive WASM demos deferred to a follow-up plan.

**Architecture:** Docusaurus 3 TypeScript preset at `/home/peter/code/argumentation/website/`. Six Diataxis-aligned content tracks: **getting-started** (tutorial), **concepts** (explanation), **examples** (narrative walkthroughs + one marquee demo), **guides** (how-to), **reference** (overview + rustdoc), **academic** (bibliography + explanation). Interactive components render from pre-computed scene traces (JSON fixtures) via `react-flow` and small React widgets; no runtime Rust execution in the browser in this phase. Rustdoc output is built separately and linked from the navbar. GitHub Pages deploy via Actions.

**Tech Stack:** Docusaurus 3.x (TypeScript preset), Node 20+, React 18, MDX, `react-flow` for graph visualization, Prism for Rust syntax highlighting.

---

## Pre-flight checklist for the executor

1. Work from `/home/peter/code/argumentation/`. Create everything under `website/` — a new top-level directory alongside `crates/`, `docs/`, `src/`.
2. Node 20+ required. If `node --version` reports < 20, stop and escalate.
3. Site dev server runs on `localhost:3000` by default. Keep it running in a background shell while iterating.
4. Every task ends with `cd website && npm run build` succeeding (catches broken links, missing images, MDX syntax errors). Treat any build warning as a failure.
5. Content style: prose should be precise, not salesy. Assume the reader is technically literate but new to argumentation. Avoid jargon without defining it on first use; link to the concepts page when a term is introduced elsewhere.
6. All academic references link to open-access copies when available (arXiv, university mirrors, author homepages). Pay-walled DOIs only as fallback.

## Design decisions locked in (pre-plan)

- **D1 — Docusaurus over VitePress / Nextra.** Docusaurus has better versioning (we'll want this as the library evolves), mature MDX support, and a plugin ecosystem. Default TypeScript preset.
- **D2 — `website/` not `docs-site/` or `docs/site/`.** Matches Docusaurus convention. Our existing `docs/` stays for engineering docs (plans, decisions).
- **D3 — Pre-computed scene traces (JSON) feed interactive components.** No WASM in this phase. A scene trace is a list of beats with embedded argument-framework state per beat. Generated via a small Rust binary (`scene-tracer`) that writes JSON to `website/static/traces/`.
- **D4 — Academic references as a first-class section, not a footnote.** Bibliography is browsable, with reading-order recommendations and a "papers → concepts" cross-reference. This is the "appeal to authority" backbone.
- **D5 — The marquee demo is the thermostat scene.** Everyday-domain multi-scheme unfolding is the research-gap slot the canon has never filled (per the literature review). Canonical examples (Hal & Carla, Nixon diamond, Tweety) also ship as reference demos.
- **D6 — Landing page is a custom React page, not a markdown doc.** Needs a hero, feature grid, and a small inline β-slider teaser. Sub-pages are MDX.
- **D7 — Versioned docs disabled initially.** We'll add versioning when we cut the first post-0.3 release with a breaking change. Single-version setup reduces maintenance until then.
- **D8 — No analytics, no cookies, no Google Fonts.** Self-host any fonts. Privacy-respecting by default.
- **D9 — Diataxis-aligned IA.** Every page declares its Diataxis type (tutorial / how-to / reference / explanation) and, for tutorials and how-tos, a behavioral learning objective. This prevents the #1 doc antipattern: mixing types on one page. The track-to-type mapping is:

  | Track | Diataxis type | Reader's question |
  |---|---|---|
  | `getting-started/` | **TUTORIAL** | "I want to learn by doing" |
  | `guides/` | **HOW-TO** | "I have a specific problem to solve" |
  | `concepts/` | **EXPLANATION** | "I want to understand why/how this works" |
  | `examples/` | **EXPLANATION** (narrative) + 1 **DEMO** (thermostat) | "Show me what this looks like in action" |
  | `reference/` + rustdoc | **REFERENCE** | "I need to look something up" |
  | `academic/bibliography.md` | **REFERENCE** | "I need to find the source paper" |
  | `academic/history.md`, `academic/reading-order.md` | **EXPLANATION** | "How did this field evolve?" |

  Tasks in Phases C–F cite their Diataxis type and template (from `~/.claude/skills/doc-writer/references/templates.md`) explicitly.

## File structure

### New directory layout

```
argumentation/
├── website/                              (new)
│   ├── docusaurus.config.ts
│   ├── sidebars.ts
│   ├── package.json
│   ├── tsconfig.json
│   ├── docs/
│   │   ├── intro.md
│   │   ├── getting-started/                  # TUTORIAL
│   │   │   ├── _category_.json
│   │   │   └── first-scene.md
│   │   ├── concepts/                         # EXPLANATION
│   │   │   ├── _category_.json
│   │   │   ├── what-is-argumentation.md
│   │   │   ├── walton-schemes.md
│   │   │   ├── attacks-and-supports.md
│   │   │   ├── semantics.md
│   │   │   ├── weighted-and-beta.md
│   │   │   ├── aspic-plus.md
│   │   │   └── encounter-integration.md
│   │   ├── examples/                         # EXPLANATION (narrative) + 1 DEMO
│   │   │   ├── _category_.json
│   │   │   ├── hal-and-carla.mdx
│   │   │   ├── nixon-diamond.mdx
│   │   │   ├── tweety-penguin.mdx
│   │   │   ├── thermostat.mdx                # ← DEMO
│   │   │   └── courtroom.mdx
│   │   ├── guides/                           # HOW-TO
│   │   │   ├── _category_.json
│   │   │   ├── installation.md
│   │   │   ├── catalog-authoring.md
│   │   │   ├── implementing-action-scorer.md
│   │   │   ├── implementing-acceptance-eval.md
│   │   │   └── tuning-beta.md
│   │   ├── reference/                        # REFERENCE (curated synthesis; full API is rustdoc)
│   │   │   ├── _category_.json
│   │   │   └── overview.md
│   │   └── academic/
│   │       ├── _category_.json
│   │       ├── bibliography.md               # REFERENCE
│   │       ├── reading-order.md              # EXPLANATION
│   │       └── history.md                    # EXPLANATION
│   ├── src/
│   │   ├── pages/
│   │   │   └── index.tsx                 (custom landing)
│   │   ├── components/
│   │   │   ├── SchemeCard/
│   │   │   │   ├── index.tsx
│   │   │   │   └── styles.module.css
│   │   │   ├── AttackGraph/
│   │   │   │   ├── index.tsx
│   │   │   │   └── styles.module.css
│   │   │   ├── SceneTrace/
│   │   │   │   ├── index.tsx
│   │   │   │   └── styles.module.css
│   │   │   ├── BetaSlider/
│   │   │   │   ├── index.tsx
│   │   │   │   └── styles.module.css
│   │   │   └── HomepageFeatures/
│   │   │       ├── index.tsx
│   │   │       └── styles.module.css
│   │   └── css/
│   │       └── custom.css
│   ├── static/
│   │   ├── img/                           (diagrams, logos)
│   │   └── traces/                        (scene-trace JSON fixtures)
│   │       ├── thermostat.json
│   │       ├── hal-and-carla.json
│   │       └── nixon-diamond.json
│   └── README.md
├── tools/
│   └── scene-tracer/                      (new Rust binary)
│       ├── Cargo.toml
│       └── src/main.rs
└── .github/
    └── workflows/
        └── deploy-site.yml                (new)
```

### File responsibilities

| File / directory | Responsibility |
|---|---|
| `website/docusaurus.config.ts` | Site metadata, theme config, navbar, footer, plugin setup |
| `website/sidebars.ts` | Sidebar structure for the four content tracks |
| `website/src/pages/index.tsx` | Custom landing (hero + feature grid + teaser) |
| `website/src/components/SchemeCard` | Pretty presentation of a Walton scheme (premises, conclusion, CQs) |
| `website/src/components/AttackGraph` | `react-flow`-based interactive argument graph (read-only, no runtime eval) |
| `website/src/components/SceneTrace` | Beat-by-beat playback of a pre-computed scene |
| `website/src/components/BetaSlider` | Discrete β selector that swaps between pre-computed trace variants |
| `website/docs/getting-started/*.md` | TUTORIAL — one guided first-scene build |
| `website/docs/concepts/*.md` | EXPLANATION — concept deep-dives |
| `website/docs/examples/*.mdx` | Narrative walkthroughs (mostly EXPLANATION) + thermostat DEMO |
| `website/docs/guides/*.md` | HOW-TO — specific developer tasks |
| `website/docs/reference/*.md` | REFERENCE — curated type/method synthesis; full API at `/api/` rustdoc |
| `website/docs/academic/bibliography.md` | REFERENCE — citation list |
| `website/docs/academic/{history,reading-order}.md` | EXPLANATION — historical context + curriculum |
| `website/static/traces/*.json` | Pre-computed scene traces |
| `tools/scene-tracer/` | Rust binary that runs a scene and emits a JSON trace fixture |
| `.github/workflows/deploy-site.yml` | Build + deploy to GitHub Pages on push to main |

---

## Phase A — Docusaurus setup

### Task 1: Initialize Docusaurus site

**Files:**
- Create: `website/` (entire scaffold via Docusaurus init)

- [ ] **Step 1: Run the init command**

```bash
cd /home/peter/code/argumentation
npx --yes create-docusaurus@latest website classic --typescript
```

This scaffolds `website/` with TypeScript preset. It pulls deps via npm; let it finish.

- [ ] **Step 2: Verify the dev server runs**

```bash
cd /home/peter/code/argumentation/website
npm run start -- --port 3000 --no-open
```

Expect: "Docusaurus website is running at: http://localhost:3000/". Kill with Ctrl-C.

- [ ] **Step 3: Verify production build succeeds**

```bash
cd /home/peter/code/argumentation/website
npm run build
```

Expect: "Generated static files in "build"."

- [ ] **Step 4: Add `.gitignore` entries for the site's build artifacts**

Append to `/home/peter/code/argumentation/.gitignore` (create if absent):

```
website/node_modules/
website/.docusaurus/
website/build/
```

- [ ] **Step 5: Commit**

```bash
cd /home/peter/code/argumentation
git add website/ .gitignore
git commit -m "feat(website): initialize Docusaurus 3 site"
```

---

### Task 2: Configure site metadata and theme

**Files:**
- Modify: `website/docusaurus.config.ts`
- Modify: `website/src/css/custom.css`

- [ ] **Step 1: Rewrite `docusaurus.config.ts`**

Replace the template with:

```typescript
import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'argumentation.dev',
  tagline: 'Formal argumentation for scene AI — auditable reasoning, interpretable beats',
  favicon: 'img/favicon.svg',
  url: 'https://argumentation.dev',
  baseUrl: '/',
  organizationName: 'patricker',
  projectName: 'argumentation',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'throw',
  i18n: {defaultLocale: 'en', locales: ['en']},
  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/patricker/argumentation/tree/main/website/',
          routeBasePath: '/',
        },
        blog: false,
        theme: {customCss: './src/css/custom.css'},
      } satisfies Preset.Options,
    ],
  ],
  themeConfig: {
    image: 'img/social-card.png',
    colorMode: {defaultMode: 'dark', respectPrefersColorScheme: true},
    navbar: {
      title: 'argumentation.dev',
      logo: {alt: 'argumentation.dev logo', src: 'img/logo.svg'},
      items: [
        {type: 'docSidebar', sidebarId: 'gettingStartedSidebar', label: 'Getting Started', position: 'left'},
        {type: 'docSidebar', sidebarId: 'conceptsSidebar', label: 'Concepts', position: 'left'},
        {type: 'docSidebar', sidebarId: 'examplesSidebar', label: 'Examples', position: 'left'},
        {type: 'docSidebar', sidebarId: 'guidesSidebar', label: 'Guides', position: 'left'},
        {type: 'docSidebar', sidebarId: 'referenceSidebar', label: 'Reference', position: 'left'},
        {type: 'docSidebar', sidebarId: 'academicSidebar', label: 'Academic', position: 'left'},
        {href: '/api/', label: 'Rustdoc', position: 'right'},
        {href: 'https://github.com/patricker/argumentation', label: 'GitHub', position: 'right'},
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {label: 'Getting Started', to: '/getting-started/first-scene'},
            {label: 'Concepts', to: '/concepts/what-is-argumentation'},
            {label: 'Examples', to: '/examples/thermostat'},
            {label: 'Guides', to: '/guides/installation'},
            {label: 'Reference', to: '/reference/overview'},
          ],
        },
        {
          title: 'Academic',
          items: [
            {label: 'Bibliography', to: '/academic/bibliography'},
            {label: 'Reading order', to: '/academic/reading-order'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'GitHub', href: 'https://github.com/patricker/argumentation'},
            {label: 'Crates.io', href: 'https://crates.io/crates/argumentation'},
          ],
        },
      ],
      copyright: `MIT / Apache-2.0 dual licensed. Built on the shoulders of Dung, Walton, and the whole argumentation community.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'toml', 'bash'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
```

- [ ] **Step 2: Replace `src/css/custom.css`** with a minimal palette:

```css
:root {
  --ifm-color-primary: #6b4ee6;
  --ifm-color-primary-dark: #5538dc;
  --ifm-color-primary-darker: #4f32d2;
  --ifm-color-primary-darkest: #4029ab;
  --ifm-color-primary-light: #8163ea;
  --ifm-color-primary-lighter: #8c71ec;
  --ifm-color-primary-lightest: #ad9bf1;
  --ifm-code-font-size: 92%;
  --docusaurus-highlighted-code-line-bg: rgba(0, 0, 0, 0.1);
}

[data-theme='dark'] {
  --ifm-color-primary: #a994ff;
  --ifm-color-primary-dark: #8c70ff;
  --ifm-color-primary-darker: #7d5eff;
  --ifm-color-primary-darkest: #4f28ff;
  --ifm-color-primary-light: #c6b8ff;
  --ifm-color-primary-lighter: #d4c9ff;
  --ifm-color-primary-lightest: #ffffff;
  --docusaurus-highlighted-code-line-bg: rgba(0, 0, 0, 0.3);
}

.hero--primary {
  background: linear-gradient(135deg, var(--ifm-color-primary-dark), var(--ifm-color-primary-lightest));
}
```

- [ ] **Step 3: Build + dev-run to confirm no errors**

```bash
cd /home/peter/code/argumentation/website
npm run build
```

- [ ] **Step 4: Commit**

```bash
git add website/docusaurus.config.ts website/src/css/custom.css
git commit -m "feat(website): configure theme, navbar, footer, four content tracks"
```

---

### Task 3: Configure sidebars for the four content tracks

**Files:**
- Rewrite: `website/sidebars.ts`
- Create: `website/docs/concepts/_category_.json`
- Create: `website/docs/examples/_category_.json`
- Create: `website/docs/guides/_category_.json`
- Create: `website/docs/academic/_category_.json`

- [ ] **Step 1: Write `sidebars.ts`**

```typescript
import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  gettingStartedSidebar: [
    'getting-started/first-scene',
  ],
  conceptsSidebar: [
    'concepts/what-is-argumentation',
    'concepts/walton-schemes',
    'concepts/attacks-and-supports',
    'concepts/semantics',
    'concepts/weighted-and-beta',
    'concepts/aspic-plus',
    'concepts/encounter-integration',
  ],
  examplesSidebar: [
    {
      type: 'category',
      label: 'Canonical',
      items: [
        'examples/hal-and-carla',
        'examples/nixon-diamond',
        'examples/tweety-penguin',
      ],
    },
    {
      type: 'category',
      label: 'Applied',
      items: ['examples/thermostat', 'examples/courtroom'],
    },
  ],
  guidesSidebar: [
    'guides/installation',
    'guides/catalog-authoring',
    'guides/implementing-action-scorer',
    'guides/implementing-acceptance-eval',
    'guides/tuning-beta',
  ],
  referenceSidebar: [
    'reference/overview',
  ],
  academicSidebar: [
    'academic/reading-order',
    'academic/bibliography',
    'academic/history',
  ],
};

export default sidebars;
```

- [ ] **Step 2: Create the six `_category_.json` files**

`website/docs/getting-started/_category_.json`:
```json
{
  "label": "Getting Started",
  "position": 1,
  "link": {"type": "generated-index", "description": "Learn by doing — build your first scene end-to-end."}
}
```

`website/docs/concepts/_category_.json`:
```json
{
  "label": "Concepts",
  "position": 2,
  "link": {"type": "generated-index", "description": "Core ideas in formal argumentation, each page under ~10 minutes of reading."}
}
```

`website/docs/examples/_category_.json`:
```json
{
  "label": "Examples",
  "position": 3,
  "link": {"type": "generated-index", "description": "Worked scenarios — canonical (Hal & Carla, Nixon, Tweety) plus applied (thermostat demo, courtroom)."}
}
```

`website/docs/guides/_category_.json`:
```json
{
  "label": "Guides",
  "position": 4,
  "link": {"type": "generated-index", "description": "How to build with the library: install, author catalogs, wire scorers, tune β."}
}
```

`website/docs/reference/_category_.json`:
```json
{
  "label": "Reference",
  "position": 5,
  "link": {"type": "generated-index", "description": "Curated overview of the key types and methods. Full API is in the rustdoc link."}
}
```

`website/docs/academic/_category_.json`:
```json
{
  "label": "Academic",
  "position": 6,
  "link": {"type": "generated-index", "description": "Bibliography, reading order, and the history of formal argumentation."}
}
```

- [ ] **Step 3: Delete the default `docs/intro.md`** that the init command created — we replace with per-track content in later tasks. Touch a placeholder so the build doesn't break meanwhile.

```bash
cd /home/peter/code/argumentation/website/docs
rm -f intro.md tutorial-basics/*.md tutorial-extras/*.md
rmdir tutorial-basics tutorial-extras 2>/dev/null || true
```

The new sidebars reference files we'll create in Phase B; for now the build will fail until those exist. We'll stub them now:

```bash
cd /home/peter/code/argumentation/website/docs
mkdir -p getting-started concepts examples guides reference academic
for f in getting-started/first-scene; do
  echo "# $(basename $f)" > "$f.md"
  echo "" >> "$f.md"
  echo "Coming soon." >> "$f.md"
done
for f in concepts/what-is-argumentation concepts/walton-schemes concepts/attacks-and-supports concepts/semantics concepts/weighted-and-beta concepts/aspic-plus concepts/encounter-integration; do
  echo "# $(basename $f | tr '-' ' ' | sed 's/.*/\u&/')" > "$f.md"
  echo "" >> "$f.md"
  echo "Coming soon." >> "$f.md"
done
for f in examples/hal-and-carla examples/nixon-diamond examples/tweety-penguin examples/thermostat examples/courtroom; do
  echo "# $(basename $f)" > "$f.mdx"
  echo "" >> "$f.mdx"
  echo "Coming soon." >> "$f.mdx"
done
for f in guides/installation guides/catalog-authoring guides/implementing-action-scorer guides/implementing-acceptance-eval guides/tuning-beta; do
  echo "# $(basename $f)" > "$f.md"
  echo "" >> "$f.md"
  echo "Coming soon." >> "$f.md"
done
for f in reference/overview; do
  echo "# $(basename $f)" > "$f.md"
  echo "" >> "$f.md"
  echo "Coming soon." >> "$f.md"
done
for f in academic/bibliography academic/reading-order academic/history; do
  echo "# $(basename $f)" > "$f.md"
  echo "" >> "$f.md"
  echo "Coming soon." >> "$f.md"
done
```

- [ ] **Step 4: Build to confirm sidebars resolve**

```bash
cd /home/peter/code/argumentation/website
npm run build
```

Expect: success (with mostly-empty pages — each later task fills them in).

- [ ] **Step 5: Commit**

```bash
cd /home/peter/code/argumentation
git add website/sidebars.ts website/docs/
git commit -m "feat(website): sidebars + four-track content skeleton"
```

---

### Task 4: Custom landing page

**Files:**
- Replace: `website/src/pages/index.tsx`
- Create: `website/src/components/HomepageFeatures/index.tsx`
- Create: `website/src/components/HomepageFeatures/styles.module.css`
- Create: `website/src/pages/index.module.css`

- [ ] **Step 1: Write `website/src/pages/index.tsx`**

```tsx
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import styles from './index.module.css';

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <Heading as="h1" className="hero__title">{siteConfig.title}</Heading>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className={styles.buttons}>
          <Link className="button button--secondary button--lg" to="/concepts/what-is-argumentation">
            Start with concepts →
          </Link>
          <Link className="button button--outline button--lg" to="/examples/thermostat">
            See the marquee demo
          </Link>
        </div>
      </div>
    </header>
  );
}

export default function Home(): React.ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout title={siteConfig.title} description="Formal argumentation for scene AI">
      <HomepageHeader />
      <main>
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
```

- [ ] **Step 2: Write `website/src/pages/index.module.css`**

```css
.heroBanner {
  padding: 4rem 0;
  text-align: center;
  position: relative;
  overflow: hidden;
}

@media screen and (max-width: 996px) {
  .heroBanner { padding: 2rem; }
}

.buttons {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  flex-wrap: wrap;
  margin-top: 1.5rem;
}
```

- [ ] **Step 3: Write `website/src/components/HomepageFeatures/index.tsx`**

```tsx
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  description: React.JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Auditable scene AI',
    description: (
      <>
        Every beat has a receipt: which arguments fired, which attacks bound,
        which residual produced the acceptance. No black-box LLM hallucinations.
      </>
    ),
  },
  {
    title: '60+ Walton schemes out of the box',
    description: (
      <>
        Argument from expert opinion, analogy, cause-to-effect, slippery slope —
        all as composable scheme instances with premises, conclusions, and critical
        questions.
      </>
    ),
  },
  {
    title: 'β — scene intensity as a first-class dial',
    description: (
      <>
        Tune how strictly attacks bind. Low β = every counter bites (courtroom
        energy); high β = counters slide off (boardroom cordiality). One knob,
        radically different scenes.
      </>
    ),
  },
  {
    title: 'Trait-inverted encounter bridge',
    description: (
      <>
        <code>StateActionScorer</code> and <code>StateAcceptanceEval</code> plug
        into any consumer's scene engine through the <code>encounter</code> crate's
        trait-inverted interface.
      </>
    ),
  },
  {
    title: 'Grounded in the canon',
    description: (
      <>
        Implements Dung (1995), Walton-Reed-Macagno (2008), Cayrol &
        Lagasquie-Schiex (2005), Dunne et al. (2011), Modgil-Prakken ASPIC+ (2014).
        Every primitive traces back to a paper.
      </>
    ),
  },
  {
    title: 'Rust-first, WASM-ready (soon)',
    description: (
      <>
        Workspace of small, composable crates. Zero unsafe. Deterministic by
        default. A WASM build is on the roadmap for browser-native demos.
      </>
    ),
  },
];

function Feature({title, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): React.JSX.Element {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
```

- [ ] **Step 4: Write `website/src/components/HomepageFeatures/styles.module.css`**

```css
.features {
  display: flex;
  align-items: center;
  padding: 2rem 0;
  width: 100%;
}
```

- [ ] **Step 5: Build and verify**

```bash
cd /home/peter/code/argumentation/website
npm run build
```

Visit `http://localhost:3000/` in dev mode to confirm the hero + feature grid render. Kill dev server after checking.

- [ ] **Step 6: Commit**

```bash
git add website/src/pages/ website/src/components/HomepageFeatures/
git commit -m "feat(website): custom landing page with hero + feature grid"
```

---

## Phase B — Interactive components

We build the React components BEFORE writing the content that uses them, so concept/example pages can embed them as they're authored.

### Task 5: Scene-tracer Rust binary (generates fixture JSON)

**Files:**
- Create: `tools/scene-tracer/Cargo.toml`
- Create: `tools/scene-tracer/src/main.rs`
- Modify: `Cargo.toml` (workspace root) — add `tools/scene-tracer` to workspace members

**Why:** The `SceneTrace` React component needs pre-computed JSON. Generating them by hand is error-prone. A tiny Rust binary runs a scene and writes the trace.

- [ ] **Step 1: Add to workspace**

In `/home/peter/code/argumentation/Cargo.toml`, extend `[workspace].members`:

```toml
members = [
    ".",
    "crates/argumentation-schemes",
    "crates/argumentation-bipolar",
    "crates/argumentation-weighted",
    "crates/argumentation-weighted-bipolar",
    "crates/encounter-argumentation",
    "tools/scene-tracer",
]
```

- [ ] **Step 2: Write `tools/scene-tracer/Cargo.toml`**

```toml
[package]
name = "scene-tracer"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
serde = {version = "1", features = ["derive"]}
serde_json = "1"
argumentation-schemes = {path = "../../crates/argumentation-schemes"}
argumentation-weighted = {path = "../../crates/argumentation-weighted"}
argumentation-weighted-bipolar = {path = "../../crates/argumentation-weighted-bipolar"}
encounter-argumentation = {path = "../../crates/encounter-argumentation"}
encounter = {git = "https://github.com/patricker/encounter", tag = "v0.1.0"}
```

- [ ] **Step 3: Write `tools/scene-tracer/src/main.rs`**

Basic structure — takes a scene name and a β value, runs it via `MultiBeat.resolve`, emits a JSON file. Start with the thermostat scene (we'll replicate the uc_multibeat_scene.rs fixture):

```rust
//! scene-tracer: pre-renders argumentation scenes to JSON for the website.
//!
//! Usage: `cargo run -p scene-tracer -- thermostat 0.5 website/static/traces/thermostat-b05.json`

use argumentation_schemes::catalog::default_catalog;
use argumentation_weighted::types::Budget;
use encounter::affordance::{AffordanceSpec, CatalogEntry};
use encounter::practice::{DurationPolicy, PracticeSpec, TurnPolicy};
use encounter::resolution::MultiBeat;
use encounter::scoring::{ActionScorer, ScoredAffordance};
use encounter_argumentation::{
    EncounterArgumentationState, StateAcceptanceEval, StateActionScorer,
};
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Serialize)]
struct Trace {
    scene_name: String,
    beta: f64,
    participants: Vec<String>,
    seeded_arguments: Vec<SeededArg>,
    attacks: Vec<AttackEdge>,
    beats: Vec<BeatRecord>,
    errors: Vec<String>,
}

#[derive(Serialize)]
struct SeededArg {
    actor: String,
    affordance_name: String,
    conclusion: String,
}

#[derive(Serialize)]
struct AttackEdge {
    attacker: String,
    target: String,
    weight: f64,
}

#[derive(Serialize)]
struct BeatRecord {
    actor: String,
    action: String,
    accepted: bool,
}

struct UniformScorer;

impl<P: Clone> ActionScorer<P> for UniformScorer {
    fn score_actions(
        &self,
        actor: &str,
        available: &[CatalogEntry<P>],
        _participants: &[String],
    ) -> Vec<ScoredAffordance<P>> {
        available.iter().map(|e| {
            let mut bindings = HashMap::new();
            bindings.insert("self".into(), actor.to_string());
            let claim = if e.spec.name == "argue_fortify_east" {
                "fortify_east"
            } else {
                "abandon_east"
            };
            bindings.insert("claim".into(), claim.into());
            bindings.insert("expert".into(), actor.to_string());
            bindings.insert("domain".into(), "military".into());
            ScoredAffordance { entry: e.clone(), score: 1.0, bindings }
        }).collect()
    }
}

fn trace_thermostat(beta: f64) -> Trace {
    // For MVP, reuse the uc_multibeat_scene.rs fixture and rename it "thermostat".
    // A later task (see plan Task 16) will author a real thermostat-domain scene.
    let registry = default_catalog();
    let scheme = registry.by_key("argument_from_expert_opinion").unwrap();

    let mut alice_b = HashMap::new();
    alice_b.insert("expert".into(), "alice".into());
    alice_b.insert("domain".into(), "military".into());
    alice_b.insert("claim".into(), "fortify_east".into());
    let alice_instance = scheme.instantiate(&alice_b).unwrap();

    let mut bob_b = HashMap::new();
    bob_b.insert("expert".into(), "bob".into());
    bob_b.insert("domain".into(), "logistics".into());
    bob_b.insert("claim".into(), "abandon_east".into());
    let bob_instance = scheme.instantiate(&bob_b).unwrap();

    let mut state = EncounterArgumentationState::new(registry);
    let mut alice_af = alice_b.clone();
    alice_af.insert("self".into(), "alice".into());
    let alice_id = state.add_scheme_instance_for_affordance(
        "alice", "argue_fortify_east", &alice_af, alice_instance,
    );
    let mut bob_af = bob_b.clone();
    bob_af.insert("self".into(), "bob".into());
    let bob_id = state.add_scheme_instance_for_affordance(
        "bob", "argue_abandon_east", &bob_af, bob_instance,
    );
    state.add_weighted_attack(&bob_id, &alice_id, 0.4).unwrap();
    state.set_intensity(Budget::new(beta).unwrap());

    let make_spec = |name: &str| AffordanceSpec {
        name: name.into(),
        domain: "persuasion".into(),
        bindings: vec!["self".into(), "expert".into(), "domain".into(), "claim".into()],
        considerations: Vec::new(),
        effects_on_accept: Vec::new(),
        effects_on_reject: Vec::new(),
        drive_alignment: Vec::new(),
    };
    let catalog = vec![
        CatalogEntry {spec: make_spec("argue_fortify_east"), precondition: String::new()},
        CatalogEntry {spec: make_spec("argue_abandon_east"), precondition: String::new()},
    ];
    let practice = PracticeSpec {
        name: "debate".into(),
        affordances: vec!["argue_fortify_east".into(), "argue_abandon_east".into()],
        turn_policy: TurnPolicy::RoundRobin,
        duration_policy: DurationPolicy::MultiBeat {max_beats: 4},
        entry_condition_source: String::new(),
    };
    let scorer = StateActionScorer::new(&state, UniformScorer, 0.5);
    let acceptance = StateAcceptanceEval::new(&state);
    let participants = vec!["alice".into(), "bob".into()];
    let result = MultiBeat.resolve(&participants, &practice, &catalog, &scorer, &acceptance);

    Trace {
        scene_name: "thermostat".into(),
        beta,
        participants,
        seeded_arguments: vec![
            SeededArg {actor: "alice".into(), affordance_name: "argue_fortify_east".into(), conclusion: "fortify_east".into()},
            SeededArg {actor: "bob".into(), affordance_name: "argue_abandon_east".into(), conclusion: "abandon_east".into()},
        ],
        attacks: vec![AttackEdge {attacker: "abandon_east".into(), target: "fortify_east".into(), weight: 0.4}],
        beats: result.beats.iter().map(|b| BeatRecord {
            actor: b.actor.clone(), action: b.action.clone(), accepted: b.accepted,
        }).collect(),
        errors: state.drain_errors().iter().map(|e| e.to_string()).collect(),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: scene-tracer <scene> <beta> <out-path>");
        std::process::exit(2);
    }
    let beta: f64 = args[2].parse().expect("beta must be a float");
    let trace = match args[1].as_str() {
        "thermostat" => trace_thermostat(beta),
        other => {
            eprintln!("unknown scene: {}", other);
            std::process::exit(2);
        }
    };
    let json = serde_json::to_string_pretty(&trace).unwrap();
    fs::write(&args[3], json).expect("write failed");
    println!("wrote {}", args[3]);
}
```

- [ ] **Step 4: Build the binary**

```bash
cd /home/peter/code/argumentation
cargo build -p scene-tracer
```

- [ ] **Step 5: Generate the first two traces**

```bash
cd /home/peter/code/argumentation
mkdir -p website/static/traces
cargo run -p scene-tracer -- thermostat 0.0 website/static/traces/thermostat-b00.json
cargo run -p scene-tracer -- thermostat 0.4 website/static/traces/thermostat-b04.json
cargo run -p scene-tracer -- thermostat 0.5 website/static/traces/thermostat-b05.json
cargo run -p scene-tracer -- thermostat 1.0 website/static/traces/thermostat-b10.json
```

All four files must exist and be valid JSON afterwards (spot-check with `jq . < ...`).

- [ ] **Step 6: Commit**

```bash
git add tools/scene-tracer/ Cargo.toml website/static/traces/
git commit -m "feat(tools): scene-tracer binary + thermostat traces at four β values"
```

---

### Task 6: `AttackGraph` component

**Files:**
- Create: `website/src/components/AttackGraph/index.tsx`
- Create: `website/src/components/AttackGraph/styles.module.css`
- Modify: `website/package.json` — add `reactflow`

- [ ] **Step 1: Install `reactflow`**

```bash
cd /home/peter/code/argumentation/website
npm install reactflow
```

- [ ] **Step 2: Write `AttackGraph/index.tsx`**

A read-only graph viewer. Takes `arguments` (nodes, with `id` and optional `label`), `attacks` (edges), and optional `supports`. Uses `reactflow` for layout and interaction.

```tsx
import React from 'react';
import ReactFlow, {Background, Controls, MarkerType, type Edge, type Node} from 'reactflow';
import 'reactflow/dist/style.css';
import styles from './styles.module.css';

export type ArgNode = {id: string; label?: string; accepted?: boolean};
export type ArgEdge = {from: string; to: string; weight?: number; kind?: 'attack' | 'support'};

type Props = {
  arguments: ArgNode[];
  attacks?: ArgEdge[];
  supports?: ArgEdge[];
  height?: number;
  title?: string;
};

export default function AttackGraph({arguments: args, attacks = [], supports = [], height = 300, title}: Props) {
  // Simple circular layout.
  const count = args.length;
  const radius = Math.max(80, 40 * count);
  const nodes: Node[] = args.map((a, i) => {
    const angle = (i / Math.max(count, 1)) * Math.PI * 2 - Math.PI / 2;
    return {
      id: a.id,
      data: {label: a.label ?? a.id},
      position: {x: 200 + radius * Math.cos(angle), y: 150 + radius * Math.sin(angle)},
      style: {
        background: a.accepted === true ? 'var(--ifm-color-success-contrast-background)'
                  : a.accepted === false ? 'var(--ifm-color-danger-contrast-background)'
                  : undefined,
        border: '1px solid var(--ifm-color-emphasis-400)',
        borderRadius: 8,
        padding: 8,
        fontSize: 13,
      },
    };
  });

  const attackEdges: Edge[] = attacks.map((e, i) => ({
    id: `a-${i}`,
    source: e.from,
    target: e.to,
    label: e.weight !== undefined ? e.weight.toFixed(2) : undefined,
    style: {stroke: 'var(--ifm-color-danger)'},
    markerEnd: {type: MarkerType.ArrowClosed, color: 'var(--ifm-color-danger)'},
  }));
  const supportEdges: Edge[] = supports.map((e, i) => ({
    id: `s-${i}`,
    source: e.from,
    target: e.to,
    label: e.weight !== undefined ? e.weight.toFixed(2) : undefined,
    style: {stroke: 'var(--ifm-color-success)'},
    markerEnd: {type: MarkerType.ArrowClosed, color: 'var(--ifm-color-success)'},
  }));

  return (
    <div className={styles.wrapper} style={{height}}>
      {title && <div className={styles.title}>{title}</div>}
      <ReactFlow
        nodes={nodes}
        edges={[...attackEdges, ...supportEdges]}
        fitView
        nodesDraggable
        proOptions={{hideAttribution: true}}
      >
        <Background />
        <Controls showInteractive={false} />
      </ReactFlow>
    </div>
  );
}
```

- [ ] **Step 3: Write `AttackGraph/styles.module.css`**

```css
.wrapper {
  border: 1px solid var(--ifm-color-emphasis-300);
  border-radius: 8px;
  margin: 1.5rem 0;
  overflow: hidden;
  background: var(--ifm-background-surface-color);
}

.title {
  padding: 0.5rem 1rem;
  border-bottom: 1px solid var(--ifm-color-emphasis-200);
  font-weight: 600;
  font-size: 0.9rem;
}
```

- [ ] **Step 4: Smoke-test by embedding it in a scratch MDX file**

Temporarily add to `website/docs/concepts/what-is-argumentation.md` (convert to `.mdx`):

```mdx
import AttackGraph from '@site/src/components/AttackGraph';

<AttackGraph
  title="Nixon diamond"
  arguments={[
    {id: 'A', label: 'Republican → not pacifist'},
    {id: 'B', label: 'Quaker → pacifist'},
  ]}
  attacks={[{from: 'A', to: 'B'}, {from: 'B', to: 'A'}]}
/>
```

Run `npm run build` and confirm. We'll remove the scratch content in Task 9.

- [ ] **Step 5: Commit**

```bash
git add website/src/components/AttackGraph/ website/package.json website/package-lock.json website/docs/concepts/what-is-argumentation.md
git commit -m "feat(website): AttackGraph component + reactflow dep"
```

---

### Task 7: `SchemeCard` component

**Files:**
- Create: `website/src/components/SchemeCard/index.tsx`
- Create: `website/src/components/SchemeCard/styles.module.css`

A pretty presentation of a Walton scheme: premises as a bulleted list, conclusion prominent, critical questions collapsible.

- [ ] **Step 1: Write `index.tsx`**

```tsx
import React, {useState} from 'react';
import styles from './styles.module.css';

type Props = {
  name: string;
  premises: string[];
  conclusion: string;
  criticalQuestions?: string[];
};

export default function SchemeCard({name, premises, conclusion, criticalQuestions = []}: Props) {
  const [openCQ, setOpenCQ] = useState(false);
  return (
    <div className={styles.card}>
      <div className={styles.header}>{name}</div>
      <div className={styles.body}>
        <div className={styles.section}>
          <div className={styles.label}>Premises</div>
          <ul className={styles.list}>
            {premises.map((p, i) => <li key={i}>{p}</li>)}
          </ul>
        </div>
        <div className={styles.section}>
          <div className={styles.label}>Conclusion</div>
          <div className={styles.conclusion}>∴ {conclusion}</div>
        </div>
        {criticalQuestions.length > 0 && (
          <div className={styles.section}>
            <button className={styles.cqToggle} onClick={() => setOpenCQ(!openCQ)}>
              {openCQ ? '▾' : '▸'} Critical questions ({criticalQuestions.length})
            </button>
            {openCQ && (
              <ol className={styles.list}>
                {criticalQuestions.map((cq, i) => <li key={i}>{cq}</li>)}
              </ol>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Write `styles.module.css`**

```css
.card {
  border: 1px solid var(--ifm-color-emphasis-300);
  border-radius: 8px;
  margin: 1.5rem 0;
  overflow: hidden;
  background: var(--ifm-background-surface-color);
}

.header {
  padding: 0.75rem 1rem;
  font-weight: 600;
  background: var(--ifm-color-primary-darker);
  color: #fff;
}

.body { padding: 1rem; }
.section { margin-bottom: 1rem; }
.section:last-child { margin-bottom: 0; }
.label {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--ifm-color-emphasis-600);
  margin-bottom: 0.25rem;
}
.list { margin: 0; padding-left: 1.25rem; }
.conclusion { font-weight: 600; font-size: 1.05rem; }
.cqToggle {
  background: none;
  border: none;
  color: var(--ifm-color-primary);
  font-family: inherit;
  font-size: inherit;
  cursor: pointer;
  padding: 0;
}
.cqToggle:hover { text-decoration: underline; }
```

- [ ] **Step 3: Build to confirm**

```bash
cd /home/peter/code/argumentation/website && npm run build
```

- [ ] **Step 4: Commit**

```bash
git add website/src/components/SchemeCard/
git commit -m "feat(website): SchemeCard component"
```

---

### Task 8: `SceneTrace` + `BetaSlider` components

**Files:**
- Create: `website/src/components/SceneTrace/index.tsx`
- Create: `website/src/components/SceneTrace/styles.module.css`
- Create: `website/src/components/BetaSlider/index.tsx`
- Create: `website/src/components/BetaSlider/styles.module.css`

`SceneTrace` plays back a pre-computed scene (beats appear one by one as the user clicks "next beat" or plays). `BetaSlider` swaps between pre-computed trace variants at discrete β values.

- [ ] **Step 1: Write `SceneTrace/index.tsx`**

```tsx
import React, {useEffect, useState} from 'react';
import styles from './styles.module.css';
import AttackGraph, {type ArgNode, type ArgEdge} from '../AttackGraph';

export type Trace = {
  scene_name: string;
  beta: number;
  participants: string[];
  seeded_arguments: {actor: string; affordance_name: string; conclusion: string}[];
  attacks: {attacker: string; target: string; weight: number}[];
  beats: {actor: string; action: string; accepted: boolean}[];
  errors: string[];
};

type Props = {trace: Trace};

export default function SceneTrace({trace}: Props) {
  const [beatIdx, setBeatIdx] = useState(0);
  const [playing, setPlaying] = useState(false);
  const totalBeats = trace.beats.length;

  useEffect(() => {
    if (!playing) return;
    if (beatIdx >= totalBeats) {setPlaying(false); return;}
    const t = setTimeout(() => setBeatIdx(i => i + 1), 900);
    return () => clearTimeout(t);
  }, [playing, beatIdx, totalBeats]);

  const visibleBeats = trace.beats.slice(0, beatIdx);

  const argNodes: ArgNode[] = trace.seeded_arguments.map(a => ({
    id: a.conclusion,
    label: `${a.conclusion}\n(${a.actor})`,
  }));
  const argEdges: ArgEdge[] = trace.attacks.map(e => ({
    from: e.attacker, to: e.target, weight: e.weight,
  }));

  return (
    <div className={styles.wrapper}>
      <div className={styles.header}>
        <strong>Scene: {trace.scene_name}</strong>
        <span className={styles.beta}>β = {trace.beta.toFixed(2)}</span>
      </div>
      <AttackGraph arguments={argNodes} attacks={argEdges} height={240} />
      <div className={styles.controls}>
        <button onClick={() => setBeatIdx(0)} disabled={beatIdx === 0}>⟲ Reset</button>
        <button onClick={() => setBeatIdx(i => Math.max(0, i - 1))} disabled={beatIdx === 0}>← Back</button>
        <button onClick={() => setBeatIdx(i => Math.min(totalBeats, i + 1))} disabled={beatIdx >= totalBeats}>Next →</button>
        <button onClick={() => {setBeatIdx(0); setPlaying(true);}}>▶ Play</button>
        <span className={styles.counter}>Beat {beatIdx} / {totalBeats}</span>
      </div>
      <ol className={styles.beatList}>
        {visibleBeats.map((b, i) => (
          <li key={i} className={b.accepted ? styles.accepted : styles.rejected}>
            <strong>{b.actor}</strong> proposed <code>{b.action}</code>
            {b.accepted ? ' — accepted' : ' — rejected'}
          </li>
        ))}
      </ol>
      {trace.errors.length > 0 && beatIdx === totalBeats && (
        <div className={styles.errors}>
          <strong>Latched errors:</strong>
          <ul>{trace.errors.map((e, i) => <li key={i}>{e}</li>)}</ul>
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 2: Write `SceneTrace/styles.module.css`**

```css
.wrapper {
  border: 1px solid var(--ifm-color-emphasis-300);
  border-radius: 8px;
  padding: 1rem;
  margin: 1.5rem 0;
  background: var(--ifm-background-surface-color);
}
.header {
  display: flex; justify-content: space-between; align-items: baseline;
  margin-bottom: 0.75rem;
}
.beta { font-family: var(--ifm-font-family-monospace); color: var(--ifm-color-primary); }
.controls { display: flex; gap: 0.5rem; margin: 0.75rem 0; align-items: center; flex-wrap: wrap; }
.controls button { padding: 0.25rem 0.75rem; border-radius: 6px; border: 1px solid var(--ifm-color-emphasis-300); background: var(--ifm-color-emphasis-100); cursor: pointer; font-family: inherit; }
.controls button:disabled { opacity: 0.4; cursor: not-allowed; }
.counter { margin-left: auto; font-size: 0.85rem; color: var(--ifm-color-emphasis-700); }
.beatList { margin-top: 0.5rem; padding-left: 1.5rem; }
.beatList li { margin-bottom: 0.25rem; }
.accepted { color: var(--ifm-color-success); }
.rejected { color: var(--ifm-color-danger); }
.errors { margin-top: 1rem; padding: 0.5rem; background: var(--ifm-color-warning-contrast-background); border-radius: 6px; font-size: 0.85rem; }
```

- [ ] **Step 3: Write `BetaSlider/index.tsx`**

```tsx
import React, {useEffect, useState} from 'react';
import styles from './styles.module.css';
import SceneTrace, {type Trace} from '../SceneTrace';

type Props = {
  tracePaths: {beta: number; path: string}[];
  title?: string;
};

export default function BetaSlider({tracePaths, title}: Props) {
  const [idx, setIdx] = useState(0);
  const [traces, setTraces] = useState<Record<number, Trace>>({});

  useEffect(() => {
    tracePaths.forEach(tp => {
      if (traces[tp.beta]) return;
      fetch(tp.path).then(r => r.json()).then(json => {
        setTraces(prev => ({...prev, [tp.beta]: json}));
      });
    });
  }, [tracePaths]);

  const current = tracePaths[idx];
  const trace = traces[current?.beta];

  return (
    <div className={styles.wrapper}>
      {title && <div className={styles.title}>{title}</div>}
      <div className={styles.sliderRow}>
        <span>β:</span>
        <input
          type="range"
          min={0}
          max={tracePaths.length - 1}
          step={1}
          value={idx}
          onChange={e => setIdx(Number(e.target.value))}
        />
        <span className={styles.value}>{current?.beta.toFixed(2)}</span>
      </div>
      {trace ? <SceneTrace trace={trace} /> : <div>Loading…</div>}
    </div>
  );
}
```

- [ ] **Step 4: Write `BetaSlider/styles.module.css`**

```css
.wrapper { margin: 1.5rem 0; }
.title { font-weight: 600; margin-bottom: 0.5rem; }
.sliderRow {
  display: flex; align-items: center; gap: 0.75rem;
  padding: 0.75rem; background: var(--ifm-color-emphasis-100);
  border-radius: 8px;
}
.sliderRow input[type=range] { flex: 1; }
.value { font-family: var(--ifm-font-family-monospace); min-width: 3em; text-align: right; color: var(--ifm-color-primary); }
```

- [ ] **Step 5: Build to confirm**

```bash
cd /home/peter/code/argumentation/website && npm run build
```

- [ ] **Step 6: Commit**

```bash
git add website/src/components/SceneTrace/ website/src/components/BetaSlider/
git commit -m "feat(website): SceneTrace + BetaSlider components"
```

---

## Phase C — Concept content

**Diataxis type:** EXPLANATION for every page in this phase.  
**Template to load:** `## EXPLANATION TEMPLATE` from `~/.claude/skills/doc-writer/references/templates.md`.  
**No learning objectives** — explanation docs don't carry them (they're not task-anchored).

Each concept page follows this local template (a specialization of the Diataxis EXPLANATION template for our domain):
1. **One-sentence definition** (bolded).
2. **"Why it matters"** (1–2 paragraphs, framed for a dev who doesn't know argumentation).
3. **Mechanics** (the actual content — usually with a diagram or a SchemeCard).
4. **In our library** (how this concept maps to a type or method in our crates).
5. **Further reading** (linked to the academic/bibliography page anchors).

Antipattern to avoid: do NOT slip numbered steps or parameter tables into these pages. If the page catches itself doing that, the content belongs in `guides/` (how-to) or `reference/` instead — split and link.

### Task 9: "What is argumentation?" (the primer)

**Diataxis:** EXPLANATION.

**File:** `website/docs/concepts/what-is-argumentation.mdx` (rename from `.md`)

- [ ] **Step 1: Replace file content**

Rename `what-is-argumentation.md` to `.mdx`. Write:

```mdx
---
sidebar_position: 1
title: What is argumentation?
---

import AttackGraph from '@site/src/components/AttackGraph';

**Formal argumentation is the study of how arguments attack and support each other, and which sets of arguments can stand together without contradicting themselves.**

It sounds abstract, but it's the formal machinery behind auditable reasoning. When a courtroom jury decides which testimony to believe, or when a scientific review board weighs competing studies, they're doing argumentation informally. Formal argumentation gives you primitives — *argument*, *attack*, *support*, *acceptance* — that a computer can reason about.

## Why it matters for scene AI

Most AI scene generation today hands the steering wheel to a language model: prompt it with the setup, let it hallucinate beats. The results can be fluent, but the reasoning is opaque. Why did the villain concede? Why did the witness break down? The answer is "the LLM decided," which is neither auditable nor reproducible.

Formal argumentation replaces the hallucination with a graph and a semantics. Every beat has a receipt:

- Which arguments were asserted by whom.
- Which attacks bound under the current scene tension.
- Which acceptance semantics the beat outcome came from.

You can replay the scene deterministically. You can tune one parameter (β, scene intensity) and watch the beats change in an explainable way. You can mix argumentation with LLM-generated surface prose — argumentation gives you the skeleton, the LLM gives you the dialogue.

## The smallest example

Two arguments that attack each other — the [Nixon diamond](/examples/nixon-diamond):

<AttackGraph
  title="Nixon diamond"
  arguments={[
    {id: 'A', label: 'Republican\n→ not pacifist'},
    {id: 'B', label: 'Quaker\n→ pacifist'},
  ]}
  attacks={[{from: 'A', to: 'B'}, {from: 'B', to: 'A'}]}
  height={260}
/>

Neither argument "wins" in isolation. Dung's acceptance semantics tell you which subsets of these arguments can coherently stand together — in this case, either-or, but not both. The canonical examples page walks through this in detail.

## In our library

- `argumentation` — the core Dung + ASPIC+ implementation.
- `argumentation-bipolar` — attack + support graph structure.
- `argumentation-weighted` — edge weights and β-budgets.
- `argumentation-weighted-bipolar` — the composition of the two.
- `argumentation-schemes` — Walton's 60+ presumptive argument schemes.
- `encounter-argumentation` — the bridge into scene AI via the [`encounter`](https://github.com/patricker/encounter) crate.

## Further reading

Start with [Dung (1995)](/academic/bibliography#dung1995), the paper that founded the field. Then [Walton, Reed & Macagno (2008)](/academic/bibliography#walton2008) for schemes. See [the reading order](/academic/reading-order) for a full curriculum.
```

- [ ] **Step 2: Delete the scratch `what-is-argumentation.md` and any leftover AttackGraph import** from Task 6's smoke test. (The above content supersedes it.)

```bash
cd /home/peter/code/argumentation/website/docs/concepts
rm -f what-is-argumentation.md
```

- [ ] **Step 3: Build + spot-check**

```bash
cd /home/peter/code/argumentation/website && npm run build
```

Dev-mode view the page; the AttackGraph should render.

- [ ] **Step 4: Commit**

```bash
git add website/docs/concepts/what-is-argumentation.mdx
git rm website/docs/concepts/what-is-argumentation.md 2>/dev/null || true
git commit -m "docs(website): concepts/what-is-argumentation primer"
```

---

### Task 10: Walton schemes concept page

**File:** `website/docs/concepts/walton-schemes.mdx`

- [ ] **Step 1: Write content**

```mdx
---
sidebar_position: 2
title: Walton schemes
---

import SchemeCard from '@site/src/components/SchemeCard';

**A Walton scheme is a reusable template of argument — a named pattern of premises and conclusion that people actually use to reason.**

Walton, Reed, and Macagno catalogued about 60 schemes in their 2008 book. Each scheme also carries a list of **critical questions** — the standard ways that argument can be challenged. Our [`argumentation-schemes`](https://crates.io/crates/argumentation-schemes) crate ships instantiable versions of the canon.

## The flagship: argument from expert opinion

<SchemeCard
  name="Argument from expert opinion"
  premises={[
    "Source E is an expert in domain D containing proposition A.",
    "E asserts that A is true.",
  ]}
  conclusion="A is true."
  criticalQuestions={[
    "Expertise: How credible is E as an expert source?",
    "Field: Is E an expert in the field D that A is in?",
    "Opinion: What did E actually assert about A?",
    "Trustworthiness: Is E personally reliable as a source?",
    "Consistency: Is A consistent with what other experts assert?",
    "Backup: Is E's assertion based on evidence?",
  ]}
/>

Instantiate it by binding `expert`, `domain`, and `claim`:

```rust
use argumentation_schemes::catalog::default_catalog;

let registry = default_catalog();
let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
let instance = scheme.instantiate(&[
    ("expert".into(), "Dr. Vance".into()),
    ("domain".into(), "structural engineering".into()),
    ("claim".into(), "the bridge is unsafe".into()),
].into_iter().collect()).unwrap();
```

## Other flagship schemes

Walton's catalogue is large; a few names you'll recognize:

- **Argument from analogy** — "A and B are similar in respects X, Y, Z; A has property P; therefore B probably has P."
- **Argument from cause to effect** — "C generally causes E; C happened; therefore E."
- **Slippery slope** — "A leads to B, B leads to C, …, therefore A leads to something bad."
- **Ad hominem** — "E says A; E has property X; therefore A is suspect."
- **Practical reasoning** — "I want goal G; action M achieves G; therefore I should do M."

Each of these has a `SchemeCard` in the [full scheme gallery](/academic/bibliography#walton2008) (planned for a later release). For the MVP we focus on **expert opinion** because it underpins most of the scene examples in this site.

## Critical questions as attack templates

When a responder rejects an assertion, they often do so by picking a critical question. "Expert says the bridge is unsafe" is defeated by any of:

- **Field:** Dr. Vance is a civil engineer, but this bridge is suspension — different discipline. → undermines premise 1.
- **Consistency:** The other six structural consultants said the bridge was safe. → rebuts the conclusion.
- **Trustworthiness:** Dr. Vance was paid by the bridge's demolition contractor. → rebuts via source.

In our encounter bridge, rejection in `StateAcceptanceEval` corresponds to "the responder has put forward an argument that (credulously) attacks the proposer's." Critical questions are the *template* for those attacks. See [the encounter integration page](/concepts/encounter-integration) for the wiring.

## In our library

- [`argumentation_schemes::Scheme`](/api/) — the scheme trait.
- [`argumentation_schemes::catalog::default_catalog()`](/api/) — the bundled 60+ schemes.
- [`argumentation_schemes::instance::SchemeInstance`](/api/) — an instantiated scheme with bindings.

## Further reading

- Walton, Reed, Macagno (2008) — [the canonical catalogue](/academic/bibliography#walton2008).
- Walton's [*Fundamentals of Critical Argumentation*](/academic/bibliography#walton2006) for shorter teaching examples.
```

- [ ] **Step 2: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/concepts/walton-schemes.mdx
git rm website/docs/concepts/walton-schemes.md 2>/dev/null || true
git commit -m "docs(website): concepts/walton-schemes"
```

---

### Task 11: "Attacks and supports" concept page

**File:** `website/docs/concepts/attacks-and-supports.mdx`

- [ ] **Step 1: Write content**

```mdx
---
sidebar_position: 3
title: Attacks and supports
---

import AttackGraph from '@site/src/components/AttackGraph';

**An *attack* is a directed edge from one argument to another, saying: "if A holds, B fails." A *support* is its dual: "if A holds, B gains strength."**

Attacks give you contradiction. Supports give you reinforcement. Together they form a **bipolar argumentation framework** — the structure that all semantics operate on.

## Attacks alone (Dung, 1995)

Dung's original paper cared only about attacks. An argument is **acceptable** (roughly) if every argument that attacks it is itself attacked by some argument that survives. Recursion and fixed-points make this interesting:

<AttackGraph
  title="Simple attack chain"
  arguments={[
    {id: 'A', accepted: true},
    {id: 'B', accepted: false},
    {id: 'C', accepted: true},
  ]}
  attacks={[{from: 'A', to: 'B'}, {from: 'B', to: 'C'}]}
/>

A attacks B; B attacks C. Under the standard (grounded) semantics, A is accepted, C is accepted (because its attacker B is itself attacked), and B is rejected.

## Why supports?

Cayrol & Lagasquie-Schiex (2005) argued that attack alone misses something: two arguments can **reinforce** each other without either being an attack. "Dr. Vance (engineer) says X" and "Dr. Chen (engineer) independently says X" are distinct arguments that **support** the same conclusion.

Supports form the backbone of **coalition detection**: strongly-connected components of the support graph are arguments that stand or fall together.

<AttackGraph
  title="Mutual support (coalition)"
  arguments={[
    {id: 'A'}, {id: 'B'}, {id: 'C'},
  ]}
  supports={[
    {from: 'A', to: 'B'},
    {from: 'B', to: 'A'},
  ]}
  attacks={[{from: 'C', to: 'A'}]}
/>

A and B mutually support → they are in a coalition. C attacks A, which by the coalition indirectly weakens B too.

## In our library

- [`argumentation_bipolar::BipolarFramework`](/api/) — attacks + supports.
- [`argumentation_weighted_bipolar::WeightedBipolarFramework`](/api/) — adds edge weights (see the [weighted & β page](/concepts/weighted-and-beta)).
- [`EncounterArgumentationState::add_weighted_attack`](/api/) / [`add_weighted_support`](/api/) — the consumer-facing API in the encounter bridge.
- [`EncounterArgumentationState::coalitions`](/api/) — strongly-connected components over the support graph.

## Further reading

- [Dung (1995)](/academic/bibliography#dung1995) — attacks only; the seminal paper.
- [Cayrol & Lagasquie-Schiex (2005)](/academic/bibliography#cayrol2005) — bipolar frameworks.
- [Amgoud et al. (2008)](/academic/bibliography#amgoud2008) — "On bipolarity in argumentation frameworks" (survey).
```

- [ ] **Step 2: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/concepts/attacks-and-supports.mdx
git rm website/docs/concepts/attacks-and-supports.md 2>/dev/null || true
git commit -m "docs(website): concepts/attacks-and-supports"
```

---

### Task 12: "Semantics" concept page

**File:** `website/docs/concepts/semantics.mdx`

- [ ] **Step 1: Write content**

The full page walks through the four standard Dung semantics (conflict-free, admissible, complete, preferred, stable, grounded) plus credulous vs skeptical acceptance. Use `AttackGraph` for two worked examples: a simple chain where all semantics agree, and the Nixon diamond where they diverge.

Frontmatter + opening:

```mdx
---
sidebar_position: 4
title: Acceptance semantics
---

import AttackGraph from '@site/src/components/AttackGraph';

**An acceptance semantics is a rule for deciding which subsets of arguments in a framework can coherently "stand together."** Dung's original paper defined four main ones; we'll walk through them with examples.

## The building blocks

- **Conflict-free set** — no argument in the set attacks another in the set.
- **Admissible set** — conflict-free AND defends itself: every attacker of a member is itself attacked by some member.
- **Complete extension** — an admissible set that contains every argument it defends. (The minimum complete extension is the **grounded extension**; maximum complete extensions are **preferred**.)
- **Stable extension** — a conflict-free set that attacks every argument outside it.

## Credulous vs skeptical

- **Credulously accepted** — the argument is in **at least one** preferred extension.
- **Skeptically accepted** — the argument is in **every** preferred extension.

Skeptical is strictly stronger than credulous. Our library exposes both via [`EncounterArgumentationState::is_credulously_accepted`](/api/) and [`is_skeptically_accepted`](/api/).

## When they diverge

Most of the time, all four semantics (grounded / preferred / stable / complete) agree on which arguments are accepted. The **Nixon diamond** is the canonical case where they diverge:

<AttackGraph
  title="Nixon diamond"
  arguments={[{id: 'A', label: 'Republican'}, {id: 'B', label: 'Quaker'}]}
  attacks={[{from: 'A', to: 'B'}, {from: 'B', to: 'A'}]}
/>

- Grounded: {} (no argument is "unconditionally acceptable").
- Preferred: {A} and {B} (two maximal admissible sets).
- Stable: {A} and {B} (both are stable).
- Complete: {}, {A}, {B}.

Credulous acceptance holds for both A and B (each is in some preferred extension). Skeptical acceptance holds for neither (neither is in every preferred extension).

## Why we care in scene AI

Credulous acceptance is a natural fit for "is this claim live right now?" A beat can proceed as long as the proposer's argument is credulously accepted at the current β. Skeptical acceptance is too strict — it would stall scenes the moment any counter-argument exists.

Our bridge's [`StateActionScorer`](/api/) boosts affordances whose argument is credulously accepted; [`StateAcceptanceEval`](/api/) rejects only when the responder has put forward a credulously-accepted counter.

## Further reading

- [Dung (1995)](/academic/bibliography#dung1995) — definitive.
- [Baroni, Caminada, Giacomin (2011)](/academic/bibliography#baroni2011) — a survey of semantics beyond the original four.
```

- [ ] **Step 2: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/concepts/semantics.mdx
git rm website/docs/concepts/semantics.md 2>/dev/null || true
git commit -m "docs(website): concepts/semantics"
```

---

### Task 13: "Weighted and β" concept page

**File:** `website/docs/concepts/weighted-and-beta.mdx`

- [ ] **Step 1: Write content**

```mdx
---
sidebar_position: 5
title: Weighted frameworks and β
---

import AttackGraph from '@site/src/components/AttackGraph';
import BetaSlider from '@site/src/components/BetaSlider';

**Weighted argumentation assigns a real-valued strength to each attack. A *budget* β lets you drop attacks whose total weight fits in the budget — turning acceptance from a binary question into a continuous one.**

Introduced by Dunne, Hunter, McBurney, Parsons & Wooldridge (2011), weighted argument systems are the mechanism that lets scene tension modulate outcome. In our bridge, β is the **scene intensity** dial.

## The semantic chain

1. A framework has attacks with weights `w(a, b) > 0`.
2. Given a budget β, you consider all *residuals* — frameworks obtained by dropping some subset of attacks whose total cost ≤ β.
3. An argument is **credulously accepted at β** if it is credulously accepted in at least one such residual.

At β = 0 you can drop nothing (equivalent to ordinary Dung semantics). At β large enough to cover every attack, every non-self-attacking argument becomes acceptable.

The boundary is **inclusive**: attacks whose weight exactly equals β are droppable.

## As scene intensity

Think of β as how much the scene can "let slide":

- **β = 0.0** — every counter bites. Tense courtroom energy.
- **β = 0.4** — mid-range; some counters survive, some are waved off.
- **β = 1.0** — counters slide off; amicable boardroom cordiality.

Here's the marquee demo (the [thermostat scene](/examples/thermostat)) at four discrete β values:

<BetaSlider
  title="Thermostat scene across β"
  tracePaths={[
    {beta: 0.0, path: '/traces/thermostat-b00.json'},
    {beta: 0.4, path: '/traces/thermostat-b04.json'},
    {beta: 0.5, path: '/traces/thermostat-b05.json'},
    {beta: 1.0, path: '/traces/thermostat-b10.json'},
  ]}
/>

Drag the slider. At β = 0 both characters accept everything (no attacks survive the budget check below zero — so neither argument is credulous, no boost fires, both fall back to the same action and accept each other). At β = 0.4 and above, alice's argument becomes credulous — the scorer boosts it, she switches to her signature action, and bob rejects with his accepted counter.

## Weighted support (bipolar)

Supports can also carry weights. They don't consume budget — they reinforce the supported argument in the weighted-bipolar semantics of [argumentation-weighted-bipolar](/api/).

## In our library

- [`argumentation_weighted::types::Budget`](/api/) — a validated β value in [0, 1].
- [`argumentation_weighted_bipolar::WeightedBipolarFramework`](/api/) — the underlying graph.
- [`EncounterArgumentationState::set_intensity`](/api/) — set β mid-scene through a shared reference.
- [`EncounterArgumentationState::is_credulously_accepted`](/api/) — acceptance at current β.

## Further reading

- [Dunne, Hunter, McBurney, Parsons, Wooldridge (2011)](/academic/bibliography#dunne2011) — weighted argument systems.
- [Amgoud, Ben-Naim (2016)](/academic/bibliography#amgoud2016) — graded semantics in weighted frameworks.
```

- [ ] **Step 2: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/concepts/weighted-and-beta.mdx
git rm website/docs/concepts/weighted-and-beta.md 2>/dev/null || true
git commit -m "docs(website): concepts/weighted-and-beta + embedded β slider"
```

---

### Task 14: "ASPIC+" concept page

**File:** `website/docs/concepts/aspic-plus.mdx`

- [ ] **Step 1: Write content**

Short page — ASPIC+ is the bridge between Dung's abstract framework and structured arguments built from rules. Keep it to ~500 words, link aggressively to the Modgil-Prakken tutorial.

```mdx
---
sidebar_position: 6
title: ASPIC+ structured arguments
---

**ASPIC+ is a framework that builds Dung-style abstract arguments out of *structured* pieces: strict rules, defeasible rules, premises, and an ordering over them.**

Dung (1995) took arguments as atomic — you have them, they attack each other, you find extensions. ASPIC+ (Modgil & Prakken, 2014) says: let's look inside each argument. An argument is a derivation from premises via strict or defeasible rules. Attacks come in three flavors:

- **Rebutting** — attack the conclusion. "Your argument concludes *fly*; mine concludes *not fly*."
- **Undermining** — attack a premise. "Your argument assumes *Tweety is a bird*; I deny that."
- **Undercutting** — attack a defeasible rule. "Your argument uses *birds generally fly*; I argue the rule doesn't apply here."

## The penguin example

The canonical ASPIC+ example (also Tweety; see [Modgil & Prakken (2014)](/academic/bibliography#modgil2014)):

- Strict: *penguins are birds*.
- Defeasible: *birds generally fly*.
- Defeasible: *penguins generally don't fly*.
- Premise: *Tweety is a penguin*.

Build argument A₁: Tweety → bird (strict) → flies (defeasible, birds-fly rule).
Build argument A₂: Tweety → penguin → doesn't fly (defeasible, penguins-don't-fly rule).

A₁ and A₂ rebut each other. The **preference ordering** (more specific rules beat more general) tells you A₂ wins. The resulting abstract framework has A₂ accepted, A₁ rejected.

## Why our library doesn't force ASPIC+

We support ASPIC+ as a formalism but **don't require it**. In many practical scenes the rule structure is implicit — a scheme instance has premises and a conclusion, and attacks come from schemes attacking schemes. ASPIC+ shines when you need to audit *which rule* got undercut, which the structured view exposes cleanly.

For a full ASPIC+ walkthrough, see [Modgil & Prakken's tutorial](/academic/bibliography#modgil2014) — it is the friendliest entry point to the formalism.

## In our library

- [`argumentation::aspic`](/api/) — structured arguments, strict/defeasible rules, preferences.
- [`argumentation::dung`](/api/) — the abstract layer ASPIC+ reduces to.

## Further reading

- [Modgil & Prakken (2014)](/academic/bibliography#modgil2014) — *The ASPIC+ framework for structured argumentation: a tutorial*.
- [Prakken (2010)](/academic/bibliography#prakken2010) — the original ASPIC+ paper.
```

- [ ] **Step 2: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/concepts/aspic-plus.mdx
git rm website/docs/concepts/aspic-plus.md 2>/dev/null || true
git commit -m "docs(website): concepts/aspic-plus"
```

---

### Task 15: "Encounter integration" concept page

**File:** `website/docs/concepts/encounter-integration.mdx`

- [ ] **Step 1: Write content**

The page explaining *our* contribution — how the bridge crate wires argumentation into the `encounter` scene engine. Covers the ActionScorer/AcceptanceEval split (D4), β as scene intensity, seed-via-bindings, error latch.

```mdx
---
sidebar_position: 7
title: Encounter integration
---

**The `encounter-argumentation` crate bridges formal argumentation into a trait-inverted scene engine.** It is the reason this library exists: to turn the research corpus into a production-ready building block for auditable scene AI.

## The two-question architecture

Scene engines like [`encounter`](https://github.com/patricker/encounter) are trait-inverted — they don't know how to score actions or decide acceptance. They delegate via traits:

- `ActionScorer<P>::score_actions` — how much does actor X want to take action Y right now?
- `AcceptanceEval<P>::evaluate` — does the responder accept Y's proposal?

Our bridge provides one impl of each:

- [`StateActionScorer<'a, S>`](/api/) wraps any inner scorer and **boosts** affordances whose backing argument is credulously accepted at current β. Proposer-side salience.
- [`StateAcceptanceEval<'a>`](/api/) **rejects** when the responder has put forward a credulously-accepted counter-argument. Responder-side gate.

These ask genuinely different questions. The scorer is "what's in the proposer's vocabulary right now?" — a global-β credulous check. The eval is "does the responder have ammunition?" — a per-responder check using [`has_accepted_counter_by`](/api/).

## Lifecycle

1. Build an [`EncounterArgumentationState`](/api/) with your scheme catalog.
2. Set β via [`set_intensity`](/api/).
3. For each (actor, affordance) pair in the scene, call [`add_scheme_instance_for_affordance`](/api/) — this seeds the forward index that the bridge uses at resolve time.
4. Construct a `StateActionScorer` and `StateAcceptanceEval`, both borrowing the state.
5. Hand them to `encounter::resolution::MultiBeat::resolve` (or `SingleExchange`).
6. After resolve returns, call [`drain_errors`](/api/) to collect any latched errors.

See the [first-scene guide](/guides/first-scene) for a complete worked example.

## Why the error latch?

Bridge traits return `bool`, not `Result<bool, Error>`. If `has_accepted_counter_by` fails internally (e.g., framework exceeds the weighted-bipolar enumeration limit), we default to **accept** — permissive — and **append** the error to a per-state buffer. Consumers drain via `drain_errors()` after the scene. This keeps scenes flowing even under internal failure, and exposes failures on the normal drain path.

A common latch entry is [`Error::MissingProposerBinding`](/api/): the bridge uses `"self"` as the proposer slot by convention, and will surface this error when an affordance's bindings don't contain one.

## Zero `encounter` changes

The bridge was designed with the constraint that the sibling `encounter` crate could not be modified. Every Phase B feature — forward index, β dial, error latch — lives on our side of the trait boundary. That meant some interesting workarounds: `Sync` is required by encounter's trait bounds, so our state uses `Mutex<Budget>` and `Mutex<Vec<Error>>` internally (see the [plan doc](https://github.com/patricker/argumentation/blob/main/docs/superpowers/plans/2026-04-20-phase-b-state-scorer-bridge.md) for the full story).

## Further reading

- [encounter crate docs](https://docs.rs/encounter) — the scene engine.
- [Our Phase B plan](https://github.com/patricker/argumentation/blob/main/docs/superpowers/plans/2026-04-20-phase-b-state-scorer-bridge.md) — the architectural walkthrough.
- [First scene guide](/guides/first-scene) — practical walkthrough.
```

- [ ] **Step 2: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/concepts/encounter-integration.mdx
git rm website/docs/concepts/encounter-integration.md 2>/dev/null || true
git commit -m "docs(website): concepts/encounter-integration"
```

---

## Phase D — Example scenarios

**Diataxis type:** EXPLANATION (narrative walkthrough) for Nixon diamond, Hal & Carla, Tweety, and Courtroom. DEMO for Thermostat (marquee).  
**Templates to load:**
- For the four narrative examples: `## EXPLANATION TEMPLATE` adapted to example-walkthrough shape (see each task for the local shape).
- For Thermostat: `## DEMO TEMPLATE` from `~/.claude/skills/doc-writer/references/templates.md`.

Antipattern to avoid: the narrative examples exist to build intuition about canonical scenarios — they should NOT drift into tutorial territory (numbered build steps) or reference (parameter tables). Show the framework, explain why the semantics behave as they do, link to further reading.

### Task 16: Example — Nixon diamond

**Diataxis:** EXPLANATION (narrative walkthrough of a canonical scenario).

**File:** `website/docs/examples/nixon-diamond.mdx`

- [ ] **Step 1: Write content**

```mdx
---
sidebar_position: 1
title: Nixon diamond
---

import AttackGraph from '@site/src/components/AttackGraph';

> "Nixon is anti-pacifist since he is a Republican." / "Nixon is a pacifist since he is a Quaker." — the canonical example for frameworks with multiple extensions, introduced in Dung (1995).

## The setup

Two arguments, each attacking the other. Neither attacks anything else.

<AttackGraph
  arguments={[
    {id: 'A', label: 'Republican → not pacifist'},
    {id: 'B', label: 'Quaker → pacifist'},
  ]}
  attacks={[{from: 'A', to: 'B'}, {from: 'B', to: 'A'}]}
/>

## Which sets stand?

- **Grounded extension:** ∅. Neither A nor B is unconditionally defensible.
- **Preferred extensions:** {A}, {B}. Two maximal admissible sets.
- **Stable extensions:** {A}, {B}. Each stably defeats the other.

Credulously accepted: both A and B (each is in some preferred).
Skeptically accepted: neither (neither is in every preferred).

## In code

```rust
use argumentation::dung::{Framework, grounded};

let mut fw = Framework::new();
fw.add_argument("A");
fw.add_argument("B");
fw.add_attack("A", "B");
fw.add_attack("B", "A");

let g = grounded(&fw);
assert!(g.is_empty());  // grounded extension is empty

let prefs = fw.preferred_extensions();
assert_eq!(prefs.len(), 2);  // two preferred extensions
```

## Why it matters

The diamond is the smallest case where semantics disagree. It's the argument-theoretic equivalent of "should I believe he flies or doesn't?" — a forced choice. Scene AI designers should notice that diamonds create **indecision**: either branch is equally defensible, so the scene resolution depends on *which* preferred extension you pick. In practice, you'd break the tie by adding a weighted attack, a preference ordering, or β — turning the diamond into a one-sided framework.

## Further reading

- [Dung (1995)](/academic/bibliography#dung1995), Example 6.
- [Nixon diamond, Wikipedia](https://en.wikipedia.org/wiki/Nixon_diamond).
```

- [ ] **Step 2: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/examples/nixon-diamond.mdx
git commit -m "docs(website): examples/nixon-diamond"
```

---

### Task 17: Example — Hal & Carla

**Diataxis:** EXPLANATION (narrative walkthrough).  
**File:** `website/docs/examples/hal-and-carla.mdx`

- [ ] **Step 1: Write content**

```mdx
---
sidebar_position: 2
title: Hal & Carla
---

import AttackGraph from '@site/src/components/AttackGraph';

> Hal, a diabetic, loses his insulin. Before collapsing, he enters Carla's house and uses some of her insulin. Carla is another diabetic, not home. Should Hal be punished?

— Trevor Bench-Capon, introducing **value-based argumentation frameworks** (2003). This is the field's most-cited worked example for reasoning about values.

## The arguments

<AttackGraph
  arguments={[
    {id: 'H1', label: 'Hal: life > property'},
    {id: 'C1', label: 'Carla: property rights'},
    {id: 'H2', label: 'Hal: too poor to compensate'},
    {id: 'C2', label: 'Carla: he endangered me (my only dose)'},
  ]}
  attacks={[
    {from: 'C1', to: 'H1'},
    {from: 'H1', to: 'C1'},
    {from: 'C2', to: 'H2'},
    {from: 'H2', to: 'C1'},
  ]}
  height={320}
/>

## Why values matter

Pure Dung semantics can't resolve this — symmetric attacks give you multiple extensions. But you can attach **values** to arguments: H1 promotes *life*, C1 promotes *property*. Different audiences with different value orderings reach different stable positions *rationally*:

- An audience that ranks life > property → H1 accepted, C1 rejected.
- An audience that ranks property > life → the opposite.

Bench-Capon's VAF framework is the machinery that makes "different audiences, same framework, different conclusions" formally precise.

## In our library

Value-based argumentation is on the roadmap for a future crate (`argumentation-values`). Currently our [`argumentation-schemes`](/api/) supports a related mechanism: `PracticalReasoning` schemes carry a value dimension in their bindings, and `encounter-argumentation`'s `StateActionScorer` can be composed with a value-aware inner scorer to produce audience-conditioned outcomes.

## Further reading

- [Bench-Capon (2003)](/academic/bibliography#benchcapon2003) — value-based argumentation frameworks.
- [Atkinson & Bench-Capon (2007)](/academic/bibliography#atkinson2007) — practical reasoning over VAFs.
```

- [ ] **Step 2: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/examples/hal-and-carla.mdx
git commit -m "docs(website): examples/hal-and-carla"
```

---

### Task 18: Example — Tweety penguin

**Diataxis:** EXPLANATION (narrative walkthrough).  
**File:** `website/docs/examples/tweety-penguin.mdx`

- [ ] **Step 1: Write content**

```mdx
---
sidebar_position: 3
title: Tweety the penguin
---

import SchemeCard from '@site/src/components/SchemeCard';

> Birds fly. Penguins are birds. Penguins don't fly. Tweety is a penguin. Does Tweety fly?

The canonical **defeasible reasoning** example — everyone in non-monotonic logic uses it, and it's the go-to demo for ASPIC+.

## The rules

- **Strict rule:** penguins are birds.
- **Defeasible rule R1:** birds generally fly.
- **Defeasible rule R2:** penguins generally don't fly.
- **Premise:** Tweety is a penguin.

## Two arguments

**A₁:** Tweety → bird (strict) → flies (defeasible via R1).
**A₂:** Tweety → penguin → doesn't fly (defeasible via R2).

A₁ and A₂ **rebut** each other (opposite conclusions about Tweety's flying).

## Which wins?

ASPIC+ adds a **preference ordering**: more specific rules dominate more general ones. R2 (about penguins specifically) beats R1 (about birds in general). Therefore A₂ defeats A₁. Tweety does **not** fly.

Without the preference ordering, the framework has two preferred extensions — same symmetric-attack problem as [Nixon](/examples/nixon-diamond). Preferences are the tool ASPIC+ gives you to resolve this kind of draw structurally.

## In code

Full example in our [`argumentation::aspic`](/api/) module — the `aspic::demo::tweety` helper builds the framework, applies the preference, and reports A₂ as the winner.

## Why it matters

Tweety is the smallest case where you need something *beyond* Dung's attack-only abstraction to get an intuitive answer. ASPIC+'s innovation was packaging that "something beyond" as structured arguments + preferences, all of which reduce to an abstract framework that standard semantics can chew on.

## Further reading

- [Modgil & Prakken (2014)](/academic/bibliography#modgil2014) — ASPIC+ tutorial, uses Tweety as the opening example.
- [Reiter (1980)](/academic/bibliography#reiter1980) — the default logic paper that first posed the penguin problem.
```

- [ ] **Step 2: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/examples/tweety-penguin.mdx
git commit -m "docs(website): examples/tweety-penguin"
```

---

### Task 19: Example — Thermostat (marquee)

**Diataxis:** DEMO. Load `## DEMO TEMPLATE` from `~/.claude/skills/doc-writer/references/templates.md` and adapt — this is the one page on the site that genuinely qualifies as a demo (full interactive walkthrough of a realistic use case).  
**File:** `website/docs/examples/thermostat.mdx`

- [ ] **Step 1: Write content**

This is the marquee. Everyday-domain, multi-scheme, β-modulated. For MVP the scene reuses the "fortify east" fixture (wired as thermostat skin in Task 5); a future task can author a true thermostat-domain scheme catalogue.

```mdx
---
sidebar_position: 1
title: The thermostat (marquee)
---

import BetaSlider from '@site/src/components/BetaSlider';

> Two roommates argue about the thermostat. No lawyers, no scientists, no courtroom. Just the world's most relatable disagreement — modeled with formal argumentation.

This is the example the argumentation literature has never built: an **everyday-domain, multi-scheme, single-unfolding-scene** demo. Walton's book has 60+ schemes taught in isolation; academic papers prefer courtroom or medical settings; nobody has shown what happens when the full machinery is turned on "my roommate wants the heat at 68°F and I want it at 72°F."

## Drag β to change the scene

<BetaSlider
  title="The thermostat across β"
  tracePaths={[
    {beta: 0.0, path: '/traces/thermostat-b00.json'},
    {beta: 0.4, path: '/traces/thermostat-b04.json'},
    {beta: 0.5, path: '/traces/thermostat-b05.json'},
    {beta: 1.0, path: '/traces/thermostat-b10.json'},
  ]}
/>

## What's happening

At **β = 0**, every argument's counter binds. Neither character can assert their position without rejection, so they both fall back to a non-contentious action (accepting each other's default).

At **β = 0.4**, the 0.4-weight attack becomes droppable (inclusive boundary). Alice's preferred argument (`argue_fortify_east` — mapped here to her thermostat preference) becomes credulously accepted → the scorer boosts it → she picks it. Bob still has his counter, though, and the counter is still credulously accepted, so he rejects her.

At **β = 1.0**, same dynamic as β = 0.4 for this fixture (the attack weight is 0.4, well below 1.0; no additional edges bind differently).

## Multiple schemes in sequence (planned)

In v0.4 we'll extend the scene so that, when alice's first argument is rejected, she escalates to a second scheme — say, `argument_from_cause_to_effect` ("if we don't fortify east, morale will collapse"). The point of the marquee demo is to show multiple Walton schemes firing in sequence on the same scene — a demonstration that the academic literature has not attempted.

## The receipt

Every beat has a receipt visible in the trace JSON — check `/traces/thermostat-b05.json` in the browser. You can see the seeded arguments, the attack graph, the per-beat acceptance decision, and any errors latched by the bridge. No LLM hallucinations. Replayable.

## What this buys you

If you're building scene AI, the thermostat is a proof point: you don't have to choose between "fluent but opaque" (LLM free-text) and "rigid scripted branching" (state machines). Argumentation gives you formal reasoning with auditable receipts, and β gives you a single dial to move between tension registers — courtroom sharp vs boardroom smooth vs family-dinner relaxed.

## Further reading

- [The encounter integration concept](/concepts/encounter-integration) — how the bridge composes with your scene engine.
- [β as scene intensity](/concepts/weighted-and-beta) — the mechanics of the dial.
- [The first-scene guide](/guides/first-scene) — build your own thermostat.
```

- [ ] **Step 2: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/examples/thermostat.mdx
git commit -m "docs(website): examples/thermostat — the marquee"
```

---

### Task 20: Example — Courtroom

**Diataxis:** EXPLANATION (narrative walkthrough).  
**File:** `website/docs/examples/courtroom.mdx`

- [ ] **Step 1: Write content**

Short page — courtroom is the field's home turf, so we point to the literature for depth and show a compact "snoring witness" vignette.

```mdx
---
sidebar_position: 2
title: Courtroom (snoring witness)
---

import SchemeCard from '@site/src/components/SchemeCard';

> Bob testifies that he saw the defendant at the scene. Alice argues Bob was asleep at the time and could not have seen anything.

A compact illustration of **undercutting** — attacking a premise rather than a conclusion. Adapted from Modgil & Prakken's ASPIC+ tutorial.

## The arguments

Alice's argument undercuts Bob's source-credibility premise (not Bob's conclusion). If Bob was asleep, Bob cannot be a witness; his testimony provides no evidence either way. This is different from rebutting Bob — Alice isn't claiming the defendant *wasn't* at the scene, she's claiming Bob's evidence is void.

## The scheme

<SchemeCard
  name="Argument from witness testimony"
  premises={[
    "W was in position to observe E.",
    "W asserts having observed E.",
  ]}
  conclusion="E happened."
  criticalQuestions={[
    "Was W actually in position to observe (awake, unobstructed, etc.)?",
    "Is W a reliable source (honest, no motive to lie)?",
    "Is W's recollection consistent with other testimony?",
  ]}
/>

Alice's challenge keys on the first critical question. If the framework accepts "Bob was asleep at the time" as credulously accepted, then the premise-attacker wins and Bob's testimony is undercut.

## Further reading

- [Bex, Prakken, Reed, Walton (2003)](/academic/bibliography#bex2003) — argumentation in legal/forensic reasoning.
- [Modgil & Prakken (2014)](/academic/bibliography#modgil2014) — undercutting in ASPIC+.
```

- [ ] **Step 2: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/examples/courtroom.mdx
git commit -m "docs(website): examples/courtroom"
```

---

## Phase E — Developer guides + getting-started tutorial

**Diataxis types:** TUTORIAL for `getting-started/first-scene.md` (Task 21). HOW-TO for every page under `guides/` (Tasks 21b, 22).  
**Templates to load:**
- Tutorial: `## TUTORIAL TEMPLATE` from `~/.claude/skills/doc-writer/references/templates.md`.
- How-to: `## HOW-TO TEMPLATE` from the same file.

**Diataxis boundary rule:** the tutorial page MUST NOT drift into parameter-table territory (that's reference) and MUST NOT skip steps the reader hasn't seen yet (that's how-to). Each how-to page assumes the reader has done the tutorial OR already knows the library — no theory-building inside how-tos.

### Task 21: Getting-started tutorial — your first scene

**Diataxis:** TUTORIAL. Load `## TUTORIAL TEMPLATE`.  
**Learning objective:** By the end of this tutorial, the reader will be able to *build a working roommate-thermostat scene with `encounter-argumentation` in under 10 minutes, without prior argumentation-theory knowledge, using only the commands and code shown.*  
**File:** `website/docs/getting-started/first-scene.md`

- [ ] **Step 1: Write `first-scene.md`** (moved from `guides/` to `getting-started/`; content matches the original Task 21 Step 2 block but adapted to the tutorial template — "What you'll build" header, explicit time+difficulty, numbered steps, complete runnable example at the end, "What you learned" tied back to the learning objective, "Next steps" linking to relevant how-tos and concepts).

Use the body previously drafted (see prior version of this plan). Structural requirements, per Diataxis tutorial discipline:
- Opens with **What you'll build** (one sentence + outcome description)
- Lists prerequisites explicitly (Rust 1.80+, a working terminal)
- Numbered steps (state setup → seed arguments → build catalog → resolve → drain errors)
- Each step has code + "expected output" block
- Complete code at the end as one paste-ready snippet
- Closes with "What you learned" (references the learning objective) and "Next steps" linking to `guides/tuning-beta`, `guides/catalog-authoring`, and `concepts/encounter-integration`

- [ ] **Step 2: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/getting-started/first-scene.md
git commit -m "docs(website): getting-started/first-scene tutorial"
```

### Task 21b: Guide — installation

**Diataxis:** HOW-TO (installing a dependency is a specific task, not a learning arc).  
**Learning objective:** *Install the `argumentation` and `encounter-argumentation` crates into a new or existing Rust project and verify the build, in under 2 minutes.*  
**Files:** `website/docs/guides/installation.md`

- [ ] **Step 1: Write `installation.md`**

```markdown
---
sidebar_position: 1
title: Installation
---

## Rust

Add the core crate:

```toml
[dependencies]
argumentation = "0.2"
```

For weighted bipolar frameworks:

```toml
argumentation-weighted-bipolar = "0.2"
```

For the encounter bridge (needs the sibling [`encounter`](https://github.com/patricker/encounter) crate):

```toml
encounter-argumentation = "0.3"
encounter = "0.1"
```

Minimum supported Rust version: 1.80 (matches workspace edition 2024).

## Cargo features

No features required. The crates are minimal by design — `petgraph` and `thiserror` are the only runtime deps.

## Verify

```bash
cargo check
cargo test
```

## Next steps

- [Build your first scene](/getting-started/first-scene) — guided tutorial.
- [Catalog authoring](/guides/catalog-authoring) — defining your own affordances.
```

- [ ] **Step 2: Build + commit (installation only)**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/guides/installation.md
git commit -m "docs(website): guides/installation"
```

**(The first-scene tutorial is authored in Task 21, not here.)**

---

### Task 22: Guides — catalog, scorer, acceptance, β tuning

**Diataxis:** HOW-TO for every page in this task. Load `## HOW-TO TEMPLATE`.  
**Learning objectives (one per file):**

| File | Learning objective |
|---|---|
| `catalog-authoring.md` | *Define a scheme-backed affordance catalog in TOML and load it at runtime, without writing Rust scheme-registration code.* |
| `implementing-action-scorer.md` | *Wire a custom `ActionScorer<P>` inner implementation through `StateActionScorer` so base scores flow correctly to the bridge's boost logic.* |
| `implementing-acceptance-eval.md` | *Decide whether to use `StateAcceptanceEval` as-is or wrap it, and implement a custom `AcceptanceEval<P>` that composes with it.* |
| `tuning-beta.md` | *Pick an appropriate β for a scene by register (courtroom vs boardroom vs family dinner) and escalate β mid-scene when tension rises.* |

Antipattern to avoid: none of these pages should teach the underlying theory. Link to `concepts/` for that. How-tos assume the reader knows what an `ActionScorer` is.

**Files:**
- `website/docs/guides/catalog-authoring.md`
- `website/docs/guides/implementing-action-scorer.md`
- `website/docs/guides/implementing-acceptance-eval.md`
- `website/docs/guides/tuning-beta.md`

- [ ] **Step 1: Write `catalog-authoring.md`**

Content: how to define affordances in TOML (referencing the `encounter` crate's `AffordanceSpec` serde support), how to associate scheme_id (not-yet-implemented; cross-reference the Phase B plan note about the "no AffordanceSpec change" decision), how to load a catalog at runtime.

~400 words; reference the `encounter` crate's TOML format; include one full worked TOML snippet.

- [ ] **Step 2: Write `implementing-action-scorer.md`**

Content: what the `ActionScorer<P>` trait expects, how `ScoredAffordance` bindings flow to the bridge's `AffordanceKey` lookup, why your inner scorer should still produce meaningful base scores (the bridge only boosts; it doesn't replace your judgment). Include a concrete example of a utility-based inner scorer.

~400 words.

- [ ] **Step 3: Write `implementing-acceptance-eval.md`**

Content: when you'd write your own `AcceptanceEval` vs just use `StateAcceptanceEval` — e.g., you want per-personality responders, or non-"self" proposer slots. Show how to compose: run `StateAcceptanceEval` first, fall through to your own logic on `true`, or intercept on `false`.

~350 words.

- [ ] **Step 4: Write `tuning-beta.md`**

Content: β as a narrative register. Rule-of-thumb table:

| Scene type | Suggested β |
|---|---|
| Courtroom cross-examination | 0.0 – 0.2 |
| Formal meeting | 0.3 – 0.5 |
| Family dinner | 0.4 – 0.7 |
| Boardroom cordiality | 0.7 – 0.9 |
| Celebration / reception | 0.9 – 1.0 |

Discuss β transitions mid-scene (via `set_intensity`), and when you should NOT use β (when the scene's whole point is that a specific attack *must* bind).

~400 words.

- [ ] **Step 5: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/guides/catalog-authoring.md website/docs/guides/implementing-action-scorer.md website/docs/guides/implementing-acceptance-eval.md website/docs/guides/tuning-beta.md
git commit -m "docs(website): guides/catalog-authoring + scorer + acceptance + beta-tuning"
```

---

## Phase F — Reference + academic

**Diataxis types:**
- `reference/overview.md` (Task 24b) → **REFERENCE**. Curated synthesis of the public API; the definitive full reference is rustdoc at `/api/`.
- `academic/bibliography.md` (Task 23) → **REFERENCE**. Browsable citation list.
- `academic/reading-order.md` + `academic/history.md` (Task 24) → **EXPLANATION**. Meta-commentary on the field.

**Templates to load:**
- REFERENCE pages: `## REFERENCE TEMPLATE (Function/Method)` from the doc-writer skill, adapted. Bibliography uses a list-of-entries shape rather than a function-signature shape, but the same scannability principles apply (anchored IDs, complete fields, no prose padding).
- EXPLANATION pages: `## EXPLANATION TEMPLATE`.

### Task 23: Bibliography (the authoritative reference list)

**Diataxis:** REFERENCE. Scannable. Every entry gets an anchor, a full citation, a one-sentence summary, and an open-access link when available. No editorial prose between entries — save that for the reading-order page.  
**File:** `website/docs/academic/bibliography.md`

- [ ] **Step 1: Write the bibliography**

Browsable, anchored by paper ID (`dung1995`, `walton2008`, etc.) so concept pages can link to specific entries. Full citations, open-access links, one-sentence summaries.

```markdown
---
sidebar_position: 1
title: Bibliography
---

The complete reference list for the formalisms this library implements. Anchored by paper ID; concept pages link here.

## Foundational

### dung1995
**Dung, P. M. (1995).** *On the acceptability of arguments and its fundamental role in nonmonotonic reasoning, logic programming and n-person games.* Artificial Intelligence, 77(2), 321–357.
[[PDF]](https://cse-robotics.engr.tamu.edu/dshell/cs631/papers/dung95acceptability.pdf)

The paper that founded the field. Defines abstract argumentation frameworks, conflict-free / admissible / preferred / stable / grounded semantics. Uses the Nixon diamond.

### reiter1980
**Reiter, R. (1980).** *A logic for default reasoning.* Artificial Intelligence, 13(1–2), 81–132.

The default logic paper that motivated much of the field. Home of the Tweety-flies-because-bird problem.

## Walton's canon

### walton2008
**Walton, D., Reed, C., Macagno, F. (2008).** *Argumentation Schemes.* Cambridge University Press.
[[Publisher]](https://www.cambridge.org/core/books/argumentation-schemes/9AE7E4E6ABDE690565442B2BD516A8B6)

The definitive catalogue of ~60 presumptive argument schemes. Each scheme comes with premises, conclusion, and critical questions. Our `argumentation-schemes` crate ships a subset.

### walton2006
**Walton, D. (2006).** *Fundamentals of Critical Argumentation.* Cambridge University Press.

A teaching-focused book — short worked dialogues (parent-child, doctor-patient, courtroom). Good entry point if the 2008 book feels dense.

## Bipolar

### cayrol2005
**Cayrol, C., Lagasquie-Schiex, M.-C. (2005).** *On the acceptability of arguments in bipolar argumentation frameworks.* ECSQARU 2005.
[[Springer]](https://link.springer.com/chapter/10.1007/11518655_33)

Introduces support edges alongside attacks. Argues supports aren't reducible to attacks. The foundation for our `argumentation-bipolar` crate.

### amgoud2008
**Amgoud, L., Cayrol, C., Lagasquie-Schiex, M.-C., Livet, P. (2008).** *On bipolarity in argumentation frameworks.* International Journal of Intelligent Systems, 23(10), 1062–1093.
[[PDF]](https://www.irit.fr/~Leila.Amgoud/docfinal-v4.pdf)

The survey paper on bipolar. Covers multiple support semantics — worth reading if you want to understand *which* support semantics our implementation picks.

## Weighted

### dunne2011
**Dunne, P. E., Hunter, A., McBurney, P., Parsons, S., Wooldridge, M. (2011).** *Weighted argument systems: Basic definitions, algorithms, and complexity results.* Artificial Intelligence, 175(2), 457–486.
[[PDF]](http://www.cs.ox.ac.uk/people/michael.wooldridge/pubs/aij2011a.pdf)

Introduces weighted argument systems and β-budget inconsistency tolerance. The semantic foundation of our `argumentation-weighted` crate. Our β-as-scene-intensity is a direct application of this paper's machinery.

### amgoud2016
**Amgoud, L., Ben-Naim, J. (2016).** *Axiomatic foundations of acceptability semantics.* KR 2016.

Graded semantics in weighted frameworks — ordering arguments by acceptance strength. Relevant if you want per-argument acceptance *scores* rather than binary accept/reject.

## ASPIC+

### prakken2010
**Prakken, H. (2010).** *An abstract framework for argumentation with structured arguments.* Argument and Computation, 1(2), 93–124.

The original ASPIC+ paper. Builds structured arguments from strict/defeasible rules and reduces to Dung frameworks.

### modgil2014
**Modgil, S., Prakken, H. (2014).** *The ASPIC+ framework for structured argumentation: a tutorial.* Argument and Computation, 5(1), 31–62.
[[Publisher]](https://journals.sagepub.com/doi/10.1080/19462166.2013.869766)

The friendlier entry point. Uses the Tweety penguin example. The tutorial we most recommend for newcomers to ASPIC+.

## Values & practical reasoning

### benchcapon2003
**Bench-Capon, T. (2003).** *Persuasion in practical argument using value-based argumentation frameworks.* Journal of Logic and Computation, 13(3), 429–448.
[[arXiv]](https://arxiv.org/pdf/cs/0207059)

Introduces value-based argumentation. Home of the Hal & Carla example.

### atkinson2007
**Atkinson, K., Bench-Capon, T. (2007).** *Practical reasoning as presumptive argumentation using action based alternating transition systems.* Artificial Intelligence, 171(10–15), 855–874.
[[ScienceDirect]](https://www.sciencedirect.com/science/article/pii/S0004370207000689)

Practical reasoning (deliberation between actions) modeled as argumentation over action-transitions. Relevant to scene AI where characters deliberate.

## Legal / forensic

### bex2003
**Bex, F., Prakken, H., Reed, C., Walton, D. (2003).** *Towards a formal account of reasoning about evidence: argumentation schemes and generalisations.* Artificial Intelligence and Law, 11, 125–165.

Argumentation applied to legal evidence. Source of the snoring-witness / undercutting examples.

## Semantics surveys

### baroni2011
**Baroni, P., Caminada, M., Giacomin, M. (2011).** *An introduction to argumentation semantics.* The Knowledge Engineering Review, 26(4), 365–410.

The field's best survey of Dung + post-Dung semantics. Covers complete, preferred, stable, grounded, plus ideal, eager, CF2, stage, etc.
```

- [ ] **Step 2: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/academic/bibliography.md
git commit -m "docs(website): academic/bibliography"
```

---

### Task 24: Reading order + history

**Diataxis:** EXPLANATION for both.  
**Files:** `website/docs/academic/reading-order.md`, `website/docs/academic/history.md`

- [ ] **Step 1: Write `reading-order.md`**

```markdown
---
sidebar_position: 2
title: Reading order
---

The field spans 30 years of papers, most of them technically dense. Here's a curated curriculum.

## 1-hour overview

If you only read one paper:

- [**Baroni, Caminada, Giacomin (2011)**](/academic/bibliography#baroni2011) — *An introduction to argumentation semantics.* The best field-wide survey.

## Full curriculum

Read in order:

1. [**Dung (1995)**](/academic/bibliography#dung1995) — *On the acceptability of arguments…* — the foundational paper. Focus on §2–§4; skim §5 onward.
2. [**Modgil & Prakken (2014)**](/academic/bibliography#modgil2014) — *ASPIC+ tutorial.* Bridges the gap from abstract to structured. Read end-to-end.
3. [**Walton, Reed, Macagno (2008)**](/academic/bibliography#walton2008) — *Argumentation Schemes.* Read Chapter 1 + Chapter 9 (Expert Opinion); skim the catalogue for a taste.
4. [**Cayrol & Lagasquie-Schiex (2005)**](/academic/bibliography#cayrol2005) — *On the acceptability of arguments in bipolar frameworks.* Short (~15 pages).
5. [**Dunne, Hunter, McBurney, Parsons, Wooldridge (2011)**](/academic/bibliography#dunne2011) — *Weighted argument systems.* §1–§3 for definitions; skim complexity results unless you care.
6. [**Bench-Capon (2003)**](/academic/bibliography#benchcapon2003) — *Persuasion in practical argument.* Hal & Carla + values.

## If you want to teach this to someone

Use Walton (2006) — *Fundamentals of Critical Argumentation* — as the textbook. It has the short dialogues that make the formalism click for non-specialists. Then bring in the Modgil-Prakken tutorial for the formal mechanics.

## If you want to build with this

Read the Modgil-Prakken tutorial, then this library's [guides](/guides/installation). The `encounter-argumentation` bridge is the primary entry point.
```

- [ ] **Step 2: Write `history.md`**

~500 words. Narrative arc:

- 1980s: Reiter, McCarthy, etc. — non-monotonic logic establishing the need for defeasible reasoning.
- 1990s: Dung (1995) founds abstract argumentation; parallel developments in AI (Gordon, Verheij, Vreeswijk).
- 2000s: structured arguments (Prakken's early work, ASPIC), bipolar (Cayrol-Lagasquie), value-based (Bench-Capon), schemes (Walton's books).
- 2010s: weighted systems (Dunne et al.), ASPIC+ matures, graded semantics.
- 2020s: applications to scene AI, deliberative systems, legal informatics.

End with: "The library you're reading about treats 30 years of this literature as primitives and asks a new question — what if we use all of this to drive *scenes*, not just resolve arguments?"

- [ ] **Step 3: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/academic/reading-order.md website/docs/academic/history.md
git commit -m "docs(website): academic/reading-order + history"
```

---

### Task 24b: Reference overview (curated API synthesis)

**Diataxis:** REFERENCE. Load `## REFERENCE TEMPLATE (Function/Method)` from the doc-writer skill and adapt to a "key types" shape.  
**Why this page exists:** rustdoc at `/api/` is the full reference, but cold-hitting rustdoc without orientation is harsh. This page is the curated entry point — the 8–12 types a new user will meet first, each with a one-liner and a deep-link to rustdoc.  
**File:** `website/docs/reference/overview.md`

- [ ] **Step 1: Write `overview.md`**

Structure:

```markdown
---
sidebar_position: 1
title: Reference overview
---

Curated entry point into the argumentation workspace. For exhaustive API docs, see [rustdoc](/api/).

## Core types

### `EncounterArgumentationState`
The central state object for the encounter bridge. Composes schemes + bipolar + weighted.
→ [Full docs](/api/encounter_argumentation/state/struct.EncounterArgumentationState.html)

### `StateActionScorer<'a, S>`
Wraps an inner `ActionScorer` and boosts affordances whose argument is credulously accepted at current β.
→ [Full docs](/api/encounter_argumentation/state_scorer/struct.StateActionScorer.html)

### `StateAcceptanceEval<'a>`
Encounter's `AcceptanceEval<P>` impl backed by a live state reference. Rejects on credulously-accepted counter-arguments.
→ [Full docs](/api/encounter_argumentation/state_acceptance/struct.StateAcceptanceEval.html)

### `AffordanceKey`
Canonical `(actor, affordance_name, bindings)` triple used for forward-index lookup.
→ [Full docs](/api/encounter_argumentation/affordance_key/struct.AffordanceKey.html)

### `Budget`
A validated scene-intensity value in [0.0, 1.0]. Construct with `Budget::new(f64)`.
→ [Full docs](/api/argumentation_weighted/types/struct.Budget.html)

### `Scheme` + `SchemeInstance`
A Walton scheme template and its bound instantiation. Instantiate via `Scheme::instantiate(&bindings)`.
→ [Full docs](/api/argumentation_schemes/)

### `WeightedBipolarFramework<A>`
The underlying attack+support+weights graph. Usually accessed through `EncounterArgumentationState`.
→ [Full docs](/api/argumentation_weighted_bipolar/framework/struct.WeightedBipolarFramework.html)

### `Error` (encounter-argumentation)
Error enum for the bridge. Variants include `MissingProposerBinding` — surfaces when an affordance has no `"self"` binding. Drained via `state.drain_errors()`.
→ [Full docs](/api/encounter_argumentation/error/enum.Error.html)

## Core methods

| Method | What it does |
|---|---|
| `EncounterArgumentationState::new(registry)` | Construct with a scheme catalog. |
| `set_intensity(&self, Budget)` | Set β through a shared reference. |
| `add_scheme_instance_for_affordance(...)` | Seed the forward index. Required before `resolve`. |
| `is_credulously_accepted(&id)` | Acceptance check at current β. |
| `has_accepted_counter_by(responder, &target)` | Per-responder attacker-credulity check. |
| `drain_errors()` | Drain the latched error buffer after resolve. |

## Crate map

| Crate | Purpose |
|---|---|
| `argumentation` | Dung + ASPIC+ core. |
| `argumentation-bipolar` | Attacks + supports. |
| `argumentation-weighted` | Weighted edges + `Budget`. |
| `argumentation-weighted-bipolar` | Composition; β-residual semantics. |
| `argumentation-schemes` | Walton's 60+ schemes + catalog. |
| `encounter-argumentation` | The bridge crate. |

## See also

- [Full rustdoc](/api/)
- [Guides](/guides/installation) — how to use these types in practice.
- [Concepts](/concepts/what-is-argumentation) — why these types exist.
```

- [ ] **Step 2: Build + commit**

```bash
cd /home/peter/code/argumentation/website && npm run build
git add website/docs/reference/overview.md
git commit -m "docs(website): reference/overview — curated API synthesis"
```

---

## Phase G — Deployment

### Task 25: Rustdoc build + link

**Files:**
- Create: `.github/workflows/deploy-site.yml`

- [ ] **Step 1: Write the deploy workflow**

```yaml
name: Deploy site

on:
  push:
    branches: [main]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: true

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install Node
        uses: actions/setup-node@v4
        with: {node-version: '20'}

      - name: Build rustdoc
        run: cargo doc --workspace --no-deps --document-private-items

      - name: Build Docusaurus
        working-directory: website
        run: |
          npm ci
          npm run build

      - name: Combine outputs
        run: |
          mkdir -p public
          cp -r website/build/* public/
          mkdir -p public/api
          cp -r target/doc/* public/api/

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: public

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - uses: actions/deploy-pages@v4
        id: deployment
```

- [ ] **Step 2: Update `docusaurus.config.ts`** — the `url`/`baseUrl` may need to change depending on whether the GitHub Pages site is at a custom domain or `username.github.io/argumentation/`. For now, add a comment and leave the custom-domain config.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/deploy-site.yml
git commit -m "ci(website): GitHub Pages deploy workflow"
```

- [ ] **Step 4: Local rustdoc check**

```bash
cd /home/peter/code/argumentation
cargo doc --workspace --no-deps --document-private-items
```

Verify `target/doc/encounter_argumentation/index.html` exists and opens.

---

### Task 26: Final build + link check + polish

- [ ] **Step 1: Full build from clean**

```bash
cd /home/peter/code/argumentation/website
rm -rf node_modules build .docusaurus
npm ci
npm run build
```

Expected: zero warnings, zero broken links.

- [ ] **Step 2: Link check**

```bash
cd /home/peter/code/argumentation/website
npx -y link-check-pkg --recursive build/
```

Or spot-check: click every link in the navbar, each sidebar category root, and a sample of cross-links on concept pages.

- [ ] **Step 3: Lighthouse / accessibility check (manual)**

Run the site locally (`npm run serve`) and use Chrome DevTools Lighthouse. Aim for ≥90 on Accessibility and ≥80 on Performance.

- [ ] **Step 4: README in `website/`**

Write a short `website/README.md` explaining how to build, serve, and deploy locally. (Copy the Docusaurus-generated one and trim it.)

- [ ] **Step 5: Commit**

```bash
git add website/
git commit -m "docs(website): final polish + README"
```

- [ ] **Step 6: Merge to main, trigger deploy**

If on a feature branch, PR to main; once merged the workflow runs and deploys.

---

## Self-review

**Spec coverage:**
- Docusaurus site ✓ (Task 1–4)
- Academic references "appeal to authority" ✓ (Task 23, 24)
- Learning-focused (not just API reference) ✓ (Phases C, D, E)
- Interactive exploration ✓ (Task 6–8 components, Task 19 marquee demo)
- Academic history / reading order ✓ (Task 24)
- Rustdoc integration + deploy ✓ (Task 25)
- **Diataxis-aligned IA** ✓ (D9, per-task Diataxis labels, separated `getting-started/` tutorial from `guides/` how-to, added `reference/overview.md` via Task 24b)

**Explicit non-goals (follow-up plan):**
- WASM compilation of argumentation crates for in-browser live execution. The scene-tracer binary in Task 5 produces pre-computed traces; that's the MVP. A Phase 2 plan should cover `wasm-pack build -p argumentation-weighted-bipolar` and wiring it into `AttackGraph` for live acceptance updates.
- Versioned docs. Enable when v0.4 cuts a breaking change.
- Multi-scheme unfolding in the thermostat marquee. Current marquee shows β modulation on a single pair of arguments; extending to a 6-beat scene where multiple schemes fire sequentially is a post-MVP content task.
- A full scheme gallery (all 60+ Walton schemes with SchemeCards). We ship expert-opinion in Task 10; a scrollable gallery is a post-MVP content task.
- Blog / changelog surface. Docusaurus has a `blog/` plugin; enable when we have release notes to post.
- Search. Algolia DocSearch or local search; defer until site is public and we can prove it's useful.

**Placeholder scan:** No TBDs. Guide pages in Task 22 are word-count-specified rather than word-for-word (content-dense developer guides are genuine prose work; the task fully defines structure, audience, and facts to cover, which is what the implementer needs).

**Type consistency:** Component prop types match across their appearances (`Trace`, `ArgNode`, `ArgEdge` imported from the component files in which they're declared). MDX imports resolve via `@site/src/components/*`. Rust crate versions in `first-scene.md` and `Cargo.toml` snippets match our shipped 0.2 / 0.3 numbers.

**Worktree note:** This plan assumes execution on a feature branch in the main checkout (no worktree). If an executor wants isolation, run `git worktree add ../argumentation-website feat/website` first — but the work touches only `website/`, `tools/scene-tracer/`, and `.github/workflows/deploy-site.yml`, so it's a clean diff against main anyway.

---

## Execution handoff

**Plan complete and saved to `docs/superpowers/plans/2026-04-23-learning-site-docusaurus.md`. Two execution options:**

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints.

**Which approach?**
