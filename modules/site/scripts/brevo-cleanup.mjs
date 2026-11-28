// The bounds docs/privacy.md promises for the release's list, kept by a job (ADR-0021), run each day by
// .github/workflows/site-brevo-cleanup.yml: a contact of the list is deleted 12 months after it was added; a contact
// in no list at all, an address never confirmed if Brevo keeps one (its documentation does not say), 30 days after.
// A contact of another list is never touched. Without --apply it only says what it would delete.
// It reads every contact (GET /v3/contacts, 1000 a page), deletes by id (DELETE /v3/contacts/<id>), one call at a
// time, each with a timeout, no retry: tomorrow's run takes what today's left. It says counts, never an address or
// the key, and stops at Brevo's first error, which fails the run and so notifies the repository's maintainers.
//
//   BREVO_API_KEY=… BREVO_LIST_ID=… node scripts/brevo-cleanup.mjs [--apply]

const API = "https://api.brevo.com/v3/contacts";
const PAGE = 1000;
const TIMEOUT_MS = 10_000;
const DAY = 86_400_000;
const LISTED_DAYS = 365; // « at the latest 12 months after you signed up »
const UNCONFIRMED_DAYS = 30; // « an address not confirmed within 30 days is deleted »

export async function cleanup({ key, list, apply, now = new Date(), fetch = globalThis.fetch, log = console.log }) {
  if (!key) throw new Error("brevo-cleanup: BREVO_API_KEY is not set");
  if (!Number.isInteger(list)) throw new Error("brevo-cleanup: BREVO_LIST_ID is not a list's number");
  const call = async (url, method = "GET") => {
    const answer = await fetch(url, { method, headers: { "api-key": key, accept: "application/json" }, signal: AbortSignal.timeout(TIMEOUT_MS) });
    if (!answer.ok) throw new Error(`brevo-cleanup: Brevo answered ${answer.status} to ${method} ${new URL(url).pathname.replace(/\d+$/, "<id>")}`);
    return answer;
  };

  const contacts = [];
  for (let offset = 0; ; offset += PAGE) {
    const page = await (await call(`${API}?limit=${PAGE}&offset=${offset}&sort=asc`)).json();
    contacts.push(...page.contacts);
    if (page.contacts.length < PAGE || contacts.length >= page.count) break;
  }

  const older = (created, days) => now.getTime() - new Date(created).getTime() > days * DAY;
  const listed = contacts.filter((c) => c.listIds.includes(list));
  const unconfirmed = contacts.filter((c) => c.listIds.length === 0);
  const goListed = listed.filter((c) => older(c.createdAt, LISTED_DAYS));
  const goUnconfirmed = unconfirmed.filter((c) => older(c.createdAt, UNCONFIRMED_DAYS));

  if (apply) for (const c of [...goListed, ...goUnconfirmed]) await call(`${API}/${c.id}`, "DELETE");

  const summary = { applied: Boolean(apply), seen: contacts.length, listed: listed.length, unconfirmed: unconfirmed.length,
    deletedListed: goListed.length, deletedUnconfirmed: goUnconfirmed.length };
  log(JSON.stringify({ event: "brevo-cleanup", ...summary }));
  return summary;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  cleanup({ key: process.env.BREVO_API_KEY, list: Number(process.env.BREVO_LIST_ID), apply: process.argv.includes("--apply") })
    .catch((error) => { console.error(JSON.stringify({ event: "brevo-cleanup", outcome: "failed", error: String(error.message) })); process.exit(1); });
}
