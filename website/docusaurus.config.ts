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
