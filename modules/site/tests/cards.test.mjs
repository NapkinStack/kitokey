// The « Fonctions » cards (the decider, 2026-11-27, the planche v4 of kitokey-team/site-proto/fonctions-vignettes.html):
// each card a small scene of its own, drawn with the site's tokens, hidden from a screen reader; their texts of a near
// length, 80 to 100 characters; the formats' title « Personnaliser le format de vos comptes rendus »; the folder's card
// keeping that a confidential meeting's summary is never written there (D109, D60).
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { dist } from "./helpers.mjs";
import { landing } from "../src/lib/landing.mjs";

const page = (lang) => readFile(join(dist, lang, "index.html"), "utf8");
const KINDS = ["share", "calendar", "headphones", "folder", "history", "file", "search", "format"];

for (const lang of ["fr", "en"]) {
  test(`/${lang}/: each of the eight cards draws its own scene, hidden from a screen reader`, async () => {
    const cards = [...(await page(lang)).matchAll(/<article class="ft"[^>]*>([\s\S]*?)<\/article>/g)].map(([, card]) => card);
    assert.equal(cards.length, 8);
    cards.forEach((card, i) => {
      const scene = card.match(/<svg class="v v-([a-z]+)"[^>]*>/);
      assert.ok(scene, `card ${i + 1}: no scene`);
      assert.equal(scene[1], KINDS[i], `card ${i + 1}`);
      assert.match(scene[0], /aria-hidden="true"/);
      assert.match(scene[0], /viewBox="0 0 240 112"/);
    });
  });

  test(`/${lang}/: the eight cards' texts are of a near length, 80 to 100 characters`, () => {
    for (const [, title, said] of landing.ready[lang].feats) {
      assert.ok(said.length >= 80 && said.length <= 100, `« ${title} »: ${said.length} characters`);
    }
  });

  test(`/${lang}/: the formats' title says what it lets one do; the folder's card keeps the confidential meetings out`, () => {
    const feats = Object.fromEntries(landing.ready[lang].feats.map(([kind, title, said]) => [kind, [title, said]]));
    assert.equal(feats.format[0], lang === "fr" ? "Personnaliser le format de vos comptes rendus" : "Customize your minutes’ format");
    assert.match(feats.folder[1], lang === "fr" ? /sauf les confidentiels/ : /except confidential ones/);
  });
}
