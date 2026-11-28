// Site v7, the coordinator's audit of 2026-11-27 and the decider's answers that day: what the built pages hold.
// The price in dollars in English (PDR-0009, amended in #1311); the policy linked from the footer alone; the
// hero's note with v7's sparkles; the app's own page in place of the capture's placeholder; the systems' logos
// only where their owners allow it, each credited (BRANDS.md); the titles' face with its optical axis.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import { existsSync } from "node:fs";
import { brotliDecompressSync } from "node:zlib";
import { join } from "node:path";
import { dist, root, pages, route } from "./helpers.mjs";

const page = (lang) => readFile(join(dist, lang, "index.html"), "utf8");
const text = (html) => html.replace(/<svg[\s\S]*?<\/svg>/g, "").replace(/<[^>]+>/g, " ").replace(/&nbsp;| | /g, " ").replace(/\s+/g, " ");
const section = (html, id) => html.match(new RegExp(`<section[^>]*id="${id}"[\\s\\S]*?</section>`))?.[0] ?? "";

test("the price is said in dollars in English, in euros in French, wherever it is said", async () => {
  const en = text(await page("en"));
  const fr = text(await page("fr"));
  assert.match(en, /Free trial, then \$69 once\./);
  assert.equal((en.match(/\$69/g) ?? []).length, 3, "the hero, the card and the last call");
  // The price alone: the made-up meetings quote their own sums (« €18,400 »), never Kitokey's price.
  assert.doesNotMatch(en, /€ ?69|69 ?€/);
  assert.equal((fr.match(/69 €/g) ?? []).length, 3);
  assert.doesNotMatch(fr, /\$ ?69|69 ?\$/);
});

test("the privacy section links no policy: the footer does", async () => {
  for (const [lang, id] of [["fr", "confidentialite"], ["en", "privacy"]]) {
    const html = await page(lang);
    const privacy = section(html, id);
    assert.ok(privacy, `/${lang}/: no #${id}`);
    assert.doesNotMatch(privacy, new RegExp(`href="/${lang}/privacy/"`));
    assert.match(html.match(/<footer[\s\S]*<\/footer>/)[0], new RegExp(`href="/${lang}/privacy/"`));
  }
});

test("the hero's note begins with v7's sparkles", async () => {
  for (const lang of ["fr", "en"]) {
    const prov = (await page(lang)).match(/<article class="mcard"[\s\S]*?<div class="prov">([\s\S]*?)<\/div>/)?.[1] ?? "";
    assert.match(prov, /^<svg[^>]*><use href="#i-sparkles"\/><\/svg>/, `/${lang}/: no sparkles before « ${text(prov).trim()} »`);
  }
});

test("the app's page stands where its placeholder was: the Store's screenshot in the page's language, lazy, sized, described", async () => {
  for (const lang of ["fr", "en"]) {
    const html = await page(lang);
    assert.doesNotMatch(html, /class="shot-ph"/);
    const shots = [...html.matchAll(/<img class="shot"[^>]*>/g)].map(([tag]) => ({ tag, src: tag.match(/src="([^"]+)"/)?.[1], alt: tag.match(/alt="([^"]*)"/)?.[1] }));
    assert.equal(shots.length, 1, `/${lang}/`);
    const [shot] = shots;
    assert.equal(shot.src, `/img/app/meeting-${lang}.webp`);
    assert.ok(existsSync(join(dist, shot.src)), `${shot.src} not built`);
    assert.match(shot.tag, /loading="lazy"/);
    assert.match(shot.tag, /width="1920" height="1080"/);
    assert.ok(shot.alt && shot.alt.length > 20, `/${lang}/: alt « ${shot.alt} »`);
  }
});

test("Fedora, Ubuntu and Debian: their logo in their tab, decorative; Fedora's and Debian's projects linked in their panel", async () => {
  for (const lang of ["fr", "en"]) {
    const html = await page(lang);
    const tab = (os) => html.match(new RegExp(`<button type="button" class="os-tab" data-os="${os}"[\\s\\S]*?</button>`))?.[0] ?? "";
    for (const [os, logo, site] of [["fedora", "fedora-icon", "https://fedoraproject.org/"], ["debian", "debian-openlogo", "https://www.debian.org/"]]) {
      assert.match(tab(os), new RegExp(`<img[^>]*src="/img/brands/${logo}[^"]*\\.svg"[^>]*alt=""`), `/${lang}/: ${os}'s tab has no logo`);
      assert.match(text(tab(os)), new RegExp(os === "fedora" ? "Fedora" : "Debian"), `/${lang}/: ${os}'s tab lost its name`);
      assert.doesNotMatch(tab(os), /<a /, `/${lang}/: a link in ${os}'s tab`);
      const panel = html.match(new RegExp(`<p class="os-note" data-os="${os}"[\\s\\S]*?</p>`))?.[0] ?? "";
      assert.match(panel, new RegExp(`<a href="${site.replace(/[.]/g, "\\.")}"`), `/${lang}/: ${os}'s panel does not link ${site}`);
    }
    assert.match(tab("ubuntu"), /<img[^>]*src="\/img\/brands\/ubuntu-cof-white\.svg"[^>]*alt=""/, `/${lang}/: Ubuntu's tab has no logo`);
  }
});

test("each page's footer says whose marks Fedora and Debian are, and credits Debian's logo", async () => {
  for (const file of await pages()) {
    if (route(file) === "/") continue;
    const footer = text((await readFile(file, "utf8")).match(/<footer[\s\S]*<\/footer>/)?.[0] ?? "");
    assert.match(footer, /Fedora is a trademark or registered trademark of Red Hat, Inc\., or its subsidiaries in the United States and other countries\./, route(file));
    assert.match(footer, /Debian is a registered trademark owned by Software in the Public Interest, Inc\./, route(file));
    assert.match(footer, /Debian Open Use Logo © 1999 Software in the Public Interest, Inc\., CC BY-SA 3\.0/, route(file));
  }
});

test("BRANDS.md names each brand's file, its source, its sha256, the day it was read and whom it is credited to", async () => {
  const brands = await readFile(join(root, "BRANDS.md"), "utf8");
  const header = brands.split("\n").find((l) => l.startsWith("| File"));
  assert.match(header, /\| File \| Source \| sha256 \| Read, attribution \|/);
  for (const file of await readdir(join(root, "public/img/brands"))) {
    const row = brands.split("\n").find((l) => l.startsWith(`| \`${file}\``));
    assert.ok(row, `${file} not in BRANDS.md`);
    assert.match(row.split("|")[4] ?? "", /^ 2026-\d\d-\d\d; \S/, `${file}: no day read, no attribution`);
  }
});

// WOFF2 (W3C, §5): a table directory, then one Brotli stream of the tables; `fvar` is never transformed.
function axes(woff2) {
  const tags = ["cmap", "head", "hhea", "hmtx", "maxp", "name", "OS/2", "post", "cvt ", "fpgm", "glyf", "loca", "prep", "CFF ", "VORG", "EBDT", "EBLC", "gasp", "hdmx", "kern", "LTSH", "PCLT", "VDMX", "vhea", "vmtx", "BASE", "GDEF", "GPOS", "GSUB", "EBSC", "JSTF", "MATH", "CBDT", "CBLC", "COLR", "CPAL", "SVG ", "sbix", "acnt", "avar", "bdat", "bloc", "bsln", "cvar", "fdsc", "feat", "fmtx", "fvar", "gvar", "hsty", "just", "lcar", "mort", "morx", "opbd", "prop", "trak", "Zapf", "Silf", "Glat", "Gloc", "Feat", "Sill"];
  const count = woff2.readUInt16BE(12);
  let at = 48;
  const base128 = () => { let v = 0; for (;;) { const b = woff2[at++]; v = (v << 7) | (b & 0x7f); if (!(b & 0x80)) return v; } };
  const tables = [];
  for (let i = 0; i < count; i++) {
    const flags = woff2[at++];
    let tag = tags[flags & 0x3f];
    if ((flags & 0x3f) === 0x3f) { tag = woff2.toString("latin1", at, at + 4); at += 4; }
    const length = base128();
    const version = flags >> 6;
    const transformed = (tag === "glyf" || tag === "loca") ? version === 0 : (tag === "hmtx" ? version === 1 : false);
    tables.push({ tag, length: transformed ? base128() : length });
  }
  const data = brotliDecompressSync(woff2.subarray(at, at + woff2.readUInt32BE(20)));
  let offset = 0;
  for (const { tag, length } of tables) {
    if (tag === "fvar") {
      const fvar = data.subarray(offset, offset + length);
      const [start, n, size] = [fvar.readUInt16BE(4), fvar.readUInt16BE(8), fvar.readUInt16BE(10)];
      return Array.from({ length: n }, (_, i) => fvar.toString("latin1", start + i * size, start + i * size + 4));
    }
    offset += length;
  }
  return [];
}

test("the titles' face keeps Bricolage's optical axis, as v7 loads it: opsz and wght", async () => {
  const woff2 = await readFile(join(root, "public/fonts/bricolage-latin.woff2"));
  assert.deepEqual(axes(woff2).sort(), ["opsz", "wght"]);
});

// A compound word never breaks at its hyphen (the coordinator, 2026-11-28: « revenez- / y »), and stays the word one
// searches, copies and the titles' face draws: the ASCII hyphen kept, the word wrapped in <span class="nb">
// (white-space: nowrap) by one function at build time, in every page's text but code.
const body = (html) => html.slice(html.indexOf("<body")).replace(/<(pre|code|script|style|svg|textarea)[\s\S]*?<\/\1>/g, "");
test("every compound word of a page's text is kept whole, its hyphen the ASCII one", async () => {
  for (const file of await pages()) {
    const html = body(await readFile(file, "utf8"));
    assert.doesNotMatch(html, /‑/, `${route(file)}: a non-breaking hyphen, which no one searches nor types`);
    const loose = html.replace(/<span class="nb">[^<]*<\/span>/g, " ").replace(/<[^>]+>/g, " ")
      .match(/[\p{L}\p{N}’]+(?:-[\p{L}\p{N}’]+)+/gu) ?? [];
    assert.deepEqual(loose, [], `${route(file)}: compound words left to break`);
  }
});
