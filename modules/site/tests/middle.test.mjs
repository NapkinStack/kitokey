// Site v7, the middle (validated by the decider on 2026-11-26): « Vous restez souverain. », « Pour qui », « Après
// l'appel, tout est prêt. ». Each claim as the app holds it: what stays, what leaves only on a choice, « Bientôt » on
// what main does not hold yet, nothing about importing, no health or law promise; the capture's place kept.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { dist } from "./helpers.mjs";
import { landing } from "../src/lib/landing.mjs";

const page = (lang) => readFile(join(dist, lang, "index.html"), "utf8");
const sections = async (lang) => [...(await page(lang)).matchAll(/<section class="mid[^"]*"[\s\S]*?<\/section>/g)].map((m) => m[0]);
const text = (html) => html.replace(/<svg[\s\S]*?<\/svg>/g, "").replace(/<[^>]+>/g, " ").replace(/\s+/g, " ");

// What main holds today, and what is still to come in cycle 12 (D121 PDF, D127 search, D129 and D130 formats).
const SOON = { fr: ["Exporter en PDF", "Chercher dans ce qui a été dit", "Personnaliser le format de vos comptes rendus"], en: ["Export to PDF", "Search what was said", "Customize your minutes’ format"] };

for (const lang of ["fr", "en"]) {
  test(`/${lang}/: three sections in their order, each a heading of its own`, async () => {
    const all = await sections(lang);
    assert.equal(all.length, 3);
    const titles = all.map((s) => s.match(/<h2 id="[^"]+">([^<]+)<\/h2>/)?.[1]);
    assert.deepEqual(titles, [landing.sovereign[lang].h, landing.who[lang].h, landing.ready[lang].h]);
  });

  test(`/${lang}/: what stays is the audio, the transcript and the minutes; what leaves is off at first, the service and the send`, async () => {
    const [sovereign] = await sections(lang);
    const stays = [...sovereign.matchAll(/<div class="item">[\s\S]*?<b>([^<]+)<\/b>[\s\S]*?(role="img" aria-label="([^"]+)"[^>]*>)/g)].map((m) => [m[1], m[3]]);
    assert.deepEqual(stays.map(([what]) => what), [...landing.sovereign[lang].stays, ...landing.sovereign[lang].chosen].map(([, what]) => what));
    assert.deepEqual(stays.map(([, label]) => label), [...Array(3).fill(landing.sovereign[lang].here[2]), ...Array(2).fill(landing.sovereign[lang].leaves[2])]);
    assert.ok(sovereign.includes(landing.sovereign[lang].leaves[1]), "what leaves is not said off at first");
  });

  test(`/${lang}/: the app's own page in the section, in its language, described by its alt; the policy left to the footer`, async () => {
    const [sovereign] = await sections(lang);
    assert.doesNotMatch(sovereign, new RegExp(`href="/${lang}/privacy/"`));
    const alts = [...sovereign.matchAll(/<img class="shot"[^>]*alt="([^"]*)"/g)].map(([, alt]) => alt);
    assert.deepEqual(alts, [landing.sovereign[lang].shot]);
  });

  test(`/${lang}/: three cases, each with its people, its pictogram and a made-up meeting hidden from a screen reader`, async () => {
    const [, who] = await sections(lang);
    const cases = [...who.matchAll(/<article class="case">[\s\S]*?<p class="who-l">([^<]+)<\/p>\s*<h3>([^<]+)<\/h3>[\s\S]*?<div class="lrow" aria-hidden="true">/g)];
    assert.deepEqual(cases.map((m) => [m[1], m[2]]), landing.who[lang].cases.map(([, people, title]) => [people, title]));
  });

  test(`/${lang}/: eight features, « ${lang === "fr" ? "Bientôt" : "Soon"} » on the PDF, the search and the formats alone, nothing about importing`, async () => {
    const [, , ready] = await sections(lang);
    const feats = [...ready.matchAll(/<article class="ft">[\s\S]*?<h3>([^<]+)(<span class="chip">([^<]+)<\/span>)?<\/h3>/g)];
    assert.equal(feats.length, 8);
    assert.deepEqual(feats.filter((m) => m[3]).map((m) => m[1]), SOON[lang]);
    for (const m of feats.filter((m) => m[3])) assert.equal(m[3], landing.ready[lang].soon);
    assert.doesNotMatch(text(ready), /import/i, "a feature about importing");
  });

  test(`/${lang}/: no promise the app does not keep: no health, no law's name, no « passage exact », no Word or subtitles`, async () => {
    const said = text((await sections(lang)).join(" "));
    for (const word of ["médecin", "doctor", "RGPD", "GDPR", "HIPAA", "HDS", "conforme", "compliant", "passage exact", "Word", "sous-titres", "subtitles"]) {
      assert.ok(!said.includes(word), `« ${word} » is said`);
    }
  });
}

// sam's review of #1292: D109's folder never holds a confidential meeting (D60, e2e CF), so no card says every
// summary goes there without that exception.
test("the Markdown folder is never said to hold every summary: confidential meetings stay in Kitokey", () => {
  for (const lang of ["fr", "en"]) {
    const [, , said] = landing.ready[lang].feats.find(([icon]) => icon === "folder");
    assert.match(said, lang === "fr" ? /Réglages/ : /Settings/, said);
    assert.match(said, /confidenti/i, `${lang}: the confidential meetings' exception is not said`);
    for (const [, , card] of landing.ready[lang].feats) {
      assert.ok(!/chaque compte rendu|each summary|every summary/i.test(card) || /confidenti/i.test(card), `${lang}: « ${card} »`);
    }
  }
});

test("#still in the address keeps every group in place, for a capture", async () => {
  const js = await readFile(join(dist, "site.js"), "utf8");
  assert.match(js, /location\.hash !== "#still"/);
  assert.match(js, /querySelectorAll\("\.rise"\)/);
});

// As for the hero and the walkthrough: the old page's rules reach these sections only by the classes named here,
// each set again under .mid in middle.css.
const SHARED = { wrap: "the page's container", ico: "a pictogram's box", tile: "a pictogram's tile", kicker: "the line over the title" };

test("the middle takes no style of the old page by a class they share, but the ones named", async () => {
  const source = await readFile(new URL("../src/components/Landing.astro", import.meta.url), "utf8");
  const start = source.indexOf('<section class="mid"');
  const part = source.slice(start, source.indexOf('<section class="end'));
  const classes = new Set([...part.matchAll(/class=\{?["`]([^"`]+)["`]/g)].flatMap((m) => m[1].replace(/\$\{[^}]*\}/g, "").split(/\s+/)).filter(Boolean));
  for (const name of ["ico", "tile", "priv", "lock"]) classes.add(name);
  const css = (await readFile(new URL("../src/styles/site.css", import.meta.url), "utf8")).replace(/\/\*[\s\S]*?\*\//g, "");
  const styled = new Set([...css.matchAll(/([^{}]+)\{/g)].flatMap((m) => [...m[1].matchAll(/\.([a-zA-Z][\w-]*)/g)].map((c) => c[1])));
  const shared = [...classes].filter((c) => styled.has(c));
  assert.deepEqual(shared.filter((c) => !(c in SHARED)), [], "the middle takes the old page's style by these classes");
  const own = await readFile(new URL("../src/styles/middle.css", import.meta.url), "utf8");
  for (const c of shared.filter((c) => c !== "wrap")) assert.match(own, new RegExp(`\\.mid \\.${c}\\b`), `.${c}: not set again under .mid`);
});
