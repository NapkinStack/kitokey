// The bounds docs/privacy.md promises, kept by a job, never by a person (the coordinator, 2026-11-28: no recurring
// manual step): scripts/brevo-cleanup.mjs, run each day by .github/workflows/site-brevo-cleanup.yml. Against a
// stand-in for Brevo's API. Only this form's sign-ups are its business: the contacts marked NOTIFY_SINCE, the day
// /notify asked Brevo, never a contact of the account's other uses, never a blocklisted one (its record not to be
// written to again is the person's). Twelve months after that day, a confirmed sign-up leaves the release's list:
// deleted when that list is its only one, else taken out of it and its mark cleared. Thirty days after, an unconfirmed
// one (not in the list) is deleted when it is in no list, else its mark cleared (nora's reading of #1343). Without
// --apply it changes nothing and says what it would. It never says an address, and stops on Brevo's first error.
// The bounds in whole days, the run's day in UTC: gone on the 365th and on the 30th day after the mark.
import { test } from "node:test";
import assert from "node:assert/strict";
import { cleanup } from "../scripts/brevo-cleanup.mjs";

const NOW = new Date("2027-06-15T12:00:00.000Z");
const DAY = 86_400_000;
const ago = (days) => new Date(NOW.getTime() - days * DAY).toISOString();
const LIST = 7;
const KEY = "fake-brevo-key-for-tests";

// A Brevo of `contacts`, answering GET /v3/contacts by pages, DELETE and PUT /v3/contacts/<id>.
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
    if (init.method === "PUT" && /^\/v3\/contacts\/\d+$/.test(u.pathname)) { calls.at(-1).push(JSON.parse(init.body)); return new Response(null, { status: 204 }); }
    return new Response("", { status: 404 });
  };
  return { fetch, calls };
}

// `since` is the form's mark, a day; `created` the contact's creation in the account, which never counts.
const contact = (id, since, listIds, { blocked = false, created = ago(2000) } = {}) =>
  ({ id, email: `person${id}@example.com`, createdAt: created, listIds, emailBlacklisted: blocked,
    attributes: since === undefined ? {} : { NOTIFY_SINCE: since.slice(0, 10) } });

const PEOPLE = [
  contact(1, ago(10), [LIST]), // confirmed lately: kept
  contact(2, ago(364), [LIST]), // confirmed 364 days ago: kept, though the account has known it for years
  contact(3, ago(365), [LIST]), // confirmed 365 days ago, the list its only one: deleted
  contact(4, ago(29), []), // unconfirmed 29 days ago: kept
  contact(5, ago(30), []), // unconfirmed 30 days ago, in no list: deleted
  contact(6, undefined, [], { created: ago(900) }), // the account's other use, no mark: never touched
  contact(7, ago(366), [99, LIST]), // confirmed, over 12 months, also in another list: out of ours, mark cleared
  contact(8, ago(31), [99]), // unconfirmed, over 30 days, in another list: mark cleared, not deleted
  contact(9, ago(900), [LIST], { blocked: true }), // blocklisted: never touched
  contact(10, ago(900), [], { blocked: true }), // blocklisted: never touched
];

function run(people, options = {}) {
  const lines = [];
  const { fetch, calls } = brevo(people, options);
  const done = cleanup({ key: KEY, list: LIST, apply: options.apply ?? false, now: NOW, fetch, log: (l) => lines.push(l) });
  return { done, calls, lines };
}

const deleted = (calls) => calls.filter(([m]) => m === "DELETE").map(([, path]) => Number(path.split("/").pop())).sort((a, b) => a - b);
const updated = (calls) => calls.filter(([m]) => m === "PUT").map(([, path, , , body]) => [Number(path.split("/").pop()), body]).sort((a, b) => a[0] - b[0]);

const SUMMARY = { seen: 10, marked: 9, blocked: 2, deleted: 2, unlisted: 1, cleared: 1 };

test("with --apply: this form's sign-ups past their bound go, by the sign-up's day; nothing of anyone else's", async () => {
  const { done, calls } = run(PEOPLE, { apply: true });
  const summary = await done;
  assert.deepEqual(deleted(calls), [3, 5]);
  assert.deepEqual(updated(calls), [
    [7, { unlinkListIds: [LIST], attributes: { NOTIFY_SINCE: "" } }],
    [8, { attributes: { NOTIFY_SINCE: "" } }],
  ]);
  assert.deepEqual(summary, { applied: true, ...SUMMARY });
});

test("a blocklisted contact and an unmarked one are never written to, whatever their age", async () => {
  const { done, calls } = run(PEOPLE, { apply: true });
  await done;
  const touched = calls.filter(([m]) => m !== "GET").map(([, path]) => Number(path.split("/").pop()));
  for (const id of [6, 9, 10]) assert.ok(!touched.includes(id), `contact ${id} touched`);
});

test("without --apply (the dry run): nothing is deleted, the same counts are said", async () => {
  const { done, calls, lines } = run(PEOPLE);
  const summary = await done;
  assert.deepEqual(deleted(calls), []);
  assert.deepEqual(updated(calls), []);
  assert.deepEqual(summary, { applied: false, ...SUMMARY });
  assert.ok(lines.some((l) => JSON.parse(l).applied === false));
});

test("every page of contacts is read, 1000 at a time", async () => {
  const many = Array.from({ length: 2345 }, (_, i) => contact(100 + i, ago(400), [LIST]));
  const { done, calls } = run(many, { apply: true });
  const summary = await done;
  const gets = calls.filter(([m]) => m === "GET").map(([, path]) => path);
  assert.deepEqual(gets, [0, 1000, 2000].map((o) => `/v3/contacts?limit=1000&offset=${o}&sort=asc`));
  assert.equal(summary.deleted, 2345);
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
  assert.equal(calls.filter(([m]) => m !== "GET").length, 1, "written after Brevo's error");
});

test("an account with no contact: Brevo's answer without `contacts` is no contact", async () => {
  const fetch = async () => Response.json({ count: 0 });
  const summary = await cleanup({ key: KEY, list: LIST, apply: true, now: NOW, fetch, log() {} });
  assert.equal(summary.seen, 0);
});

test("without its key or its list, it refuses to run", async () => {
  const { fetch } = brevo(PEOPLE);
  await assert.rejects(cleanup({ key: "", list: LIST, apply: true, now: NOW, fetch, log() {} }), /BREVO_API_KEY/);
  await assert.rejects(cleanup({ key: KEY, list: NaN, apply: true, now: NOW, fetch, log() {} }), /BREVO_LIST_ID/);
});
