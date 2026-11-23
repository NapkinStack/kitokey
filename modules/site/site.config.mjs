// The one path out of this module: the privacy policy's source, docs/privacy.md of the repository
// (D114, criterion 1). Split out of the repository by `git subtree split`, the build takes it from
// PRIVACY_SOURCE instead.
import { resolve } from "node:path";

export const privacySource =
  resolve(process.env.PRIVACY_SOURCE ?? "../../docs/privacy.md");
// Relative to the module's folder, where npm runs every script.

// Closed until the decider opens it (D114, criterion 6): every page noindex, robots.txt « Disallow: / ».
// SITE_OPEN=1 builds the opening's site, for Lighthouse's SEO score alone.
export const open = process.env.SITE_OPEN === "1";

export const languages = ["en", "fr"];

// Where the policy's relative links (support.md, …) point once on the site: the source's folder on GitHub.
export const privacyLinks = process.env.PRIVACY_LINKS ?? "https://github.com/NapkinStack/kitokey/blob/main/docs/";
export const privacyHistory = process.env.PRIVACY_HISTORY ?? "https://github.com/NapkinStack/kitokey/commits/main/docs/privacy.md";
