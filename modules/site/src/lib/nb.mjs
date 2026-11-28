// One function keeps each compound word whole (the coordinator, 2026-11-28: « revenez- / y » broke at its hyphen):
// in a page's text, outside code, scripts, styles, drawings and the head, a word joined by ASCII hyphens is wrapped
// in <span class="nb">, which never breaks (white-space: nowrap, site.css). The hyphen stays the ASCII one: the word
// one searches, copies, and the titles' face draws (its latin cut holds no U+2011).
const SKIP = new Set(["head", "pre", "code", "script", "style", "svg", "textarea", "title"]);
const WORD = /[\p{L}\p{N}’]+(?:-[\p{L}\p{N}’]+)+/gu;

export function keepWhole(html) {
  const parts = html.split(/(<[^>]+>)/);
  const open = [];
  for (let i = 0; i < parts.length; i++) {
    const part = parts[i];
    if (part.startsWith("<")) {
      const tag = part.match(/^<\/?([a-zA-Z][\w-]*)/)?.[1]?.toLowerCase();
      if (!tag || part.endsWith("/>")) continue;
      if (part.startsWith("</")) {
        if (open.at(-1) === tag) open.pop();
      } else if (SKIP.has(tag)) open.push(tag);
      continue;
    }
    if (open.length) continue;
    parts[i] = part.replace(WORD, (word) => `<span class="nb">${word}</span>`);
  }
  return parts.join("");
}

// The build's last step: every page of dist/ read once and written back, its compound words kept whole.
export const wholeWords = () => ({
  name: "kitokey-whole-words",
  hooks: {
    "astro:build:done": async ({ dir }) => {
      const { readdir, readFile, writeFile } = await import("node:fs/promises");
      const { fileURLToPath } = await import("node:url");
      const { join } = await import("node:path");
      const walk = async (folder) => {
        for (const entry of await readdir(folder, { withFileTypes: true })) {
          const path = join(folder, entry.name);
          if (entry.isDirectory()) await walk(path);
          else if (entry.name.endsWith(".html")) await writeFile(path, keepWhole(await readFile(path, "utf8")));
        }
      };
      await walk(fileURLToPath(dir));
    },
  },
});
