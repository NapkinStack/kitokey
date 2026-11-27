// kitokey.com, site v7 (validated by the decider on 2026-11-26): the walkthrough. Four steps in a loop (record 3 s,
// confidential 3.5 s, the two voices 5 s, the minutes 6 s), playing only while the scene is visible and the tab shown,
// held while hovered or focused. A tab shows its step and the loop goes on from there; the arrows, Home and End move
// between the tabs (WAI-ARIA's tabs pattern). On a narrow screen and under reduced motion the scene is not displayed
// (walk.css): four static cards stand for it, and nothing here runs. #walk-1 to #walk-4 in the address shows that
// step at its end and holds it, for a capture.
(() => {
  "use strict";
  const walk = document.getElementById("walk");
  const stage = document.getElementById("wstage");
  if (!walk || !stage) return;
  const reduce = matchMedia("(prefers-reduced-motion: reduce)");
  const tabs = [...walk.querySelectorAll(".wtab")];
  const clock = document.getElementById("wtime");
  const lanes = [...stage.querySelectorAll("[data-lane]")];
  const meters = [...stage.querySelectorAll(".w-bar [data-meter]")];
  const tiles = [...stage.querySelectorAll(".w-call .ct")];
  const page = stage.querySelector(".w-page");
  const STEPS = [3000, 3500, 5000, 6000];
  const N = 44;
  let step = 1, held = false, visible = false, elapsed = 0, secs = 0, lt = 0, ph = 0, tick = 0, started = false;

  const bars = (el, n, side) => {
    for (let i = 0; i < n; i++) {
      const bar = document.createElement("i");
      bar.style.background = `var(--track-${side})`;
      el.append(bar);
    }
  };
  for (const lane of lanes) bars(lane, N, lane.dataset.lane === "me" ? "me" : "others");
  for (const meter of document.querySelectorAll(".walk-band [data-meter]")) bars(meter, Number(meter.dataset.n), meter.dataset.meter === "me" ? "me" : "others");

  // Who speaks alternates every 2.6 s: the speaking lane is tall, the other low, and the speaker's tile is lit, its
  // ring swelling with the voice.
  const paint = () => {
    const turn = Math.floor(ph / 2.6), mine = turn % 2 === 0, other = (turn >> 1) % 3;
    for (const lane of lanes) {
      const me = lane.dataset.lane === "me", loud = me === mine;
      [...lane.children].forEach((bar, i) => {
        const v = Math.abs(Math.sin(i * 0.47 + ph * (me ? 3.1 : 2.6)) * Math.sin(i * 0.13 - ph * 1.3 + (me ? 0 : 1.2)));
        bar.style.height = `${(3 + v * (loud ? 40 : 8)).toFixed(1)}px`;
      });
    }
    for (const meter of meters) {
      const me = meter.dataset.meter === "me", loud = me === mine;
      [...meter.children].forEach((bar, i) => {
        bar.style.height = `${Math.max(2, Math.abs(Math.sin(i * 0.8 + ph * 4 + (me ? 0 : 2))) * (loud ? 12 : 4)).toFixed(1)}px`;
      });
    }
    const lvl = Math.abs(Math.sin(ph * 5.3) * Math.sin(ph * 2.1 + 0.7));
    tiles.forEach((tile, i) => {
      const on = mine ? tile.dataset.who === "me" : i === other;
      tile.classList.toggle("sp", on);
      if (on) tile.style.setProperty("--lvl", lvl.toFixed(2));
    });
    const s = secs | 0;
    clock.textContent = `00:${String((s / 60) | 0).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;
  };

  const show = (n) => {
    step = n;
    elapsed = 0;
    // Only a repeated step restarts its sequences; a new step keeps the shared layers in place.
    if (stage.dataset.step === String(n)) { delete stage.dataset.step; void stage.offsetWidth; }
    stage.dataset.step = String(n);
    stage.setAttribute("aria-labelledby", tabs[n - 1].id);
    page.classList.toggle("on", n >= 2);
    tabs.forEach((tab, i) => {
      tab.setAttribute("aria-selected", String(i + 1 === n));
      tab.tabIndex = i + 1 === n ? 0 : -1;
    });
    walk.style.setProperty("--step-ms", `${STEPS[n - 1]}ms`);
    walk.classList.remove("auto");
    void walk.offsetWidth;
    walk.classList.add("auto");
    if (n === 1) secs = 0;
  };

  // A timer, not requestAnimationFrame: eight paints a second are enough, and it runs only while the scene is visible.
  const loop = () => {
    if (!visible || document.hidden || reduce.matches) { clearInterval(tick); tick = 0; lt = 0; return; }
    const now = Date.now(), dt = lt ? Math.min(0.25, (now - lt) / 1000) : 0;
    lt = now;
    if (step <= 3) { ph += dt; secs += dt; paint(); }
    if (!held) { elapsed += dt * 1000; if (elapsed >= STEPS[step - 1]) show((step % 4) + 1); }
  };
  const run = () => { if (visible && !tick && !document.hidden && !reduce.matches) tick = setInterval(loop, 125); };

  tabs.forEach((tab, i) => {
    tab.addEventListener("click", () => { show(i + 1); run(); });
    tab.addEventListener("keydown", (e) => {
      const to = { ArrowDown: i + 1, ArrowRight: i + 1, ArrowUp: i - 1, ArrowLeft: i - 1, Home: 0, End: 3 }[e.key];
      if (to === undefined) return;
      e.preventDefault();
      const j = (to + 4) % 4;
      tabs[j].focus();
      tabs[j].click();
    });
  });
  const hold = (on) => { held = on; walk.classList.toggle("held", on); };
  walk.addEventListener("mouseenter", () => hold(true));
  walk.addEventListener("mouseleave", () => hold(walk.contains(document.activeElement)));
  walk.addEventListener("focusin", () => hold(true));
  walk.addEventListener("focusout", (e) => hold(walk.contains(e.relatedTarget) || walk.matches(":hover")));

  const frozen = /^#walk-([1-4])$/.exec(location.hash);
  if (frozen) {
    walk.classList.add("frozen");
    ph = 1.3; secs = 12;
    paint();
    show(Number(frozen[1]));
    walk.classList.remove("auto");
    return;
  }
  paint();
  new IntersectionObserver((entries) => {
    visible = entries[0].isIntersecting;
    if (visible && !started && !reduce.matches) { started = true; show(1); }
    run();
  }, { threshold: 0.35 }).observe(stage);
  document.addEventListener("visibilitychange", run);
  reduce.addEventListener("change", run);
})();
