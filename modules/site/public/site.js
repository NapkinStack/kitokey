// kitokey.com's landing (D114, site v7). Progressive only: the page is whole without this file.
// 1. The install's tabs: the pressed one shows its commands. 2. The copy button. 3. Blocks rise into place as
// they enter, unless motion is reduced, or #still is in the address (a capture whose clock does not run them).
(() => {
  "use strict";
  const tabs = [...document.querySelectorAll(".os-tab")];
  const show = (tab) => {
    for (const t of tabs) t.setAttribute("aria-pressed", String(t === tab));
    for (const el of document.querySelectorAll(".os-code, .os-note")) el.hidden = el.dataset.os !== tab.dataset.os;
  };
  for (const tab of tabs) tab.addEventListener("click", () => show(tab));
  if (tabs.length) show(tabs.find((t) => t.getAttribute("aria-pressed") === "true") ?? tabs[0]);

  const box = document.querySelector(".code");
  const button = box?.querySelector("button.copy");
  if (button) {
    button.hidden = false;
    const label = button.textContent;
    const status = document.getElementById("copy-status");
    button.addEventListener("click", async () => {
      const source = box.querySelector(".os-code:not([hidden])");
      if (!source) return;
      try {
        await navigator.clipboard.writeText(source.textContent.replace(/^\$ /gm, "").trim());
        button.textContent = box.dataset.copied;
      } catch {
        button.textContent = box.dataset.failed;
      }
      if (status) status.textContent = button.textContent;
      setTimeout(() => { button.textContent = label; }, 2000);
    });
  }

  if (!matchMedia("(prefers-reduced-motion: reduce)").matches && location.hash !== "#still" && "IntersectionObserver" in window) {
    document.documentElement.classList.add("js-reveal");
    const io = new IntersectionObserver((entries) => {
      for (const e of entries) if (e.isIntersecting) { e.target.classList.add("in"); io.unobserve(e.target); }
    }, { rootMargin: "0px 0px -8% 0px" });
    document.querySelectorAll(".rise").forEach((el) => io.observe(el));
  }
})();
