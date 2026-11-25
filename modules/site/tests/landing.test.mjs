// D114, the landing (proposal D, validated by the decider on 2026-11-25): /en/ and /fr/ hold every section
// their nav names, nothing left to measure, the install's commands as the module gives them, and the drawn
// call as the hero.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { join } from "node:path";
import { dist } from "./helpers.mjs";
import { installSources } from "../site.config.mjs";
import { channels, landing } from "../src/lib/landing.mjs";

// Every command line the READMEs' bash blocks give, its comment dropped.
const given = new Set();
for (const source of installSources) {
  for (const [, block] of (await readFile(source, "utf8")).matchAll(/```bash\n([\s\S]*?)```/g)) {
    for (const line of block.split("\n")) if (line.trim()) given.add(line.replace(/\s+#.*$/, "").trim());
  }
}

const page = (lang) => readFile(join(dist, lang, "index.html"), "utf8");
const text = (html) => html.replace(/<[^>]+>/g, "").replace(/&amp;/g, "&").replace(/&gt;/g, ">").replace(/&lt;/g, "<");

for (const lang of ["en", "fr"]) {
  test(`/${lang}/: each link of its nav leads to a section of the page`, async () => {
    const html = await page(lang);
    const nav = html.match(/<ul class="nav-links">([\s\S]*?)<\/ul>/)?.[1];
    assert.ok(nav, "no nav links");
    const targets = [...nav.matchAll(/href="#([^"]+)"/g)].map((m) => m[1]);
    assert.deepEqual(targets, landing.nav[lang].map(([id]) => id));
    for (const id of targets) assert.match(html, new RegExp(`<section[^>]* id="${id}"`), `no section #${id}`);
  });

  test(`/${lang}/: nothing left to measure or to do`, async () => {
    assert.doesNotMatch(await page(lang), /\[(À MESURER|TO MEASURE)\]|class="todo"|TODO|XXX/);
  });

  test(`/${lang}/: the install's commands, each a line of the install's READMEs`, async () => {
    const html = await page(lang);
    for (const [os, , commands] of channels) {
      const shown = html.match(new RegExp(`<pre class="os-code" data-os="${os}"[^>]*>([\\s\\S]*?)</pre>`))?.[1];
      assert.ok(shown, `no commands for ${os}`);
      assert.equal(text(shown).replace(/^\$ /gm, ""), commands);
      for (const line of commands.split("\n")) assert.ok(given.has(line), `${os}: « ${line} » is in no README`);
    }
  });

  test(`/${lang}/: the hero is the drawn call, described, with its four portraits`, async () => {
    const html = await page(lang);
    const call = html.match(/<figure class="window hero-window">[\s\S]*?<\/figure>/)?.[0];
    assert.ok(call, "no hero");
    assert.ok(call.includes(`<div class="call" role="img" aria-label="${landing.call[lang][2]}"`), "the call is not described");
    const portraits = [...call.matchAll(/<img class="ptr" src="(\/img\/call-[a-z]+\.webp)"/g)].map((m) => m[1]);
    assert.equal(portraits.length, 4);
    for (const src of portraits) assert.ok((await stat(join(dist, src))).size > 0, `${src} not built`);
  });
}
