// The one path out of this module: the privacy policy's source, docs/privacy.md of the repository
// (D114, criterion 1). Split out of the repository by `git subtree split`, the build takes it from
// PRIVACY_SOURCE instead.
import { resolve } from "node:path";

export const privacySource =
  resolve(process.env.PRIVACY_SOURCE ?? "../../docs/privacy.md");
// Relative to the module's folder, where npm runs every script.

// Open (D114, criterion 6; the decider's word relayed by the coordinator, 2026-11-28): robots.txt lets every robot in
// and names the sitemap, no page but the 404 asks not to be indexed. Opened here, in the repository, never by a build's
// variable.
export const open = true;

export const languages = ["en", "fr"];

// Where the policy's relative links (support.md, …) point once on the site: the source's folder on GitHub.
export const privacyLinks = process.env.PRIVACY_LINKS ?? "https://github.com/NapkinStack/kitokey/blob/main/docs/";
export const privacyHistory = process.env.PRIVACY_HISTORY ?? "https://github.com/NapkinStack/kitokey/commits/main/docs/privacy.md";

// Where the landing's install commands come from, read by its test alone (D114): the repository's README.md, which
// no module owns. Split out of the repository, INSTALL_SOURCE names it.
export const installSource = resolve(process.env.INSTALL_SOURCE ?? "../../README.md");
