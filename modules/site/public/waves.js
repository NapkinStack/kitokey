// kitokey.com's background « A + C » (D114, proposal D, the decider's round 9 of 2026-11-25): 14 thin
// sound-wave lines behind the headline, moving for ever without visible repetition. Each line sums three sines with
// incommensurate periods whose phases drift at irrational rates, and its amplitude breathes slowly. Drawn at most 30
// times a second, only while on screen and the tab is visible; one still frame under prefers-reduced-motion; a small
// pause button in the hero stops it (WCAG 2.2.2).
(() => {
  "use strict";
  const copy = document.querySelector(".hero-copy");
  const hero = document.querySelector(".hero");
  if (!copy || !hero) return;
  const fr = document.documentElement.lang !== "en";
  const reduce = matchMedia("(prefers-reduced-motion: reduce)");
  const canvas = document.createElement("canvas");
  canvas.className = "waves";
  canvas.setAttribute("aria-hidden", "true");
  copy.prepend(canvas);
  const button = document.createElement("button");
  button.type = "button";
  button.className = "waves-pause";
  button.setAttribute("aria-pressed", "false");
  button.setAttribute("aria-label", fr ? "Mettre l’animation du fond en pause" : "Pause the background animation");
  button.innerHTML = "<i aria-hidden=\"true\"></i>";
  hero.append(button);
  const ctx = canvas.getContext("2d");
  const N = 14;
  const hue = ["34, 83, 207", "74, 98, 235", "111, 151, 255", "122, 92, 255", "164, 120, 255", "190, 140, 250", "255, 138, 122",
               "255, 170, 150", "63, 185, 126", "90, 200, 160", "111, 151, 255", "150, 175, 255", "122, 92, 255", "34, 83, 207"];
  // Per line: three wavelengths and drift rates, scaled by irrational numbers so nothing lines up again.
  const lines = Array.from({ length: N }, (_, k) => ({
    k: [1 / (170 + k * 23), 1 / (97 + k * 13.7), 1 / (311 + k * 17.3)],
    v: [0.37 + k * 0.0271, -0.23 - k * 0.0173 * Math.SQRT2, 0.11 + k * 0.011 * Math.PI],
    a: [16 + k * 2.2, 7 + (k % 5) * 1.7, 11 + (k % 3) * 2.9],
    p: k * 1.618,
  }));
  let w = 0, h = 0, t = 0, last = 0, raf = 0, onScreen = true, paused = false;

  const size = () => {
    const r = Math.min(window.devicePixelRatio || 1, 2);
    const box = canvas.getBoundingClientRect();
    w = box.width; h = box.height;
    canvas.width = Math.round(w * r); canvas.height = Math.round(h * r);
    ctx.setTransform(r, 0, 0, r, 0, 0);
    draw();
  };
  const draw = () => {
    ctx.clearRect(0, 0, w, h);
    ctx.lineWidth = 1.1;
    for (let i = 0; i < N; i++) {
      const L = lines[i];
      const breathe = 0.75 + 0.25 * Math.sin(t * 0.07 * (1 + i * 0.031) + L.p);
      const base = h * (0.5 + 0.034 * (i - (N - 1) / 2));
      ctx.beginPath();
      for (let x = 0; x <= w; x += 12) {
        const y = base + breathe * Math.sin((x / w) * Math.PI) *
          (L.a[0] * Math.sin(x * L.k[0] + t * L.v[0] + L.p) +
           L.a[1] * Math.sin(x * L.k[1] + t * L.v[1] + L.p * 2.3) +
           L.a[2] * Math.sin(x * L.k[2] + t * L.v[2] + L.p * 0.7));
        x ? ctx.lineTo(x, y) : ctx.moveTo(x, y);
      }
      ctx.strokeStyle = `rgb(${hue[i]})`;
      ctx.stroke();
    }
  };
  const running = () => onScreen && !document.hidden && !reduce.matches && !paused;
  const frame = (now) => {
    raf = 0;
    if (!running()) { last = 0; return; }
    if (!last || now - last >= 33) { t += last ? (now - last) / 1000 : 0; last = now; draw(); }
    raf = requestAnimationFrame(frame);
  };
  const sync = () => { if (running()) { if (!raf) raf = requestAnimationFrame(frame); } else if (raf) { cancelAnimationFrame(raf); raf = 0; last = 0; } };

  button.addEventListener("click", () => { paused = !paused; button.setAttribute("aria-pressed", String(paused)); sync(); });
  new IntersectionObserver(([e]) => { onScreen = e.isIntersecting; sync(); }).observe(canvas);
  new ResizeObserver(size).observe(canvas);
  document.addEventListener("visibilitychange", sync);
  reduce.addEventListener("change", () => { size(); sync(); });
  button.hidden = reduce.matches;
  size();
  sync();
})();
