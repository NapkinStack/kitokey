// Site v7 (validated by the decider on 2026-11-26), the coordinator's audit of 2026-11-27 against it
// (kitokey-team/site-audit-v7.md): what a visitor sees at 1440 px, measured in the machine's Chromium.
// Its values are v7's own CSS: the kickers 14 px, bold, in the accent (v7 l. 177); every section on one grid;
// the dark theme's grounds (v7 l. 48-50); the price's two columns 1.2fr / 1fr (v7 l. 526, the rule that
// applies); the walk's titles in Bricolage at their optical size (v7 loads opsz 12..96), its first step's track seen.
import { test } from "node:test";
import assert from "node:assert/strict";
import { chromium as executablePath, serve } from "./helpers.mjs";
import { chromium } from "playwright-core";

// v7 l. 48-50, the dark theme's grounds and lines.
const DARK = { "--bg-canvas": "#0F1115", "--bg-surface": "#171A21", "--bg-sunken": "#12141A", "--bg-raised": "#1E222B",
  "--bg-hover": "#252A35", "--bg-pressed": "#2E3441", "--bg-tint": "#1B2540", "--border-subtle": "#2A2F3A", "--border-control": "#717A8C" };

async function look(fn) {
  const { server, origin } = await serve();
  let browser;
  try {
    browser = await chromium.launch({ executablePath, args: ["--no-sandbox"] });
    return await fn(browser, origin);
  } finally {
    await browser?.close();
    server.close();
  }
}

async function open(browser, origin, lang, colorScheme) {
  const page = await browser.newPage({ colorScheme, viewport: { width: 1440, height: 900 } });
  await page.goto(`${origin}/${lang}/#still`, { waitUntil: "networkidle" });
  await page.evaluate(() => document.fonts.ready);
  return page;
}

test("each section's kicker is v7's: 14 px, bold, in the accent, in both themes and languages", async () => {
  const found = [];
  await look(async (browser, origin) => {
    for (const lang of ["fr", "en"]) {
      for (const scheme of ["light", "dark"]) {
        const page = await open(browser, origin, lang, scheme);
        const kickers = await page.evaluate(() => {
          const accent = document.createElement("i");
          accent.style.color = "var(--accent)";
          document.body.append(accent);
          const want = getComputedStyle(accent).color;
          accent.remove();
          return [...document.querySelectorAll("main .kicker")].map((k) => {
            const s = getComputedStyle(k);
            return [k.textContent.trim(), s.color === want, s.fontWeight, s.fontSize];
          });
        });
        assert.ok(kickers.length >= 6, `/${lang}/ ${scheme}: ${kickers.length} kickers`);
        for (const [said, accent, weight, size] of kickers) {
          if (!accent || weight !== "700" || size !== "14px") found.push([`/${lang}/ ${scheme}`, said, accent, weight, size]);
        }
        await page.close();
      }
    }
  });
  assert.deepEqual(found, []);
});

test("« Installer » stands on the grid of every other section", async () => {
  await look(async (browser, origin) => {
    for (const lang of ["fr", "en"]) {
      const page = await open(browser, origin, lang, "light");
      const lefts = await page.evaluate(() => [...document.querySelectorAll("main section .sh")].map((sh) =>
        [sh.closest("section").id || sh.closest("section").className, Math.round(sh.getBoundingClientRect().left)]));
      const install = lefts.find(([id]) => /^(install|installer)$/.test(id));
      assert.ok(install, `/${lang}/: no « Installer » heading`);
      for (const [id, left] of lefts) assert.equal(left, install[1], `/${lang}/: ${id} at ${left} px, « Installer » at ${install[1]} px`);
      await page.close();
    }
  });
});

test("the dark theme's grounds are v7's, its bands still told from the canvas", async () => {
  await look(async (browser, origin) => {
    const page = await open(browser, origin, "fr", "dark");
    const got = await page.evaluate((names) => {
      const s = getComputedStyle(document.documentElement);
      return Object.fromEntries(names.map((n) => [n, s.getPropertyValue(n).trim().toUpperCase()]));
    }, Object.keys(DARK));
    assert.deepEqual(got, DARK);
    const grounds = await page.evaluate(() => [getComputedStyle(document.body).backgroundColor,
      ...[...document.querySelectorAll("main section.mid-band, main section.end-band, main section.walk-band")].map((s) => getComputedStyle(s).backgroundColor)]);
    assert.equal(grounds[0], "rgb(15, 17, 21)");
    for (const band of grounds.slice(1)) assert.notEqual(band, grounds[0], "a band no longer told from the canvas");
    await page.close();
  });
});

test("the price's two cards share 1440 px as v7 does, 1.2fr and 1fr", async () => {
  await look(async (browser, origin) => {
    for (const lang of ["fr", "en"]) {
      const page = await open(browser, origin, lang, "light");
      const [main, linux] = await page.evaluate(() => [".pcard.main", ".pcard.linux"].map((c) => Math.round(document.querySelector(c).getBoundingClientRect().width)));
      assert.ok(Math.abs(main - 609) <= 2 && Math.abs(linux - 508) <= 2, `/${lang}/: ${main} / ${linux} px`);
      await page.close();
    }
  });
});

test("the walk's titles at their optical size: « Les deux voix se transcrivent » on two lines, as in v7", async () => {
  await look(async (browser, origin) => {
    const page = await open(browser, origin, "fr", "light");
    const lines = await page.evaluate(() => {
      const b = document.querySelector("#wtab-3 b");
      return Math.round(b.getBoundingClientRect().height / parseFloat(getComputedStyle(b).lineHeight));
    });
    assert.equal(lines, 2);
    await page.close();
  });
});

test("the first step's track is seen against its tab, in both themes", async () => {
  await look(async (browser, origin) => {
    for (const scheme of ["light", "dark"]) {
      const page = await open(browser, origin, "fr", scheme);
      const ratio = await page.evaluate(() => {
        const lum = (c) => {
          const [r, g, b] = c.match(/\d+(\.\d+)?/g).slice(0, 3).map(Number).map((v) => v / 255).map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
          return 0.2126 * r + 0.7152 * g + 0.0722 * b;
        };
        const track = getComputedStyle(document.querySelector("#wtab-1 .pg")).backgroundColor;
        const tab = getComputedStyle(document.querySelector("#wtab-1")).backgroundColor;
        const [a, b] = [lum(track), lum(tab)].sort((x, y) => y - x);
        return (a + 0.05) / (b + 0.05);
      });
      assert.ok(ratio >= 1.2, `${scheme}: the track at ${ratio.toFixed(2)}:1 against its tab`);
      await page.close();
    }
  });
});

test("in the dark theme, the install block stands out of the canvas: its edge at 3:1 at least", async () => {
  await look(async (browser, origin) => {
    const page = await open(browser, origin, "fr", "dark");
    const ratio = await page.evaluate(() => {
      const lum = (c) => {
        const [r, g, b] = c.match(/\d+(\.\d+)?/g).slice(0, 3).map(Number).map((v) => v / 255).map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
        return 0.2126 * r + 0.7152 * g + 0.0722 * b;
      };
      const code = getComputedStyle(document.querySelector(".get .code"));
      // An edge that is drawn: an outline or a border at least 1 px wide. With neither, a border's colour is the text's
      // (currentColor) and measures nothing: no edge, 1:1.
      const edge = code.outlineStyle !== "none" && parseFloat(code.outlineWidth) >= 1 ? code.outlineColor
        : code.borderTopStyle !== "none" && parseFloat(code.borderTopWidth) >= 1 ? code.borderTopColor : getComputedStyle(document.body).backgroundColor;
      const [a, b] = [lum(edge), lum(getComputedStyle(document.body).backgroundColor)].sort((x, y) => y - x);
      return (a + 0.05) / (b + 0.05);
    });
    assert.ok(ratio >= 3, `the install block's edge at ${ratio.toFixed(2)}:1 against the canvas`);
    await page.close();
  });
});

test("the formats' scene: its switches edged at 3:1 at least against their ground, in both themes", async () => {
  await look(async (browser, origin) => {
    for (const scheme of ["light", "dark"]) {
      const page = await open(browser, origin, "fr", scheme);
      const ratio = await page.evaluate(() => {
        const lum = (c) => {
          const [r, g, b] = c.match(/\d+(\.\d+)?/g).slice(0, 3).map(Number).map((v) => v / 255).map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
          return 0.2126 * r + 0.7152 * g + 0.0722 * b;
        };
        const sw = document.querySelector(".v-format .sw");
        const ground = getComputedStyle(sw.closest("svg")).backgroundColor;
        const [a, b] = [lum(getComputedStyle(sw).stroke), lum(ground)].sort((x, y) => y - x);
        return (a + 0.05) / (b + 0.05);
      });
      assert.ok(ratio >= 3, `${scheme}: the switches' edge at ${ratio.toFixed(2)}:1`);
      await page.close();
    }
  });
});
