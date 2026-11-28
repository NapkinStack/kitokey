// B4 (ADR-0021): « Me prévenir de la sortie ». The function at /notify, run here against a stand-in for Brevo (a
// fetch double): it calls Brevo's double opt-in only for a well-formed request, with its key, list, template and a
// timeout; it answers with the site's pages; its log and its answer never hold the address nor the key. Then the
// built page: the form posts to the site itself, the browser's CSP lets it, and the pages it lands on exist.
import { test, beforeEach, afterEach } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { dist, unwrapped } from "./helpers.mjs";

const KEY = "fake-brevo-key-for-tests";
const ENV = { BREVO_API_KEY: KEY, BREVO_LIST_ID: "7", BREVO_TEMPLATE_ID: "3" };
const EMAIL = "someone@example.com";
const BREVO = "https://api.brevo.com/v3/contacts/doubleOptinConfirmation";

let calls, logs, answer, realFetch, realLog, realError;
beforeEach(() => {
  calls = [];
  logs = [];
  answer = () => new Response("{}", { status: 201 });
  realFetch = globalThis.fetch;
  realLog = console.log;
  realError = console.error;
  globalThis.fetch = async (url, init) => { calls.push([String(url), init]); return answer(); };
  console.log = console.error = (...args) => logs.push(args.join(" "));
});
afterEach(() => {
  globalThis.fetch = realFetch;
  console.log = realLog;
  console.error = realError;
});

const form = (fields) => new URLSearchParams({ email: EMAIL, lang: "fr", hp_field: "", ...fields }).toString();
// Imported when called, so that each test says what it finds, the page's tests too, before the function exists.
// A browser's POST from the site's own page says Sec-Fetch-Site: same-origin and its Origin (nora's reading of #1343).
const SAME = { "sec-fetch-site": "same-origin", origin: "https://kitokey.com" };
async function post(body, { env = ENV, type = "application/x-www-form-urlencoded", from = SAME, url = "https://kitokey.com/notify" } = {}) {
  const request = new Request(url, { method: "POST", headers: { "content-type": type, ...from }, body });
  const { onRequestPost } = await import("../functions/notify.js");
  return onRequestPost({ request, env });
}
const where = (response) => [response.status, response.headers.get("location")];
async function nothingLeaks(response) {
  const said = [await response.clone().text(), ...response.headers.values(), ...logs].join("\n");
  assert.ok(!said.includes(EMAIL), "the address in the answer or the log");
  assert.ok(!said.includes(KEY), "the key in the answer or the log");
  assert.ok(logs.length >= 1, "nothing logged");
  for (const line of logs) assert.equal(JSON.parse(line).event, "notify", `a log line not structured: ${line}`);
}

test("a well-formed request: Brevo's double opt-in is asked once, with the key, the list, the template and a timeout; then « Vérifiez votre boîte »", async () => {
  const response = await post(form({}));
  assert.deepEqual(where(response), [303, "/fr/notify/sent/"]);
  assert.equal(calls.length, 1);
  const [url, init] = calls[0];
  assert.equal(url, BREVO);
  assert.equal(init.method, "POST");
  assert.equal(new Headers(init.headers).get("api-key"), KEY);
  assert.equal(new Headers(init.headers).get("content-type"), "application/json");
  assert.ok(init.signal instanceof AbortSignal, "no timeout on the call to Brevo");
  // The form's mark, the day it asked (UTC): what the daily clean-up counts its bounds from, and the only contacts it
  // touches (scripts/brevo-cleanup.mjs, nora's reading of #1343).
  const today = new Date().toISOString().slice(0, 10);
  assert.deepEqual(JSON.parse(init.body), { email: EMAIL, includeListIds: [7], templateId: 3, redirectionUrl: "https://kitokey.com/fr/notify/confirmed/", attributes: { NOTIFY_SINCE: today } });
  await nothingLeaks(response);
});

test("in English, the pages are the English ones; an unknown language is English", async () => {
  assert.deepEqual(where(await post(form({ lang: "en" }))), [303, "/en/notify/sent/"]);
  assert.equal(JSON.parse(calls[0][1].body).redirectionUrl, "https://kitokey.com/en/notify/confirmed/");
  assert.deepEqual(where(await post(form({ lang: "../x" }))), [303, "/en/notify/sent/"]);
});

test("the address is taken trimmed", async () => {
  await post(form({ email: `  ${EMAIL} ` }));
  assert.equal(JSON.parse(calls[0][1].body).email, EMAIL);
});

for (const [why, body, type, lang] of [
  ["no address", form({ email: "" })],
  ["no @", form({ email: "someone.example.com" })],
  ["no domain's dot", form({ email: "someone@example" })],
  ["a space inside", form({ email: "some one@example.com" })],
  ["two @", form({ email: "a@b@example.com" })],
  ["longer than 254", form({ email: `${"a".repeat(243)}@example.com` })],
  ["a body over 2 KB", form({ pad: "x".repeat(2100) }), undefined, "en"],
  ["not a form", JSON.stringify({ email: EMAIL }), "application/json", "en"],
  ["plain text", form({}), "text/plain", "en"],
  ["a space in the domain", form({ email: "someone@exam ple.com" })],
  // Raw UTF-8, as a client may send it: about 1150 characters, about 2250 bytes.
  ["over 2 KB in bytes, under it in characters", `${form({})}&pad=${"é".repeat(1100)}`, undefined, "en"],
]) {
  test(`refused, Brevo never called: ${why}`, async () => {
    const response = await post(body, type ? { type } : undefined);
    // Refused before the form is read, the language unknown: English, as the root's.
    assert.deepEqual(where(response), [303, `/${lang ?? "fr"}/notify/failed/`]);
    assert.equal(calls.length, 0);
    await nothingLeaks(response);
  });
}

test("on the bounds: an address of exactly 254 characters and a body of exactly 2048 bytes are taken", async () => {
  const at254 = `${"a".repeat(242)}@example.com`;
  assert.equal(at254.length, 254);
  assert.deepEqual(where(await post(form({ email: at254 }))), [303, "/fr/notify/sent/"]);
  const base = form({});
  const at2048 = `${base}&pad=${"x".repeat(2048 - Buffer.byteLength(base) - 5)}`;
  assert.equal(Buffer.byteLength(at2048), 2048);
  assert.deepEqual(where(await post(at2048)), [303, "/fr/notify/sent/"]);
  assert.equal(calls.length, 2);
});

for (const [why, from] of [
  ["another site's form (Sec-Fetch-Site: cross-site)", { "sec-fetch-site": "cross-site", origin: "https://evil.example" }],
  ["a sibling site (same-site is not same-origin)", { "sec-fetch-site": "same-site", origin: "https://preview.kitokey.com" }],
  ["another Origin, no Sec-Fetch-Site", { origin: "https://evil.example" }],
  ["neither header", {}],
]) {
  test(`refused with a 403, Brevo never called: ${why}`, async () => {
    const response = await post(form({}), { from });
    assert.equal(response.status, 403);
    assert.equal(calls.length, 0);
    await nothingLeaks(response);
  });
}

for (const [why, from] of [
  ["Sec-Fetch-Site: same-origin alone", { "sec-fetch-site": "same-origin" }],
  ["the site's Origin alone (a browser without Fetch Metadata)", { origin: "https://kitokey.com" }],
]) {
  test(`taken from the site itself: ${why}`, async () => {
    assert.deepEqual(where(await post(form({}), { from })), [303, "/fr/notify/sent/"]);
    assert.equal(calls.length, 1);
  });
}

test("Brevo's link lands on kitokey.com whatever host the request came by", async () => {
  await post(form({}), { url: "https://some-preview.kitokey.pages.dev/notify", from: { "sec-fetch-site": "same-origin" } });
  assert.equal(JSON.parse(calls[0][1].body).redirectionUrl, "https://kitokey.com/fr/notify/confirmed/");
});

test("the function's origin is the site's own, as astro.config.mjs names it", async () => {
  const config = await readFile(new URL("../astro.config.mjs", import.meta.url), "utf8");
  const site = config.match(/site: "([^"]+)"/)?.[1];
  const fn = await readFile(new URL("../functions/notify.js", import.meta.url), "utf8");
  assert.equal(fn.match(/const SITE = "([^"]+)"/)?.[1], site);
});

test("the honeypot filled: Brevo never called, and the robot is told it worked", async () => {
  const response = await post(form({ hp_field: "https://spam.example" }));
  assert.deepEqual(where(response), [303, "/fr/notify/sent/"]);
  assert.equal(calls.length, 0);
  await nothingLeaks(response);
});

test("without the key (the preview), nothing is called and the failure is said", async () => {
  const response = await post(form({}), { env: { BREVO_LIST_ID: "7", BREVO_TEMPLATE_ID: "3" } });
  assert.deepEqual(where(response), [303, "/fr/notify/failed/"]);
  assert.equal(calls.length, 0);
  await nothingLeaks(response);
});

for (const [why, brevo] of [
  ["Brevo refuses", () => new Response('{"code":"invalid_parameter"}', { status: 400 })],
  ["Brevo fails", () => new Response("", { status: 503 })],
  ["Brevo answers a redirect", () => new Response("", { status: 302, headers: { location: "https://example.com/" } })],
  ["Brevo does not answer in time", () => { throw new DOMException("timed out", "TimeoutError"); }],
]) {
  test(`${why}: the generic failure, once, no retry, the status in the log`, async () => {
    answer = brevo;
    const response = await post(form({}));
    assert.deepEqual(where(response), [303, "/fr/notify/failed/"]);
    assert.equal(calls.length, 1, "retried");
    await nothingLeaks(response);
    assert.ok(logs.some((line) => JSON.parse(line).outcome !== "sent"), "the failure not logged");
  });
}

const page = (lang, path = "") => readFile(join(dist, lang, path, "index.html"), "utf8").then(unwrapped);

for (const lang of ["fr", "en"]) {
  test(`/${lang}/: one form posts to /notify on the site itself: the address, the language, the honeypot hidden from everyone`, async () => {
    const html = await page(lang);
    const forms = html.match(/<form\b[^>]*>[\s\S]*?<\/form>/g) ?? [];
    assert.equal(forms.length, 1);
    const [f] = forms;
    assert.match(f, /^<form[^>]* id="notify"/);
    assert.match(f, /^<form[^>]* method="post"/);
    assert.match(f, /^<form[^>]* action="\/notify"/);
    assert.match(f, /<input[^>]* type="email"[^>]* name="email"|<input[^>]* name="email"[^>]* type="email"/);
    assert.match(f, /<input[^>]* name="email"[^>]* required/);
    assert.match(f, /<input[^>]* name="email"[^>]* maxlength="254"/);
    assert.match(f, new RegExp(`<input type="hidden" name="lang" value="${lang}"`));
    const trap = f.match(/<div class="trap" aria-hidden="true">[\s\S]*?<\/div>/)?.[0] ?? "";
    assert.match(trap, /<input[^>]* name="hp_field"[^>]* tabindex="-1"[^>]* autocomplete="off"/);
    assert.doesNotMatch(trap.match(/name="([^"]*)"/)?.[1] ?? "", /web|url|site|home|mail|name|phone|tel|addr/i, "a trap autofill would fill");
    assert.match(f, new RegExp(`<a href="/${lang}/privacy/">`), "the form does not say where its words are kept");
  });

  test(`/${lang}/: every « Me prévenir de la sortie » leads to the form`, async () => {
    const html = await page(lang);
    const links = [...html.matchAll(/<a class="store-soon" href="([^"]+)"/g)].map((m) => m[1]);
    assert.ok(links.length >= 2, `${links.length} Store buttons`);
    assert.deepEqual([...new Set(links)], ["#notify"]);
  });

  for (const state of ["sent", "failed", "confirmed"]) {
    test(`/${lang}/notify/${state}/ is a page of the site, not indexed`, async () => {
      assert.ok(existsSync(join(dist, lang, "notify", state, "index.html")), "not built");
      const html = await page(lang, `notify/${state}`);
      assert.match(html, /<meta name="robots" content="noindex"/);
      assert.match(html, /<h1[^>]*>[^<]+<\/h1>/);
      assert.match(html, new RegExp(`<a class="home" href="/${lang}/">`), "no way back to the home page");
    });
  }
}

test("the pages /notify answers with are not in the sitemap", async () => {
  const { readdir } = await import("node:fs/promises");
  const maps = (await readdir(dist)).filter((f) => /^sitemap.*\.xml$/.test(f));
  assert.ok(maps.length > 0, "no sitemap");
  for (const f of maps) assert.doesNotMatch(await readFile(join(dist, f), "utf8"), /\/notify\//, f);
});

test("_headers lets a form post to the site itself, and nowhere else", async () => {
  const headers = await readFile(join(dist, "_headers"), "utf8");
  assert.match(headers, /form-action 'self'(;|$)/m);
  assert.doesNotMatch(headers, /form-action 'none'/);
});

test("the privacy policy says what the form keeps, who sends, for how long, on what basis, in both languages", async () => {
  for (const [lang, said] of [
    // nora's reading against GDPR art. 13: the right to complain to the CNIL, how long an unconfirmed address and the
    // list are kept (the decider and the coordinator, 2026-11-28, what a quarterly clean-up by hand holds: an unconfirmed
    // address at the latest 4 months after the request, the list right after the one message and at the latest 12 months
    // after the sign-up), every right that applies under consent. The CNIL's complaint page is in French alone (a complaint is
    // written in French), so both halves link it.
    ["en", ["Brevo", "European Union", "consent", "6(1)(a)", "delete it", "at the latest 4 months after", "right after", "at the latest 12 months after", "restrict", "object", "portab", "CNIL", 'href="https://www.cnil.fr/fr/adresser-une-plainte"', "never write to it again"] ],
    ["fr", ["Brevo", "Union européenne", "consentement", "6.1.a", "l’effaçons", "au plus tard 4 mois après", "aussitôt après", "au plus tard 12 mois après", "limiter", "opposer", "portab", "CNIL", 'href="https://www.cnil.fr/fr/adresser-une-plainte"', "ne plus jamais lui écrire"] ],
  ]) {
    const html = await page(lang, "privacy");
    for (const words of said) assert.ok(html.includes(words), `/${lang}/privacy/ does not say « ${words} »`);
  }
});
