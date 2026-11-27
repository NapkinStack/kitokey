// D126: the first run, its first screen (mock-up v5's screen 1, criterion 1 as #1280 amended it). Who writes
// the summaries, as three cards by kind of engine (`engines.js`, scope "first"), « Auto » checked; what « Auto »
// needs to hear downloads at once, while the user reads — what needs an install (Ollama) is only said. « Je
// choisirai plus tard » keeps « Auto » and writes nothing; either way the window ends on its ready screen (CU).
// « Refaire la configuration » keeps the assistant before it (`showWelcome`, D58) until D122b.
// It uses the window's own `element`, `fill`, `icon` and `firstRunEnded`, and Settings' `focusMinutes` (window.js).

async function showFirstRun(words) {
  const page = document.getElementById("welcome");
  const { invoke } = window.__TAURI__.core;
  const offered = await invoke("models_offered", { name: null }).catch(() => null);
  const made = element("section", "card welcome first-run");
  made.id = "first-choice";
  const top = element("div", "welcome-top");
  // Where the user is among the screens, as v5 draws it: three marks, a screen reader told « Étape 1 sur 3 ».
  const steps = element("span", "first-steps");
  steps.id = "first-steps";
  steps.setAttribute("role", "img");
  steps.setAttribute("aria-label", fill(words.first_step, 1));
  steps.append(...[1, 2, 3].map((step) => element("i", step === 1 ? "on" : "")));
  top.append(document.querySelector("#sidebar .brand").cloneNode(true), steps);
  const heading = element("h1", "", words.first_title);
  heading.id = "welcome-heading";
  heading.tabIndex = -1;
  made.append(top, heading, element("p", "says", words.first_lead));
  const cards = offered ? engineCards({ job: "write", offered, words, scope: "first" }) : null;
  if (cards) made.append(cards.box);
  const said = element("p", "says welcome-said");
  said.setAttribute("role", "status");
  const ready = () => {
    firstRunEnded = true;
    location.hash = "#/";
  };
  const button = (id, className, text, act) => {
    const one = element("button", className, text);
    one.type = "button";
    one.id = id;
    one.addEventListener("click", act);
    return one;
  };
  const actions = element("div", "welcome-actions");
  actions.append(
    button("first-later", "link", words.first_later, ready),
    button("first-next", "primary with-icon", words.first_next, async () => {
      const chosen = cards?.value() ?? "";
      // An online service: its key is asked where Settings ask it, nothing saved here.
      if (chosen === "online") {
        focusMinutes = true;
        location.hash = "#/settings";
        return;
      }
      // This computer, named: Ollama's model « Auto » resolves to, saved as Settings save it.
      if (chosen === "local") {
        try {
          await invoke("minutes_saved", { provider: offered.ollama_address, model: offered.auto_writing.model, key: null });
        } catch (why) {
          said.textContent = fill(words.not_saved, why);
          return;
        }
      }
      ready();
    }),
  );
  // « Continuer » goes on: its chevron after its words, as v5 draws it, hidden from a screen reader.
  actions.lastChild.append(icon("chevron-right"));
  made.append(actions, said);
  page.replaceChildren(made);
  heading.focus();
  // What « Auto » needs to hear, downloaded at once: what Settings would fetch (`model_fetch`).
  if (offered?.auto_hearing.state === "download") invoke("model_fetch").catch(() => {});
}
