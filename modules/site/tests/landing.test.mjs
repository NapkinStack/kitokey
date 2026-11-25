// D114, the landing (proposal D, validated by the decider on 2026-11-25): /en/ and /fr/ hold every section
// their nav names, nothing left to measure, the install's commands as the module gives them, and a carousel, the
// drawn call then the app filmed (c16), each slide with its poster and its words; the clips c16's alone; the root to French.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile, readdir, stat } from "node:fs/promises";
import { join } from "node:path";
import { dist } from "./helpers.mjs";
import { installSources } from "../site.config.mjs";
import { channels, landing } from "../src/lib/landing.mjs";

const KEYS = ["transcript", "summary", "dates", "send", "confidential"]; // the record slide is drawn

// The READMEs' bash blocks, cut in paragraphs at blank lines, each with the system it is for: the one its
// comments name, else the one its section's heading names, else none — a paragraph for every system.
const SYSTEMS = ["fedora", "ubuntu"];
const named = (text) => SYSTEMS.filter((s) => text.toLowerCase().includes(s));
const blocks = [];
for (const source of installSources) {
  let heading = "";
  for (const [, title, block] of (await readFile(source, "utf8")).matchAll(/^#+ (.*)$|```bash\n([\s\S]*?)```/gm)) {
    if (title !== undefined) { heading = title; continue; }
    blocks.push(block.split(/\n\s*\n/).map((paragraph) => {
      const lines = paragraph.split("\n").filter((l) => l.trim());
      const own = named(lines.map((l) => l.match(/#(.*)$/)?.[1] ?? "").join(" "));
      const systems = own.length ? own : named(heading).length === 1 ? named(heading) : [];
      return { systems, lines: lines.map((l) => l.replace(/\s+#.*$/, "").trim()) };
    }));
  }
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

  test(`/${lang}/: the install's commands, line for line, as a README block gives them for that system`, async () => {
    const html = await page(lang);
    for (const [os, , commands] of channels) {
      const shown = html.match(new RegExp(`<pre class="os-code" data-os="${os}"[^>]*>([\\s\\S]*?)</pre>`))?.[1];
      assert.ok(shown, `no commands for ${os}`);
      assert.equal(text(shown).replace(/^\$ /gm, ""), commands);
      assert.ok(given(os, commands.split("\n")), `${os}: its commands are not a README block's lines for ${os}`);
    }
  });

  test(`/${lang}/: the hero's carousel, the drawn call then five c16 clips in their order, each with its poster and its words`, async () => {
    const html = await page(lang);
    const slides = [...html.matchAll(/<article class="slide"[\s\S]*?<\/article>/g)].map((m) => m[0]);
    assert.equal(slides.length, 6);
    const [call, ...clips] = slides;
    assert.doesNotMatch(call, /data-clip=/);
    assert.ok(call.includes(`role="img" aria-label="${landing.slides[lang][0][5]}"`), "the call is not described");
    assert.equal([...call.matchAll(/<img class="ptr" src="\/img\/call-[a-z]+\.webp"/g)].length, 4);
    assert.deepEqual(clips.map((s) => s.match(/data-clip="\/clips\/([a-z]+)-/)?.[1]), KEYS);
    for (const [i, s] of clips.entries()) {
      const [key, , , , , alt] = landing.slides[lang][i + 1];
      assert.match(s, new RegExp(`<img class="still" src="/clips/${key}-${lang}-light\\.jpg" width="1280" height="720"`));
      assert.ok(s.includes(`alt="${alt}"`), `slide ${key}: its alt is not its words`);
    }
  });
}

test("the waves move for ever but pause on a button, and stand still under reduced motion (WCAG 2.2.2)", async () => {
  const js = await readFile(join(dist, "waves.js"), "utf8");
  assert.match(js, /className = "waves-pause"/);
  assert.match(js, /aria-pressed/);
  assert.match(js, /prefers-reduced-motion: reduce/);
  assert.match(js, /document\.hidden/);
  assert.match(js, /IntersectionObserver/);
});

test("the clips are c16's alone, each in AV1, H.264 and its poster, none over 500 KiB", async () => {
  const want = KEYS.flatMap((k) => ["en", "fr"].flatMap((l) => ["jpg", "mp4", "webm"].map((e) => `${k}-${l}-light.${e}`))).sort();
  const have = (await readdir(join(dist, "clips"))).sort();
  assert.deepEqual(have, want);
  for (const name of have) assert.ok((await stat(join(dist, "clips", name))).size <= 500 * 1024, `${name} is over 500 KiB`);
});

test("the root sends a reader to the French landing, English one link away", async () => {
  const html = await readFile(join(dist, "index.html"), "utf8");
  assert.match(html, /<meta http-equiv="refresh" content="0; url=\/fr\/"/);
  assert.match(html, /href="\/en\/" hreflang="en"/);
});
