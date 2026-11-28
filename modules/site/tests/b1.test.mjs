// Site B1, the decider's look at kitokey.com (relayed by the coordinator, 2026-11-28): the root a 301 to /en/, never a
// page; English first for x-default; the language switch a globe and the other language's name; one 404 in English; the
// features' title the decider's; GDPR said as a fact, never a badge; the lock before what stays; the Store's and the
// Mac's own marks on their lines.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { dist, root, pages, route, unindexed } from "./helpers.mjs";

const page = (lang) => readFile(join(dist, lang, "index.html"), "utf8");
const text = (html) => html.replace(/<svg[\s\S]*?<\/svg>/g, "").replace(/<[^>]+>/g, " ").replace(/\s+/g, " ");

test("the root is a 301 to /en/, and no page is built there", async () => {
  const redirects = await readFile(join(root, "public", "_redirects"), "utf8");
  assert.match(redirects, /^\/ \/en\/ 301$/m);
  assert.ok(!existsSync(join(dist, "index.html")), "a root page is built beside the redirect");
});

test("every page names English its x-default", async () => {
  for (const file of await pages()) {
    const html = await readFile(file, "utf8");
    const x = html.match(/<link rel="alternate" hreflang="x-default" href="([^"]+)"/)?.[1];
    if (unindexed(route(file))) continue;
    assert.equal(x, "https://kitokey.com/en/", route(file));
  }
});

test("the sitemap lists no root, only canonical pages", async () => {
  const index = await readFile(join(dist, "sitemap-index.xml"), "utf8");
  const maps = [...index.matchAll(/<loc>https:\/\/kitokey\.com\/([^<]+)<\/loc>/g)].map(([, p]) => p);
  const urls = (await Promise.all(maps.map((m) => readFile(join(dist, m), "utf8")))).join("");
  assert.doesNotMatch(urls, /<loc>https:\/\/kitokey\.com\/<\/loc>/);
});

test("the language switch: a globe, then the other language's name in full, in that language; no flag anywhere", async () => {
  for (const [lang, other, name] of [["fr", "en", "English"], ["en", "fr", "Français"]]) {
    const html = await page(lang);
    const switcher = html.match(/<a class="lang"[^>]*>[\s\S]*?<\/a>/)?.[0] ?? "";
    assert.match(switcher, new RegExp(`^<a class="lang" href="/${other}/" hreflang="${other}">`), `/${lang}/: ${switcher.slice(0, 80)}`);
    assert.match(switcher, /^<a[^>]*><svg class="ico"[^>]*aria-hidden="true"/, `/${lang}/: no globe first`);
    assert.match(switcher, new RegExp(`<span lang="${other}">${name}</span></a>$`), `/${lang}/: « ${name} »`);
    assert.doesNotMatch(switcher, /aria-label/, `/${lang}/: a name other than the one shown`);
    assert.doesNotMatch(html, /class="flag"|[\u{1F1E6}-\u{1F1FF}]/u, `/${lang}/: a flag`);
  }
});

test("one 404, in English: « 404 », « Not found », the way home, noindex", async () => {
  const html = await readFile(join(dist, "404.html"), "utf8");
  assert.match(html, /<html lang="en"/);
  const said = text(html.match(/<main[\s\S]*?<\/main>/)?.[0] ?? "");
  assert.match(said, /404/);
  assert.match(said, /Not found/);
  assert.match(html.match(/<main[\s\S]*?<\/main>/)?.[0] ?? "", /href="\/en\/"/);
  assert.doesNotMatch(html.match(/<main[\s\S]*?<\/main>/)?.[0] ?? "", /lang="fr"/);
  assert.match(html, /<meta name="robots" content="noindex"/);
});

test("the features' title is the decider's", async () => {
  for (const [lang, h] of [["fr", "Votre assistant de réunion, au quotidien."], ["en", "Your everyday meeting assistant."]]) {
    assert.match(await page(lang), new RegExp(`<h2 id="[^"]+">${h.replace(/\./g, "\\.")}</h2>`), `/${lang}/`);
  }
});

test("the privacy section says what GDPR is spared, as a fact, never a badge of compliance", async () => {
  for (const [lang, id, law] of [["fr", "confidentialite", "RGPD"], ["en", "privacy", "GDPR"]]) {
    const html = await page(lang);
    const said = text(html.match(new RegExp(`<section[^>]*id="${id}"[\\s\\S]*?</section>`))?.[0] ?? "");
    assert.match(said, new RegExp(law), `/${lang}/: ${law} not said`);
    assert.doesNotMatch(said, /conforme|compliant|certifi|compliance/i, `/${lang}/: a badge of compliance`);
  }
});

test("« Reste sur votre ordinateur » begins with the lock", async () => {
  for (const lang of ["fr", "en"]) {
    const h3 = (await page(lang)).match(/<div class="box here">[\s\S]*?<h3>([\s\S]*?)<\/h3>/)?.[1] ?? "";
    assert.match(h3, /^<svg[^>]*><use href="#i-lock"\/><\/svg>/, `/${lang}/: « ${text(h3)} »`);
  }
});

test("the Store's line bears the Windows mark (the coordinator, 2026-11-28: the Store's bag comes with its badge), the Mac's the Apple's", async () => {
  for (const lang of ["fr", "en"]) {
    const html = await page(lang);
    const store = html.match(/<a class="store-soon"[^>]*>[\s\S]*?<\/a>/)?.[0] ?? "";
    assert.match(store, /<span class="mark" aria-hidden="true"><\/span>/, `/${lang}/: the Windows mark`);
    assert.doesNotMatch(store, /<img/, `/${lang}/: the mark in its own colours`);
    const mac = html.match(/<p class="cta-note">[\s\S]*?<\/p>/)?.[0] ?? "";
    assert.match(mac, /<img[^>]*src="\/img\/brands\/apple-logo-(black|white)\.svg"[^>]*alt=""/, `/${lang}/: the Mac's mark`);
  }
});

test("the Windows mark is the button's one colour, its owner's file as a mask, no plate, as tall as the capitals", async () => {
  const css = await readFile(join(root, "src", "styles", "hero.css"), "utf8");
  const rule = css.match(/\.store-soon \.mark \{[^}]*\}/)?.[0] ?? "";
  assert.match(rule, /background: currentColor/);
  assert.match(rule, /(?<!-webkit-)mask: url\(\/img\/brands\/windows-logo-2021\.svg\)/);
  assert.match(rule, /-webkit-mask: url\(\/img\/brands\/windows-logo-2021\.svg\)/);
  assert.match(rule, /height: 1cap/);
  assert.doesNotMatch(css, /#FFFFFF/i, "a white plate");
});

test("each FAQ question is one item of its summary: a word held whole (.nb) never a flex item of its own (the audit, 2026-11-28)", async () => {
  for (const lang of ["fr", "en"]) {
    const html = await page(lang);
    const summaries = [...html.matchAll(/<div class="faq">[\s\S]*?<\/div><\/details><\/div>/g)].join("").match(/<summary>[\s\S]*?<\/summary>/g) ?? [];
    assert.ok(summaries.length >= 9, `/${lang}/: ${summaries.length} questions`);
    for (const s of summaries) {
      assert.match(s, /^<summary><span>(?:[^<]|<span class="nb">[^<]*<\/span>)+<\/span><svg class="ico chev"/, `/${lang}/: ${s.slice(0, 90)}`);
    }
  }
});

test("the walk's selected tab draws the keyboard's focus as every control does, after its own edge (the audit, 2026-11-28)", async () => {
  const css = await readFile(join(root, "src", "styles", "walk.css"), "utf8");
  const edge = css.indexOf('.wtab[aria-selected="true"] {');
  const focus = css.indexOf(".wtab:focus-visible { outline: 3px solid var(--accent)");
  assert.ok(edge >= 0 && focus > edge, `edge at ${edge}, focus at ${focus}`);
});

test("the 404, kept out of the index, names no canonical and no alternate (the audit, 2026-11-28)", async () => {
  const html = await readFile(join(dist, "404.html"), "utf8");
  assert.doesNotMatch(html, /<link rel="(canonical|alternate)"/);
  assert.match(await page("en"), /<link rel="canonical" href="https:\/\/kitokey\.com\/en\/"/);
});

test("the legal notice's values wrap anywhere, each under its name at a phone's width (the audit, 2026-11-28)", async () => {
  const css = await readFile(join(root, "src", "styles", "site.css"), "utf8");
  assert.match(css, /\.legal dd \{[^}]*overflow-wrap: anywhere/);
  assert.match(css, /@media \(max-width: 480px\) \{ \.legal \{ grid-template-columns: 1fr;/);
});

test("Fedora's mark is its « f » alone; Debian's swirl on a plate on the dark tabs; the price card's distributions bear their marks", async () => {
  const html = await page("en");
  const tab = (os) => html.match(new RegExp(`<button type="button" class="os-tab" data-os="${os}"[\\s\\S]*?</button>`))?.[0] ?? "";
  assert.match(tab("fedora"), /src="\/img\/brands\/fedora-icon-2021\.svg"/);
  assert.match(tab("debian"), /<img class="mark-debian"/);
  const site = await readFile(join(root, "src", "styles", "site.css"), "utf8");
  assert.match(site, /\.os-tab \.mark-debian \{ background: #FFFFFF;/);
  const end = await readFile(join(root, "src", "styles", "end.css"), "utf8");
  assert.match(end, /\.end \.strip \.mark-ubuntu \{ background: #E95420;/);
  assert.ok(!existsSync(join(root, "public", "img", "brands", "fedora-logo-official.svg")), "the wordmark kept");
});
