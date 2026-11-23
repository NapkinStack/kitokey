// D114, criterion 3: nothing promised that the product does not keep — every sentence of the site
// names what keeps it; no price before the decider announces one.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { words } from "../src/lib/words.mjs";
import { dist } from "./helpers.mjs";

test("every sentence says what keeps its promise, in both languages", () => {
  for (const [key, w] of Object.entries(words)) {
    assert.ok(w.en && w.fr, `${key} lacks a language`);
    assert.ok(w.keeps?.trim(), `${key} names nothing that keeps it`);
  }
});

test("no price is said before the decider announces it", async () => {
  for (const lang of ["en", "fr"]) {
    const html = await readFile(join(dist, lang, "index.html"), "utf8");
    assert.doesNotMatch(html, /\d\s?(€|EUR|\$|USD)|(€|\$)\s?\d/, `/${lang}/ says a price`);
  }
});
