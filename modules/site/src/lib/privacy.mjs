// The privacy policy's two halves, read from its one source (site.config.mjs): from « ## English »
// to « ## Français », then to the end, each rendered as Astro renders Markdown — made a web page:
// its headings one level up under the page's h1, its relative links to GitHub, and the sentence on
// « this file's history » turned into a link to that history.
import { createMarkdownProcessor } from "@astrojs/markdown-remark";
import { privacyLinks, privacyHistory } from "../../site.config.mjs";

const heads = { en: "## English", fr: "## Français" };
const history = {
  en: [/Changes to this policy are in this file's history\./, `Every change to this policy is in [its history, on GitHub](${privacyHistory}).`],
  fr: [/Les\s+changements de cette politique sont dans l'historique de ce fichier\./, `Chaque changement de cette politique est dans [son historique, sur GitHub](${privacyHistory}).`],
};

export function section(source, lang) {
  const start = source.indexOf(`\n${heads[lang]}\n`);
  if (start < 0) throw new Error(`the policy has no « ${heads[lang]} » heading`);
  const body = source.slice(start + heads[lang].length + 2);
  const next = lang === "en" ? body.indexOf(`\n${heads.fr}\n`) : -1;
  let half = next < 0 ? body : body.slice(0, next);
  const [said, link] = history[lang];
  if (!said.test(half)) throw new Error(`the policy's ${lang} half no longer says where its changes are: update src/lib/privacy.mjs`);
  half = half.replace(said, link);
  return half
    .replace(/^(#{3,6}) /gm, (_, h) => `${h.slice(1)} `)
    // A link relative to the source's folder points to it on GitHub: the site has no such file.
    .replace(/\]\((?!https?:|mailto:|#)([^)]+)\)/g, (_, to) => `](${privacyLinks}${to})`);
}

// The day the policy last changed, as its first lines say it.
export function changed(source) {
  const day = source.match(/^Last changed \/ dernière modification: (\d{4}-\d{2}-\d{2})\./m)?.[1];
  if (!day) throw new Error("the policy no longer says when it last changed: update src/lib/privacy.mjs");
  return day;
}

let processor;
export async function half(source, lang) {
  processor ??= await createMarkdownProcessor({ syntaxHighlight: false });
  return (await processor.render(section(source, lang))).code;
}

// A page's words, tags and spacing aside: what the test compares with its half of the source.
export function words(html) {
  return html
    .replace(/<[^>]+>/g, " ")
    .replace(/&#x([0-9a-f]+);/gi, (_, h) => String.fromCodePoint(parseInt(h, 16)))
    .replace(/&#(\d+);/g, (_, d) => String.fromCodePoint(Number(d)))
    .replace(/&quot;/g, '"').replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&nbsp;/g, " ").replace(/&amp;/g, "&")
    .replace(/\s+/g, " ")
    .trim();
}
