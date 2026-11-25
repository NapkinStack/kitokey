// D114, the landing (proposal D, validated by the decider on 2026-11-25): /en/ and /fr/ hold every section
// their nav names, nothing left to measure, the install's commands as README.md gives them, and the drawn
// call as the hero.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { join } from "node:path";
import { dist } from "./helpers.mjs";
import { installSource } from "../site.config.mjs";
import { channels, landing } from "../src/lib/landing.mjs";

// README.md's bash blocks, cut in paragraphs at blank lines, each with the system it is for: the one its
// comments name, else those the prose just before the block names (« On Fedora 43, 44 and 45 », « On Ubuntu
// 24.04 … »), else every system.
const SYSTEMS = ["fedora", "ubuntu"];
const named = (text) => SYSTEMS.filter((s) => text.toLowerCase().includes(s));
const blocks = [];
for (const [, before, block] of (await readFile(installSource, "utf8")).matchAll(/([^\n]+(?:\n[^\n]+)*)\n+```bash\n([\s\S]*?)```/g)) {
  blocks.push(block.split(/\n\s*\n/).map((paragraph) => {
    const lines = paragraph.split("\n").filter((l) => l.trim());
    const own = named(lines.map((l) => l.match(/#(.*)$/)?.[1] ?? "").join(" "));
    return { systems: own.length ? own : named(before), lines: lines.map((l) => l.replace(/\s+#.*$/, "").trim()) };
  }));
}
// A tab's commands are, line for line, one block's lines meant for its system or for every system.
const given = (os, commands) => blocks.some((block) => {
  const lines = block.filter((p) => !p.systems.length || p.systems.includes(os)).flatMap((p) => p.lines);
  return lines.join("\n") === commands.join("\n");
});

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

  test(`/${lang}/: the install's commands, line for line, as a block of README.md gives them for that system`, async () => {
    const html = await page(lang);
    for (const [os, , commands] of channels) {
      const shown = html.match(new RegExp(`<pre class="os-code" data-os="${os}"[^>]*>([\\s\\S]*?)</pre>`))?.[1];
      assert.ok(shown, `no commands for ${os}`);
      assert.equal(text(shown).replace(/^\$ /gm, ""), commands);
      assert.ok(given(os, commands.split("\n")), `${os}: its commands are not a README.md block's lines for ${os}`);
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
