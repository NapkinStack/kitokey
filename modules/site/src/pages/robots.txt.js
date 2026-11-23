// Closed until the decider opens it (D114, criterion 6); once open, every robot and the sitemap.
import { open } from "../../site.config.mjs";

export const GET = ({ site }) =>
  new Response(open
    ? `User-agent: *\nAllow: /\n\nSitemap: ${new URL("/sitemap-index.xml", site).href}\n`
    : "User-agent: *\nDisallow: /\n");
