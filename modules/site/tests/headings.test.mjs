// WCAG 1.3.1 and 2.4.6: each page has one h1, and its headings go down one level at a time.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { pages, route } from "./helpers.mjs";

test("each page has one h1 and no heading skips a level", async () => {
  for (const file of await pages()) {
    const levels = [...(await readFile(file, "utf8")).matchAll(/<h([1-6])[\s>]/g)].map((m) => Number(m[1]));
    assert.equal(levels.filter((l) => l === 1).length, 1, `${route(file)}: ${levels.filter((l) => l === 1).length} h1`);
    assert.equal(levels[0], 1, `${route(file)} starts at h${levels[0]}`);
    levels.forEach((l, i) => i && assert.ok(l <= levels[i - 1] + 1, `${route(file)}: h${levels[i - 1]} then h${l}`));
  }
});
