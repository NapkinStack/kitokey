// Site v7, the hero (validated by the decider on 2026-11-26): PDR-0009's promise as the h1, its price and trial, the
// Store's words until its listing exists, Linux free, the reassurance the app keeps, a drawn scene described to a
// screen reader, and the two voices' spectrogram, decorative, still under reduced motion.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import { join } from "node:path";
import vm from "node:vm";
import { dist } from "./helpers.mjs";
import { landing } from "../src/lib/landing.mjs";

const page = (lang) => readFile(join(dist, lang, "index.html"), "utf8");
const hero = async (lang) => (await page(lang)).match(/<section class="hero"[\s\S]*?<\/section>/)?.[0] ?? "";
const PROMISE = {
  fr: ["Vos rendez-vous clients ont leur compte rendu.", "Rien ne quitte votre ordinateur."],
  en: ["Every client call gets its minutes.", "Nothing leaves your computer."],
};

for (const lang of ["fr", "en"]) {
  test(`/${lang}/: the h1 is PDR-0009's promise, its two lines`, async () => {
    const h1 = (await hero(lang)).match(/<h1[^>]*>([\s\S]*?)<\/h1>/)?.[1] ?? "";
    assert.deepEqual([...h1.matchAll(/<span class="l[^"]*">([^<]+)<\/span>/g)].map((m) => m[1]), PROMISE[lang]);
  });

  test(`/${lang}/: the price is PDR-0009's, 69 € once after a free trial, 69 $ in English (#1311); no Store badge before its listing; Linux free`, async () => {
    const html = await hero(lang);
    const [line, store, linux, mac] = landing.buy[lang];
    assert.equal(line, lang === "fr" ? "Essai gratuit, puis 69 € une fois." : "Free trial, then $69 once.");
    assert.ok(html.includes(`<p class="buy-line">${line}</p>`));
    assert.match(html, new RegExp(`<a class="store-soon" href="#[a-z]+">${store}</a>`));
    assert.doesNotMatch(html, /ms-store|get\.microsoft\.com/, "a Store badge before the listing exists");
    assert.match(html, new RegExp(`<a class="quiet" href="#[a-z]+"><img class="tux" src="/img/brands/tux\\.svg"[^>]*>${linux}</a>`));
    assert.ok(html.includes(mac));
    for (const [, target] of html.matchAll(/<a class="(?:store-soon|quiet)" href="#([a-z]+)"/g)) {
      assert.match(await page(lang), new RegExp(`<section[^>]* id="${target}"`), `#${target} is no section`);
    }
  });

  test(`/${lang}/: the reassurance says what the app keeps: no account, no subscription, no bot`, async () => {
    const said = [...(await hero(lang)).matchAll(/<span><svg[^>]*>[\s\S]*?<\/svg>([^<]+)<\/span>/g)].map((m) => m[1]);
    assert.deepEqual(said, lang === "fr" ? ["Sans compte.", "Sans abonnement.", "Aucun robot dans l’appel."] : ["No account.", "No subscription.", "No bot in the call."]);
  });

  test(`/${lang}/: the scene is one image to a screen reader, its parts hidden, no heading of its own; the canvas decorative`, async () => {
    const html = await hero(lang);
    assert.ok(html.includes(`<div class="scene" role="img" aria-label="${landing.scene[lang][0]}">`));
    assert.match(html, /<div class="cbar" aria-hidden="true">/);
    assert.match(html, /<div class="callwin" aria-hidden="true">/);
    assert.match(html, /<article class="mcard" aria-hidden="true">/);
    assert.doesNotMatch(html.slice(html.indexOf('class="scene"')), /<h[1-6]/);
    assert.match(html, /<canvas class="voices" aria-hidden="true"><\/canvas>/);
    assert.doesNotMatch(html, /class="slider"|data-clip=/, "the old carousel is still drawn");
  });
}

test("the spectrogram runs only on screen and with the tab visible, at most 30 frames a second, and stands still under reduced motion", async () => {
  const js = await readFile(join(dist, "hero.js"), "utf8");
  assert.match(js, /prefers-reduced-motion: reduce/);
  assert.match(js, /!reduce\.matches/);
  assert.match(js, /document\.hidden/);
  assert.match(js, /IntersectionObserver/);
  assert.match(js, /now - last >= 33/);
});

// hero.js run against a stand-in page: it asks for frames when motion is allowed, and none under reduced motion.
async function run(reduced) {
  const source = await readFile(join(dist, "hero.js"), "utf8");
  const frames = [];
  const el = (extra = {}) => ({ dataset: {}, style: {}, append() {}, getBoundingClientRect: () => ({ left: 0, top: 0, width: 800, height: 600, right: 800, bottom: 600 }), ...extra });
  const ctx = { setTransform() {}, clearRect() {}, fillRect() {} };
  const canvas = el({ offsetWidth: 800, getContext: () => ctx });
  const hero = el({ querySelector: (q) => ({ ".voices": canvas, ".scene .mcard": el(), ".pitch": el() })[q] });
  const page = {
    document: { documentElement: el(), hidden: false, addEventListener() {}, querySelector: (q) => (q === ".hero" ? hero : null), querySelectorAll: () => [], createElement: () => el() },
    matchMedia: () => ({ matches: reduced, addEventListener() {} }),
    getComputedStyle: () => ({ getPropertyValue: () => "#000" }),
    IntersectionObserver: class { observe() {} }, MutationObserver: class { observe() {} },
    requestAnimationFrame: (fn) => frames.push(fn), cancelAnimationFrame() {},
    addEventListener() {}, devicePixelRatio: 1, scrollY: 0,
  };
  page.window = page;
  vm.runInNewContext(source, page);
  return frames;
}

test("the spectrogram's loop starts when motion is allowed and never under reduced motion", async () => {
  assert.equal((await run(false)).length, 1, "the loop never started");
  assert.equal((await run(true)).length, 0, "it moves under reduced motion");
});

test("nothing of the old hero is built: no carousel, no clips, no waves", async () => {
  const files = await readdir(dist);
  for (const gone of ["slider.js", "waves.js", "clips"]) assert.ok(!files.includes(gone), `${gone} is still built`);
});

// #1290 at 419b131: the hero's text column was class « copy », which site.css styles as the install's dark, round
// copy button, so a dark disc sat behind the whole headline in the light theme. The hero shares no class with the
// old page's rules but those named here, each overridden by hero.css under .hero.
const SHARED = { wrap: "the page's container, the same width everywhere", ico: "a pictogram's box, sized under .hero", lead: "the lead's size, margin and width, set again by .hero .lead" };

test("the hero takes no style of the old page by a class they share, but the ones named", async () => {
  const source = await readFile(new URL("../src/components/Landing.astro", import.meta.url), "utf8");
  const hero = source.slice(source.indexOf('<section class="hero"'), source.indexOf("</section>", source.indexOf('<section class="hero"')));
  const classes = new Set([...hero.matchAll(/class=\{?["`]([^"`]+)["`]/g)].flatMap((m) => m[1].replace(/\$\{[^}]*\}/g, "").split(/\s+/)).filter(Boolean));
  for (const name of hero.match(/ico\("[a-z]+"\)/g) ? ["ico"] : []) classes.add(name);
  const css = (await readFile(new URL("../src/styles/site.css", import.meta.url), "utf8")).replace(/\/\*[\s\S]*?\*\//g, "");
  const styled = new Set([...css.matchAll(/([^{}]+)\{/g)].flatMap((m) => [...m[1].matchAll(/\.([a-zA-Z][\w-]*)/g)].map((c) => c[1])));
  const shared = [...classes].filter((c) => styled.has(c));
  assert.deepEqual(shared.filter((c) => !(c in SHARED)), [], "the hero takes the old page's style by these classes");
  const css2 = await readFile(new URL("../src/styles/hero.css", import.meta.url), "utf8");
  for (const c of shared) assert.match(css2, new RegExp(`\\.hero \\.${c}\\b|^\\.${c}\\b`, "m"), `.${c}: ${SHARED[c]}, but hero.css does not set it`);
});
