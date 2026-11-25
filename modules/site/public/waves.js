// kitokey.com's background « A + C » (D114, proposal D, the decider's choice of 2026-11-25): nine thin sound-wave
// lines behind the headline, over the mesh. They drift for five seconds after the page opens, then move only as the
// page scrolls (nothing loops by itself, WCAG 2.2.2); one still frame under prefers-reduced-motion. A frame is drawn
// only while the lines are on screen and the tab is visible.
(() => {
  "use strict";
  const copy = document.querySelector(".hero-copy");
  if (!copy) return;
  const reduce = matchMedia("(prefers-reduced-motion: reduce)");
  const canvas = document.createElement("canvas");
  canvas.className = "waves";
  canvas.setAttribute("aria-hidden", "true");
  copy.prepend(canvas);
  const ctx = canvas.getContext("2d");
  const colours = ["34, 83, 207", "91, 108, 255", "122, 92, 255", "164, 120, 255", "255, 138, 122", "255, 170, 150", "63, 185, 126", "111, 151, 255", "150, 175, 255"];
  let w = 0, h = 0, phase = 0, raf = 0, onScreen = true, start = 0;

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
    ctx.lineWidth = 1.2;
    const n = colours.length;
    for (let k = 0; k < n; k++) {
      const amp = 14 + k * 5;
      const base = h * (0.5 + 0.045 * (k - (n - 1) / 2));
      ctx.beginPath();
      for (let x = 0; x <= w; x += 10) {
        const y = base + Math.sin(x / (200 + k * 28) + phase * (1 + k * 0.12) + k * 1.3) * amp * Math.sin((x / w) * Math.PI);
        x ? ctx.lineTo(x, y) : ctx.moveTo(x, y);
      }
      ctx.strokeStyle = `rgb(${colours[k]})`;
      ctx.stroke();
    }
  };
  const visible = () => onScreen && !document.hidden;
  const intro = (now) => {
    raf = 0;
    if (!start) start = now;
    if (reduce.matches || !visible() || now - start > 5000) return;
    phase = (now - start) / 1000 * 0.5;
    draw();
    raf = requestAnimationFrame(intro);
  };
  const onScroll = () => {
    if (reduce.matches || !visible() || raf) return;
    raf = requestAnimationFrame(() => { raf = 0; phase = 2.5 + scrollY / 260; draw(); });
  };

  new IntersectionObserver(([e]) => { onScreen = e.isIntersecting; }).observe(canvas);
  new ResizeObserver(size).observe(canvas);
  addEventListener("scroll", onScroll, { passive: true });
  size();
  if (!reduce.matches) raf = requestAnimationFrame(intro);
})();
