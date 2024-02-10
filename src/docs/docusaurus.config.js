// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/dracula');

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Logging meets the 2nd dimension',
  tagline: 'Export, visualize, and inspect data from anywhere in your program' ,
  url: 'https://observation.tools',
  baseUrl: '/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',
  favicon: 'img/favicon.svg',
  organizationName: 'Dig-Doug',
  projectName: 'observation-tools-client',
  plugins: ['docusaurus-plugin-sass'],

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          editUrl: 'https://github.com/Dig-Doug/observation-tools-client/src/docs',
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      navbar: {
        title: 'observation.tools',
        logo: {
          alt: 'observation.tools Logo',
          src: 'img/logo.svg',
          srcDark: 'img/logo_dark.svg',
        },
        items: [
          {
            href: 'https://docs.rs/observation-tools/',
            position: 'left',
            label: 'Docs',
          },
          {
            href: 'https://app.observation.tools',
            label: 'Login',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'More',
            items: [
              {
                label: 'Docs',
                href: 'https://docs.rs/observation-tools/',
              },
              {
                label: 'GitHub',
                href: 'https://github.com/Dig-Doug/observation-tools-client',
              },
              {
                label: 'crates.io',
                href: 'https://crates.io/crates/observation-tools'
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Douglas Roeper. Built with Docusaurus.`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
        additionalLanguages: ['rust'],
      },
    }),
};

module.exports = config;
