// Site B1, the decider's look at kitokey.com (relayed by the coordinator, 2026-11-28): the root a 301 to /en/, never a
// page; English first for x-default; the language switch a globe and the other language's name; one 404 in English; the
// features' title the decider's; GDPR said as a fact, never a badge; the lock before what stays; the Store's and the
// Mac's own marks on their lines.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { dist, root, pages, route } from "./helpers.mjs";

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
    if (route(file) === "/404.html") continue;
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
