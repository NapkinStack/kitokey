// The page an unknown address gets (the coordinator's walk of 2026-11-27, the decider that day): Cloudflare Pages
// answers 404 with /404.html once it exists, instead of the landing in 200. In the site's own frame and ground,
// in French and in English, its brand, one human sentence each, and the two landings. Its typography is held by
// tests/typography.test.mjs, its noindex by tests/closed.test.mjs, as every page's.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { dist } from "./helpers.mjs";

const page = () => readFile(join(dist, "404.html"), "utf8");

test("/404.html is built, in the site's frame: its stylesheet, its theme, its brand", async () => {
  const html = await page();
  assert.match(html, /<link rel="stylesheet" href="\/_astro\/[^"]+\.css"/);
  assert.match(html.slice(0, html.indexOf("</head>")), /<script src="\/theme\.js"><\/script>/);
  assert.match(html, /<div class="halo"/);
  assert.match(html, /src="\/kitokey\.svg"/);
});

test("/404.html speaks both languages, each its sentence and its way home", async () => {
  const html = await page();
  const main = html.match(/<main id="main"[\s\S]*?<\/main>/)?.[0] ?? "";
  for (const lang of ["fr", "en"]) {
    const part = main.match(new RegExp(`<div class="lost" lang="${lang}">([\\s\\S]*?)</div>`))?.[1] ?? "";
    assert.match(part, /<h1|<p/, `no ${lang} part`);
    assert.match(part, new RegExp(`<a [^>]*href="/${lang}/"`), `the ${lang} part does not lead to /${lang}/`);
  }
});
