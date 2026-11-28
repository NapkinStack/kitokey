// Site v7, the base (D114, the decider's rule of 2026-11-26): a brand's mark is its owner's own file, unaltered,
// listed in BRANDS.md with its source and its sha256; none is drawn by the site.
import { test } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, readdir } from "node:fs/promises";
import { join } from "node:path";
import { root, dist, pages } from "./helpers.mjs";
import { ICONS } from "../src/lib/icons.mjs";

const listed = async () =>
  [...(await readFile(join(root, "BRANDS.md"), "utf8")).matchAll(/^\| `([^`]+)` \| (.+?) \| `([0-9a-f]{64})` \|(?: .+? \|)?$/gm)].map(
    ([, file, source, sha]) => ({ file, source, sha }));

test("each brand file is BRANDS.md's, byte for byte, with its source, and BRANDS.md lists every one", async () => {
  const rows = await listed();
  const files = (await readdir(join(root, "public", "img", "brands"))).sort();
  assert.deepEqual(rows.map((r) => r.file).sort(), files);
  for (const { file, source, sha } of rows) {
    assert.match(source, /https:\/\//, `${file}: no source`);
    const bytes = await readFile(join(root, "public", "img", "brands", file));
    assert.equal(createHash("sha256").update(bytes).digest("hex"), sha, file);
  }
});

test("no brand is drawn by the site: the pictograms hold no system's or channel's mark", async () => {
  const marks = ["apple", "windows", "linux", "tux", "fedora", "ubuntu", "debian", "slack", "discord", "google"];
  for (const name of Object.keys(ICONS)) assert.ok(!marks.includes(name), `icons.mjs draws ${name}`);
  for (const file of await pages()) {
    const html = await readFile(file, "utf8");
    for (const [, id] of html.matchAll(/<symbol id="i-([a-z-]+)"/g)) assert.ok(!marks.includes(id), `${file}: draws ${id}`);
    for (const [, src] of html.matchAll(/<img[^>]+src="\/img\/brands\/([^"]+)"/g)) {
      assert.ok((await listed()).some((r) => r.file === src), `${file}: ${src} is not in BRANDS.md`);
    }
  }
});

test("each system is shown by its owner's own mark: Tux, Windows, Apple's black and its white for the dark theme", async () => {
  for (const lang of ["en", "fr"]) {
    const html = await readFile(join(dist, lang, "index.html"), "utf8");
    for (const mark of ["tux.svg", "windows-logo-2021.svg", "apple-logo-black.svg", "apple-logo-white.svg"]) {
      assert.ok(html.includes(`src="/img/brands/${mark}"`), `/${lang}/: ${mark}`);
    }
  }
});
