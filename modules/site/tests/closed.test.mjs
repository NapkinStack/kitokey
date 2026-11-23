// D114, criterion 6: closed until the decider opens it — robots.txt refuses every robot, every page
// asks not to be indexed. The opening removes both in one commit that changes this test with them.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { dist, pages, route } from "./helpers.mjs";

test("robots.txt refuses every robot", async () => {
  const robots = await readFile(join(dist, "robots.txt"), "utf8");
  assert.match(robots, /^User-agent: \*\s*\nDisallow: \/\s*$/m);
});

test("every page asks not to be indexed", async () => {
  const all = await pages();
  assert.ok(all.length > 0, "no page built");
  for (const file of all) {
    const html = await readFile(file, "utf8");
    assert.match(html, /<meta name="robots" content="noindex"/, `${route(file)} is indexable`);
  }
});
