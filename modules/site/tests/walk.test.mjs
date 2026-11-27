// Site v7, the walkthrough (validated by the decider on 2026-11-26): record, confidential, the two voices, the minutes,
// as WAI-ARIA's tabs over one drawn scene; four static cards on a narrow screen and under reduced motion; the loop
// 3, 3.5, 5 then 6 s, only while visible; no replay button and no record button in the scene.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import vm from "node:vm";
import { dist } from "./helpers.mjs";
import { landing } from "../src/lib/landing.mjs";

const section = async (lang) => {
  const html = await readFile(join(dist, lang, "index.html"), "utf8");
  return html.match(/<section class="walk-band"[\s\S]*?<\/section>/)?.[0] ?? "";
};

for (const lang of ["fr", "en"]) {
  test(`/${lang}/: four tabs in their order, the first selected, the panel labelled by its tab`, async () => {
    const html = await section(lang);
    const tabs = [...html.matchAll(/<button type="button" class="wtab[^"]*" role="tab" id="wtab-(\d)" aria-selected="(true|false)" aria-controls="wstage" tabindex="(-?\d)"><span class="wn">\d<\/span><b>([^<]+)<\/b>/g)];
    assert.deepEqual(tabs.map((m) => m[4]), landing.walk[lang].tabs.map(([title]) => title));
    assert.deepEqual(tabs.map((m) => [m[2], m[3]]), [["true", "0"], ["false", "-1"], ["false", "-1"], ["false", "-1"]]);
    assert.match(html, /<div class="wstage" id="wstage" role="tabpanel" data-step="1" aria-labelledby="wtab-1">/);
    assert.match(html, new RegExp(`role="tablist" aria-label="${landing.walk[lang].steps}"`));
  });

  test(`/${lang}/: the scene's layers are hidden from a screen reader, the four static cards say the same steps`, async () => {
    const html = await section(lang);
    const stage = html.slice(html.indexOf('id="wstage"'), html.indexOf('<div class="flow">'));
    for (const layer of ["w-bar", "w-lockchip", "w-cloud", "w-lane me", "w-lane ot", "w-flow", "w-note", "w-ban2", "w-call", "w-page"]) {
      assert.match(stage, new RegExp(`class="${layer}[^"]*" aria-hidden="true"`), layer);
    }
    const cards = [...html.matchAll(/<article class="step[^"]*">[\s\S]*?<h3>(?:<svg[\s\S]*?<\/svg>)?([^<]+)<\/h3>\s*<p>([^<]+)<\/p>/g)];
    assert.deepEqual(cards.map((m) => m[1]), landing.walk[lang].tabs.map(([title]) => title));
    assert.deepEqual(cards.map((m) => m[2]), landing.walk[lang].cards);
  });

  test(`/${lang}/: no replay button, no record button, no inline style (the CSP refuses it)`, async () => {
    const html = await section(lang);
    assert.equal([...html.matchAll(/<button\b/g)].length, 4, "a button other than the four tabs");
    assert.doesNotMatch(html, /btn-primary|Revoir|Replay|class="btn\b/);
    assert.doesNotMatch(html, /\sstyle="/);
  });
}

test("the scene gives way to the four cards on a narrow screen and under reduced motion", async () => {
  const css = await readFile(new URL("../src/styles/walk.css", import.meta.url), "utf8");
  assert.match(css, /@media \(max-width: 900px\), \(prefers-reduced-motion: reduce\) \{\s*\.walk \{ display: none; \}\s*\.flow \{ display: grid; \}/);
});

// walk.js run against a stand-in page and a stand-in clock: the steps follow at 3, 3.5, 5 and 6 s once the scene is
// visible; nothing runs before, nor under reduced motion, nor with the tab hidden.
function element(props = {}) {
  const el = { children: [], dataset: {}, attrs: {}, classes: new Set(), style: { setProperty() {} }, listeners: {}, textContent: "", offsetWidth: 1, ...props };
  el.append = (...kids) => el.children.push(...kids);
  el.setAttribute = (k, v) => { el.attrs[k] = v; };
  el.removeAttribute = (k) => { delete el.attrs[k]; };
  el.addEventListener = (type, fn) => { el.listeners[type] = fn; };
  el.classList = { add: (c) => el.classes.add(c), remove: (c) => el.classes.delete(c), toggle: (c, on) => (on ?? !el.classes.has(c)) ? el.classes.add(c) : el.classes.delete(c) };
  el.matches = () => false;
  el.contains = () => false;
  el.focus = () => {};
  return el;
}

async function page({ reduced = false } = {}) {
  const source = await readFile(join(dist, "walk.js"), "utf8");
  const tabs = [1, 2, 3, 4].map((n) => element({ id: `wtab-${n}`, querySelector: () => ({ textContent: `t${n}` }) }));
  const pageEl = element();
  const stage = element({ querySelectorAll: () => [], querySelector: () => pageEl });
  stage.dataset.step = "1";
  const walk = element({ querySelectorAll: () => tabs });
  let now = 0, observer = null, timer = null;
  const world = {
    document: { hidden: false, getElementById: (id) => ({ walk, wstage: stage, wtime: element() })[id], querySelectorAll: () => [], createElement: () => element(), addEventListener() {}, activeElement: null },
    matchMedia: () => ({ matches: reduced, addEventListener() {} }),
    IntersectionObserver: class { constructor(fn) { observer = fn; } observe() {} },
    setInterval: (fn) => { timer = fn; return 1; }, clearInterval: () => { timer = null; },
    Date: { now: () => now },
  };
  vm.runInNewContext(source, world);
  const at = (ms) => { while (now < ms) { now += 125; timer?.(); } };
  return { stage, world, at, show: () => observer([{ isIntersecting: true }]), running: () => timer !== null };
}

test("the loop waits until the scene is visible, then steps at 3, 3.5, 5 and 6 s and starts again", async () => {
  const p = await page();
  assert.equal(p.running(), false, "it runs before the scene is seen");
  p.show();
  assert.equal(p.running(), true);
  const seen = [];
  let last = p.stage.dataset.step;
  for (let t = 0; t <= 18000; t += 125) {
    p.at(t);
    if (p.stage.dataset.step !== last) { last = p.stage.dataset.step; seen.push([t, last]); }
  }
  // Each step is reached within one tick of its time: 3 s, then 3.5, 5 and 6 more, then step 1 again.
  const near = (t, want) => Math.abs(t - want) <= 250;
  assert.deepEqual(seen.map(([, s]) => s), ["2", "3", "4", "1"]);
  for (const [[t], want] of seen.map((s, i) => [s, [3000, 6500, 11500, 17500][i]])) assert.ok(near(t, want), `${t} for ${want}`);
});

test("under reduced motion, or with the tab hidden, the loop does not run", async () => {
  const still = await page({ reduced: true });
  still.show();
  assert.equal(still.running(), false, "it runs under reduced motion");
  const hidden = await page();
  hidden.world.document.hidden = true;
  hidden.show();
  assert.equal(hidden.running(), false, "it runs with the tab hidden");
});

// As for the hero (#1290): the walkthrough shares no class with the old page's rules but these, each set again under
// .walk-band in walk.css.
const SHARED = { wrap: "the page's container", ico: "a pictogram's box", kicker: "the line over the title" };

test("the walkthrough takes no style of the old page by a class they share, but the ones named", async () => {
  const source = await readFile(new URL("../src/components/Landing.astro", import.meta.url), "utf8");
  const start = source.indexOf('<section class="walk-band"');
  const part = source.slice(start, source.indexOf("</section>", start));
  const classes = new Set([...part.matchAll(/class=\{?["`]([^"`]+)["`]/g)].flatMap((m) => m[1].replace(/\$\{[^}]*\}/g, "").split(/\s+/)).filter(Boolean));
  classes.add("ico");
  const css = (await readFile(new URL("../src/styles/site.css", import.meta.url), "utf8")).replace(/\/\*[\s\S]*?\*\//g, "");
  const styled = new Set([...css.matchAll(/([^{}]+)\{/g)].flatMap((m) => [...m[1].matchAll(/\.([a-zA-Z][\w-]*)/g)].map((c) => c[1])));
  const shared = [...classes].filter((c) => styled.has(c));
  assert.deepEqual(shared.filter((c) => !(c in SHARED)), [], "the walkthrough takes the old page's style by these classes");
  const own = await readFile(new URL("../src/styles/walk.css", import.meta.url), "utf8");
  for (const c of shared.filter((c) => c !== "wrap")) assert.match(own, new RegExp(`\\.walk-band \\.${c}\\b`), `.${c}: ${SHARED[c]}, not set again under .walk-band`);
});
