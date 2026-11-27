// Site v7, the hero (validated by the decider on 2026-11-26): PDR-0009's promise as the h1, its price and trial, the
// Store's words until its listing exists, Linux free, the reassurance the app keeps, a drawn scene described to a
// screen reader, and the two voices' spectrogram, decorative, still under reduced motion.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import { join } from "node:path";
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

  test(`/${lang}/: the price is PDR-0009's, 69 € once after a free trial; no Store badge before its listing; Linux free`, async () => {
    const html = await hero(lang);
    const [line, store, linux, mac] = landing.buy[lang];
    assert.equal(line, lang === "fr" ? "Essai gratuit, puis 69 € une fois." : "Free trial, then €69 once.");
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

test("nothing of the old hero is built: no carousel, no clips, no waves", async () => {
  const files = await readdir(dist);
  for (const gone of ["slider.js", "waves.js", "clips"]) assert.ok(!files.includes(gone), `${gone} is still built`);
});
