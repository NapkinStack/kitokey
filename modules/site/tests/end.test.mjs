// Site v7, the end (validated by the decider on 2026-11-26): the price and the trial as PDR-0009 and PDR-0005 set
// them, no Store badge before its listing, Linux free from its distributions, what a computer needs as README.md
// says, the nine questions, the last call and the footer with Tux's credit. Nothing the app does not keep.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { dist, unwrapped } from "./helpers.mjs";
import { landing } from "../src/lib/landing.mjs";
import { words } from "../src/lib/words.mjs";

const page = (lang) => readFile(join(dist, lang, "index.html"), "utf8").then(unwrapped);
const section = async (lang, id) => (await page(lang)).match(new RegExp(`<section class="end[^"]*"[^>]* id="${id}"[\\s\\S]*?</section>`))?.[0] ?? "";
const text = (html) => html.replace(/<svg[\s\S]*?<\/svg>/g, "").replace(/<[^>]+>/g, " ").replace(/\s+/g, " ");
const ids = { fr: ["prix", "besoins", "faq"], en: ["price", "needs", "faq"] };

for (const lang of ["fr", "en"]) {
  test(`/${lang}/: the price is 69 € once, 69 $ in English (PDR-0009, #1311), the trial 14 days and 10 meetings at least, no account, no subscription`, async () => {
    const html = await section(lang, ids[lang][0]);
    const said = text(html);
    assert.match(said, lang === "fr" ? /69 € une fois/ : /\$69 once/);
    assert.match(said, lang === "fr" ? /14 jours d’essai au moins/ : /14 days of trial at least/);
    assert.match(said, lang === "fr" ? /10 réunions au moins/ : /10 meetings at least/);
    assert.match(said, lang === "fr" ? /Pas d’abonnement/ : /No subscription/);
    assert.match(said, lang === "fr" ? /Sur macOS bientôt, au même prix/ : /On macOS soon, at the same price/);
    assert.doesNotMatch(said, /exportable|export/i, "the meetings said exportable after the trial");
  });

  test(`/${lang}/: no Store badge or link before its listing; Linux free, its three distributions as README.md builds them`, async () => {
    const html = await section(lang, ids[lang][0]);
    assert.doesNotMatch(html, /ms-store|get\.microsoft\.com|apps\.microsoft\.com/);
    // Said as plain text, the Windows mark before it, not a grey pill that reads as a disabled button (the coordinator,
    // 2026-11-28, after #1343): the form under it is what can be done.
    assert.match(html, new RegExp(`<p class="soon-line"><span class="mark" aria-hidden="true"></span>${landing.price[lang].main[9]}</p>`));
    assert.doesNotMatch(html, /is-soon/, "the grey pill still drawn");
    // Each distribution after its mark, as the install tabs show it (the audit, 2026-11-28).
    const distros = [...html.matchAll(/<span><b><img class="mark-(fedora|ubuntu|debian)" src="\/img\/brands\/[a-z0-9-]+\.svg" alt=""[^>]*>(Fedora|Ubuntu|Debian)<\/b>([^<]+)<\/span>/g)]
      .map((m) => (m[1] === m[2].toLowerCase() ? [m[2], m[3]] : m));
    assert.deepEqual(distros, landing.price[lang].linux[4]);
    assert.match(html, new RegExp(`<a class="install-l" href="#(installer|install)">`));
    for (const sys of ["windows-logo-2021.svg", "apple-logo-black.svg", "apple-logo-white.svg", "tux.svg"]) assert.ok(html.includes(`/img/brands/${sys}`), sys);
  });

  test(`/${lang}/: what a computer needs is folded, its two ways README.md's, Ollama said for the private minutes`, async () => {
    const html = await section(lang, ids[lang][1]);
    assert.match(html, /<details class="need">/);
    const ways = [...html.matchAll(/<div><h3>([^<]+)<\/h3>/g)].map((m) => m[1]);
    assert.deepEqual(ways, landing.hardware[lang].map(([, h]) => h));
    assert.match(text(html), /Ollama/);
    assert.doesNotMatch(text(html), /Confidentialité assurée|Confidentiality assured/);
  });

  test(`/${lang}/: nine questions, the first open, the client's and the professional secrecy's among them, no compliance promised`, async () => {
    const html = await section(lang, ids[lang][2]);
    const asked = [...html.matchAll(/<details(?: open)?><summary><span>([^<]+)<\/span><svg/g)].map((m) => m[1]);
    assert.deepEqual(asked, landing.faq[lang][2].map(([q]) => q));
    assert.equal(asked.length, 9);
    assert.match(html, /<details open><summary>/);
    const said = text(html);
    for (const word of ["conforme", "compliant", "RGPD", "GDPR", "HIPAA", "HDS", "certifi"]) assert.ok(!said.includes(word), word);
    assert.doesNotMatch(said, /les exportez|export them/, "past meetings said exportable");
  });

  test(`/${lang}/: the last call, its price PDR-0009's, its words for the Store, Linux free`, async () => {
    const html = (await page(lang)).match(/<section class="end end-band final"[\s\S]*?<\/section>/)?.[0] ?? "";
    assert.ok(html.includes(`<h2 id="final-h">${landing.final[lang][0]}</h2>`));
    assert.match(text(html), lang === "fr" ? /69 € une fois/ : /\$69 once/);
    assert.doesNotMatch(html, /ms-store|get\.microsoft\.com/);
    assert.match(html, /<svg class="illus" viewBox="0 0 320 128" aria-hidden="true">/);
  });

  test(`/${lang}/: the footer names the publisher, the source, the policy, the other language and Tux's author`, async () => {
    const footer = (await page(lang)).match(/<footer class="foot">[\s\S]*?<\/footer>/)?.[0] ?? "";
    for (const key of ["publisher", "source", "privacy", "other", "credit"]) assert.ok(text(footer).includes(words[key][lang]), key);
    assert.match(footer, /Larry Ewing/);
  });
}

// As for the other v7 sections: the old page's rules reach these only by the classes named, each set again under .end.
const SHARED = { wrap: "the page's container", ico: "a pictogram's box", "sr-only": "a heading read and not seen" };

test("the end takes no style of the old page by a class they share, but the ones named", async () => {
  const source = await readFile(new URL("../src/components/Landing.astro", import.meta.url), "utf8");
  // The install section is site.css's, its commands' block styled there.
  const parts = [...source.matchAll(/<section class="end[\s\S]*?<\/section>/g)].map((m) => m[0]).filter((p) => !p.includes("id={installId}")).join("\n");
  const classes = new Set([...parts.matchAll(/class=\{?["`]([^"`]+)["`]/g)].flatMap((m) => m[1].replace(/\$\{[^}]*\}/g, "").split(/\s+/)).filter(Boolean));
  for (const name of ["ico", "chev"]) classes.add(name);
  const css = (await readFile(new URL("../src/styles/site.css", import.meta.url), "utf8")).replace(/\/\*[\s\S]*?\*\//g, "");
  const styled = new Set([...css.matchAll(/([^{}]+)\{/g)].flatMap((m) => [...m[1].matchAll(/\.([a-zA-Z][\w-]*)/g)].map((c) => c[1])));
  const shared = [...classes].filter((c) => styled.has(c));
  assert.deepEqual(shared.filter((c) => !(c in SHARED)), [], "the end takes the old page's style by these classes");
});

test("« Bientôt dans le Microsoft Store » is text: no ground, no edge, no button's height", async () => {
  const { readFile } = await import("node:fs/promises");
  const css = (await readFile(new URL("../src/styles/end.css", import.meta.url), "utf8")).replace(/\/\*[\s\S]*?\*\//g, "");
  // The line's own rules (the selector ends on it), not its mark's, whose ground is the text's colour through a mask.
  const rules = [...css.matchAll(/([^{}]*)\{([^}]*)\}/g)].filter((m) => /\.soon-line\s*$/.test(m[1])).map((m) => m[2]).join(";");
  assert.ok(rules, "no rule for .soon-line");
  assert.doesNotMatch(rules, /background|outline|border(?!-radius)|min-height/, "soon-line has no ground, no edge");
});
