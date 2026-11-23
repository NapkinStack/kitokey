// D114, criterion 3: no advertising, no script, font or image from another domain — every built page
// is opened in a headless Chromium and any request to a host other than the site's fails the test;
// the browser refuses them too, by the Content-Security-Policy in _headers.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { chromium as executablePath, dist, pages, route, serve } from "./helpers.mjs";
import { chromium } from "playwright-core";

test("_headers sets a Content-Security-Policy of the site's own origin", async () => {
  const headers = await readFile(join(dist, "_headers"), "utf8");
  assert.match(headers, /^\/\*\n(\s+.+\n)*\s+Content-Security-Policy: default-src 'self'[;\n]/m);
  assert.doesNotMatch(headers, /unsafe-inline|unsafe-eval|https?:\/\//);
});

test("no page requests another host", async () => {
  const { server, origin } = await serve();
  let browser;
  try {
    browser = await chromium.launch({ executablePath, args: ["--no-sandbox"] });
    const all = await pages();
    assert.ok(all.length > 0, "no page built");
    for (const file of all) {
      const page = await browser.newPage();
      const foreign = [];
      page.on("request", (r) => { if (!r.url().startsWith(origin) && !r.url().startsWith("data:")) foreign.push(r.url()); });
      await page.goto(origin + route(file), { waitUntil: "networkidle" });
      assert.deepEqual(foreign, [], `${route(file)} asked another host`);
      await page.close();
    }
  } finally {
    await browser?.close();
    server.close();
  }
});
