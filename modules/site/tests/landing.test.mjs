// D114, the landing (proposal D, validated by the decider on 2026-11-25): /en/ and /fr/ hold every section
// their nav names, nothing left to measure, the install's commands as the module gives them, and a hero still
// that is the app filmed, with its size and its words.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { join } from "node:path";
import { dist } from "./helpers.mjs";
import { channels, landing } from "../src/lib/landing.mjs";

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

  test(`/${lang}/: the install's commands, as the module gives them`, async () => {
    const html = await page(lang);
    for (const [os, , commands] of channels) {
      const shown = html.match(new RegExp(`<pre class="os-code" data-os="${os}"[^>]*>([\\s\\S]*?)</pre>`))?.[1];
      assert.ok(shown, `no commands for ${os}`);
      assert.equal(text(shown).replace(/^\$ /gm, ""), commands);
    }
  });

  test(`/${lang}/: the hero's still is the record clip's first image, sized, and described`, async () => {
    const html = await page(lang);
    const img = html.match(/<img class="still hero-still"[^>]*>/)?.[0];
    assert.ok(img, "no hero still");
    assert.match(img, new RegExp(`src="/clips/record-${lang}-light\\.jpg"`));
    assert.match(img, /width="1280" height="720"/);
    assert.ok(img.includes(`alt="${landing.still[lang]}"`), "its alt is not the still's words");
    assert.ok((await stat(join(dist, "clips", `record-${lang}-light.jpg`))).size <= 500 * 1024);
  });
}
