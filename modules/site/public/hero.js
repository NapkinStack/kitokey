// kitokey.com, site v7 (validated by the decider on 2026-11-26): the header's hairline once the page has scrolled,
// the compact bar's two meters, and the hero's background, a dot-matrix spectrogram of the two voices. Decorative:
// the canvas is aria-hidden. It runs at most 30 frames a second, only while the hero is on screen and the tab
// visible; under reduced motion it draws one still frame (WCAG 2.2.2).
(() => {
  "use strict";
  const root = document.documentElement;
  const reduce = matchMedia("(prefers-reduced-motion: reduce)");

  const top = document.querySelector(".top");
  if (top) {
    const onScroll = () => top.classList.toggle("scrolled", window.scrollY > 8);
    addEventListener("scroll", onScroll, { passive: true });
    onScroll();
  }

  // The bar's two meters, static, from the app's own trace: « Vous » in the accent, « Les autres » in their colour.
  for (const el of document.querySelectorAll("[data-meter]")) {
    const me = el.dataset.meter === "me";
    for (let i = 0; i < Number(el.dataset.n); i++) {
      const bar = document.createElement("i");
      bar.style.height = `${Math.max(2, Math.abs(Math.sin(i * (me ? 0.6 : 0.45))) * 12).toFixed(1)}px`;
      bar.style.background = `var(--track-${me ? "me" : "others"})`;
      el.append(bar);
    }
  }

  const hero = document.querySelector(".hero");
  const cv = hero?.querySelector(".voices");
  const card = hero?.querySelector(".scene .mcard");
  const pitch = hero?.querySelector(".pitch");
  if (!cv?.getContext || !card || !pitch) return;
  const ctx = cv.getContext("2d");
  const cols = {};
  let w = 0, h = 0, t = 4, last = 0, raf = 0, onScreen = true;
  const dark = () => root.dataset.site === "dark";
  const readCols = () => {
    const cs = getComputedStyle(root);
    cols.me = cs.getPropertyValue("--track-me").trim();
    cols.ot = cs.getPropertyValue("--track-others").trim();
    cols.base = cs.getPropertyValue("--text-secondary").trim();
  };
  // A quiet grid of dots over the hero; two bands of light run through it, « Vous » above, « Les autres » below. Time
  // runs left to right, so speech flows towards the minutes card; the voices take turns, and past the middle both
  // bands bend into the card's left edge: two voices becoming a page. Dots under the headline are dimmed, so the
  // words keep their contrast.
  const ease = (v) => { v = Math.max(0, Math.min(1, v)); return v < 0.5 ? 2 * v * v : 1 - (-2 * v + 2) ** 2 / 2; };
  const draw = () => {
    if (!w) return;
    const narrow = w < 1040, g = narrow ? 12 : 14, night = dark();
    const hb = hero.getBoundingClientRect(), cb = card.getBoundingClientRect(), pb = pitch.getBoundingClientRect();
    const x1 = narrow ? w : cb.left - hb.left + cb.width * 0.35, y0 = narrow ? 10 : 18;
    const y1 = narrow ? pb.top - hb.top + 150 : cb.top - hb.top + cb.height * 0.5;
    const px0 = pb.left - hb.left - 16, px1 = pb.right - hb.left + 16, py0 = pb.top - hb.top - 8, py1 = pb.bottom - hb.top + 8;
    ctx.clearRect(0, 0, w, h);
    for (let x = g / 2; x < w; x += g) {
      const u = x / x1, on = u <= 1;
      const cyl = y0 + (y1 - y0) * ease(u), spread = (narrow ? 20 : 46) * (1 - 0.8 * Math.min(1, u));
      const sig = (narrow ? 18 : 34) * (1 - 0.45 * Math.min(1, u)), sx = x * 0.011 - t * 0.5;
      const em = 0.15 + 0.85 * Math.max(0, Math.sin(sx * 0.55)) * (0.55 + 0.45 * Math.abs(Math.sin(sx * 2.9 + 0.4)));
      const eo = 0.15 + 0.85 * Math.max(0, Math.sin(sx * 0.55 + Math.PI)) * (0.55 + 0.45 * Math.abs(Math.sin(sx * 2.3 + 1.7)));
      const cm = cyl - spread, co = cyl + spread, edge = Math.min(1, 0.35 + x / (w * 0.15));
      for (let y = g / 2; y < h; y += g) {
        const under = x > px0 && x < px1 && y > py0 && y < py1 ? (narrow ? 0.85 : 0.6) : 1;
        const vf = Math.min(1, y / 40, (h - y) / 80);
        ctx.globalAlpha = (night ? 0.16 : 0.14) * vf;
        ctx.fillStyle = cols.base;
        ctx.fillRect(x - 0.6, y - 0.6, 1.2, 1.2);
        if (!on) continue;
        const tex = 0.72 + 0.28 * Math.sin(y * 0.21 + sx * 4.1) * Math.sin(y * 0.07 - sx * 1.3);
        const dm = (y - cm) / sig, dO = (y - co) / sig;
        const a = em * Math.exp(-dm * dm) * tex, b = eo * Math.exp(-dO * dO) * tex, e = Math.max(a, b);
        if (e < 0.05) continue;
        const r = 0.9 + 1.9 * e;
        ctx.globalAlpha = Math.min(1, e * (night ? 1.15 : 1.3) * (narrow ? 1.2 : 1)) * under * edge * vf;
        ctx.fillStyle = a >= b ? cols.me : cols.ot;
        ctx.fillRect(x - r, y - r, r * 2, r * 2);
      }
    }
    ctx.globalAlpha = 1;
  };
  const size = () => {
    const r = Math.min(devicePixelRatio || 1, 2), b = hero.getBoundingClientRect();
    w = b.width; h = b.height;
    cv.width = Math.round(w * r); cv.height = Math.round(h * r);
    ctx.setTransform(r, 0, 0, r, 0, 0);
    readCols(); draw();
  };
  const running = () => onScreen && !document.hidden && !reduce.matches && cv.offsetWidth > 0;
  const frame = (now) => {
    raf = 0;
    if (!running()) { last = 0; return; }
    if (!last || now - last >= 33) { t += last ? (now - last) / 1000 : 0; last = now; draw(); }
    raf = requestAnimationFrame(frame);
  };
  const sync = () => {
    if (running()) { if (!raf) raf = requestAnimationFrame(frame); }
    else if (raf) { cancelAnimationFrame(raf); raf = 0; last = 0; }
  };
  new IntersectionObserver((e) => { onScreen = e[0].isIntersecting; sync(); }).observe(hero);
  if ("ResizeObserver" in window) new ResizeObserver(size).observe(hero);
  document.addEventListener("visibilitychange", sync);
  reduce.addEventListener("change", () => { size(); sync(); });
  new MutationObserver(() => { readCols(); draw(); }).observe(root, { attributes: true, attributeFilter: ["data-site"] });
  size(); sync();
})();
