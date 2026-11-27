// D126: the first run, its screens (1) and (2) (mock-up v5's screens 1 and 1b, criterion 1 as #1280 amended it).
// (1) Who writes the summaries, as three cards by kind of engine (`engines.js`, scope "first"), « Auto » checked;
// what « Auto » needs to hear downloads at once, while the user reads — what needs an install (Ollama) is only said.
// « Je choisirai plus tard » keeps « Auto » and writes nothing. (2) The sound checked while it downloads: the two
// meters Settings' « Vérifier le son » reads (`sound_check_*`), listened to only while the screen shows, and under
// them how far the download came, its size and the time left; « Ça marche » ends on the ready screen (CU).
// « Refaire la configuration » keeps the assistant before it (`showWelcome`, D58) until D122b.
// It uses the window's own `element`, `fill`, `icon`, `size`, `length`, `levelTrack`, `paintLevel` and
// `firstRunEnded`, and Settings' `focusMinutes` (window.js).

// Where the user is among the screens, as v5 draws it: three marks, a screen reader told « Étape 1 sur 3 ».
function firstSteps(words, at) {
  const steps = element("span", "first-steps");
  steps.id = "first-steps";
  steps.setAttribute("role", "img");
  steps.setAttribute("aria-label", fill(words.first_step, at));
  steps.append(...[1, 2, 3].map((step) => element("i", step <= at ? "on" : "")));
  return steps;
}

function firstButton(id, className, text, act) {
  const one = element("button", className, text);
  one.type = "button";
  one.id = id;
  one.addEventListener("click", act);
  return one;
}

async function showFirstRun(words) {
  const page = document.getElementById("welcome");
  const { invoke } = window.__TAURI__.core;
  const offered = await invoke("models_offered", { name: null }).catch(() => null);
  const made = element("section", "card welcome first-run");
  made.id = "first-choice";
  const top = element("div", "welcome-top");
  top.append(document.querySelector("#sidebar .brand").cloneNode(true), firstSteps(words, 1));
  const heading = element("h1", "", words.first_title);
  heading.id = "welcome-heading";
  heading.tabIndex = -1;
  made.append(top, heading, element("p", "says", words.first_lead));
  const cards = offered ? engineCards({ job: "write", offered, words, scope: "first" }) : null;
  if (cards) made.append(cards.box);
  const said = element("p", "says welcome-said");
  said.setAttribute("role", "status");
  // Screen (2), and from it back to this one as it was left: its choice, its focus on its heading.
  const on = () => showFirstSound(words, offered, () => {
    page.replaceChildren(made);
    heading.focus();
  });
  const actions = element("div", "welcome-actions");
  actions.append(
    firstButton("first-later", "link", words.first_later, on),
    firstButton("first-next", "primary with-icon", words.first_next, async () => {
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
      on();
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

// Screen (2): the sound checked while what « Auto » needs to hear downloads. `back` draws screen (1) again.
function showFirstSound(words, offered, back) {
  const page = document.getElementById("welcome");
  const { invoke } = window.__TAURI__.core;
  const made = element("section", "card welcome first-run");
  made.id = "first-sound";
  const top = element("div", "welcome-top");
  top.append(document.querySelector("#sidebar .brand").cloneNode(true), firstSteps(words, 2));
  const heading = element("h1", "", words.first_sound_title);
  heading.id = "first-sound-heading";
  heading.tabIndex = -1;
  made.append(top, heading, element("p", "says", words.first_sound_lead));
  // The two sides as the recording bar names them; each meter Settings' own (`levelTrack`).
  const meters = element("div", "first-meters");
  const side = (glyph, name, kind) => {
    const said = element("span", "first-side");
    said.append(icon(glyph), element("span", "", name));
    const track = levelTrack("first-meter", name);
    track.dataset.side = kind;
    meters.append(said, track);
    return track;
  };
  const near = side("mic", words.small_mic, "near");
  const far = side("volume-2", words.small_meeting, "far");
  made.append(meters);
  // What is prepared meanwhile: the engine that hears, how far it came, its size and the time left.
  const getting = element("div", "first-getting");
  const label = element("b", "", words.first_getting);
  label.id = "first-getting";
  const bar = element("div", "first-progress");
  bar.id = "first-progress";
  bar.setAttribute("role", "progressbar");
  bar.setAttribute("aria-labelledby", label.id);
  bar.setAttribute("aria-valuemin", "0");
  bar.setAttribute("aria-valuemax", "100");
  bar.append(element("i"));
  const got = element("span", "says", "");
  got.id = "first-progress-said";
  getting.append(label, bar, got, element("span", "says first-once", words.first_once));
  getting.hidden = offered?.auto_hearing.state !== "download";
  made.append(getting);
  let first = null;
  let last = null;
  const paintGot = (fetching) => {
    if (!fetching) return;
    const { held, bytes } = fetching;
    const share = Math.min(100, Math.floor((100 * held) / Math.max(bytes, 1)));
    bar.setAttribute("aria-valuenow", String(share));
    bar.firstChild.style.width = `${share}%`;
    const now = performance.now();
    if (first === null || held < first.held) first = { held, at: now };
    // The time left from the speed seen here, once a second of it is known.
    const speed = now - first.at >= 1000 ? (held - first.held) / ((now - first.at) / 1000) : 0;
    got.textContent = speed > 0 && held < bytes
      ? fill(words.first_got_left, size(held, words), size(bytes, words), length((bytes - held) / speed))
      : fill(words.first_got, size(held, words), size(bytes, words));
  };
  const works = firstButton("first-works", "primary with-icon", words.first_works, () => end(() => {
    firstRunEnded = true;
    location.hash = "#/";
  }));
  works.prepend(icon("check"));
  const actions = element("div", "welcome-actions");
  actions.append(firstButton("first-back", "link with-icon", words.first_back, () => end(back)), works);
  actions.firstChild.prepend(icon("chevron-left"));
  made.append(actions);
  page.replaceChildren(made);
  heading.focus();
  // Listened to only while this screen shows: left by its buttons, by another page, or hidden, it stops.
  let polls = [];
  let open = true;
  const end = async (then) => {
    if (!open) return;
    open = false;
    for (const poll of polls) clearInterval(poll);
    window.removeEventListener("hashchange", left);
    document.removeEventListener("visibilitychange", hidden);
    await invoke("sound_check_close").catch(() => {});
    then?.();
  };
  const left = () => end();
  const hidden = () => document.visibilityState === "hidden" && end();
  window.addEventListener("hashchange", left);
  document.addEventListener("visibilitychange", hidden);
  invoke("sound_check_open").catch(() => ({ opened: false })).then(({ opened }) => {
    // Left while it opened: what opened is closed now.
    if (!open) return opened && invoke("sound_check_close").catch(() => {});
    polls.push(setInterval(async () => {
      const now = await invoke("sound_check_now").catch(() => null);
      if (!open || !now?.open) return;
      paintLevel(near, now.heard.levels[0]);
      paintLevel(far, now.heard.levels[1]);
    }, 100));
  });
  if (!getting.hidden) {
    polls.push(setInterval(async () => {
      const read = await invoke("makings").catch(() => null);
      if (!open || !read) return;
      // Come whole: said whole, and no longer asked.
      if (read.model_fetching) last = read.model_fetching;
      else if (last) paintGot({ held: last.bytes, bytes: last.bytes });
      if (read.model_fetching) paintGot(read.model_fetching);
    }, 250));
  }
}
