// D114, the frame every page shares (proposal D, validated by the decider on 2026-11-25): a skip link to
// the content, the brand, the other language, a footer naming the source and the policy; the titles' face
// shipped with its licence; nothing of the decider's review tool left in the build.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import { join } from "node:path";
import { dist, pages, route } from "./helpers.mjs";

test("each page skips to its content, offers the other language and names the source and the policy", async () => {
  const all = await pages();
  assert.ok(all.length > 0, "no page built");
  for (const file of all) {
    if (route(file) === "/") continue; // the root, the two policies' links until the landing
    const html = await readFile(file, "utf8");
    const lang = html.match(/<html lang="(en|fr)"/)?.[1];
    const other = lang === "en" ? "fr" : "en";
    assert.match(html, /<a class="skip" href="#main">/, `${route(file)}: no skip link`);
    assert.match(html, /<main id="main"/, `${route(file)}: no #main`);
    assert.match(html, new RegExp(`<a class="lang" href="/${other}/[^"]*" hreflang="${other}">[\\s\\S]*?<span lang="${other}">`), `${route(file)}: no link to ${other}`);
    const footer = html.match(/<footer[\s\S]*<\/footer>/)?.[0] ?? "";
    assert.match(footer, /href="https:\/\/github\.com\/NapkinStack\/kitokey"/, `${route(file)}: the footer names no source`);
    assert.match(footer, new RegExp(`href="/${lang}/privacy/"`), `${route(file)}: the footer names no policy`);
  }
});

test("each page offers the moon and the sun, named, hidden until its script runs, the theme set before the page paints", async () => {
  for (const file of await pages()) {
    if (route(file) === "/") continue;
    const html = await readFile(file, "utf8");
    const toggle = html.match(/<button type="button" class="theme-toggle"[^>]*>/)?.[0];
    assert.ok(toggle, `${route(file)}: no theme toggle`);
    assert.match(toggle, /aria-pressed="false"/);
    assert.match(toggle, /data-dark="[^"]+" data-light="[^"]+" aria-label="[^"]+" hidden/);
    const head = html.slice(0, html.indexOf("</head>"));
    assert.match(head, /<script src="\/theme\.js"><\/script>/, `${route(file)}: theme.js not in <head>, or deferred`);
  }
});

test("the titles' face ships with its licence", async () => {
  const fonts = await readdir(join(dist, "fonts"));
  assert.ok(fonts.includes("bricolage-latin.woff2"), "no face");
  assert.match(await readFile(join(dist, "fonts", "OFL-Bricolage.txt"), "utf8"), /SIL Open Font License, Version 1\.1/);
});

test("nothing of the review tool is built", async () => {
  async function files(dir) {
    const out = [];
    for (const e of await readdir(dir, { withFileTypes: true })) {
      const path = join(dir, e.name);
      if (e.isDirectory()) out.push(...(await files(path)));
      else if (/\.(html|css|js)$/.test(e.name)) out.push(path);
    }
    return out;
  }
  for (const file of await files(dist)) {
    const text = await readFile(file, "utf8");
    assert.doesNotMatch(text, /review-bar|kitokey-review-|data-screens|[?&](site|screens|bg|slide)=/, `${file.slice(dist.length)} keeps the review tool`);
  }
});

test("the skip link hides off screen and shows at the top once focused", async () => {
  const html = await readFile(join(dist, "en", "privacy", "index.html"), "utf8");
  let css = "";
  for (const [, href] of html.matchAll(/<link rel="stylesheet" href="([^"]+)"/g)) css += await readFile(join(dist, href), "utf8");
  const rule = (selector) => css.match(new RegExp(`(?:^|})${selector.replace(/[.:]/g, "\\$&")}\\{([^}]*)\\}`))?.[1] ?? "";
  const top = (decls) => decls.match(/(?:^|;)top:(-?[\d.]+)(?:px)?(?:;|$)/)?.[1];
  assert.ok(Number(top(rule(".skip"))) < 0, `.skip is not off screen: ${rule(".skip")}`);
  assert.ok(Number(top(rule(".skip:focus"))) >= 0, `.skip:focus does not bring it back: ${rule(".skip:focus") || "no rule"}`);
});
