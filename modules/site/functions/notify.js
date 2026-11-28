// POST /notify (ADR-0021): « Me prévenir de la sortie ». The site's one function, run by Cloudflare Pages. It takes
// the landing's form, asks Brevo to send the address a confirmation link (double opt-in: only a confirmed address
// joins the list), and answers with a page of the site. All input is hostile until checked; Brevo is called once,
// with a timeout, never retried (the visitor may send again; Brevo answers the same for an address it has).
// Its log is one JSON line per request, the outcome and Brevo's status: never the address, never the key.
// Set on the Pages project by the decider (production only): BREVO_API_KEY, a secret; BREVO_LIST_ID and
// BREVO_TEMPLATE_ID, plain variables. Without the key (the preview), nothing is called and the failure is said.

const BREVO = "https://api.brevo.com/v3/contacts/doubleOptinConfirmation";
// The site's origin, astro.config.mjs's `site` (tests/notify.test.mjs holds them equal): where Brevo's link lands,
// never taken from the request's host; and the one Origin a POST is taken from.
const SITE = "https://kitokey.com";
const TIMEOUT_MS = 5000;
const MAX_BODY = 2048; // bytes
const MAX_EMAIL = 254; // RFC 5321's path limit
const LANGS = new Set(["en", "fr"]);
// One @, a local part and a domain with a dot, no space: what a mail can be sent to, not RFC 5322 whole. Brevo checks
// it again; the confirmation link proves it.
const EMAIL = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

// Brevo's error `code` on a refusal, its enum (`invalid_parameter`, …): what a log needs to say why, read from at
// most 4 KB of its answer. Never its `message`, which may say the address back; anything else is « unknown ».
async function brevoCode(answer) {
  try {
    const code = JSON.parse((await answer.text()).slice(0, 4096))?.code;
    return typeof code === "string" && /^[a-z_]{1,64}$/.test(code) ? code : "unknown";
  } catch {
    return "unknown";
  }
}

const log = (outcome, extra = {}) => console.log(JSON.stringify({ event: "notify", outcome, ...extra }));
const go = (lang, page) => new Response(null, { status: 303, headers: { location: `/${lang}/notify/${page}/`, "cache-control": "no-store" } });

// A POST from the site's own page: the browser says Sec-Fetch-Site: same-origin, or, without Fetch Metadata, sends the
// site's Origin. Another site's form, or a sibling host, is refused before anything is read: it would make the site
// mail a confirmation to any address (nora's reading of #1343).
const fromTheSite = (headers) => headers.get("sec-fetch-site") === "same-origin" || headers.get("origin") === SITE;

export async function onRequestPost({ request, env }) {
  if (!fromTheSite(request.headers)) {
    log("refused", { why: "origin" });
    return new Response("Forbidden", { status: 403, headers: { "content-type": "text/plain", "cache-control": "no-store" } });
  }
  const type = (request.headers.get("content-type") ?? "").split(";")[0].trim().toLowerCase();
  if (type !== "application/x-www-form-urlencoded") {
    log("refused", { why: "type" });
    return go("en", "failed");
  }
  if (Number(request.headers.get("content-length") ?? 0) > MAX_BODY) {
    log("refused", { why: "size" });
    return go("en", "failed");
  }
  const bytes = await request.arrayBuffer();
  if (bytes.byteLength > MAX_BODY) {
    log("refused", { why: "size" });
    return go("en", "failed");
  }
  const raw = new TextDecoder().decode(bytes);
  const form = new URLSearchParams(raw);
  const lang = LANGS.has(form.get("lang")) ? form.get("lang") : "en";

  if ((form.get("hp_field") ?? "") !== "") {
    log("trapped");
    return go(lang, "sent");
  }
  const email = (form.get("email") ?? "").trim();
  if (email.length > MAX_EMAIL || !EMAIL.test(email)) {
    log("refused", { why: "address" });
    return go(lang, "failed");
  }
  const list = Number(env.BREVO_LIST_ID);
  const template = Number(env.BREVO_TEMPLATE_ID);
  if (!env.BREVO_API_KEY || !Number.isInteger(list) || !Number.isInteger(template)) {
    log("unconfigured");
    return go(lang, "failed");
  }

  let status, code;
  try {
    const answer = await fetch(BREVO, {
      method: "POST",
      headers: { "api-key": env.BREVO_API_KEY, "content-type": "application/json", accept: "application/json" },
      // NOTIFY_SINCE, the day it asked: the mark scripts/brevo-cleanup.mjs counts the policy's bounds from, and the
      // only contacts it touches.
      body: JSON.stringify({ email, includeListIds: [list], templateId: template, redirectionUrl: `${SITE}/${lang}/notify/confirmed/`,
        attributes: { NOTIFY_SINCE: new Date().toISOString().slice(0, 10) } }),
      signal: AbortSignal.timeout(TIMEOUT_MS),
    });
    status = answer.status;
    if (status < 200 || status > 299) code = await brevoCode(answer);
  } catch (error) {
    log("unreached", { error: error?.name ?? "error" });
    return go(lang, "failed");
  }
  if (status < 200 || status > 299) {
    log("failed", { status, code });
    return go(lang, "failed");
  }
  log("sent", { status });
  return go(lang, "sent");
}
