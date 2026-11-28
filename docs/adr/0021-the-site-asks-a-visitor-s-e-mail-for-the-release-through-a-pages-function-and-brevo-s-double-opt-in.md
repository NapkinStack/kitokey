# ADR-0021 — The site asks a visitor's e-mail for the release, through a Pages Function and Brevo's double opt-in

> The framework's words: `.nstack/handbook/glossary.md`.

- **Status**: Accepted by the decider, 2026-11-28 (relayed by the coordinator); proposed by marc (session
  0dfe2d51-5e28-4fd0-a8c4-964470174259) the same day
- **Date**: 2026-11-28
- **Decision makers**: the decider, `@napkinstack-admin`; the form on the site and the criticality chosen by them
- **Scope**: module `site` — amends [ADR-0019](0019-kitokey-com-a-static-site-built-by-astro-served-by-cloudflare-pages.md)
  on two points: a server function, and the module's criticality
- **Reversibility**: easy — one function and one form; the list is deleted at the release

---

## Context

Kitokey is not in the Microsoft Store yet; the site's Store button says so and leads nowhere. The decider wants
it to become « Me prévenir de la sortie », a form that keeps a visitor's e-mail address to tell them once, at the
release (the decider, 2026-11-28, B4 of the site's follow-ups). ADR-0019 built the site without a server ("nothing to compute per
request; a server is a cost and a surface") and set `criticality: prototype` because the site "collects nobody's
data". A form that keeps an address breaks both, so it needs its own decision.

## Problem

Where does the address go, through what, and with what guarantees, so that a visitor who asks is told once and
nobody else is ever mailed?

## Constraints

- The GDPR: a consent that can be shown and withdrawn (art. 6(1)(a), 7), a processor under contract (art. 28),
  the data kept in the European Union, the visitor told what is kept and why (art. 13).
- No request to another domain from a page (ADR-0019, `tests/own-domain.test.mjs`): the browser loads nothing
  from the provider.
- No secret in the repository (`security.md` SEC1); a Cloudflare secret holds the provider's key.
- No account, contract or key is an agent's: the decider makes them (*What the decider does*, below).
- 0 €: the decider's, for the site (ADR-0019).

---

## Prior art

**Dominant convention of the field:** a "notify me" list kept by an e-mail provider, with double opt-in (the
address joins the list only once its owner clicks a link sent to it), the form either hosted by the provider or
posted by the site's own server to the provider's API.

**References examined:**

| Reference | What it does | Applicable here? |
|---|---|---|
| [Brevo](https://www.brevo.com/) (Sendinblue SAS, Paris) | Contact lists, campaigns; servers in the EU ([data location](https://help.brevo.com/hc/en-us/articles/360001005510)); an API creating a contact by double opt-in, `POST /v3/contacts/doubleOptinConfirmation` with the list, the confirmation template and the page to land on ([reference](https://developers.brevo.com/reference/create-doi-contact)); hosted forms too | Yes — chosen |
| [MailerLite](https://mailerlite.com/legal/security-statement) (Lithuania) | The same, data in Google's EU regions; double opt-in for API sign-ups is a separate setting ([help](https://mailerlite.com/help/how-to-use-double-opt-in-when-collecting-subscribers)) | Yes — second choice |
| [Buttondown](https://buttondown.com/legal/subprocessors) | A newsletter service, data in AWS us-west-2 | No — not kept in the EU |
| Brevo's hosted form, the button a link to it | No code of ours; the visitor fills the form on Brevo's page | Weighed, rejected below |
| [Cloudflare Pages Functions](https://developers.cloudflare.com/pages/functions/) | A file under `functions/` answers a route of the Pages project; a secret set on the project is read in `context.env` ([bindings](https://developers.cloudflare.com/pages/functions/bindings/)) | Yes — the host's own way to run a few lines per request |
| [Cloudflare's rate limiting rules](https://developers.cloudflare.com/waf/rate-limiting-rules/) | One rule on the free plan, matching a path, counted over 10 s | Yes — on `/notify` |

---

## Options considered

### Option 1 — The site's own form, posted to a Pages Function that calls Brevo
- Description: the Store button opens a form of one field on the page; it posts to `/notify` on kitokey.com; a
  function checks it and asks Brevo to send the confirmation; Brevo keeps the list.
- Advantages: the visitor stays on kitokey.com, in the site's words, look and language; the browser talks to
  kitokey.com alone.
- Drawbacks: a server function and a secret, where ADR-0019 had none: a surface to guard (abuse, logs).
- Cost of setup / of maintenance / **of getting out**: a day / Brevo's API, read when it changes / remove
  `functions/` and the form; export or delete the list in Brevo.

### Option 2 — Brevo's hosted form, the button a link to it
- Description: the button links to a form Brevo hosts; no code of ours.
- Advantages: no function, no secret, no rate limit to set.
- Drawbacks: the visitor leaves kitokey.com for a page of Brevo's, in Brevo's frame, to give their address.
- Cost of getting out: change one link.

### Option 3 — Do nothing
- The button keeps saying « Bientôt dans le Microsoft Store » and leads to the price; a visitor who wants to know
  must come back.

---

## Decision

Option 1, the decider's, 2026-11-28, knowing it costs a function and a secret.

- **The form.** The Store buttons, « Me prévenir de la sortie », lead to a form in the price's card, where the
  Store's line is: one field, the address, the sentence saying what it is kept for, and a link to the privacy
  page. In the page rather than in a dialog: it needs no script and has no focus to manage, as the newsletter
  forms of most product pages. It posts to `/notify`, the site's own origin: `_headers`' CSP becomes
  `form-action 'self'`, nothing else in it changes.
- **The function**, `modules/site/functions/notify.js`. It takes only a POST from the site's own page
  (`Sec-Fetch-Site: same-origin`, or the site's `Origin` from a browser without Fetch Metadata); another site's
  form gets a 403, since it could make the site mail a confirmation to any address. It takes only a form-encoded
  body of at most 2 KB, counted in bytes; it refuses an address longer than 254 characters or not of the form
  `local@domain.tld`, and a filled honeypot field (`hp_field`, a name no autofill fills), without calling Brevo. Otherwise it asks Brevo's double opt-in endpoint, with an explicit timeout of
  5 s and no retry (the visitor may send again; Brevo answers the same for an address already there). Brevo's confirmation link lands on the
  function's fixed origin, `https://kitokey.com` (held equal to `astro.config.mjs`'s `site` by the test), never on
  the request's host. It answers a 303 to a page of the site: « Vérifiez votre boîte » when Brevo accepted, a generic failure otherwise; the
  detail goes to the log. Its log line is structured, says the outcome and Brevo's status, never the address nor
  the key.
- **The secret.** `BREVO_API_KEY`, set by the decider on the Pages project, production only. Without it (the
  preview) the function says the generic failure and calls nothing. The list's and the template's ids are not
  secrets but belong to the same account: plain variables of the Pages project, `BREVO_LIST_ID` and
  `BREVO_TEMPLATE_ID`, so that the decider sets the three together and no pull request waits on them.
- **Abuse.** The honeypot; one rate limiting rule on `/notify` per IP (the free plan's one rule); double opt-in,
  so that an address typed by someone else receives one mail and never joins the list.
- **The list.** Only confirmed addresses; one campaign at the release, then the whole list is deleted the same day.
  An address not confirmed is deleted at the latest 4 months after the request; if there is no release, an address
  is deleted at the latest 12 months after the sign-up (`docs/privacy.md`, *This website*, which also names every right under consent and the CNIL). Brevo's
  confirmation link expires after 30 days ([its help](https://help.brevo.com/hc/en-us/articles/208733449)), but its
  documentation does not say what it keeps of an address created through the API and never confirmed, nor does it
  delete a list's contacts after a time. **The decider holds these bounds by hand**, each quarter (the decider,
  2026-11-28: few sign-ups are expected before the launch; the coordinator keeps the reminder). The promise rests on
  the run's cutoffs (the coordinator, 2026-11-28): each run deletes every unconfirmed address older than 28 days, within
  the confirmation link's 30, so at worst 28 days plus a quarter of at most 92, 120 days after the request, which no
  4 months fall short of, February included (nora's remark); and every sign-up whose
  mark is more than 9 months old, so at worst 12 months. `/notify` marks the
  contact `NOTIFY_SINCE`, a text attribute, the day it asked: in Brevo, a filter on it finds this form's sign-ups
  alone, never a contact of the account's other uses (step 7). A contact who withdrew by the link stays on Brevo's
  list not to write to, which the policy says.
- **The criticality.** The module now holds personal data: `criticality: standard`, the decider's, 2026-11-28,
  knowing it costs a verifier's sheet on each site pull request (`.nstack/handbook/05-workflow.md` §7).

ADR-0019 stands for the rest: the pages stay static, no analytics, no request to another domain from a page.

---

## Success criterion

> We will consider this was the right call if, **by 2027-01-31**, **at least 1 % of the landings' visitors who
> reached the form confirm their address**, with **no address in a log** and **no mail sent but the
> confirmation and the release's**.

To read it: Cloudflare's own count of visits to the landings (the host's, no script of ours) against the number
of confirmed contacts in Brevo's list. What we do if it is not met: revisit — Option 2 costs one link, or the form
goes.

---

## Consequences

**Rejected as accepted risks** (the coordinator, 2026-11-28, on nora's reading): a cross-site POST, a host-derived
link, a bound in characters, an autofillable trap; each is fixed above and tested.

**Positive:** a visitor who is interested now has a way to be told, in the site's own words, and the
addresses stay in the EU under a processor's contract.

**Negative and accepted debt:** a function, a secret and a WAF rule to keep; Brevo's API to follow; a
verifier's sheet on every site pull request.

**Impacts on other modules or contracts:** none. `docs/privacy.md`, read by the site, gains the paragraph that
says what is kept.

**Rule to automate:** `tests/notify.test.mjs` — the function never calls Brevo for an invalid, empty, overlong
or honeypot request, never writes the address or the key in its log or its answer, and answers with the site's
pages; `tests/own-domain.test.mjs` keeps every page free of another domain.

---

## What the decider does (no agent's: an account, a contract, a secret)

1. A Brevo account for the company, free plan; accept its data processing agreement.
2. kitokey.com authenticated as a sender: Brevo's DKIM and DMARC records in Cloudflare's DNS.
3. The list « Kitokey — sortie », the contact attribute `NOTIFY_SINCE` (text), and a double opt-in template
   (sender support@kitokey.com), with the words written in the pull request.
4. An API key, set as the secret `BREVO_API_KEY` on the Pages project, production only.
5. The list's and the template's ids, set as the plain variables `BREVO_LIST_ID` and `BREVO_TEMPLATE_ID` on the
   Pages project, production only.
6. A rate limiting rule on the path `/notify` in Cloudflare's WAF.
7. Each quarter, by hand: delete the contacts marked `NOTIFY_SINCE` more than 28 days ago that are not in the list
   (unconfirmed), and the list's contacts marked more than 9 months ago. At the release: the one campaign, then the
   list and its contacts deleted the same day.

---

## Rejected alternatives

| Alternative | Why not |
|---|---|
| Brevo's hosted form, the button a link to it | The visitor would leave kitokey.com, for a page in Brevo's frame, to give their address; the decider wants them to stay on the site (2026-11-28) |
| A Worker and a database of our own, sending the mails ourselves | Builds what a provider does — tokens, expiry, sending, unsubscribing — and still needs one to send (`security.md` SEC5's spirit, law 3) |
| A provider outside the EU (Buttondown) | The addresses would leave the EU |
| A daily job deleting past the bounds (a script and a GitHub Actions workflow, written and tested on 2026-11-28) | It needs a second Brevo key, which reads the whole list, held by GitHub for a public repository's Actions, for a list expected small before the launch; the decider prefers to hold the bounds by hand (2026-11-28). **Revisit** if the confirmed sign-ups pass about 500, or a quarter's clean-up is missed |
| The form posting to Brevo's domain from the page | A request to another domain from the page, which ADR-0019 and its test forbid |
