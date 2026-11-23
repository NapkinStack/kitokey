// D114, criterion 2: /en/privacy/ and /fr/privacy/ are docs/privacy.md's English and French halves,
// the one source; the test reads the built page's text against its half of the source.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { dist } from "./helpers.mjs";
import { privacySource } from "../site.config.mjs";
import { half, words } from "../src/lib/privacy.mjs";

const source = await readFile(privacySource, "utf8");

for (const lang of ["en", "fr"]) {
  test(`/${lang}/privacy/ says its half of the policy, word for word`, async () => {
    const html = await readFile(join(dist, lang, "privacy", "index.html"), "utf8");
    const main = html.match(/<main[^>]*>([\s\S]*)<\/main>/)?.[1];
    assert.ok(main, "no <main> on the page");
    assert.equal(words(main), words(await half(source, lang)));
  });
}
