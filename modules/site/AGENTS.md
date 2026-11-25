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

- It runs nothing on a server: `astro build` writes static files, no adapter, no function.
- It writes no policy of its own: `/privacy/` is `docs/privacy.md`, read at build time.
- It asks nothing of another domain: no analytics, no CDN, no font from another host, no embed.

## Internal conventions nobody could guess

- **One path out of the module**: `site.config.mjs`, `privacySource` (`docs/privacy.md`). No other file
  reaches out of `modules/site/` (`tests/space.test.mjs`).
- **Every sentence lives in `src/lib/words.mjs`** — the landing's in `src/lib/landing.mjs` —, in English and French, with `keeps`: what keeps its
  promise, a merged deliverable or a test, as `docs/store/listing.md` does. The voice:
  `modules/shell/docs/voice.md`.
- **The look is proposal D's** (the decider, 2026-11-25): light only, the titles in Bricolage Grotesque served from
  `public/fonts/` with its licence beside it, the text in the system's font, the aurora mesh behind every page.
  `src/layouts/Page.astro` is every page's frame: a page passes `kind` for its `<main>`.
- **Closed until the decider opens it**: every page `noindex`, `robots.txt` `Disallow: /`. `SITE_OPEN=1`
  builds the opening's site; the opening's commit makes it the default and changes `tests/closed.test.mjs`.

## Business invariants

- No request to another host from any page (`tests/own-domain.test.mjs`, `_headers`' CSP).
- `/en/privacy/` and `/fr/privacy/` say their half of the source, word for word (`tests/privacy.test.mjs`).

## Known traps

- The tests that open a page need a Chromium the machine has (`CHROME_PATH`, else the usual paths); in a
  sandbox that refuses `socket()`, Chromium aborts: run them where the attest runs.
- A path in `site.config.mjs` resolves from the module's folder, where npm runs every script: run them from there.

## Non-standard commands

Out of the repository, whole (D114, criterion 1):

```sh
git subtree split --prefix=modules/site -b site-alone
git worktree add ../site-alone site-alone && cd ../site-alone
npm ci && PRIVACY_SOURCE=/path/to/privacy.md npm test
```
