// D114, criterion 1: everything lives in this module; one path leaves it, the policy's, in
// site.config.mjs alone — so that `git subtree split --prefix=modules/site` builds alone.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile, readdir, stat } from "node:fs/promises";
import { join, relative } from "node:path";
import { root } from "./helpers.mjs";

const skip = new Set(["node_modules", "dist", ".astro", ".lighthouseci", "site.config.mjs", "package-lock.json"]);

async function files(dir) {
  const out = [];
  for (const name of await readdir(dir)) {
    if (skip.has(name)) continue;
    const path = join(dir, name);
    if ((await stat(path)).isDirectory()) out.push(...(await files(path)));
    else if (/\.(astro|mjs|js|ts|json|css)$/.test(name)) out.push(path);
  }
  return out;
}

test("no file but site.config.mjs reaches out of the module", async () => {
  const out = [];
  for (const file of await files(root)) {
    for (const [, path] of (await readFile(file, "utf8")).matchAll(/["'`]((?:\.\.\/)+[^"'`]*)["'`]/g)) {
      const depth = relative(root, join(file, "..", path));
      if (depth.startsWith("..")) out.push(`${relative(root, file)}: ${path}`);
    }
  }
  assert.deepEqual(out, []);
});
