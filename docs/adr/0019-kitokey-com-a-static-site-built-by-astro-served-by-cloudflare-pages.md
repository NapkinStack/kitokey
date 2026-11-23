# ADR-0019 — kitokey.com, a static site built by Astro, served by Cloudflare Pages

> The framework's words: `.nstack/handbook/glossary.md`.

- **Status**: Accepted — by the decider's approval of the framing pull request (#1143), 2026-11-23; proposed by
  sam (session wholeheard-sam-a2), framer of D114, the same day
- **Date**: 2026-11-23
- **Decision makers**: the decider, `@napkinstack-admin`; the host decided by them, 2026-11-23
- **Scope**: module `site` — its creation, its stack, its host, its criticality
- **Reversibility**: easy — the build is a folder of static files

---

## Context

The Microsoft Store asks a privacy policy's address at submission ([Store Policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies)
10.5.1, read 2026-11-23). The policy is written ([`docs/privacy.md`](../privacy.md), D103) and linked on GitHub.
The decider wants it on kitokey.com, Kitokey's shop window, with a landing that is professional and modern, playful,
free of advertising, animated and indexable by search engines (the decider, 2026-11-23, cycle 11's D114). The domain's registrar and DNS are Cloudflare's;
the decider chose Cloudflare Pages, no server, 0 €. No module of the repository serves the web.

## Problem

Which module, which tool and which host serve a bilingual, indexable, animated site of a few pages, with no
third party and no cost?

## Constraints

- No server and no cost: the decider's.
- No request to another domain — no analytics, no CDN font, no embed: the charter's local-first promise, which
  the site must not contradict on the page that states it.
- One source for the policy: `docs/privacy.md`.
- This repository is public: what is pushed is published. The decider chose it for the site, in a module of
  its own that can leave it, with nothing published before its time (2026-11-23).
- Written in English in the repository (`rules/always.md` 1); the site's sentences in French and English, to the
  voice guide.
- Closed to the public and not indexed until the decider validates it (D114, criterion 6), with nothing more
  than the host's own settings.

---

## Prior art

**Dominant convention of the field:** a static site generator writing HTML at build time, served by a static
host's CDN: no server code, pages indexable as written.

**References examined:**

| Reference | What it does | Applicable here? |
|---|---|---|
| [Astro](https://docs.astro.build/en/guides/internationalization/) | Components in HTML, CSS and JS; static HTML by default, no client JavaScript unless asked; built-in i18n routing; `@astrojs/sitemap` writes hreflang alternates ([its page](https://docs.astro.build/en/guides/integrations-guide/sitemap/)) | Yes — the coordinator's proposal |
| [Hugo](https://gohugo.io/content-management/multilingual/) | A single binary, Go templates, multilingual sites | Yes — fits as well; its templates are a language the team does not write elsewhere |
| [Eleventy](https://www.11ty.dev/docs/i18n/) | A JavaScript static site generator, several template languages | Yes — fewer built-ins: i18n and sitemap by hand |
| Hand-written HTML | No tool | For one page; two languages, a sitemap with alternates and a policy read from Markdown would be written by hand |
| Cloudflare Pages | Static hosting from a Git repository, preview deployments behind an access policy, build watch paths, 500 builds a month free ([limits](https://developers.cloudflare.com/pages/platform/limits/)) | Yes — the decider's host |
| Cloudflare Workers static assets | "Cloudflare recommends using Cloudflare Workers for new projects" ([Astro, *Deploy to Cloudflare*](https://docs.astro.build/en/guides/deploy/cloudflare/), read 2026-11-23) | Yes — the exit; Pages' preview access policy holds the closed phase in one setting |

---

## Options considered

### Option 1 — Astro, static output, on Cloudflare Pages
- Description: `modules/site`, an Astro project; `astro build` writes `dist/`; Pages builds it from the repository.
- Advantages: i18n routing, sitemap with hreflang, image formats at build time — all built in; components close to
  the shell's own HTML, CSS and JS (`modules/shell/ui`).
- Drawbacks: a Node toolchain and its dependencies, in a repository of Rust.
- Cost of setup / of maintenance / **of getting out**: a day / a dependency update now and then / the `dist/`
  folder is plain HTML: any host serves it.

### Option 2 — Hugo on Cloudflare Pages
- Description: the same, with Hugo's binary.
- Advantages: one binary, no `node_modules`.
- Drawbacks: Go templates, written nowhere else here; image processing and hreflang written by hand more often.
- Cost of getting out: the same.

### Option 3 — Do nothing
- The policy stays at its GitHub address, enough for the submission. No shop window, no landing: not what the
  decider asked.

---

## Decision

Option 1. A module `site`, `criticality: prototype`, `user_facing: true`: it holds no key, places no order,
moves no money and collects nobody's data (`.nstack/handbook/05-workflow.md` §7); the decider set the lightest criticality, 2026-11-23. Astro
with its static output and no adapter; its sitemap integration; Lighthouse CI and axe-core in its tests, run in
the workstation's Chromium. No analytics and no third party: `Content-Security-Policy: default-src 'self'` in
`_headers`. Everything under `modules/site/`, its own `package.json` and lockfile, nothing imported from another module,
`docs/privacy.md` read through one configuration value, so that `git subtree split` takes it out whole.
Served by Cloudflare Pages from `modules/site`, main built as a preview behind the project's access
policy until the decider opens it (cycle 11, D114's section and *What needs a person*).

The versions are pinned in the module's lockfile at its first pull request; on 2026-11-23 the registry gave
`astro` 7.4.13, `@astrojs/sitemap` 3.7.6, `@lhci/cli` 0.16.1, `axe-core` 4.14.1.

---

## Success criterion

> We will consider this was the right call if **the site is opened on kitokey.com with every check of D114 green
> and no request to another domain** before **2026-12-04**.

What we do if it is not: correct — what fails is named at cycle 11's closure; the privacy policy stays at its
GitHub address meanwhile.

---

## Consequences

**Positive:** the policy has one source and a public address of the project's own; the landing is indexable
and fast by construction (static HTML, no client script required).

**Negative and accepted debt:** a Node toolchain to keep current; a second copy of the app's colour tokens,
copied once, which may drift from `modules/shell/ui/window.css`.

**Impacts on other modules or contracts:** none consumed. The site reads `docs/privacy.md` and copies files the
shell's e2e wrote (D103's screenshots) into itself; neither is a contract.

**Rule to automate:** the module's tests: no request to another domain, the policy equal to its source,
noindex until the opening, axe-core AA, Lighthouse at least 95, no path out of the module but the policy's,
no landing sentence without the deliverable that keeps it.

---

## Rejected alternatives

| Alternative | Why not |
|---|---|
| A server-rendered site (an adapter, Pages Functions) | Nothing to compute per request; a server is a cost and a surface |
| Analytics, even cookieless | The decider ruled out advertising, 2026-11-23; the charter: nothing leaves without the user's choice; the page stating that cannot break it |
| Zero Trust's application on the production address for the closed phase | Five steps for what one setting does on previews (the decider asked for the simplest access control, 2026-11-23) |
| GitHub Pages | The domain and its DNS are Cloudflare's; the decider chose Pages |
| A private repository for the site | Weighed so that drafts stay unread before validation. Set aside by the decider, 2026-11-23: one repository is simpler, keeps `docs/privacy.md` the single source, and the same guards hold here — no price before it is announced, no promise without a merged deliverable, screenshots of invented meetings only, no key or analytics identifier, the hooks' secret scan. The way out stays open: `git subtree split --prefix=modules/site` gives a repository that builds alone |
