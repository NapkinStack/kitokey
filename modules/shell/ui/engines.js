// D122: one engine card, wherever an engine is offered — Settings and the first run (« Auto » first),
// a meeting's Refaire (« Par défaut », Settings' own, first). Its markup is D120's (agreed with lina,
// 2026-11-26): a fieldset per job, its legend the job, each card a native radio in its label, the
// card's state and where it runs on the card itself, its id, address and size behind « Détails ».
// The radios give the group and the arrow keys (WAI-ARIA APG, Radio Group: the native control first).
// Drawn from models_offered (`offered`) and services_offered's (`services`): nothing read here.
// It uses the window's own `element`, `fill`, `size`, `spoken` and `modelSaid` (window.js).

// One card. `value` is what its radio stands for — "" for « Auto » or « Par défaut » —, `state` one of
// ready, download, install, key; `at`: here or online.
function engineCard(job, group, { engine, value, name, at, state, said, facts, mark, details, checked }) {
  const card = element("div", "engine-card");
  Object.assign(card.dataset, { engine, job, state, where: at });
  const label = element("label");
  const radio = element("input");
  radio.type = "radio";
  radio.name = group;
  radio.value = value;
  radio.checked = checked;
  label.append(radio, element("span", "name", name));
  if (mark) label.append(element("span", "default", mark));
  // Where it runs, unless the card's name already says it (« Sur cet ordinateur », the coordinator's audit of #1307).
  const where = said.where ?? (at === "online" ? said.online : said.here);
  if (where !== name) label.append(element("span", "where", where));
  label.append(element("span", "state", said.state));
  if (facts) label.append(element("span", "facts", facts));
  const more = element("details");
  more.append(element("summary", "", said.details), ...details.filter(Boolean).map((line) => element("p", "detail", line)));
  card.append(label, more);
  return card;
}

// Each group of radios its own name: radios of one name with no form are one group in the whole
// document, and two groups on one page — a dialog over Settings — would uncheck each other.
let engineGroups = 0;

function engineCards({ job, offered, words, scope, services = [] }) {
  const group = `engine-${job}-${(engineGroups += 1)}`;
  const box = element("fieldset", "engine-cards");
  box.dataset.job = job;
  box.append(element("legend", "", job === "write" ? words.engine_write : words.engine_transcribe));
  const writing = job === "write";
  const stateSaid = (state, bytes) =>
    state === "ready" ? words.engine_ready
    : state === "download" ? (bytes ? fill(words.engine_download, size(bytes, words)) : words.engine_download_unsized)
    : state === "install" ? words.engine_install
    : words.engine_key;
  const said = (state, bytes, where) => ({
    here: words.engine_here, online: words.engine_online, where, state: stateSaid(state, bytes), details: words.engine_details,
  });
  const auto = writing ? offered.auto_writing : offered.auto_hearing;
  const autoName = writing ? modelSaid(auto.model) : spoken(auto.model);
  const autoSaid = said(auto.state, auto.bytes, fill(words.engine_auto_is, autoName));
  const autoDetails = [auto.model, writing ? auto.provider : null, auto.bytes ? size(auto.bytes, words) : null];
  const cards = [];
  if (scope === "first") {
    // The first run: three cards by kind, no list of models (mock-up v6, the decider's).
    const online = services.length > 0 ? "ready" : "key";
    cards.push(
      { engine: "auto", value: "", name: words.engine_auto, at: "here", state: auto.state, said: autoSaid, facts: words.engine_auto_says, mark: words.engine_default, details: autoDetails, checked: true },
      { engine: "local", value: "local", name: words.engine_here, at: "here", state: auto.state, said: said(auto.state, auto.bytes), facts: words.engine_here_says, details: autoDetails, checked: false },
      { engine: "online", value: "online", name: words.engine_online_kind, at: "online", state: online, said: said(online), facts: words.engine_online_says, details: services.map((service) => service.name), checked: false },
    );
  } else {
    const own = scope === "meeting" ? (writing ? offered.chosen.minutes : offered.chosen.transcription) : null;
    const ownValue = own ? modelValue(writing ? "minutes" : "transcription", own) : "";
    // Settings' choice, as one value: none named is « Auto ».
    const settingsValue = writing
      ? (offered.minutes_provider && offered.minutes_model ? modelValue("minutes", { provider: offered.minutes_provider, model: offered.minutes_model }) : "")
      : offered.engine === "speechanalyzer" ? modelValue("transcription", { engine: "speechanalyzer" })
      : offered.engine === "service" && offered.transcription_model ? modelValue("transcription", { engine: "service", provider: offered.transcription_provider, model: offered.transcription_model })
      : offered.model === auto.model ? "" : modelValue("transcription", { engine: "whisper", model: offered.model });
    const chosen = scope === "meeting" ? ownValue : settingsValue;
    const card = (engine, value, name, at, state, details, extra = {}) =>
      cards.push({ engine, value, name, at, state, said: said(state), details, checked: value === chosen, ...extra });
    if (scope === "meeting") {
      // Settings' own, first, marked: what the meeting is made by when it chose none.
      const named = settingsValue === "" ? words.engine_auto : writing ? modelSaid(offered.minutes_model) : spoken(offered.model);
      const at = writing && settingsValue !== "" && offered.minutes_provider !== offered.ollama_address ? "online" : "here";
      card(settingsValue === "" ? "auto" : "settings", "", named, at, settingsValue === "" ? auto.state : "ready", settingsValue === "" ? autoDetails : [], { mark: words.engine_default, said: settingsValue === "" ? autoSaid : said("ready") });
    } else {
      card("auto", "", words.engine_auto, "here", auto.state, autoDetails, { said: autoSaid, facts: words.engine_auto_says });
    }
    // Settings' own is offered once, as the first card — unless it is the meeting's own, kept as chosen.
    const listed = new Set(cards.map((c) => c.value));
    if (scope === "meeting" && settingsValue !== ownValue) listed.add(settingsValue);
    const add = (engine, value, name, at, state, details) => {
      if (!listed.has(value) && !(offered.confidential && at === "online")) { listed.add(value); card(engine, value, name, at, state, details); }
    };
    if (writing) {
      for (const model of offered.ollama) add("ollama", modelValue("minutes", { provider: offered.ollama_address, model }), modelSaid(model), "here", "ready", [model, offered.ollama_address]);
      if (offered.minutes_provider && offered.minutes_model && offered.minutes_provider !== offered.ollama_address) {
        add("service", settingsValue, modelSaid(offered.minutes_model), "online", "ready", [offered.minutes_model, offered.minutes_provider]);
      }
      for (const service of services) for (const model of service.models) {
        add("service", modelValue("minutes", { provider: service.address, model }), `${modelSaid(model)} · ${service.name}`, "online", "ready", [model, service.address]);
      }
    } else {
      for (const model of offered.whisper) add("whisper", modelValue("transcription", { engine: "whisper", model }), spoken(model), "here", "ready", [model]);
      if (offered.apple) add("speechanalyzer", modelValue("transcription", { engine: "speechanalyzer" }), "SpeechAnalyzer", "here", "ready", []);
      if (offered.service) {
        const [provider, model] = offered.service;
        add("service", modelValue("transcription", { engine: "service", provider, model }), modelSaid(model), "online", "ready", [model, provider]);
      }
      for (const service of services) for (const model of service.models) {
        add("service", modelValue("transcription", { engine: "service", provider: service.address, model }), `${modelSaid(model)} · ${service.name}`, "online", "ready", [model, service.address]);
      }
    }
  }
  for (const card of cards) box.append(engineCard(job, group, card));
  const told = [];
  box.addEventListener("change", (event) => {
    const radio = event.target;
    if (!(radio instanceof HTMLInputElement) || radio.type !== "radio") return;
    const state = radio.closest(".engine-card").dataset.state;
    for (const fn of told) fn({ value: radio.value, state });
  });
  return {
    box,
    value: () => box.querySelector("input[type=radio]:checked")?.value ?? "",
    onChange: (fn) => told.push(fn),
  };
}
