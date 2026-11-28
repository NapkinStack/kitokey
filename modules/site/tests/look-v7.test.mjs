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

test("no compound word breaks between two lines, at 1440 and 420 px, in both languages", async () => {
  const found = [];
  await look(async (browser, origin) => {
    for (const lang of ["fr", "en"]) {
      for (const width of [1440, 420]) {
        const page = await browser.newPage({ colorScheme: "light", viewport: { width, height: 900 } });
        await page.goto(`${origin}/${lang}/#still`, { waitUntil: "networkidle" });
        await page.evaluate(() => document.fonts.ready);
        const split = await page.evaluate(() => [...document.querySelectorAll(".nb")]
          .filter((w) => w.getClientRects().length > 1).map((w) => w.textContent));
        const count = await page.evaluate(() => document.querySelectorAll(".nb").length);
        if (count === 0) found.push([lang, width, "no .nb at all"]);
        for (const word of split) found.push([lang, width, word]);
        await page.close();
      }
    }
  });
  assert.deepEqual(found, []);
});

test("each « Bientôt » pill follows its card's title, as planche v4 draws it: on the title's line, clear of the scene, the accent on its tint", async () => {
  const found = [];
  await look(async (browser, origin) => {
    for (const lang of ["fr", "en"]) {
      for (const scheme of ["light", "dark"]) {
        const page = await open(browser, origin, lang, scheme);
        const pills = await page.evaluate(() => {
          const probe = (value) => { const i = document.createElement("i"); i.style.color = value; document.body.append(i); const c = getComputedStyle(i).color; i.remove(); return c; };
          const accent = probe("var(--accent)"), tint = probe("var(--bg-tint)");
          return [...document.querySelectorAll(".ft h3 .chip")].map((chip) => {
            const c = chip.getBoundingClientRect(), h = chip.closest("h3"), v = chip.closest(".ft").querySelector("svg.v")?.getBoundingClientRect();
            const s = getComputedStyle(chip);
            const lines = [...h.getClientRects()];
            const onLine = lines.some((l) => c.top < l.bottom && c.bottom > l.top) && c.left >= h.getBoundingClientRect().left - 0.5;
            const clear = !v || c.bottom <= v.top || c.top >= v.bottom || c.right <= v.left || c.left >= v.right;
            return [h.firstChild?.textContent?.trim() ?? "", onLine && s.position !== "absolute", clear, s.color === accent, s.backgroundColor === tint, s.fontWeight === "700", s.fontSize === "12px"];
          });
        });
        if (pills.length !== 3) found.push([lang, scheme, `${pills.length} pills`]);
        for (const pill of pills) if (pill.slice(1).some((ok) => !ok)) found.push([lang, scheme, ...pill]);
        await page.close();
      }
    }
  });
  assert.deepEqual(found, []);
});

test("each section a nav link reaches lands just under the sticky header, at 1440 and 420 px", async () => {
  const found = [];
  await look(async (browser, origin) => {
    for (const [width, lang] of [[1440, "fr"], [420, "fr"], [1440, "en"]]) {
      const page = await browser.newPage({ colorScheme: "light", viewport: { width, height: 900 } });
      await page.goto(`${origin}/${lang}/`, { waitUntil: "networkidle" });
      const ids = await page.evaluate(() => [...document.querySelectorAll(".top nav a[href^='#']")].map((a) => a.getAttribute("href").slice(1)));
      for (const id of ids) {
        // Where the scroll ends, not mid-way: the page scrolls smoothly (html's scroll-behavior), and a heading that
        // rises in (.rise, 12 px, .4 s) is measured once it has risen.
        await page.evaluate((id) => document.getElementById(id).scrollIntoView({ block: "start", behavior: "instant" }), id);
        await page.waitForTimeout(700);
        const gap = await page.evaluate((id) => {
          const el = document.getElementById(id);
          const header = document.querySelector(".top").getBoundingClientRect().bottom;
          const heading = el.querySelector("h2") ?? el;
          return heading.getBoundingClientRect().top - header;
        }, id);
        if (!(gap >= 0 && gap <= 72)) found.push([lang, width, id, Math.round(gap)]);
      }
      if (!ids.length && width === 1440) found.push([lang, width, "no nav link"]);
      await page.close();
    }
  });
  assert.deepEqual(found, []);
});

test("the install tabs' marks stand at one optical height, drawn whole at 200 %", async () => {
  await look(async (browser, origin) => {
    for (const scheme of ["light", "dark"]) {
      const page = await browser.newPage({ colorScheme: scheme, viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 });
      await page.goto(`${origin}/en/#still`, { waitUntil: "networkidle" });
      const marks = await page.evaluate(() => [...document.querySelectorAll(".os-tab img")].map((i) => [i.getAttribute("src"), Math.round(i.getBoundingClientRect().height), i.complete && i.naturalWidth > 0]));
      assert.ok(marks.length >= 3, `${scheme}: ${marks.length} marks`);
      const heights = marks.map(([, h]) => h);
      assert.ok(Math.max(...heights) - Math.min(...heights) <= 1, `${scheme}: heights ${JSON.stringify(marks)}`);
      assert.ok(marks.every(([, , drawn]) => drawn), `${scheme}: a mark not drawn ${JSON.stringify(marks)}`);
      await page.close();
    }
  });
});

test("the Store's and the Mac's marks stand on their text's baseline, the Windows one as tall as the capitals, in the button's colour", async () => {
  const found = [];
  await look(async (browser, origin) => {
    for (const [lang, scheme] of [["fr", "light"], ["en", "dark"]]) {
      const page = await browser.newPage({ colorScheme: scheme, viewport: { width: 1440, height: 900 } });
      await page.goto(`${origin}/${lang}/`, { waitUntil: "networkidle" });
      const [store, mac] = await page.evaluate(() => {
        // A probe of no width at the text's end: its bottom is the baseline, its height 1cap.
        const against = (mark) => {
          const probe = document.createElement("span");
          probe.style.cssText = "display:inline-block;width:0;height:1cap";
          mark.parentElement.append(probe);
          const m = mark.getBoundingClientRect(), p = probe.getBoundingClientRect();
          probe.remove();
          return { bottom: Math.round(m.bottom - p.bottom), height: Math.round(m.height - p.height), cap: Math.round(p.height), shown: m.height > 0,
            tint: getComputedStyle(mark).backgroundColor === getComputedStyle(mark.parentElement).color };
        };
        const mac = [...document.querySelectorAll(".hero .cta-note .mark")].find((m) => m.getBoundingClientRect().height > 0);
        return [against(document.querySelector(".hero .store-soon .mark")), mac ? against(mac) : null];
      });
      if (!store.shown || Math.abs(store.bottom) > 1 || Math.abs(store.height) > 1 || !store.tint) found.push([lang, scheme, "store", store]);
      if (!mac || Math.abs(mac.bottom) > 1) found.push([lang, scheme, "mac", mac]);
      await page.close();
    }
  });
  assert.deepEqual(found, []);
});

test("the language switch stays in the header at 420 px, a 44 px target, its globe and the other language's name", async () => {
  await look(async (browser, origin) => {
    for (const [lang, name] of [["fr", "English"], ["en", "Français"]]) {
      const page = await browser.newPage({ colorScheme: "light", viewport: { width: 420, height: 900 } });
      await page.goto(`${origin}/${lang}/`, { waitUntil: "networkidle" });
      const box = await page.locator(".top .lang").boundingBox();
      assert.ok(box && box.height >= 44 && box.width >= 44 && box.x + box.width <= 420, `/${lang}/ at 420 px: ${JSON.stringify(box)}`);
      assert.equal((await page.locator(".top .lang").innerText()).trim(), name);
      await page.close();
    }
  });
});

// lina's note on #1338 (fr-desk-light, 2026-11-28): the lock touched « Reste sur votre ordinateur ». The gap between
// the icon's right edge and the title's first letter, measured, in both languages and themes.
test("« Reste sur votre ordinateur »'s lock stands apart from its words: 6 px at least", async () => {
  const found = [];
  await look(async (browser, origin) => {
    for (const lang of ["fr", "en"]) {
      for (const scheme of ["light", "dark"]) {
        const page = await open(browser, origin, lang, scheme);
        const gap = await page.evaluate(() => {
          const h3 = document.querySelector(".mid .box.here .box-h h3");
          const svg = h3.querySelector("svg");
          const words = [...h3.childNodes].find((n) => n.nodeType === Node.TEXT_NODE && n.textContent.trim());
          const range = document.createRange();
          range.setStart(words, 0);
          range.setEnd(words, 1);
          return range.getBoundingClientRect().left - svg.getBoundingClientRect().right;
        });
        if (!(gap >= 6)) found.push([lang, scheme, gap]);
        await page.close();
      }
    }
  });
  assert.deepEqual(found, []);
});

// lina's note on #1338 (2026-11-28): Fedora's cell in the Linux price strip sat about 4 px low. Each cell's name and
// mark measured against the others': the names on one line (their text's top within 1 px), each mark centred on its
// name (within 1 px), in both languages and themes. The values are said when it fails.
test("the Linux strip's three distributions stand on one line, each mark centred on its name", async () => {
  const found = [];
  await look(async (browser, origin) => {
    for (const lang of ["fr", "en"]) {
      for (const scheme of ["light", "dark"]) {
        const page = await open(browser, origin, lang, scheme);
        const cells = await page.evaluate(() => [...document.querySelectorAll(".pcard.linux .strip > span")].map((cell) => {
          const b = cell.querySelector("b"), img = b.querySelector("img");
          const words = [...b.childNodes].find((n) => n.nodeType === Node.TEXT_NODE && n.textContent.trim());
          const range = document.createRange();
          range.selectNodeContents(words);
          const t = range.getBoundingClientRect(), m = img.getBoundingClientRect(), c = cell.getBoundingClientRect();
          return { name: words.textContent.trim(), top: t.top - c.top, centre: (m.top + m.bottom) / 2 - (t.top + t.bottom) / 2 };
        }));
        if (cells.length !== 3) found.push([lang, scheme, `${cells.length} cells`]);
        const first = cells[0]?.top;
        for (const { name, top, centre } of cells) {
          if (Math.abs(top - first) > 1 || Math.abs(centre) > 1) found.push([lang, scheme, name, `top ${top.toFixed(1)} (first ${first.toFixed(1)})`, `mark off its name's centre by ${centre.toFixed(1)}`]);
        }
        await page.close();
      }
    }
  });
  assert.deepEqual(found, []);
});
