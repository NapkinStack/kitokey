// The legal notice's details (the decider, 2026-11-27): the publisher's, from the build's environment alone
// (Cloudflare Pages' variables), never in the repository; the host's, public. A build without one of them fails,
// naming it. `npm test` builds with LEGAL_TEST=1, each field then « TEST <its name> »; on Cloudflare Pages
// (CF_PAGES=1, set by Pages on every build) the tests' values are refused, given by LEGAL_TEST or by hand.
// No person's name: the publication's director is named by office unless PUBLICATION_DIRECTOR says otherwise.
export const FIELDS = ["LEGAL_NAME", "LEGAL_FORM", "LEGAL_CAPITAL", "LEGAL_REGISTRATION", "LEGAL_VAT", "LEGAL_SEAT", "LEGAL_CONTACT"];

const env = process.env;
const testing = env.LEGAL_TEST === "1";
const onPages = env.CF_PAGES === "1";

if (testing && onPages) throw new Error("legal notice: LEGAL_TEST is the tests' and is refused on Cloudflare Pages");

const read = (name) => (testing ? `TEST ${name}` : (env[name] ?? "").trim());
const missing = FIELDS.filter((name) => !read(name));
if (missing.length) throw new Error(`legal notice: ${missing.join(", ")} not set in the build's environment`);
const fake = FIELDS.filter((name) => /^TEST\b/i.test(read(name)));
if (onPages && fake.length) throw new Error(`legal notice: ${fake.join(", ")} hold a test value on Cloudflare Pages`);

export const publisher = Object.fromEntries(FIELDS.map((name) => [name, read(name)]));

// Named by office (the decider refuses their own details on the site); a name only if the build is given one.
export const director = (lang) => (env.PUBLICATION_DIRECTOR ?? "").trim() || (lang === "fr" ? "le président de C12C" : "the President of C12C");

// Cloudflare's own Terms of Use (cloudflare.com/website-terms, effective 2025-08-01, read 2026-11-27).
export const host = { name: "Cloudflare, Inc.", address: "101 Townsend St, San Francisco, CA 94107, USA", phone: "+1 (650) 319‑8930" };
