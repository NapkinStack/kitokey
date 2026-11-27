// Site v7 (validated by the decider on 2026-11-26): each landing, in French and in English, in the light theme and
// the dark one, judged by axe-core's WCAG 2.2 A and AA rules in the machine's Chromium. #still keeps every group in
// place, so axe reads the page as a visitor sees it once it has scrolled.
import { test } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { chromium as executablePath, serve } from "./helpers.mjs";
import { chromium } from "playwright-core";

// axe-core 4.14.0 from npm (package-lock.json pins its integrity), the release the shell's e2e vendors (D106).
const AXE = new URL(import.meta.resolve("axe-core/axe.min.js"));
const AXE_SHA256 = "20c09fe157a8a34a30e241aaa1fcdade657734f08ab379ecfbeb7d45cc46e878";
const TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"];

test("axe-core is the one pinned by its SHA-256", async () => {
  assert.equal(createHash("sha256").update(await readFile(AXE)).digest("hex"), AXE_SHA256);
});

test("each landing, light and dark, has no WCAG 2.2 A or AA violation axe finds", async () => {
  const source = await readFile(AXE, "utf8");
  const { server, origin } = await serve();
  let browser;
  try {
    browser = await chromium.launch({ executablePath, args: ["--no-sandbox"] });
    for (const lang of ["fr", "en"]) {
      for (const colorScheme of ["light", "dark"]) {
        const page = await browser.newPage({ colorScheme, viewport: { width: 1280, height: 900 } });
        await page.goto(`${origin}/${lang}/#still`, { waitUntil: "networkidle" });
        assert.equal(await page.evaluate(() => document.documentElement.dataset.site), colorScheme, `/${lang}/ not in its ${colorScheme} theme`);
        await page.addScriptTag({ content: source });
        const result = await page.evaluate((tags) => axe.run(document, { runOnly: { type: "tag", values: tags }, resultTypes: ["violations"] })
          .then((r) => ({ passes: r.passes.length, violations: r.violations.map((v) => [v.id, v.impact, v.nodes.slice(0, 3).map((n) => n.target.join(" "))]) })), TAGS);
        assert.ok(result.passes > 0, `/${lang}/ ${colorScheme}: axe judged no rule`);
        assert.deepEqual(result.violations, [], `/${lang}/ ${colorScheme}`);
        await page.close();
      }
    }
  } finally {
    await browser?.close();
    server.close();
  }
});
