// The bounds docs/privacy.md promises for the release's list (ADR-0021), kept by a job, run each day by
// .github/workflows/site-brevo-cleanup.yml. Only this form's sign-ups are its business: the contacts /notify marked
// NOTIFY_SINCE (a text attribute, the day it asked Brevo, YYYY-MM-DD), never a contact of the account's other uses,
// never a blocklisted one, whose record not to be written to again is the person's. The bounds count from the mark,
// not from the contact's creation in the account (nora's reading of #1343):
//   - 12 months after it, a confirmed sign-up (in the release's list) is deleted when that list is its only one, else
//     taken out of it and its mark cleared;
//   - 30 days after it, an unconfirmed one (not in the list) is deleted when it is in no list, else its mark cleared.
// Without --apply it changes nothing and says what it would. It reads every contact (GET /v3/contacts, 1000 a page),
// writes by id, one call at a time, each with a timeout, no retry: tomorrow's run takes what today's left. It says
// counts, never an address or the key, and stops at Brevo's first error, which fails the run and so notifies the
// repository's maintainers.
//
//   BREVO_API_KEY=… BREVO_LIST_ID=… node scripts/brevo-cleanup.mjs [--apply]

const API = "https://api.brevo.com/v3/contacts";
const PAGE = 1000;
const TIMEOUT_MS = 10_000;
const DAY = 86_400_000;
const MARK = "NOTIFY_SINCE";
const LISTED_DAYS = 365; // « at the latest 12 months after you signed up »
const UNCONFIRMED_DAYS = 30; // « an address not confirmed within 30 days is deleted »

export async function cleanup({ key, list, apply, now = new Date(), fetch = globalThis.fetch, log = console.log }) {
  if (!key) throw new Error("brevo-cleanup: BREVO_API_KEY is not set");
  if (!Number.isInteger(list)) throw new Error("brevo-cleanup: BREVO_LIST_ID is not a list's number");
  const call = async (url, method = "GET", body) => {
    const headers = { "api-key": key, accept: "application/json", ...(body ? { "content-type": "application/json" } : {}) };
    const answer = await fetch(url, { method, headers, body: body && JSON.stringify(body), signal: AbortSignal.timeout(TIMEOUT_MS) });
    if (!answer.ok) throw new Error(`brevo-cleanup: Brevo answered ${answer.status} to ${method} ${new URL(url).pathname.replace(/\d+$/, "<id>")}`);
    return answer;
  };

  const contacts = [];
  for (let offset = 0; ; offset += PAGE) {
    const page = await (await call(`${API}?limit=${PAGE}&offset=${offset}&sort=asc`)).json();
    const some = page.contacts ?? [];
    contacts.push(...some);
    if (some.length < PAGE || contacts.length >= (page.count ?? 0)) break;
  }

  const day = (c) => /^\d{4}-\d{2}-\d{2}$/.test(c.attributes?.[MARK] ?? "") ? Date.parse(`${c.attributes[MARK]}T00:00:00Z`) : NaN;
  const marked = contacts.filter((c) => !Number.isNaN(day(c)));
  const blocked = marked.filter((c) => c.emailBlacklisted);
  // Whole days since the mark, the run's day counted in UTC: gone on the 365th, on the 30th.
  const today = Date.parse(`${now.toISOString().slice(0, 10)}T00:00:00Z`);
  const older = (c, days) => (today - day(c)) / DAY >= days;
  const work = [];
  for (const c of marked.filter((c) => !c.emailBlacklisted)) {
    const inList = c.listIds.includes(list);
    const others = c.listIds.filter((l) => l !== list);
    if (inList && older(c, LISTED_DAYS)) work.push(others.length ? ["unlisted", c, { unlinkListIds: [list], attributes: { [MARK]: "" } }] : ["deleted", c]);
    else if (!inList && older(c, UNCONFIRMED_DAYS)) work.push(others.length ? ["cleared", c, { attributes: { [MARK]: "" } }] : ["deleted", c]);
  }

  if (apply) {
    for (const [what, c, body] of work) await call(`${API}/${c.id}`, what === "deleted" ? "DELETE" : "PUT", body);
  }

  const count = (what) => work.filter(([w]) => w === what).length;
  const summary = { applied: Boolean(apply), seen: contacts.length, marked: marked.length, blocked: blocked.length,
    deleted: count("deleted"), unlisted: count("unlisted"), cleared: count("cleared") };
  log(JSON.stringify({ event: "brevo-cleanup", ...summary }));
  return summary;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  cleanup({ key: process.env.BREVO_API_KEY, list: Number(process.env.BREVO_LIST_ID), apply: process.argv.includes("--apply") })
    .catch((error) => { console.error(JSON.stringify({ event: "brevo-cleanup", outcome: "failed", error: String(error.message) })); process.exit(1); });
}
