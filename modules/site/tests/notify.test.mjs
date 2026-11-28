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

const form = (fields) => new URLSearchParams({ email: EMAIL, lang: "fr", website: "", ...fields }).toString();
// Imported when called, so that each test says what it finds, the page's tests too, before the function exists.
async function post(body, { env = ENV, type = "application/x-www-form-urlencoded" } = {}) {
  const request = new Request("https://kitokey.com/notify", { method: "POST", headers: { "content-type": type }, body });
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
  assert.deepEqual(JSON.parse(init.body), { email: EMAIL, includeListIds: [7], templateId: 3, redirectionUrl: "https://kitokey.com/fr/notify/confirmed/" });
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

for (const [why, body, type] of [
  ["no address", form({ email: "" })],
  ["no @", form({ email: "someone.example.com" })],
  ["no domain's dot", form({ email: "someone@example" })],
  ["a space inside", form({ email: "some one@example.com" })],
  ["two @", form({ email: "a@b@example.com" })],
  ["longer than 254", form({ email: `${"a".repeat(243)}@example.com` })],
  ["a body over 2 KB", form({ pad: "x".repeat(2100) })],
  ["not a form", JSON.stringify({ email: EMAIL }), "application/json"],
]) {
  test(`refused, Brevo never called: ${why}`, async () => {
    const response = await post(body, type ? { type } : undefined);
    assert.deepEqual(where(response), [303, `/${type ? "en" : "fr"}/notify/failed/`]);
    assert.equal(calls.length, 0);
    await nothingLeaks(response);
  });
}

test("the honeypot filled: Brevo never called, and the robot is told it worked", async () => {
  const response = await post(form({ website: "https://spam.example" }));
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
    assert.match(trap, /<input[^>]* name="website"[^>]* tabindex="-1"[^>]* autocomplete="off"/);
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
      assert.match(html, new RegExp(`<a href="/${lang}/">`), "no way back to the home page");
    });
  }
}

test("_headers lets a form post to the site itself, and nowhere else", async () => {
  const headers = await readFile(join(dist, "_headers"), "utf8");
  assert.match(headers, /form-action 'self'(;|$)/m);
  assert.doesNotMatch(headers, /form-action 'none'/);
});

test("the privacy policy says what the form keeps, who sends, for how long, on what basis, in both languages", async () => {
  for (const [lang, said] of [
    ["en", ["Brevo", "European Union", "consent", "6(1)(a)", "delete it"]],
    ["fr", ["Brevo", "Union européenne", "consentement", "6.1.a", "l'effaçons"]],
  ]) {
    const html = await page(lang, "privacy");
    for (const words of said) assert.ok(html.includes(words), `/${lang}/privacy/ does not say « ${words} »`);
  }
});
