// Site v7 (the decider, 2026-11-26): no em dash, no ellipsis and no arrow in what a reader reads, in French or
// in English: each page's text and the words it gives a screen reader (alt, aria-label, title). The policy's
// own text is docs/privacy.md's, the one source (D114), outside this module: its article is left to that file.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { pages, route } from "./helpers.mjs";

const AI = /[—…←-⇿⟰-⟿⤀-⥿]|->|=>|\.\.\./;

export const read = (html) => {
  const body = html.replace(/<article class="policy">[\s\S]*?<\/article>/g, "").replace(/<script[\s\S]*?<\/script>/g, "").replace(/<style[\s\S]*?<\/style>/g, "").replace(/<svg[\s\S]*?<\/svg>/g, "");
  const said = [...body.matchAll(/\s(?:alt|aria-label|title|content)="([^"]*)"/g)].map(([, v]) => v);
  const text = body.replace(/<[^>]+>/g, " ").replace(/&amp;/g, "&").replace(/&#39;/g, "’").replace(/&quot;/g, '"');
  return [text, ...said].map((s) => s.replace(/\s+/g, " "));
};

test("no page says an em dash, an ellipsis or an arrow, in its text or to a screen reader", async () => {
  for (const file of await pages()) {
    for (const said of read(await readFile(file, "utf8"))) {
      const at = said.search(AI);
      assert.equal(at, -1, `${route(file)}: « ${said.slice(Math.max(0, at - 40), at + 40)} »`);
    }
  }
});

test("the sweep tells each mark from what it must leave alone", () => {
  for (const wrong of ["Un appel — puis", "Clients…", "Réglages → Avancé", "a -> b", "x => y", "trois...", "<p>a — b</p>"]) {
    assert.ok(read(`<p>${wrong}</p>`).some((s) => AI.test(s)), wrong);
  }
  for (const right of ["10–12 h", "Refaire › Avancé", "<script>a => b</script>", "<svg><path d=\"M1 2\"/></svg>"]) {
    assert.ok(read(right).every((s) => !AI.test(s)), right);
  }
});
