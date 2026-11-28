// The page an unknown address gets (the coordinator's walk of 2026-11-27, the decider that day): Cloudflare Pages
// answers 404 with /404.html once it exists, instead of the landing in 200. In the site's own frame and ground,
// its brand; its words, in English alone since 2026-11-28, held by tests/b1.test.mjs; its typography by
// tests/typography.test.mjs, its noindex by tests/open.test.mjs.
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

// One 404, in English (the decider, 2026-11-28): held by tests/b1.test.mjs.
