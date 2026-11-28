// The legal notice (the decider, 2026-11-27): /fr/legal/ and /en/legal/, linked from every page's footer. The
// publisher's details come from the build's environment (Cloudflare Pages), never from the repository; a build
// without them, or with the tests' values on Cloudflare, fails. No person's name: the publication's director is
// named by office unless PUBLICATION_DIRECTOR says otherwise. The host is Cloudflare, Inc., its public details.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { join } from "node:path";
import { dist, root, pages, route } from "./helpers.mjs";

// The publisher's phone, which the LCEN asks (art. 6 III-1; the decider, 2026-11-28), among them.
const FIELDS = ["LEGAL_NAME", "LEGAL_FORM", "LEGAL_CAPITAL", "LEGAL_REGISTRATION", "LEGAL_VAT", "LEGAL_SEAT", "LEGAL_CONTACT", "LEGAL_PHONE"];
const text = (html) => html.replace(/<svg[\s\S]*?<\/svg>/g, "").replace(/<[^>]+>/g, " ").replace(/&amp;/g, "&").replace(/\s+/g, " ");

// Imports src/lib/legal.mjs in a fresh process with `env` alone: what a build on that machine would get.
const loads = (env) => spawnSync(process.execPath, ["--input-type=module", "-e", "await import('./src/lib/legal.mjs')"], { cwd: root, env: { PATH: process.env.PATH, ...env }, encoding: "utf8" });
const real = Object.fromEntries(FIELDS.map((f, i) => [f, `Valeur ${i} de C12C`]));

test("each page's footer links the legal notice in its own language", async () => {
  for (const file of await pages()) {
    if (route(file) === "/") continue;
    const html = await readFile(file, "utf8");
    const lang = html.match(/<html lang="(en|fr)"/)[1];
    const footer = html.match(/<footer[\s\S]*<\/footer>/)?.[0] ?? "";
    assert.match(footer, new RegExp(`<a href="/${lang}/legal/">${lang === "fr" ? "Mentions légales" : "Legal notice"}</a>`), route(file));
  }
});

test("the legal notice says each field, the host, and names the director by office, never a person", async () => {
  for (const [lang, office] of [["fr", "le président de C12C"], ["en", "the President of C12C"]]) {
    const said = text(await readFile(join(dist, lang, "legal", "index.html"), "utf8"));
    for (const f of FIELDS) assert.match(said, new RegExp(`TEST ${f}`), `/${lang}/legal/: ${f} not said`);
    assert.match(said, new RegExp(office), `/${lang}/legal/`);
    assert.match(said, /Cloudflare, Inc\./);
  }
});

test("a build without the publisher's details fails, naming what is missing", () => {
  const got = loads({});
  assert.notEqual(got.status, 0);
  assert.match(got.stderr, /LEGAL_NAME/);
  const one = loads({ ...real, LEGAL_VAT: "" });
  assert.notEqual(one.status, 0);
  assert.match(one.stderr, /LEGAL_VAT/);
  const phone = loads({ ...real, LEGAL_PHONE: "" });
  assert.notEqual(phone.status, 0);
  assert.match(phone.stderr, /LEGAL_PHONE/);
});

test("the tests' values are refused on Cloudflare Pages, a real build's values accepted there", () => {
  const tested = loads({ LEGAL_TEST: "1", CF_PAGES: "1" });
  assert.notEqual(tested.status, 0);
  assert.match(tested.stderr, /test/i);
  const slipped = loads({ ...real, LEGAL_SEAT: "TEST LEGAL_SEAT", CF_PAGES: "1" });
  assert.notEqual(slipped.status, 0);
  assert.equal(loads({ ...real, CF_PAGES: "1" }).status, 0, loads({ ...real, CF_PAGES: "1" }).stderr);
});
