// D114, criterion 3: nothing promised that the product does not keep — every sentence of the site
// names what keeps it.
import { test } from "node:test";
import assert from "node:assert/strict";
import { words } from "../src/lib/words.mjs";

test("every sentence says what keeps its promise, in both languages", () => {
  for (const [key, w] of Object.entries(words)) {
    assert.ok(w.en && w.fr, `${key} lacks a language`);
    assert.ok(w.keeps?.trim(), `${key} names nothing that keeps it`);
  }
});

