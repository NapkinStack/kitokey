// What a link to kitokey.com shows (the audit, 2026-11-28; Open Graph, read by X too): every indexed page its
// title, description, address, language and the picture of the promise in its language, 1200 × 630; the 404 none.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { dist, root, pages, route, unindexed } from "./helpers.mjs";

const meta = (html, key) => html.match(new RegExp(`<meta (?:property|name)="${key}" content="([^"]*)"`))?.[1];

test("every indexed page says what a link to it shows, in its own language", async () => {
  for (const file of await pages()) {
    const html = await readFile(file, "utf8");
    if (unindexed(route(file))) {
      assert.equal(meta(html, "og:title"), undefined, `${route(file)} shared`);
      continue;
    }
    const lang = html.match(/<html lang="(en|fr)"/)?.[1];
    const at = route(file);
    assert.equal(meta(html, "og:title"), html.match(/<title>([^<]*)<\/title>/)?.[1], at);
    assert.equal(meta(html, "og:description"), html.match(/<meta name="description" content="([^"]*)"/)?.[1], at);
    assert.equal(meta(html, "og:url"), html.match(/<link rel="canonical" href="([^"]*)"/)?.[1], at);
    assert.equal(meta(html, "og:locale"), lang === "fr" ? "fr_FR" : "en_US", at);
    assert.equal(meta(html, "og:image"), `https://kitokey.com/og/og-${lang}.png`, at);
    assert.equal(meta(html, "twitter:card"), "summary_large_image", at);
    assert.ok((meta(html, "og:image:alt") ?? "").length > 20, at);
  }
});

test("each language's picture is a PNG of 1200 × 630, served from the site itself", async () => {
  for (const lang of ["en", "fr"]) {
    const png = await readFile(join(root, "public", "og", `og-${lang}.png`));
    assert.equal(png.toString("latin1", 1, 4), "PNG", lang);
    assert.deepEqual([png.readUInt32BE(16), png.readUInt32BE(20)], [1200, 630], lang);
    await readFile(join(dist, "og", `og-${lang}.png`));
  }
});
