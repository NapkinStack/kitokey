# site — local instructions

> Only what is **specific to this module**. Its decision: [ADR-0019](../../docs/adr/0019-kitokey-com-a-static-site-built-by-astro-served-by-cloudflare-pages.md);
> its deliverable: cycle 11's D114.

## See the site (the decider's preview)

```sh
cd modules/site && npm ci && npm run dev
```

Then open <http://localhost:4321/fr/> and <http://localhost:4321/en/>. Nothing leaves the machine:
Astro's telemetry is off in every script.

## What this module does not do

- It runs one thing on a server, `functions/notify.js` (`POST /notify`, ADR-0021): the release's form, passed to
  Brevo's double opt-in. Every page stays static: `astro build` writes files, no adapter. Its secret,
  `BREVO_API_KEY`, and the list's and template's ids are the Pages project's variables, never the repository's.
- It writes no policy of its own: `/privacy/` is `docs/privacy.md`, read at build time.
- It asks nothing of another domain: no analytics, no CDN, no font from another host, no embed.

## Internal conventions nobody could guess

- **One file reaches out of the module**: `site.config.mjs` — `privacySource` (`docs/privacy.md`), and for the
  landing's test alone, `installSource` (the repository's `README.md`, no module's). No other file
  reaches out of `modules/site/` (`tests/space.test.mjs`).
- **Every sentence lives in `src/lib/words.mjs`** — the landing's in `src/lib/landing.mjs` —, in English and French, with `keeps`: what keeps its
  promise, a merged deliverable or a test, as `docs/store/listing.md` does. The voice:
  `modules/shell/docs/voice.md`.
- **The look is proposal D's** (the decider, 2026-11-25): light only, the titles in Bricolage Grotesque served from
  `public/fonts/` with its licence beside it, the text in the system's font, the aurora mesh behind every page.
  `src/layouts/Page.astro` is every page's frame: a page passes `kind` for its `<main>`.
- **Preview, then production** (the decider, 2026-11-28): `main` deploys to the preview (`main.kitokey.pages.dev`,
  behind Access); `site-production`, kitokey.com, moves only by a fast-forward on the decider's « publie »
  (`kitokey-team/promote-site.sh`). A site pull request merges after the decider's local look at it
  (`kitokey-team/site-preview.sh`). The root is a 301 to `/en/` (`public/_redirects`), no page.
- **Open since 2026-11-28** (the decider): `open` in `site.config.mjs`, never a build's variable; `robots.txt`
  lets every robot in and names the sitemap; no page but the 404 is `noindex` (`indexed={false}`); `tests/open.test.mjs`.

## Business invariants

- No request to another host from any page (`tests/own-domain.test.mjs`, `_headers`' CSP).
- `/en/privacy/` and `/fr/privacy/` say their half of the source, word for word (`tests/privacy.test.mjs`).
- The release list's bounds (an unconfirmed address 4 months at most; the list deleted right after the release's
  message, 12 months at most) are held by the decider by hand, each quarter, with the cutoffs 28 days and 9 months on
  the `NOTIFY_SINCE` mark `/notify` sends (ADR-0021).
- `/notify` calls Brevo only for a well-formed request, once, with a timeout, and logs neither the address nor the
  key (`tests/notify.test.mjs`).

## Known traps

- The tests that open a page need a Chromium the machine has (`CHROME_PATH`, else the usual paths); in a
  sandbox that refuses `socket()`, Chromium aborts: run them where the attest runs.
- A path in `site.config.mjs` resolves from the module's folder, where npm runs every script: run them from there.

## Non-standard commands

The picture a link shows (Open Graph, `public/og/og-<lang>.png`), drawn again when the hero's promise changes:
`python3 scripts/og.py` (Pillow and ImageMagick's `magick`, on the person's machine; the build does not draw it).

Out of the repository, whole (D114, criterion 1):

```sh
git subtree split --prefix=modules/site -b site-alone
git worktree add ../site-alone site-alone && cd ../site-alone
npm ci && PRIVACY_SOURCE=/path/to/privacy.md INSTALL_SOURCE=/path/to/README.md npm test
```
