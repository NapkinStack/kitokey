// kitokey.com's theme (D114, the decider's round 9, 2026-11-25). Loaded in <head>, not deferred, so the page paints
// in its theme at once. Moon = dark site and dark app screens; sun = light. Follows the system until the visitor
// chooses; the choice is kept in localStorage, a per-viewer convenience: the page works without it.
(() => {
  "use strict";
  const root = document.documentElement;
  let saved = null;
  try { saved = localStorage.getItem("kitokey-theme"); } catch {}
  const system = matchMedia("(prefers-color-scheme: dark)");
  const apply = (theme) => {
    root.dataset.site = theme;
    root.dataset.screens = theme;
    document.dispatchEvent(new CustomEvent("kitokey:screens", { detail: theme }));
    const b = document.querySelector(".theme-toggle");
    if (!b) return;
    const dark = theme === "dark";
    b.setAttribute("aria-pressed", String(dark));
    b.setAttribute("aria-label", dark ? b.dataset.light : b.dataset.dark);
    b.title = b.getAttribute("aria-label");
  };
  apply(saved === "dark" || saved === "light" ? saved : system.matches ? "dark" : "light");
  system.addEventListener("change", () => { if (!saved) apply(system.matches ? "dark" : "light"); });
  document.addEventListener("DOMContentLoaded", () => {
    apply(root.dataset.site);
    const b = document.querySelector(".theme-toggle");
    if (!b) return;
    b.hidden = false;
    b.addEventListener("click", () => {
      saved = root.dataset.site === "dark" ? "light" : "dark";
      try { localStorage.setItem("kitokey-theme", saved); } catch {}
      apply(saved);
    });
  });
})();
