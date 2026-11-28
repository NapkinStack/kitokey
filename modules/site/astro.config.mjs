// kitokey.com (D114, ADR-0019): static HTML only, no adapter, no client script of Astro's own.
import { defineConfig } from "astro/config";
import sitemap from "@astrojs/sitemap";
import { wholeWords } from "./src/lib/nb.mjs";

export default defineConfig({
  site: "https://kitokey.com",
  output: "static",
  trailingSlash: "always",
  // Styles in files, never inlined, so that the Content-Security-Policy needs no 'unsafe-inline'.
  build: { inlineStylesheets: "never", format: "directory" },
  i18n: { locales: ["en", "fr"], defaultLocale: "en", routing: { prefixDefaultLocale: true, redirectToDefaultLocale: false } },
  integrations: [sitemap({ i18n: { defaultLocale: "en", locales: { en: "en", fr: "fr" } } }), wholeWords()],
  devToolbar: { enabled: false },
});
