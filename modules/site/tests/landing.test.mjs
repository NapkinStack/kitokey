// D114, the landing (proposal D, validated by the decider on 2026-11-25): /en/ and /fr/ hold every section
// their nav names, nothing left to measure, the install's commands as README.md gives them, and a carousel, the
// drawn call then the app filmed (c18), each slide with its poster and its words; the clips c18's alone; the root to French.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile, readdir, stat } from "node:fs/promises";
import { join } from "node:path";
import { createHash } from "node:crypto";
import { dist } from "./helpers.mjs";
import { installSource } from "../site.config.mjs";
import { channels, landing } from "../src/lib/landing.mjs";

const KEYS = ["transcript", "summary", "dates", "send", "confidential"]; // the record slide is drawn

// README.md's bash blocks, cut in paragraphs at blank lines, each with the system it is for: the one its
// comments name, else those the prose just before the block names (« On Fedora 43, 44 and 45 », « On Ubuntu
// 24.04 … »), else every system.
const SYSTEMS = ["fedora", "ubuntu"];
const named = (text) => SYSTEMS.filter((s) => text.toLowerCase().includes(s));
const blocks = [];
for (const [, before, block] of (await readFile(installSource, "utf8")).matchAll(/([^\n]+(?:\n[^\n]+)*)\n+```bash\n([\s\S]*?)```/g)) {
  blocks.push(block.split(/\n\s*\n/).map((paragraph) => {
    const lines = paragraph.split("\n").filter((l) => l.trim());
    const own = named(lines.map((l) => l.match(/#(.*)$/)?.[1] ?? "").join(" "));
    return { systems: own.length ? own : named(before), lines: lines.map((l) => l.replace(/\s+#.*$/, "").trim()) };
  }));
}
// A tab's commands are, line for line, one block's lines meant for its system or for every system. Debian 13
// takes Ubuntu's package (README.md, « Install it »).
const BUILT_AS = { debian: "ubuntu" };
const given = (os, commands) => blocks.some((block) => {
  const lines = block.filter((p) => !p.systems.length || p.systems.includes(BUILT_AS[os] ?? os)).flatMap((p) => p.lines);
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

  test(`/${lang}/: the install's commands, line for line, as a block of README.md gives them for that system`, async () => {
    const html = await page(lang);
    for (const [os, , commands] of channels) {
      const shown = html.match(new RegExp(`<pre class="os-code" data-os="${os}"[^>]*>([\\s\\S]*?)</pre>`))?.[1];
      assert.ok(shown, `no commands for ${os}`);
      assert.equal(text(shown).replace(/^\$ /gm, ""), commands);
      assert.ok(given(os, commands.split("\n")), `${os}: its commands are not a README.md block's lines for ${os}`);
    }
  });

  test(`/${lang}/: the hero's carousel, the drawn call then five c18 clips in their order, each with its poster and its words`, async () => {
    const html = await page(lang);
    const slides = [...html.matchAll(/<article class="slide"[\s\S]*?<\/article>/g)].map((m) => m[0]);
    assert.equal(slides.length, 6);
    const [call, ...clips] = slides;
    assert.doesNotMatch(call, /data-clip=/);
    assert.ok(call.includes(`role="img" aria-label="${landing.slides[lang][0][5]}"`), "the call is not described");
    const portraits = [...call.matchAll(/<img class="ptr" src="(\/img\/call-[a-z]+\.webp)"/g)].map((m) => m[1]);
    assert.equal(portraits.length, 4);
    for (const src of portraits) assert.ok((await stat(join(dist, src))).size > 0, `${src} not built`);
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

// c18's files as encoded from run i-sam-film18c-17618c9 (main 17618c9), their sha256 each: another capture's clip,
// even of the same name, fails here.
const C18 = Object.fromEntries((await readFile(new URL("./clips-c18.sha256", import.meta.url), "utf8")).trim().split("\n").map((l) => l.split(/\s+/).reverse()));

test("the clips are c18's alone, light and dark for each slide, each in AV1, H.264 and its poster, none over 500 KiB", async () => {
  const set = (theme, keys) => keys.flatMap((k) => ["en", "fr"].flatMap((l) => ["jpg", "mp4", "webm"].map((e) => `${k}-${l}-${theme}.${e}`)));
  const have = (await readdir(join(dist, "clips"))).sort();
  assert.deepEqual(have, [...set("light", KEYS), ...set("dark", KEYS)].sort());
  assert.deepEqual(Object.keys(C18).sort(), have);
  for (const name of have) {
    const bytes = await readFile(join(dist, "clips", name));
    assert.ok(bytes.length <= 500 * 1024, `${name} is over 500 KiB`);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), C18[name], `${name} is not c18's`);
  }
  for (const lang of ["en", "fr"]) {
    const html = await page(lang);
    for (const k of KEYS) assert.match(html, new RegExp(`data-clip="/clips/${k}-${lang}" data-dark`), `${lang} ${k}: the moon shows no dark clip`);
  }
});

test("the root sends a reader to the French landing, English one link away", async () => {
  const html = await readFile(join(dist, "index.html"), "utf8");
  assert.match(html, /<meta http-equiv="refresh" content="0; url=\/fr\/"/);
  assert.match(html, /href="\/en\/" hreflang="en"/);
});

// The channels' brands as their owners allow (the decider, 2026-11-26; BRANDS.md): Slack in words,
// its terms forbidding to distribute its logo without a written licence; Discord's symbol, its own file unaltered.
test("Slack is named in words alone, Discord by its own symbol, unaltered", async () => {
  const { createHash } = await import("node:crypto");
  const symbol = await readFile(join(dist, "img", "brands", "discord-symbol-blurple.svg"));
  assert.equal(createHash("sha256").update(symbol).digest("hex"), "d06eaa51e78a9984ebbd3fbc82f0245ac2ecb5bf0150f537204397c1da5c7bd8");
  for (const lang of ["en", "fr"]) {
    const html = await page(lang);
    const colours = ["#E01E5A", "#36C5F0", "#2EB67D", "#ECB22E"].filter((c) => html.toUpperCase().includes(c));
    assert.ok(colours.length < 2, `/${lang}/ draws Slack's colours together: ${colours}`);
    const logos = html.match(/<ul class="ch-logos"[^>]*>([\s\S]*?)<\/ul>/)?.[1] ?? "";
    assert.match(logos, /<li><span>Slack<\/span><\/li>/, `/${lang}/: Slack is not in words alone`);
    assert.match(logos, /<li><img class="os-logo" src="\/img\/brands\/discord-symbol-blurple\.svg"[^>]*><span>Discord<\/span><\/li>/, `/${lang}/: Discord not by its own symbol`);
  }
});

test("each hardware card says, in both languages, the figures README.md gives its way of working, and only those", async () => {
  const readme = await readFile(installSource, "utf8");
  const section = readme.slice(readme.indexOf("**What a computer needs**"));
  const way = (title) => section.match(new RegExp(`- \\*\\*${title}\\*\\*:([\\s\\S]*?)(?=\\n- \\*\\*|\\n\\n)`))?.[1] ?? "";
  const figures = (text) => [...text.matchAll(/\d+(?:[.,]\d+)?/g)].map((m) => m[0].replace(",", ".")).sort();
  const ways = [way("Everything on the computer"), way("With the service you choose")];
  assert.ok(ways.every(Boolean), "README.md names no such way");
  for (const lang of ["en", "fr"]) {
    landing.hardware[lang].forEach(([, title, , lines], i) => {
      assert.deepEqual(figures(lines.join(" ")), figures(ways[i]), `${lang}, « ${title} »: its figures are not README.md's`);
      assert.equal(/vulkan/i.test(lines.join(" ")), /vulkan/i.test(ways[i]), `${lang}, « ${title} »: Vulkan said or not as README.md does`);
    });
  }
});
