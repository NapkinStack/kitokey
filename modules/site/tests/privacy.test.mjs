// D114, criterion 2: /en/privacy/ and /fr/privacy/ are docs/privacy.md's English and French halves,
// the one source; the test reads the built page's text against its half of the source.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { dist } from "./helpers.mjs";
import { privacySource } from "../site.config.mjs";
import { changed, half, words } from "../src/lib/privacy.mjs";

const source = await readFile(privacySource, "utf8");

for (const lang of ["en", "fr"]) {
  test(`/${lang}/privacy/ says its half of the policy, word for word`, async () => {
    const html = await readFile(join(dist, lang, "privacy", "index.html"), "utf8");
    const policy = html.match(/<article class="policy"[^>]*>([\s\S]*)<\/article>/)?.[1];
    assert.ok(policy, "no policy on the page");
    assert.equal(words(policy), words(await half(source, lang)));
  });
}

for (const lang of ["en", "fr"]) {
  test(`/${lang}/privacy/ says the day the policy last changed, and no « file's history »`, async () => {
    const html = await readFile(join(dist, lang, "privacy", "index.html"), "utf8");
    assert.match(html, new RegExp(`<time datetime="${changed(source)}">`));
    assert.doesNotMatch(html, /this file's history|historique de ce fichier/);
  });
}
