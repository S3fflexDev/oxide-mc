// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

import tailwindcss from '@tailwindcss/vite';

import icon from 'astro-icon';

import mjs from '@astrojs/mdx';

// https://astro.build/config
export default defineConfig({
  integrations: [starlight({
      title: 'Oxide MC',
      customCss: ['./src/styles/custom.css'],
      social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/S3fflexDev/oxide-mc' }],
      defaultLocale: 'en',
      locales: {
          en: { label: 'English' },
          es: { label: 'Español' }
      },
      sidebar: [
          {
              label: 'Start Here',
              translations: {
                  es: 'Empieza aquí'
              },
              items: [
                  {
                      label: 'Installation',
                      link: 'start-here/installation',
                      translations: {
                          es: 'Instalación'
                      }
                  },
              ],
          },
      ],
      head: [
          {
              tag: "script",
              attrs: { isInline: true },
              content: `
        (function () {
          const stored = localStorage.getItem("starlight-theme");
          const prefersLight = window.matchMedia("(prefers-color-scheme: light)").matches;

          const theme =
            stored ||
            (prefersLight ? "light" : "dark");

          document.documentElement.setAttribute("data-theme", theme);
        })();
      `,
          },
      ],
  }), icon(), mjs()],

  vite: {
    plugins: [tailwindcss()],
  },
});