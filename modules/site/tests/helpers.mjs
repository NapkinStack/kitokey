import { createServer } from "node:http";
import { existsSync } from "node:fs";
import { readFile, readdir, stat } from "node:fs/promises";
import { join, extname } from "node:path";
import { fileURLToPath } from "node:url";

export const root = fileURLToPath(new URL("..", import.meta.url));
export const dist = join(root, "dist");

export async function pages(dir = dist) {
  const out = [];
  for (const name of await readdir(dir)) {
    const path = join(dir, name);
    if ((await stat(path)).isDirectory()) out.push(...(await pages(path)));
    else if (name.endsWith(".html")) out.push(path);
  }
  return out;
}

// A page's HTML as its words were written: the build wraps each compound word in <span class="nb"> to keep it
// whole (src/lib/nb.mjs, its own tests in v7-content and look-v7); a test of the words reads them unwrapped.
export const unwrapped = (html) => html.replace(/<span class="nb">([^<]*)<\/span>/g, "$1");

export function route(file) {
  return file.slice(dist.length).replace(/index\.html$/, "");
}

const types = { ".html": "text/html", ".css": "text/css", ".js": "text/javascript", ".xml": "application/xml",
  ".txt": "text/plain", ".png": "image/png", ".webp": "image/webp", ".avif": "image/avif", ".svg": "image/svg+xml",
  ".jpg": "image/jpeg", ".ico": "image/x-icon", ".json": "application/json" };

// Serves dist/ on 127.0.0.1, as Pages would: a folder answers with its index.html.
export async function serve() {
  const server = createServer(async (req, res) => {
    let path = join(dist, decodeURIComponent(new URL(req.url, "http://x").pathname));
    try {
      if ((await stat(path)).isDirectory()) path = join(path, "index.html");
      res.writeHead(200, { "content-type": types[extname(path)] ?? "application/octet-stream" });
      res.end(await readFile(path));
    } catch {
      res.writeHead(404).end();
    }
  });
  await new Promise((ok) => server.listen(0, "127.0.0.1", ok));
  return { server, origin: `http://127.0.0.1:${server.address().port}` };
}

// The machine's own Chromium or Chrome: playwright-core downloads no browser.
export const chromium = process.env.CHROME_PATH
  ?? ["/usr/bin/chromium-browser", "/usr/bin/chromium", "/usr/bin/google-chrome", "/usr/bin/google-chrome-stable"].find(existsSync);
