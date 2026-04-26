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
      items: ['examples/east-wall', 'examples/courtroom'],
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
