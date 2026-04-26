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
      label: 'Worked examples (literature)',
      items: [
        'examples/nixon-diamond',
        'examples/tweety-penguin',
        'examples/hal-and-carla',
        'examples/courtroom',
      ],
    },
    {
      type: 'category',
      label: 'Engine-driven scenes',
      items: ['examples/east-wall'],
    },
    {
      type: 'category',
      label: 'Interactive',
      items: ['examples/playground'],
    },
  ],
  guidesSidebar: [
    'guides/installation',
    'guides/catalog-authoring',
    'guides/implementing-action-scorer',
    'guides/implementing-acceptance-eval',
    'guides/tuning-beta',
    'guides/debugging-acceptance',
    'guides/societas-modulated-weights',
    'guides/migration-v0.4-to-v0.5',
  ],
  referenceSidebar: [
    'reference/overview',
    {
      type: 'category',
      label: 'Per-crate reference',
      items: [
        'reference/argumentation',
        'reference/argumentation-bipolar',
        'reference/argumentation-weighted',
        'reference/argumentation-weighted-bipolar',
        'reference/argumentation-schemes',
        'reference/encounter-argumentation',
      ],
    },
    'reference/changelog',
  ],
  academicSidebar: [
    'academic/reading-order',
    'academic/bibliography',
    'academic/history',
  ],
};

export default sidebars;
