// The privacy policy's two halves, read from its one source (site.config.mjs): from « ## English »
// to « ## Français », then to the end, each rendered as Astro renders Markdown.
import { createMarkdownProcessor } from "@astrojs/markdown-remark";
import { privacyLinks } from "../../site.config.mjs";

const heads = { en: "## English", fr: "## Français" };

export function section(source, lang) {
  const start = source.indexOf(`\n${heads[lang]}\n`);
  if (start < 0) throw new Error(`the policy has no « ${heads[lang]} » heading`);
  const body = source.slice(start + heads[lang].length + 2);
  const next = lang === "en" ? body.indexOf(`\n${heads.fr}\n`) : -1;
  // A link relative to the source's folder points to it on GitHub: the site has no such file.
  return (next < 0 ? body : body.slice(0, next)).replace(/\]\((?!https?:|mailto:|#)([^)]+)\)/g, (_, to) => `](${privacyLinks}${to})`);
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
