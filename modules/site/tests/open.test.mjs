// D114, criterion 6: open, on the decider's word (the coordinator, 2026-11-28: the site live on Cloudflare Pages, its
// legal values set, kitokey.com being activated). Opened by the repository, never by a build variable: robots.txt lets
// every robot in and names the sitemap; no page asks not to be indexed, but the 404, which an unknown address gets.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { dist, pages, route } from "./helpers.mjs";
import { open } from "../site.config.mjs";

test("the site is open by the repository, not by a build's variable", () => {
  assert.equal(open, true);
});

test("robots.txt lets every robot in and names the sitemap", async () => {
  const robots = await readFile(join(dist, "robots.txt"), "utf8");
  assert.match(robots, /^User-agent: \*\s*\nAllow: \/\s*$/m);
  assert.match(robots, /^Sitemap: https:\/\/kitokey\.com\/sitemap-index\.xml$/m);
  assert.doesNotMatch(robots, /Disallow/);
});

test("every page may be indexed, but the 404", async () => {
  const all = await pages();
  assert.ok(all.length > 0, "no page built");
  for (const file of all) {
    const html = await readFile(file, "utf8");
    const noindex = /<meta name="robots" content="noindex"/.test(html);
    assert.equal(noindex, route(file) === "/404.html", `${route(file)}: noindex ${noindex}`);
  }
});

test("the sitemap names each language's landing and policy, and never the 404", async () => {
  const index = await readFile(join(dist, "sitemap-index.xml"), "utf8");
  const maps = [...index.matchAll(/<loc>https:\/\/kitokey\.com\/([^<]+)<\/loc>/g)].map(([, path]) => path);
  const urls = (await Promise.all(maps.map((m) => readFile(join(dist, m), "utf8")))).join("");
  for (const lang of ["en", "fr"]) {
    assert.match(urls, new RegExp(`<loc>https://kitokey\\.com/${lang}/</loc>`));
    assert.match(urls, new RegExp(`<loc>https://kitokey\\.com/${lang}/privacy/</loc>`));
  }
  assert.doesNotMatch(urls, /404/);
});
