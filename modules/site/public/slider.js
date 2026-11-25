// kitokey.com's hero carousel (D114, proposal D). Without this file the slides still swipe (CSS scroll-snap)
// and each shows its clip's poster. With it: arrow keys, a calm crossfade on a fine pointer, and on screens of
// 700 px and more the clip itself (AV1 WebM, else H.264 MP4), each fetched as its slide approaches, in the app's
// dark theme under the moon where it was filmed so, else the light one in a dark window. Each clip
// plays once per view, then the next slide comes 0.6 s later, as it does when a clip fails; the drawn call of
// slide 1 stays 3 s, a poster alone 3.5 s. WCAG 2.2.2: hover or focus inside pauses the slide and its clip,
// autoplay never starts under reduced motion (posters only), and rests on slide 1 after two full cycles.
(() => {
  "use strict";
  const slider = document.querySelector(".slider");
  if (!slider) return;
  const track = slider.querySelector(".slides");
  const slides = [...slider.querySelectorAll(".slide")];
  const reduce = matchMedia("(prefers-reduced-motion: reduce)");
  const wide = matchMedia("(min-width: 700px)");
  const fade = matchMedia("(hover: hover) and (pointer: fine)").matches;
  const HOLD = 600, FIRST = 3000, STILL = 3500, CYCLES = 2;
  let at = 0, timer = null, seen = false, hover = false, held = false, shown = 0;
  if (fade) slider.classList.add("fade");

  const still = () => reduce.matches || held || hover || !seen;
  const video = (i) => slides[i].querySelector("video");
  const schedule = () => {
    clearTimeout(timer);
    if (still() || video(at)) return; // a clip's « ended » moves on
    timer = setTimeout(() => go(at + 1), at === 0 ? FIRST : STILL);
  };
  const mark = (n) => {
    at = n;
    ensure(n); ensure((n + 1) % slides.length);
    if (fade) slides.forEach((s, i) => { s.classList.toggle("is-active", i === n); s.inert = i !== n; });
    slides.forEach((s, i) => {
      const v = video(i);
      if (!v) return;
      if (i === n && seen && !reduce.matches && !held) { v.currentTime = 0; v.play().catch(() => {}); }
      else v.pause();
    });
    schedule();
  };
  const go = (n) => {
    const i = (n + slides.length) % slides.length;
    if (i === 0 && n >= slides.length && ++shown >= CYCLES) held = true;
    if (!fade) track.scrollTo({ left: slides[i].offsetLeft - track.offsetLeft, behavior: reduce.matches ? "auto" : "smooth" });
    mark(i);
  };
  track.addEventListener("keydown", (e) => {
    if (e.key === "ArrowRight" || e.key === "ArrowLeft") { e.preventDefault(); go(at + (e.key === "ArrowRight" ? 1 : -1)); }
  });
  // Hover or focus inside pauses the slide and its clip (WCAG 2.2.2); leaving goes on where it was.
  const pause = () => { hover = true; clearTimeout(timer); video(at)?.pause(); };
  const resume = () => {
    hover = false;
    const v = video(at);
    if (v && (v.ended || v.dataset.failed) && !still()) timer = setTimeout(() => go(at + 1), HOLD);
    else if (v && !still()) v.play().catch(() => {});
    else schedule();
  };
  slider.addEventListener("pointerenter", pause);
  slider.addEventListener("pointerleave", resume);
  slider.addEventListener("focusin", pause);
  slider.addEventListener("focusout", resume);
  reduce.addEventListener("change", () => mark(at));

  // The clips and their posters come as their slide approaches — the one shown and the next —, never all at
  // load: stacked by the crossfade, every lazy poster would otherwise be on screen at once. A clip is added only
  // where it plays (700 px and more, motion allowed), so a phone or a reduced-motion reader gets posters alone.
  const play = wide.matches && !reduce.matches;
  const ready = new Set();
  if (fade) slides.forEach((s, i) => {
    const img = s.querySelector("img.still");
    if (img && i > 1) { img.dataset.src = img.getAttribute("src"); img.removeAttribute("src"); }
  });
  // The clips' theme: the app's dark clips under the moon where they exist, else the light ones.
  const theme = (s) => (document.documentElement.dataset.screens === "dark" && "dark" in s.dataset ? "dark" : "light");
  const named = (s, ext) => `${s.dataset.clip}-${theme(s)}.${ext}`;
  function ensure(index) {
    const s = slides[index];
    if (!s.dataset.clip || ready.has(index)) return; // the drawn call has no clip
    ready.add(index);
    dress(index);
  }
  function dress(index) {
    const s = slides[index];
    const img = s.querySelector("img.still");
    if (img) { delete img.dataset.src; img.src = named(s, "jpg"); }
    if (!play) return;
    const old = s.querySelector("video") || img;
    if (old.dataset.theme === theme(s)) return;
    const v = document.createElement("video");
    Object.assign(v, { muted: true, playsInline: true, preload: "none", poster: named(s, "jpg") });
    v.setAttribute("muted", "");
    v.dataset.theme = theme(s);
    v.setAttribute("aria-label", old.getAttribute("aria-label") || old.alt);
    v.width = old.width; v.height = old.height;
    const next = () => { if (index === at && !still()) timer = setTimeout(() => go(at + 1), HOLD); };
    for (const ext of ["webm", "mp4"]) {
      const source = document.createElement("source");
      source.src = named(s, ext); source.type = ext === "webm" ? 'video/webm; codecs="av01.0.05M.08"' : "video/mp4";
      v.append(source);
    }
    // A clip that fails to load leaves its poster and lets the carousel go on.
    v.lastElementChild.addEventListener("error", () => { v.dataset.failed = "1"; next(); });
    v.addEventListener("ended", next);
    old.replaceWith(v);
  }
  // The moon or the sun pressed: each slide already fetched takes the other theme's poster and clip.
  document.addEventListener("kitokey:screens", () => { for (const i of ready) dress(i); mark(at); });


  if ("IntersectionObserver" in window) {
    if (!fade) {
      const io = new IntersectionObserver((es) => {
        for (const e of es) if (e.isIntersecting && slides.indexOf(e.target) !== at) mark(slides.indexOf(e.target));
      }, { root: track, threshold: 0.6 });
      slides.forEach((s) => io.observe(s));
    }
    new IntersectionObserver(([e]) => { const was = seen; seen = e.isIntersecting; if (seen !== was) mark(at); }, { threshold: 0.3 }).observe(slider);
  }
  mark(0);
})();
