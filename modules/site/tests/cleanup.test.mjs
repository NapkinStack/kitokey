// The bounds docs/privacy.md promises, kept by a job, never by a person (the coordinator, 2026-11-28: no recurring
// manual step): scripts/brevo-cleanup.mjs, run each day by .github/workflows/site-brevo-cleanup.yml. Against a
// stand-in for Brevo's API: a contact in the release's list is deleted 12 months after it was added; a contact in no
// list (an address never confirmed, if Brevo keeps one) 30 days after; a contact of another list never. Without
// --apply it deletes nothing and says what it would. It never says an address, and stops on Brevo's first error.
import { test } from "node:test";
import assert from "node:assert/strict";
import { cleanup } from "../scripts/brevo-cleanup.mjs";

const NOW = new Date("2027-06-15T12:00:00.000Z");
const DAY = 86_400_000;
const ago = (days) => new Date(NOW.getTime() - days * DAY).toISOString();
const LIST = 7;
const KEY = "fake-brevo-key-for-tests";

// A Brevo of `contacts`, answering GET /v3/contacts by pages and DELETE /v3/contacts/<id>.
function brevo(contacts, { failOn } = {}) {
  const calls = [];
  const fetch = async (url, init = {}) => {
    const u = new URL(url);
    calls.push([init.method ?? "GET", u.pathname + u.search, new Headers(init.headers).get("api-key"), init.signal]);
    if (failOn && failOn(init.method ?? "GET", u)) return new Response('{"code":"unauthorized"}', { status: 401 });
    if ((init.method ?? "GET") === "GET" && u.pathname === "/v3/contacts") {
      const limit = Number(u.searchParams.get("limit")), offset = Number(u.searchParams.get("offset"));
      return Response.json({ contacts: contacts.slice(offset, offset + limit), count: contacts.length });
    }
    if (init.method === "DELETE" && /^\/v3\/contacts\/\d+$/.test(u.pathname)) return new Response(null, { status: 204 });
    return new Response("", { status: 404 });
  };
  return { fetch, calls };
}

const contact = (id, created, listIds, email = `person${id}@example.com`) =>
  ({ id, email, createdAt: created, listIds, emailBlacklisted: false, attributes: {} });

const PEOPLE = [
  contact(1, ago(10), [LIST]), // confirmed lately: kept
  contact(2, ago(364), [LIST]), // confirmed, under 12 months: kept
  contact(3, ago(366), [LIST]), // confirmed, over 12 months: deleted
  contact(4, ago(29), []), // unconfirmed, under 30 days: kept
  contact(5, ago(31), []), // unconfirmed, over 30 days: deleted
  contact(6, ago(900), [99]), // another list's: never touched
  contact(7, ago(900), [99, LIST]), // in the release's list and another: the 12 months apply
];

function run(people, options = {}) {
  const lines = [];
  const { fetch, calls } = brevo(people, options);
  const done = cleanup({ key: KEY, list: LIST, apply: options.apply ?? false, now: NOW, fetch, log: (l) => lines.push(l) });
  return { done, calls, lines };
}

const deleted = (calls) => calls.filter(([m]) => m === "DELETE").map(([, path]) => Number(path.split("/").pop())).sort((a, b) => a - b);

test("with --apply: the release's list over 12 months and the unconfirmed over 30 days go; nothing else", async () => {
  const { done, calls } = run(PEOPLE, { apply: true });
  const summary = await done;
  assert.deepEqual(deleted(calls), [3, 5, 7]);
  assert.deepEqual(summary, { applied: true, seen: 7, listed: 4, unconfirmed: 2, deletedListed: 2, deletedUnconfirmed: 1 });
});

test("without --apply (the dry run): nothing is deleted, the same counts are said", async () => {
  const { done, calls, lines } = run(PEOPLE);
  const summary = await done;
  assert.deepEqual(deleted(calls), []);
  assert.deepEqual(summary, { applied: false, seen: 7, listed: 4, unconfirmed: 2, deletedListed: 2, deletedUnconfirmed: 1 });
  assert.ok(lines.some((l) => JSON.parse(l).applied === false));
});

test("every page of contacts is read, 1000 at a time", async () => {
  const many = Array.from({ length: 2345 }, (_, i) => contact(100 + i, ago(400), [LIST]));
  const { done, calls } = run(many, { apply: true });
  const summary = await done;
  const gets = calls.filter(([m]) => m === "GET").map(([, path]) => path);
  assert.deepEqual(gets, [0, 1000, 2000].map((o) => `/v3/contacts?limit=1000&offset=${o}&sort=asc`));
  assert.equal(summary.deletedListed, 2345);
});

test("each call carries the key and a timeout; a contact is deleted by its id, never its address", async () => {
  const { done, calls } = run(PEOPLE, { apply: true });
  await done;
  for (const [, path, key, signal] of calls) {
    assert.equal(key, KEY);
    assert.ok(signal instanceof AbortSignal, `${path}: no timeout`);
    assert.doesNotMatch(path, /@|%40/, `${path}: an address in the URL`);
  }
});

test("what it says holds no address nor the key", async () => {
  const { done, lines } = run(PEOPLE, { apply: true });
  await done;
  const said = lines.join("\n");
  assert.ok(lines.length >= 1);
  assert.doesNotMatch(said, /@/);
  assert.ok(!said.includes(KEY));
  for (const line of lines) assert.equal(JSON.parse(line).event, "brevo-cleanup");
});

test("Brevo's error stops it, said without the key; nothing deleted after it", async () => {
  const { done, calls } = run(PEOPLE, { apply: true, failOn: (method) => method === "DELETE" });
  await assert.rejects(done, (error) => !String(error).includes(KEY) && /401/.test(String(error)));
  assert.equal(calls.filter(([m]) => m === "DELETE").length, 1);
});

test("without its key or its list, it refuses to run", async () => {
  const { fetch } = brevo(PEOPLE);
  await assert.rejects(cleanup({ key: "", list: LIST, apply: true, now: NOW, fetch, log() {} }), /BREVO_API_KEY/);
  await assert.rejects(cleanup({ key: KEY, list: NaN, apply: true, now: NOW, fetch, log() {} }), /BREVO_LIST_ID/);
});
