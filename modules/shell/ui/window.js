// The window: what the shell's `listed` command returns, shown as D11's first board. It
// reads nothing itself and changes nothing; it asks again whenever the window comes forward.
"use strict";

// A failure of the page's own reaches the shell's log (D29): its kind and where it was thrown —
// never its sentence, which may hold a meeting's words. A rejection says where from its stack.
function pageFailed(error, file, line, column) {
  const kind = error instanceof Error ? error.name : "";
  window.__TAURI__.core.invoke("page_failed", { kind, file: file || "", line: line || 0, column: column || 0 }).catch(() => {});
}
window.addEventListener("error", (event) => pageFailed(event.error, event.filename, event.lineno, event.colno));
window.addEventListener("unhandledrejection", (event) => {
  const at = /([^\s@()]*):(\d+):(\d+)/.exec(event.reason instanceof Error ? event.reason.stack || "" : "");
  pageFailed(event.reason, at?.[1], Number(at?.[2]), Number(at?.[3]));
});

const fill = (template, ...values) => values.reduce((text, value) => text.replace("{}", value), template);

function size(bytes, words) {
  let unit = 0;
  let value = bytes;
  // The unit is chosen on the value as it will be written: 999 999 B is "1 MB", not "1,000 kB".
  const rounded = (v, u) => { const d = v < 10 && u > 0 ? 10 : 1; return Math.round(v * d) / d; };
  while (rounded(value, unit) >= 1000 && unit < 3) { value /= 1000; unit += 1; }
  const digits = value < 10 && unit > 0 ? 1 : 0;
  const number = new Intl.NumberFormat(words.locale, { maximumFractionDigits: digits }).format(value);
  return `${number} ${words.units[unit]}`;
}

// The day a session started, on this machine's clock; none when the shell could not read it.
const dayOf = (session) => (session.instant === null ? null : new Date(session.instant * 1000));

// As the notifications say it (words::length): seconds under a minute, then minutes, then hours.
function length(seconds) {
  if (seconds < 59.5) return `${Math.round(Math.max(0, seconds))} s`;
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes} min`;
  return `${Math.floor(minutes / 60)} h ${String(minutes % 60).padStart(2, "0")}`;
}

// Monday 00:00 of the week a day is in, on this machine's clock.
function weekOf(day) {
  const start = new Date(day.getFullYear(), day.getMonth(), day.getDate());
  start.setDate(start.getDate() - ((start.getDay() + 6) % 7));
  return start;
}

function group(day, now, words) {
  if (day === null) return words.undated;
  const thisWeek = weekOf(now);
  const lastWeek = new Date(thisWeek);
  lastWeek.setDate(lastWeek.getDate() - 7);
  if (day >= thisWeek) return words.this_week;
  if (day >= lastWeek) return words.last_week;
  const month = new Intl.DateTimeFormat(words.locale, { month: "long" }).format(day);
  if (day.getFullYear() === now.getFullYear() && day.getMonth() === now.getMonth()) return fill(words.earlier_in, month);
  const named = new Intl.DateTimeFormat(words.locale, { month: "long", year: "numeric" }).format(day);
  return named.charAt(0).toUpperCase() + named.slice(1);
}

function chip(text, kind) {
  const element = document.createElement("span");
  element.className = kind ? `chip ${kind}` : "chip";
  element.textContent = text;
  return element;
}

// A control waiting for its answer (#886, D101): said disabled to assistive technology and deaf to
// what would change it, never `disabled` — WebKit lets the focus of a disabled control go and never
// gives it back (ines's probe: focused, disabled three frames, then body). WAI-ARIA's practice: it
// stays focusable, inert. A click, a press or a key that would change it is held back.
function waiting(control, on) {
  if (on) control.setAttribute("aria-disabled", "true");
  else control.removeAttribute("aria-disabled");
}
const HELD_KEYS = new Set([" ", "Enter", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown"]);
for (const type of ["click", "mousedown", "keydown"]) {
  document.addEventListener(type, (event) => {
    const control = event.target instanceof Element ? event.target.closest('[aria-disabled="true"]') : null;
    if (!control || (type === "keydown" && !HELD_KEYS.has(event.key))) return;
    event.preventDefault();
    event.stopImmediatePropagation();
  }, true);
}

// A lock, Lucide's: its meaning is always in the words beside it (D60, D89).
function lock() {
  const svg = icon("lock");
  svg.classList.add("lock");
  return svg;
}

// Lucide's icons, inline (ISC, `ui/LUCIDE-LICENSE`): what the page draws for itself, a glyph being the
// font's to draw (D69). The name goes on what holds it, for assistive technology. A path's `d`, or
// another shape as Lucide 1.14.0's icon node gives it (D89).
const ICONS = {
  calendar: ["M8 2v4", "M16 2v4", ["rect", { width: 18, height: 18, x: 3, y: 4, rx: 2 }], "M3 10h18"],
  check: ["M20 6 9 17l-5-5"],
  "chevron-left": ["m15 18-6-6 6-6"],
  "chevron-down": ["m6 9 6 6 6-6"],
  "chevron-up": ["m18 15-6-6-6 6"],
  "chevron-right": ["m9 18 6-6-6-6"],
  minus: ["M5 12h14"],
  mic: ["M12 19v3", "M19 10v2a7 7 0 0 1-14 0v-2", ["rect", { x: 9, y: 2, width: 6, height: 13, rx: 3 }]],
  "mic-off": ["M12 19v3", "M15 9.34V5a3 3 0 0 0-5.68-1.33", "M16.95 16.95A7 7 0 0 1 5 12v-2", "M18.89 13.23A7 7 0 0 0 19 12v-2", "m2 2 20 20", "M9 9v3a3 3 0 0 0 5.12 2.12"],
  "volume-2": ["M11 4.702a.705.705 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.705.705 0 0 0 11 19.298z", "M16 9a5 5 0 0 1 0 6", "M19.364 18.364a9 9 0 0 0 0-12.728"],
  "volume-x": ["M11 4.702a.705.705 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.705.705 0 0 0 11 19.298z", ["line", { x1: 22, x2: 16, y1: 9, y2: 15 }], ["line", { x1: 16, x2: 22, y1: 9, y2: 15 }]],
  x: ["M18 6 6 18", "m6 6 12 12"],
  "arrow-right": ["M5 12h14", "m12 5 7 7-7 7"],
  "maximize-2": ["M15 3h6v6", "m21 3-7 7", "m3 21 7-7", "M9 21H3v-6"],
  "picture-in-picture-2": ["M21 9V6a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2v10c0 1.1.9 2 2 2h4", ["rect", { width: 10, height: 7, x: 12, y: 13, rx: 2 }]],
  "panel-left-close": [["rect", { width: 18, height: 18, x: 3, y: 3, rx: 2 }], "M9 3v18", "m16 15-3-3 3-3"],
  "panel-left-open": [["rect", { width: 18, height: 18, x: 3, y: 3, rx: 2 }], "M9 3v18", "m14 9 3 3-3 3"],
  search: ["m21 21-4.34-4.34", ["circle", { cx: 11, cy: 11, r: 8 }]],
  "hard-drive": ["M10 16h.01", "M2.212 11.577a2 2 0 0 0-.212.896V18a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-5.527a2 2 0 0 0-.212-.896L18.55 5.11A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z", "M21.946 12.013H2.054", "M6 16h.01"],
  settings: ["M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915", ["circle", { cx: 12, cy: 12, r: 3 }]],
  lock: [["rect", { width: 18, height: 11, x: 3, y: 11, rx: 2, ry: 2 }], "M7 11V7a5 5 0 0 1 10 0v4"],
  send: ["M14.536 21.686a.5.5 0 0 0 .937-.024l6.5-19a.496.496 0 0 0-.635-.635l-19 6.5a.5.5 0 0 0-.024.937l7.93 3.18a2 2 0 0 1 1.112 1.11z", "m21.854 2.147-10.94 10.939"],
  copy: [["rect", { width: 14, height: 14, x: 8, y: 8, rx: 2, ry: 2 }], "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"],
  "rotate-ccw": ["M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8", "M3 3v5h5"],
  "trash-2": ["M10 11v6", "M14 11v6", "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6", "M3 6h18", "M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"],
  // Lucide's webhook, hash and globe (ISC, ui/LUCIDE-LICENSE): a webhook, a channel, an online service — the page's own,
  // where a brand's mark is not cleared for an app (#1238).
  webhook: ["M18 16.98h-5.99c-1.1 0-1.95.94-2.48 1.9A4 4 0 0 1 2 17c.01-.7.2-1.4.57-2", "m6 17 3.13-5.78c.53-.97.1-2.18-.5-3.1a4 4 0 1 1 6.89-4.06", "m12 6 3.13 5.73C15.66 12.7 16.9 13 18 13a4 4 0 0 1 0 8"],
  hash: ["M4 9h16", "M4 15h16", "M10 3 8 21", "M16 3l-2 18"],
  globe: [["circle", { cx: 12, cy: 12, r: 10 }], "M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20", "M2 12h20"],
  "folder-open": ["m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2"],
  ellipsis: [["circle", { cx: 12, cy: 12, r: 1 }], ["circle", { cx: 19, cy: 12, r: 1 }], ["circle", { cx: 5, cy: 12, r: 1 }]],
  "message-square-text": ["M22 17a2 2 0 0 1-2 2H6.828a2 2 0 0 0-1.414.586l-2.202 2.202A.71.71 0 0 1 2 21.286V5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2z", "M7 11h10", "M7 15h6", "M7 7h8"],
  languages: ["m5 8 6 6", "m4 14 6-6 2-3", "M2 5h12", "M7 2h1", "m22 22-5-10-5 10", "M14 18h6"],
  "lock-open": [["rect", { width: 18, height: 11, x: 3, y: 11, rx: 2, ry: 2 }], "M7 11V7a5 5 0 0 1 9.9-1"],
  "file-text": ["M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z", "M14 2v5a1 1 0 0 0 1 1h5", "M10 9H8", "M16 13H8", "M16 17H8"],
  captions: [["rect", { width: 18, height: 14, x: 3, y: 5, rx: 2, ry: 2 }], "M7 15h4M15 15h2M7 11h2M13 11h4"],
  "audio-lines": ["M2 10v3", "M6 6v11", "M10 3v18", "M14 8v7", "M18 5v13", "M22 10v3"],
  bluetooth: ["m7 7 10 10-5 5V2l5 5L7 17"],
  info: [["circle", { cx: 12, cy: 12, r: 10 }], "M12 16v-4", "M12 8h.01"],
  "circle-check": [["circle", { cx: 12, cy: 12, r: 10 }], "m9 12 2 2 4-4"],
  // Lucide's user-pen (ISC): the speakers named (#1205).
  "user-pen": ["M11.5 15H7a4 4 0 0 0-4 4v2", "M21.378 16.626a1 1 0 0 0-3.004-3.004l-4.01 4.012a2 2 0 0 0-.506.854l-.837 2.87a.5.5 0 0 0 .62.62l2.87-.837a2 2 0 0 0 .854-.506z", ["circle", { cx: 10, cy: 7, r: 4 }]],
  pencil: ["M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z", "m15 5 4 4"],
};
function icon(name) {
  const ns = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(ns, "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("class", "glyph");
  svg.setAttribute("aria-hidden", "true");
  for (const part of ICONS[name]) {
    const [tag, attributes] = typeof part === "string" ? ["path", { d: part }] : part;
    const shape = document.createElementNS(ns, tag);
    for (const [key, value] of Object.entries(attributes)) shape.setAttribute(key, value);
    svg.append(shape);
  }
  return svg;
}

// « Plus d’actions » (D89): the rarer actions behind one button, as WAI-ARIA's menu button has it —
// Enter, Space or ↓ open it on its first item, ↑ on its last; ↑ ↓ Home End move in it; Escape
// closes it, the focus back on its button; Tab or a click elsewhere closes it; an item chosen
// closes it, the focus back on the button before the item acts.
function moreActions(items, words) {
  const more = element("div", "more");
  const button = iconButton("ellipsis", words.more_actions, "more-actions");
  button.setAttribute("aria-haspopup", "menu");
  button.setAttribute("aria-controls", "more-menu");
  button.setAttribute("aria-expanded", "false");
  const menu = element("div", "menu");
  menu.id = "more-menu";
  menu.setAttribute("role", "menu");
  menu.setAttribute("aria-label", words.more_actions);
  menu.hidden = true;
  const close = (back) => {
    menu.hidden = true;
    button.setAttribute("aria-expanded", "false");
    if (back) button.focus();
  };
  const entries = items.map(([id, said, act]) => {
    const item = element("button", "menu-item", said);
    item.type = "button";
    item.id = id;
    item.tabIndex = -1;
    item.setAttribute("role", "menuitem");
    item.addEventListener("click", () => {
      close(true);
      act();
    });
    return item;
  });
  const open = (at) => {
    menu.hidden = false;
    button.setAttribute("aria-expanded", "true");
    entries.at(at).focus();
  };
  button.addEventListener("click", () => (menu.hidden ? open(0) : close(true)));
  button.addEventListener("keydown", (event) => {
    const at = { ArrowDown: 0, ArrowUp: -1 }[event.key];
    if (at === undefined) return;
    event.preventDefault();
    open(at);
  });
  menu.addEventListener("keydown", (event) => {
    const now = entries.indexOf(document.activeElement);
    const last = entries.length - 1;
    const to = { ArrowDown: now === last ? 0 : now + 1, ArrowUp: now <= 0 ? last : now - 1, Home: 0, End: last }[event.key];
    if (to !== undefined) entries[to].focus();
    else if (event.key === "Escape") close(true);
    else if (event.key === "Tab") return close(false);
    else return;
    // Its own: neither a panel's Escape nor the page's keys hear it.
    event.preventDefault();
    event.stopPropagation();
  });
  // A press elsewhere: WebKit does not focus a button pressed, so the press is heard, not the focus.
  const elsewhere = (event) => {
    if (!more.isConnected) return document.removeEventListener("pointerdown", elsewhere, true);
    if (!menu.hidden && !more.contains(event.target)) close(false);
  };
  document.addEventListener("pointerdown", elsewhere, true);
  menu.append(...entries);
  more.append(button, menu);
  return more;
}

// A secondary action (D89): an icon, its tooltip and its name the same words.
function iconButton(glyph, name, id) {
  const button = element("button", "icon-button");
  button.type = "button";
  button.id = id;
  button.title = name;
  button.setAttribute("aria-label", name);
  button.append(icon(glyph));
  return button;
}

// A brand's own mark where Kitokey names its service (#1238): Discord's, OpenRouter's and Hugging Face's, the files
// their owners give, unaltered (ui/brands/BRANDS.md) — OpenRouter's Ink on light and Cloud on dark, as its page says.
// Any other brand: an icon of the page's (`fallback`), its name in the words beside it. Decorative: the words name it.
const BRANDS = { discord: ["discord-symbol.svg"], openrouter: ["openrouter-glyph-ink.svg", "openrouter-glyph-cloud.svg"], huggingface: ["huggingface.svg"] };
function brandMark(brand, fallback) {
  const files = BRANDS[brand];
  if (!files) return icon(fallback);
  const mark = element("span", "brand-mark");
  files.forEach((file, index) => {
    const image = element("img", files.length > 1 ? (index ? "on-dark" : "on-light") : "");
    image.src = `brands/${file}`;
    image.alt = "";
    mark.append(image);
  });
  return mark;
}
// A destination's kind, its mark or the page's icon: a webhook, Slack's channel.
const kindMark = (kind) => brandMark(kind, kind === "slack" ? "hash" : "webhook");
// The service chosen in a list, its mark beside it (#1238): OpenRouter's own, any other an icon of the page's — its name
// the list's. Painted again at each choice.
function serviceMarked(pick, name) {
  const row = element("span", "with-mark");
  pick.replaceWith(row);
  const mark = element("span", "service-mark");
  const paint = () => mark.replaceChildren(brandMark((name() ?? "").toLowerCase().replace(/\s+/g, ""), "globe"));
  row.append(mark, pick);
  pick.addEventListener("change", paint);
  paint();
}

function readySaid(text) {
  const said = element("span", "ready");
  said.append(icon("check"), ` ${text}`);
  return said;
}

function lockedChip(words) {
  const said = chip(words.confidential_chip, "locked");
  said.prepend(lock());
  return said;
}

// « Réunion confidentielle » (D60): the settings' own switch, a lock before its words.
function confidentialSwitch(words, id) {
  const row = element("label", "row confidential");
  const box = element("input");
  box.type = "checkbox";
  box.id = id;
  box.setAttribute("role", "switch");
  const said = element("span", "said");
  said.append(element("span", "name", words.confidential_name), element("span", "says", words.confidential_says));
  row.append(lock(), said, box);
  return { row, box };
}

// What had left before the meeting was said confidential, and to which host.
function leftSaid(words, left) {
  const said = { sound: words.confidential_left_sound, transcript: words.confidential_left_transcript, summary: words.confidential_left_summary };
  const each = left.map(({ what, host }) => fill(said[what], host));
  return each.length === 0 ? "" : fill(words.confidential_left, each.join(" · "));
}

// A meeting's language its page could not save: said on every drawing of that page until the next
// choice, saved or refused, and gone once the page is left — as Settings' refusal (#655, #841).
let languageRefused = null;

// The language a meeting is spoken in (D65): the settings' own, or one of its own — its next
// transcript heard in it. Written at once by `save`; refused, said, the choice put back. The same
// row in the bar's options (D99), its ids its own. `held`: the meeting whose page it is, its refusal said
// again on each drawing of that page (#841); the bar's, none.
// A list put back by script after a refusal: WebKit's select still holds the refused value as the
// last one it fired `change` for, so that value chosen again fires none (#843). A fresh copy holds
// none: it takes the list's place, its value and the focus, and the caller listens to it again.
function putBack(pick, value) {
  const fresh = pick.cloneNode(true);
  const focused = document.activeElement === pick;
  pick.replaceWith(fresh);
  fresh.value = value;
  if (focused) fresh.focus();
  return fresh;
}

function languageRow(save, language, languages, words, settled, apple, system, id = "meeting-language", saidId = "language-said", held = null) {
  // A setting of the meeting's row (D89): its icon, its name, its list.
  const row = element("label", "setting language");
  row.htmlFor = id;
  let pick = element("select");
  pick.id = id;
  const add = (value, text) => {
    const option = element("option", "", text);
    option.value = value;
    pick.append(option);
  };
  // What « as in Settings » means, said: the language they name, or detected — by SpeechAnalyzer
  // never: it hears French or English, the system's when none is named (D64).
  const heard = apple ? languages.filter(([code]) => code === "fr" || code === "en") : languages;
  const named = (code) => languages.find(([known]) => known === code)?.[1] ?? code;
  // Said as the result, in Settings' own words (#1074): « Détection automatique », or « Français, selon les Réglages ».
  const settledSaid = apple ? named(heard.some(([code]) => code === settled) ? settled : system) : settled ? named(settled) : null;
  add("", settledSaid ? fill(words.language_default, settledSaid) : words.language_detected);
  for (const [code, said] of heard) add(code, said);
  // A language of its own SpeechAnalyzer does not hear — written while Whisper heard —: kept, said as heard.
  if (language && !heard.some(([code]) => code === language)) add(language, apple ? fill(words.language_heard_as, named(language), named(system)) : named(language));
  pick.value = language ?? "";
  const said = element("p", "says");
  said.id = saidId;
  said.setAttribute("role", "status");
  said.textContent = held !== null && languageRefused?.name === held ? languageRefused.said : "";
  let kept = pick.value;
  const changed = async () => {
    try {
      await save(pick.value || null);
      kept = pick.value;
      if (held !== null) languageRefused = null;
      said.textContent = "";
    } catch (why) {
      pick = putBack(pick, kept);
      pick.addEventListener("change", changed);
      said.textContent = fill(words.language_not_saved, why);
      if (held !== null) languageRefused = { name: held, said: said.textContent };
    }
  };
  pick.addEventListener("change", changed);
  // Its value and a chevron, as board 4 draws the list: the system's own box not drawn.
  const chevron = icon("chevron-down");
  chevron.classList.add("chevron");
  row.append(icon("languages"), element("span", "name", fill(words.labelled, words.language_label)), pick, chevron);
  return [row, said];
}

// The switch on a meeting's page, a setting of its row (D89): the file written or removed at once;
// what had left said under the row. Its lock open while it is off, as board 4 draws it. Drawn again in place
// after a confirmed send, which it then names (D119).
let confidentialShown = null;
function confidentialCard(session, confidential, words) {
  const name = nameOf(session);
  const card = element("label", "setting confidential");
  card.title = words.confidential_says;
  const box = element("input");
  box.type = "checkbox";
  box.id = "confidential-switch";
  box.setAttribute("role", "switch");
  box.checked = confidential.on;
  // What it means, read with it by a screen reader as the tooltip shows it to the pointer; outside
  // the label, which would make it part of the switch's name.
  const described = element("span", "unseen", words.confidential_says);
  described.id = "confidential-described";
  box.setAttribute("aria-describedby", described.id);
  const left = element("p", "left");
  left.id = "confidential-left";
  const said = element("p", "says");
  said.id = "confidential-said";
  said.setAttribute("role", "status");
  const paint = (shown) => {
    card.classList.toggle("on", shown.on);
    card.querySelector("svg")?.remove();
    const drawn = shown.on ? lock() : icon("lock-open");
    drawn.classList.add("lock");
    card.prepend(drawn);
    left.textContent = leftSaid(words, shown.left);
    left.hidden = left.textContent === "";
    said.hidden = said.textContent === "";
  };
  box.addEventListener("change", async () => {
    waiting(box, true);
    try {
      const shown = await window.__TAURI__.core.invoke("confidential_set", { name, on: box.checked });
      said.textContent = "";
      paint(shown);
      // Its lists read again, in place: no service offered once said confidential (D91, D93).
      if (pageOffered?.name === name) pageOffered.again();
      // Said confidential: an open send panel closed; its sends are asked again from now on (D119).
      if (shown.on) document.getElementById("send-panel")?.remove();
    } catch {
      box.checked = !box.checked;
      said.textContent = words.confidential_not_saved;
      said.hidden = false;
    }
    waiting(box, false);
    readMakings();
  });
  card.append(element("span", "name", words.confidential_chip), box);
  paint(confidential);
  confidentialShown = { name, paint };
  return [card, left, said, described];
}

// A session's folder name: what the window calls it, and the only date it says when its start
// cannot be read.
const nameOf = (session) => session.folder.split(/[\\/]/).pop();

// An hour as each language writes it (#1075): US English « 2:30 PM », with no zero before the hour; French « 09:30 ».
const hours = (words) => ({ hour: words.locale === "en" ? "numeric" : "2-digit", minute: "2-digit" });

function heading(session, words) {
  const day = dayOf(session);
  if (day === null) return nameOf(session);
  const date = new Intl.DateTimeFormat(words.locale, { weekday: "short", day: "numeric", month: "short" }).format(day);
  const time = new Intl.DateTimeFormat(words.locale, hours(words)).format(day);
  return `${date} · ${time}`;
}

// What a meeting holds. A list's row and a meeting's header say only what is not at rest —
// recording, cut, its audio freed, a step being made —: the row's marks and the page's cards say
// the rest, where each step ran with them (D89).
function holdings(session, words, compact = false) {
  const holds = document.createElement("div");
  holds.className = "holds";
  // Said confidential (D60): the lock and its word, first — never the lock alone.
  if (session.confidential && !compact) holds.append(lockedChip(words));
  if (session.state === "recording") holds.append(chip(words.recording, "live"));
  if (session.state === "interrupted") holds.append(chip(words.interrupted, "cut"));
  if (session.minutes && !compact) {
    const made = chip(words.minutes);
    // Turned from what was being made: said by one small pop, once (D69).
    if (chipsMaking.delete(nameOf(session))) made.classList.add("turned");
    holds.append(made);
  }
  if (session.transcript && !compact) holds.append(chip(words.transcript));
  if (session.audio > 0 && !compact) holds.append(chip(fill(words.audio, size(session.audio, words))));
  // Freed is said of a meeting no longer recorded: one being recorded may have no track written yet, its
  // microphone waiting (`session-audio/v2`, a track gained later) — nothing was deleted (#1054).
  if (session.audio === 0 && session.state !== "recording") holds.append(chip(words.audio_freed, "gone"));
  // What is being made for it, in the background (D25): painted from `makings`.
  const making = chip("", "making");
  making.dataset.name = nameOf(session);
  // Minutes there already, made again: the chip that follows has not turned from anything.
  making.dataset.first = String(!session.minutes);
  making.hidden = true;
  holds.append(making);
  return holds;
}

// What Kitokey makes in the background (D25): read with the header, painted where it is shown.
let makings = null;
const chipsMaking = new Set();
// The meeting last stopped, as the header says it, and the bar's way to say it with its stage (D92).
let lastStopped = null;
let paintLast = null;

function minutesState(name) {
  if (makings === null) return null;
  if (makings.minutes?.name === name) return { kind: "making", ...makings.minutes };
  const waiting = makings.minutes_waiting.indexOf(name);
  if (waiting >= 0) return { kind: "waiting", before: waiting + 1 };
  if (makings.awaiting[name] !== undefined) return { kind: "awaiting", host: makings.awaiting[name] };
  // Confidential, waiting for Ollama: silent, or holding no model (D60).
  if (makings.ollama_waiting[name] !== undefined) {
    return { kind: "ollama", silent: makings.ollama_waiting[name] === "silent", busy: makings.model_fetching !== null };
  }
  if (makings.failed[name] !== undefined) return { kind: "failed", why: makings.failed[name], said: makings.failed_said[name] };
  // Cancelled by the user (D92): said, until it is made again.
  if (makings.stopped[name] === "minutes") return { kind: "stopped" };
  return null;
}

function transcriptState(name) {
  if (makings === null) return null;
  if (makings.transcript?.name === name) return { kind: "making", ...makings.transcript };
  const waiting = makings.transcripts_waiting.indexOf(name);
  // Nothing being made and the model not there: the line waits for it, not for a turn.
  if (waiting >= 0 && makings.transcript === null && makings.model_needed) {
    return { kind: "model", coming: makings.model_coming, busy: makings.model_fetching !== null && !makings.model_coming };
  }
  if (waiting >= 0) return { kind: "waiting", before: waiting + 1 };
  if (makings.transcripts_failed[name] !== undefined) return { kind: "failed", why: makings.transcripts_failed[name] };
  if (makings.stopped[name] === "transcript") return { kind: "stopped" };
  return null;
}

// The card whose « Annuler » was pressed, and its meeting: once it says stopped and offers to make it
// again, the focus goes there — kept until then, the offer drawn only when the page is drawn anew.
let cancelledFrom = null;

// A refused « Annuler », said for the line `of` (D92): an alert, which a screen reader says as it
// is inserted — a status says its changes, not its arrival (WAI-ARIA, the APG's alert pattern).
function refusalNote(of, text) {
  const note = element("p", "note warn", text);
  note.dataset.refused = of;
  note.setAttribute("role", "alert");
  return note;
}

// « Annuler » (D92): what is made of the meeting ends within two seconds, or leaves the line.
function cancelButton(said, id) {
  const cancel = element("button", "open", words.making_cancel);
  cancel.type = "button";
  if (id) cancel.id = id;
  cancel.addEventListener("click", async () => {
    waiting(cancel, true);
    cancelledFrom = { id: said.id, name: said.dataset.name };
    // A refusal said before was the last gesture's: this one says its own.
    document.querySelector(`.note[data-refused="${said.id}"]`)?.remove();
    const ended = await window.__TAURI__.core.invoke("making_cancelled", { name: said.dataset.name }).then(() => true, (why) => {
      // Said beside the line, which is drawn again as the makings are read: kept until the next gesture.
      said.after(refusalNote(said.id, fill(words.making_cancel_failed, why)));
      cancelledFrom = null;
      waiting(cancel, false);
      return false;
    });
    // Asked: it ends within two seconds; read again until its line says it, not at the page's
    // next reading, a second and a half later (D92).
    for (let tries = 0; tries < (ended ? 15 : 1); tries++) {
      if (tries > 0) await new Promise((wait) => setTimeout(wait, 200));
      await readMakings();
      if (!said.classList.contains("making") && !said.classList.contains("waiting")) break;
    }
  });
  return cancel;
}

// One card's state, said where its content is: what makes it and for how long, its place in
// line, the word it waits for, or why it was not made, with Try again (D25).
function paintState(said, shown, of) {
  said.hidden = shown === null;
  said.className = `making-state ${shown?.kind ?? ""}`;
  // Being made, waiting, or failed with Try again: not offered a second time. Stopped, it is.
  const offered = document.getElementById(of.offer);
  if (offered) offered.hidden = shown !== null && shown.kind !== "stopped";
  // Drawn again only when it says something else — a link or a button under the pointer or the
  // keyboard's focus stays the same one —; while it is made, only its time moves.
  const { seconds, ...same } = shown ?? {};
  const key = shown === null ? "" : JSON.stringify(same);
  const minutes = `${Math.floor((seconds ?? 0) / 60)}:${String((seconds ?? 0) % 60).padStart(2, "0")}`;
  const by = shown?.kind !== "making" ? "" : shown.host === undefined ? spoken(shown.model) : `${modelSaid(shown.model)} · ${shown.host}`;
  if (said.dataset.key === key && said.childElementCount > 0) {
    const time = said.querySelector(".by");
    if (time) time.textContent = `${by} · ${minutes}`;
    return;
  }
  said.dataset.key = key;
  if (shown === null) return said.replaceChildren();
  // Being made: said by the page's one bar across both steps, not by its card (D92, `paintBar`).
  if (shown.kind === "making") {
    said.hidden = true;
    said.replaceChildren();
  } else if (shown.kind === "waiting") {
    said.replaceChildren(element("span", "what", fill(words.minutes_waiting, shown.before)), cancelButton(said));
  } else if (shown.kind === "stopped") {
    const stopped = element("span", "what", of.stopped);
    stopped.id = "making-stopped";
    said.replaceChildren(stopped);
    if (cancelledFrom?.id === said.id && cancelledFrom.name === said.dataset.name && offered && !offered.hidden) {
      offered.focus();
      cancelledFrom = null;
    }
  } else if (shown.kind === "model") {
    if (shown.coming) said.replaceChildren(element("span", "what", words.transcript_model_coming));
    else said.replaceChildren(element("span", "what", words.transcript_model_needed), modelFetch(shown.busy, words.transcript_fetch_then));
  } else if (shown.kind === "ollama") {
    // Silent: said where to get it, and tried again on a click; holding none: its model offered,
    // as Settings offer it — once it is there, the minutes are made.
    if (shown.silent) {
      const again = element("button", "open", of.try_again);
      again.type = "button";
      again.id = of.again;
      again.addEventListener("click", () => askAgain(of, said, again));
      said.replaceChildren(element("span", "what", words.welcome_ollama_silent), again);
    } else {
      const fetch_ = element("button", "open", fill(words.minutes_fetch_then, makings.ollama_recommended));
      fetch_.type = "button";
      fetch_.id = "ollama-fetch";
      fetch_.disabled = shown.busy;
      fetch_.addEventListener("click", async () => {
        waiting(fetch_, true);
        await window.__TAURI__.core.invoke("ollama_pull_offered").catch(() => {});
        readMakings();
      });
      said.replaceChildren(element("span", "what", words.confidential_ollama_empty), fetch_);
    }
  } else if (shown.kind === "awaiting") {
    const agree = element("a", "open", words.agree_open);
    agree.id = "minutes-agree";
    agree.href = `#/agree/${said.dataset.name}`;
    said.replaceChildren(element("span", "what", fill(words.minutes_awaiting, shown.host)), agree);
  } else {
    const again = element("button", "open", of.try_again);
    again.type = "button";
    again.id = of.again;
    again.addEventListener("click", () => askAgain(of, said, again));
    // What the program said, only in developer mode: the shell sends nothing else (D29).
    const detail = shown.why ? [element("code", "detail", fill(words.program_said, shown.why))] : [];
    // Its cause and gesture, when the program named one (D29): else the step's sentence alone.
    const what = shown.said ? `${of.not_made} ${shown.said}` : of.not_made;
    said.replaceChildren(element("span", "what", what), again, ...detail);
  }
}

// The tray's own fetch of the default model, offered where the window says it is missing (D36).
// On a meeting's page it names what follows, the transcript (#933); in Settings, the download alone.
function modelFetch(busy = false, said = words.model_offer) {
  const fetch_ = element("button", "open", fill(said, size(makings.model_offer_bytes, words), makings.model_offer_host));
  fetch_.type = "button";
  fetch_.id = "model-fetch";
  // Fetched from Hugging Face: its mark before the words, which still name the host (#1238).
  if (/(^|\.)huggingface\.co$/.test(makings.model_offer_host ?? "")) fetch_.prepend(brandMark("huggingface"));
  fetch_.disabled = busy;
  fetch_.addEventListener("click", async () => {
    // Where the user is, read before the button goes: what says the model coming takes its place (D101).
    const place = placeOf();
    waiting(fetch_, true);
    await window.__TAURI__.core.invoke("model_fetch").catch(() => {});
    await readMakings();
    placeBack(place);
  });
  return fetch_;
}

// Each model of Settings' list: here, coming and how far, or offered — one fetch at a time (D37).
function paintOffered(list = document.getElementById("offered")) {
  if (!list || makings === null) return;
  const coming = makings.model_fetching;
  for (const item of list.children) {
    const far = coming?.file === item.dataset.offered ? Math.floor((100 * coming.held) / Math.max(coming.bytes, 1)) : null;
    const absent = item.dataset.by === "absent";
    const kind = item.dataset.here === "true" ? "here" : far !== null ? `far ${far}` : absent ? "absent" : coming ? "busy" : "offered";
    if (item.dataset.kind === kind) continue;
    item.dataset.kind = kind;
    const state = item.querySelector(".state");
    if (kind === "here") state.replaceChildren(readySaid(words.on_this_computer));
    else if (far !== null) state.replaceChildren(element("span", "says", fill(words.offered_far, far)));
    else if (absent) state.replaceChildren(element("span", "says", words.offered_needs_ollama));
    else {
      const fetch_ = element("button", "open", words.offered_fetch);
      fetch_.type = "button";
      fetch_.disabled = kind === "busy";
      fetch_.addEventListener("click", async () => {
        waiting(fetch_, true);
        const { invoke } = window.__TAURI__.core;
        const asked = item.dataset.by ? invoke("ollama_pull_offered") : invoke("model_offered_fetch", { file: item.dataset.offered });
        await asked.catch(() => {});
        readMakings();
      });
      state.replaceChildren(fetch_);
    }
  }
}

// Settings' model, not on this computer: coming, offered, or one the user named — theirs (D36).
function paintModel(state = document.getElementById("model-state")) {
  if (!state?.dataset.missing || makings === null) return;
  const busy = makings.model_fetching !== null && !makings.model_coming;
  const kind = makings.model_coming ? "coming" : makings.model_needed ? (busy ? "needed busy" : "needed") : "named";
  if (state.dataset.kind === kind) return;
  // Offered or coming, and now neither: the default model came — the page drawn as it now is,
  // once: the drawing paints the makings again, this card still in it.
  const came = kind === "named" && state.dataset.kind;
  state.dataset.kind = kind;
  if (came) return refresh();
  state.querySelector(".says").textContent = { coming: words.model_coming, named: words.model_named_missing }[kind] ?? words.model_missing;
  state.querySelector("#model-fetch")?.remove();
  // Another model coming: offered, not yet clickable — one fetch at a time (D37).
  if (kind.startsWith("needed")) state.append(modelFetch(busy));
}

// The model being fetched when the page was last painted: once it ends, Settings are drawn again.
let lastFetching = null;

function paintMakings() {
  paintLast?.();
  // Drawn again first, never painted then drawn over: what is under the focus stays one drawing.
  const fetching = makings?.model_fetching?.file ?? null;
  const ended = lastFetching !== null && fetching === null;
  lastFetching = fetching;
  if (ended && document.getElementById("offered")) return refresh();
  paintModel();
  paintOffered();
  for (const chip_ of document.querySelectorAll(".chip.making")) {
    // The minutes' state first; else the transcript's, which comes before them. Stopped: no chip.
    const chipped = (state) => (state?.kind === "stopped" ? null : state);
    const minutes = chipped(minutesState(chip_.dataset.name));
    const transcript = chipped(transcriptState(chip_.dataset.name));
    const state = minutes ?? transcript;
    chip_.hidden = state === null;
    if (state === null) continue;
    if (minutes && chip_.dataset.first === "true") chipsMaking.add(chip_.dataset.name);
    // Minutes kept, a redo of them failed (#772): said as the redo's failure, never as none made.
    const kept = chip_.dataset.first === "false";
    chip_.textContent = minutes
      ? { making: words.chip_making, waiting: words.chip_waiting, ollama: words.chip_waiting, awaiting: words.chip_awaiting, failed: kept ? words.chip_redo_failed : words.chip_failed }[state.kind]
      : { making: words.chip_transcribing, waiting: words.chip_waiting, model: words.chip_waiting, failed: words.chip_transcript_failed }[state.kind];
    chip_.classList.toggle("failed", state.kind === "failed");
  }
  const minutes = document.getElementById("minutes-state");
  if (minutes) {
    paintState(minutes, minutesState(minutes.dataset.name), {
      offer: "make-minutes", again: "minutes-again", try_again: words.minutes_try_again, ask: "minutes_again", making: words.minutes_making, stopped: words.minutes_stopped,
      // The minutes kept (#772): the redo failed, the ones shown are those before it.
      not_made: minutes.dataset.kept === "true" ? words.minutes_redo_failed : words.minutes_not_made,
    });
  }
  // The summary not written to the export folder, or a confidential meeting's never (D109, D60).
  const exported = document.getElementById("export-said");
  if (exported) {
    const failed = makings?.not_exported.includes(exported.dataset.name);
    const kept = makings?.export && exported.dataset.confidential === "true";
    const text = failed ? words.export_failed : kept ? words.export_confidential : "";
    if (exported.textContent !== text) exported.textContent = text;
    exported.hidden = text === "";
  }
  // No line corrected while the transcript is made or waits to be (D62): its pencils hidden.
  for (const pencil of document.querySelectorAll("#meeting .lines .edit")) {
    if (pencil.parentElement.querySelector(".correcting")) continue;
    const kind = transcriptState(pencil.dataset.name)?.kind;
    // Nor while its minutes are written: they read the transcript before (#528's verification).
    pencil.hidden = (kind !== undefined && kind !== "failed" && kind !== "stopped") || minutesState(pencil.dataset.name)?.kind === "making";
  }
  const transcript = document.getElementById("transcript-state");
  if (transcript) {
    paintState(transcript, transcriptState(transcript.dataset.name), {
      offer: "make-transcript", again: "transcript-again", try_again: words.transcript_try_again, ask: "transcript_again", making: words.transcript_making, not_made: words.transcript_not_made, stopped: words.transcript_stopped,
    });
  }
  paintContext();
  // A meeting's steps both ended — its page shown or not —: its summary made again alone later is one part.
  for (const name of barChained) {
    if (transcriptState(name)?.kind !== "making" && !makings?.between.includes(name) && minutesState(name)?.kind !== "making") barChained.delete(name);
  }
  paintBar();
}

// A meeting's page while it is made (D92): one bar across both steps — its transcript, then its summary
// when one follows —, naming the step and what makes it, each part full once done and never moving back,
// « Annuler » in it for the step running. Between the two steps, the summary's part is said running.
const barChained = new Set();

function barOf(name) {
  if (makings === null) return null;
  const heard = transcriptState(name);
  const minutes = minutesState(name);
  if (heard?.kind === "making") {
    if (heard.then) barChained.add(name);
    return { step: "transcript", parts: heard.then ? ["running", "next"] : ["running"], shown: heard };
  }
  const between = makings.between.includes(name);
  const chained = between || barChained.has(name);
  if (minutes?.kind === "making") return { step: "minutes", parts: chained ? ["done", "running"] : ["running"], shown: minutes };
  if (between) return { step: "minutes", parts: ["done", "running"], shown: null };
  return null;
}

function paintBar() {
  const bar = document.getElementById("making");
  if (!bar || document.getElementById("meeting").hidden) return;
  const state = barOf(bar.dataset.name);
  bar.hidden = state === null;
  if (state === null) return;
  // The step said by a polite live region: its text changes only when the step does.
  const step = document.getElementById("making-step");
  const said = state.step === "transcript" ? words.transcript_making : words.minutes_making;
  if (step.textContent !== said) step.textContent = said;
  const shown = state.shown;
  const by = shown === null ? "" : shown.host === undefined ? spoken(shown.model) : `${modelSaid(shown.model)} · ${shown.host}`;
  const time = shown === null ? "" : `${Math.floor(shown.seconds / 60)}:${String(shown.seconds % 60).padStart(2, "0")}`;
  bar.querySelector(".by").textContent = by ? `${by} · ${time}` : "";
  // What is left, from the speeds measured here: none known, nothing said — never a number made up.
  const minutesLeft = makings.left?.[bar.dataset.name];
  const left = bar.querySelector(".left");
  left.textContent = minutesLeft === undefined ? "" : fill(words.making_left, minutesLeft);
  left.hidden = minutesLeft === undefined;
  const parts = bar.querySelector(".segments");
  if (parts.childElementCount !== state.parts.length) parts.replaceChildren(...state.parts.map(() => element("span", "segment")));
  state.parts.forEach((part, at) => { parts.children[at].dataset.state = part; });
  // « Annuler » for the step running, its card's: cancelled, said there, as before (D92, criterion 4).
  const card = document.getElementById(state.step === "transcript" ? "transcript-state" : "minutes-state");
  const holder = bar.querySelector(".cancel");
  if (shown === null || !card) holder.replaceChildren();
  else if (holder.dataset.for !== state.step || !holder.firstChild) {
    holder.dataset.for = state.step;
    holder.replaceChildren(cancelButton(card, "making-cancel"));
  }
}

// *Réessayer*: fails only when the meeting is gone, said there (D67); a later failure is on this page.
async function askAgain(of, said, again) {
  waiting(again, true);
  try {
    await window.__TAURI__.core.invoke(of.ask, { name: said.dataset.name });
  } catch (why) {
    said.replaceChildren(element("span", "what", fill(words.again_refused, why)));
    return;
  }
  // Failing again for the same reason is said again: the state drawn anew.
  said.dataset.key = "";
  readMakings();
}

// « Enregistré », a moment, in the line of what was changed (D68, D101): the page is drawn again by
// what it says, so the sentence is put there after the drawing, by the name of what changed.
let savedFor = null;
let lastChanged = "";
const remembered = (event) => {
  const control = event.target.closest?.("input, select, textarea, button");
  if (!control) return;
  lastChanged = control.id ? `#${CSS.escape(control.id)}` : control.name ? `[name="${CSS.escape(control.name)}"][value="${CSS.escape(control.value)}"]` : "";
};
document.addEventListener("change", remembered, true);
document.addEventListener("click", remembered, true);
function savedSaid(saved) {
  clearTimeout(savedFor);
  savedGone();
  if (!saved) return;
  const savedSentence = element("span", "saved-said", words.saved);
  savedSentence.id = "saved-said";
  savedSentence.setAttribute("role", "status");
  savedSentence.setAttribute("aria-live", "polite");
  // Said in the line changed itself, at its end, as a system's settings say a setting applied on its
  // own line: over its description, hidden the while — never over its name or its control —, or on
  // its name's line when it has none, or beside a button in no line. Laid over the line, never in its
  // flow: the page neither grows nor moves (the coordinator, 2026-11-17, D101).
  const changed = lastChanged ? document.querySelector(`#settings ${lastChanged}`) : null;
  const row = changed?.closest("label, .row, .field") ?? null;
  const box = row ?? changed?.parentElement ?? document.getElementById("settings");
  box.classList.add("saying");
  box.append(savedSentence);
  const description = row?.querySelector(".says:not(:empty)") ?? null;
  const name = row?.querySelector(".name") ?? null;
  if (description) {
    // Its description's own box, the sentence at its end.
    savedSentence.style.top = `${description.offsetTop}px`;
    savedSentence.style.left = `${description.offsetLeft}px`;
    savedSentence.style.width = `${description.offsetWidth}px`;
    description.classList.add("saved-under");
  } else if (name) {
    // The name's line, at the end of the column it is in — its text left as it is.
    const column = name.parentElement === box ? null : name.parentElement;
    savedSentence.style.top = `${name.offsetTop}px`;
    savedSentence.style.left = `${column ? column.offsetLeft : 0}px`;
    savedSentence.style.width = `${column ? column.offsetWidth : box.clientWidth}px`;
  } else if (changed) {
    savedSentence.style.top = `${changed.offsetTop}px`;
    savedSentence.style.left = `${changed.offsetLeft + changed.offsetWidth + 8}px`;
  }
  savedFor = setTimeout(savedGone, 1500);
}

// « Enregistré » gone, and what it was laid over shown again.
function savedGone() {
  const sentence = document.getElementById("saved-said");
  if (!sentence) return;
  sentence.closest(".saying")?.classList.remove("saying");
  for (const under of document.querySelectorAll(".saved-under")) under.classList.remove("saved-under");
  sentence.remove();
}

// Each reading of the makings numbered: an answer a later reading has overtaken is dropped — the
// latest wins, never a making said still made over the stop that followed it (D92).
let makingsAsked = 0;

async function readMakings() {
  const asked = ++makingsAsked;
  const read = await window.__TAURI__.core.invoke("makings").catch(() => null);
  if (asked !== makingsAsked) return;
  makings = read;
  // Only the meeting shown: its page hidden, the last one opened is no longer in front.
  const opened = document.getElementById("meeting").hidden ? null : document.getElementById("minutes-state")?.dataset.name;
  // The minutes or the transcript of the meeting shown written since its page was drawn, however
  // quickly: drawn again, to show them — unless the user has a panel open there, or is typing in
  // it, which a redraw would take from under them; then once that is done.
  const busy = document.querySelector("#meeting .redo, #meeting input:focus, #meeting textarea:focus");
  const drawn = (id) => document.getElementById(id)?.dataset.made;
  const moved = opened && makings
    && ((String(makings.made[opened] ?? 0) !== drawn("minutes-state") && minutesState(opened)?.kind !== "making")
      || (String(makings.transcripts_made[opened] ?? 0) !== drawn("transcript-state") && transcriptState(opened)?.kind !== "making"));
  // A first transcript cancelled, its page drawn while it was made: no offer to make it again until
  // the page is drawn anew, which nothing made asks for — drawn once more, then (D92).
  const unoffered = cancelledFrom && !cancelledFrom.drawn && cancelledFrom.id === "transcript-state" && cancelledFrom.name === opened
    && transcriptState(opened)?.kind === "stopped" && !document.getElementById("make-transcript");
  if ((moved || unoffered) && !busy) {
    if (unoffered) cancelledFrom.drawn = true;
    refresh();
  }
  paintMakings();
}

// What a meeting holds, at a row's end (boards 2 and 4): an icon each, its word its name and tooltip.
function marks(session, words) {
  const held = element("span", "marks");
  for (const [on, glyph, name, kind] of [
    [session.confidential, "lock", words.confidential_chip, "locked"],
    [session.minutes, "file-text", words.minutes, "made"],
    [session.transcript, "captions", words.transcript, "heard"],
  ]) {
    if (!on) continue;
    const mark = element("span", `mark ${kind}`);
    mark.setAttribute("role", "img");
    mark.setAttribute("aria-label", name);
    mark.title = name;
    mark.append(icon(glyph));
    // Turned from what was being made: said by one small pop, once (D69).
    if (kind === "made" && chipsMaking.delete(nameOf(session))) mark.classList.add("turned");
    held.append(mark);
  }
  return held;
}

function row(session, words) {
  const item = document.createElement("li");
  const link = document.createElement("a");
  link.className = "card session";
  link.href = `#/meeting/${encodeURIComponent(nameOf(session))}`;
  link.dataset.state = session.state;
  const when = document.createElement("div");
  when.className = "when";
  const title = document.createElement("span");
  title.className = "day";
  // Named by its minutes (D48): the name whole in its tooltip, the date beneath it.
  title.textContent = session.title ?? heading(session, words);
  if (session.title) title.title = session.title;
  if (session.title) when.append(element("span", "on", heading(session, words)));
  else when.append(title);
  if (session.seconds !== null) {
    const lasted = document.createElement("span");
    lasted.className = "length";
    lasted.textContent = length(session.seconds);
    when.append(lasted);
  }
  // In the sidebar (D61, D89): the date and length, what it holds at their end; a state not at rest below.
  const line = element("div", "line");
  line.append(when, marks(session, words));
  // A name has the row's whole width, its date under it.
  if (session.title) link.append(title);
  link.append(line, holdings(session, words, true));
  item.append(link);
  return item;
}

// The header's button, as the shell says the recording is — or as the user just asked it to be,
// until the command has answered (D68).
let recordAsked = null;
let recordRefocus = false;
// The shortcut, said in the recording's tooltip rather than on a line under it (D89).
let recordKeys = null;
function paintRecord(recording, stopping) {
  const live = recordAsked ?? recording;
  // Ending, it waits: *Arrêt…*, a click then would ask nothing.
  const ending = recordAsked === false || (recordAsked === null && stopping);
  const said = ending ? words.record_stopping : live ? words.record_stop : words.record_start;
  document.getElementById("record-said").textContent = said;
  document.getElementById("record").title = recordKeys ? `${said} (${recordKeys})` : said;
  document.getElementById("record").classList.toggle("live", live);
  // The name says it too (D89): its dot red and larger while recording, the cut around it following.
  document.querySelector(".brand").classList.toggle("live", live);
  for (const id of ["name-dot", "name-gap"]) {
    const circle = document.getElementById(id);
    circle.setAttribute("r", live ? circle.dataset.live : circle.dataset.rest);
  }
  // Disabled, it loses the focus: given back with the button, for whoever stopped by the keyboard.
  const button = document.getElementById("record");
  if (ending && document.activeElement === button) recordRefocus = true;
  button.disabled = ending;
  if (!ending && recordRefocus) {
    recordRefocus = false;
    if (document.activeElement === document.body) button.focus();
  }
}

// A start refused, the trial over (D83): no failure, said as information; buying, the one action
// left, holds the primary style, the recording's gesture its secondary one until bought (D69).
function paintRefused(refused) {
  const record = document.getElementById("record");
  record.classList.toggle("primary", !refused);
  record.classList.toggle("open", refused);
  const said = document.getElementById("refused");
  said.textContent = words.trial_over;
  said.hidden = !refused;
  const buy = document.getElementById("buy");
  buy.textContent = words.buy;
  buy.hidden = !refused;
}

function show({ words, folder, sessions, recording, stopping, shortcut, folded, unreadable, refused }) {
  document.documentElement.lang = words.locale;
  // The header's gesture, as the board has it: start or stop, with the shortcut beside.
  // The sidebar's secondary actions: icons, each with its tooltip and its name; the header's one
  // action stays the recording's (D69, D89).
  for (const [id, glyph, name] of [["free-link", "hard-drive", words.free], ["small-link", "picture-in-picture-2", words.small], ["settings-link", "settings", words.settings]]) {
    const button = document.getElementById(id);
    if (!button.firstChild) button.append(icon(glyph));
    button.title = name;
    button.setAttribute("aria-label", name);
  }
  document.getElementById("free-link").hidden = sessions.length === 0;
  // At rest too: the small window is the user's to choose (D35).
  document.getElementById("small-link").hidden = false;
  recordKeys = shortcut ?? null;
  paintRecord(recording, stopping);
  paintRefused(refused);
  const keys = document.getElementById("empty-keys");
  keys.textContent = shortcut ?? "";
  keys.hidden = !shortcut;
  document.getElementById("sidebar").setAttribute("aria-label", words.meetings);
  document.getElementById("sections").setAttribute("aria-label", words.sections);
  document.getElementById("pane-back").replaceChildren(icon("chevron-left"), words.back);
  // Folded as the user left it (D61), each button named with its shortcut.
  document.body.classList.toggle("folded", folded);
  const folding = /Mac/.test(navigator.platform) ? "⌘B" : "Ctrl+B";
  for (const [id, name, glyph] of [["fold", words.fold, "panel-left-close"], ["unfold", words.unfold, "panel-left-open"]]) {
    const button = document.getElementById(id);
    if (!button.firstChild) button.append(icon(glyph));
    button.setAttribute("aria-label", name);
    button.title = `${name} (${folding})`;
  }
  const find = document.getElementById("find");
  find.placeholder = words.find_meetings;
  find.setAttribute("aria-label", words.find_meetings);
  // The search field's icon in it, as the board draws it (D89); the field and it hidden with no meeting.
  if (!find.previousElementSibling) find.before(icon("search"));
  find.hidden = find.parentElement.hidden = sessions.length === 0;
  document.getElementById("none-found").textContent = words.none_found;
  const weight = document.getElementById("weight");
  weight.hidden = sessions.length === 0;
  // A folder that cannot be read is no empty list: said, and tried again (D67).
  document.getElementById("none").textContent = unreadable ? fill(words.list_unread, folder) : words.none;
  // Where Kitokey looks, said under the action, never in the sentence (#775): a path is no title.
  document.getElementById("empty-folder").textContent = unreadable ? "" : fill(words.none_folder, folder);
  document.getElementById("empty-folder").hidden = unreadable;
  document.getElementById("none-do").textContent = unreadable ? words.list_unread_do : words.none_do;
  document.getElementById("empty-record").textContent = words.record_start;
  document.getElementById("empty-record").hidden = recording || unreadable;
  document.getElementById("empty-again").textContent = words.try_again;
  document.getElementById("empty-again").hidden = !unreadable;

  const audio = sessions.reduce((sum, s) => sum + s.audio, 0);
  const text = sessions.reduce((sum, s) => sum + s.text, 0);
  document.getElementById("total").textContent = size(audio + text, words);
  const taken = sessions.length === 1 ? words.weight_one : words.weight;
  document.getElementById("taken").textContent = fill(taken, sessions.length);

  const now = new Date();
  const groups = document.getElementById("groups");
  // Drawn anew at each reading: the row that held the focus holds it again.
  const focused = document.activeElement?.closest?.("#groups .session")?.getAttribute("href");
  groups.replaceChildren();
  let current = null;
  for (const session of sessions) {
    const name = group(dayOf(session), now, words);
    if (!current || current.name !== name) {
      const section = document.createElement("section");
      section.className = "group";
      const heading = document.createElement("h2");
      heading.textContent = name;
      const list = document.createElement("ol");
      section.append(heading, list);
      groups.append(section);
      current = { name, list };
    }
    current.list.append(row(session, words));
  }
  if (focused) groups.querySelector(`.session[href="${CSS.escape(focused)}"]`)?.focus({ preventScroll: true });
  sought();
}

// The sidebar's search (D61): the meetings whose row holds what is typed, the others hidden.
function sought() {
  // Accents and case aside, as the transcript's own search reads.
  const typed = folded(document.getElementById("find").value.trim());
  let any = false;
  for (const section of document.querySelectorAll("#groups .group")) {
    let some = false;
    for (const item of section.querySelectorAll("li")) {
      item.hidden = typed !== "" && !folded(item.textContent).includes(typed);
      marking(item, item.hidden ? "" : typed);
      some ||= !item.hidden;
    }
    section.hidden = !some;
    any ||= some;
  }
  document.getElementById("none-found").hidden = any || typed === "";
}

// How long the small window says the reminder, in seconds (D50).
const ANNOUNCED_FOR = 15;
// How long a side is silent before the bar says it, in seconds (D90).
const SILENT_SAID = 120;

const element = (tag, className, text) => {
  const made = document.createElement(tag);
  if (className) made.className = className;
  if (text !== undefined) made.textContent = text;
  return made;
};

// A meeting's context (D59): the field, its count, and the script it is written in. Never more
// than the shell takes, counted as it counts: in characters, not in UTF-16's units.
const CONTEXT_MOST = 400;
// Latin and Cyrillic are heard whole within the 400; another script — Greek, Arabic, Chinese… —
// may be cut by Whisper or refused by a service (#489).
const otherScript = /[^\p{Script=Latin}\p{Script=Cyrillic}\p{Script=Common}\p{Script=Inherited}]/u;

function contextField(words, text, save, id) {
  const area = element("textarea", "context-text");
  area.id = id;
  area.value = text;
  const count = element("span", "context-count");
  count.id = `${id}-count`;
  // Announced as it changes, as a status is.
  count.setAttribute("role", "status");
  area.setAttribute("aria-describedby", count.id);
  const painted = () => {
    const left = CONTEXT_MOST - [...area.value].length;
    const leftSaid = fill(words.context_left[left <= 1 ? 0 : 1], left);
    const other = otherScript.test(area.value);
    count.textContent = other ? `${leftSaid} · ${words.context_script}` : `${fill(words.context_most, CONTEXT_MOST)} · ${leftSaid}`;
    count.classList.toggle("other-script", other);
  };
  // A character typed past the most is not taken; a paste or a drop too long is let in, then cut
  // to it below — refused whole, nothing of it would be taken.
  area.addEventListener("beforeinput", (event) => {
    if (/^insertFrom(Paste|Drop)/.test(event.inputType)) return;
    const data = event.data ?? (/^insert(LineBreak|Paragraph)$/.test(event.inputType) ? "\n" : null);
    if (data === null || !event.inputType.startsWith("insert")) return;
    const replaced = [...area.value.slice(area.selectionStart, area.selectionEnd)].length;
    if ([...area.value].length - replaced + [...data].length > CONTEXT_MOST) event.preventDefault();
  });
  area.addEventListener("input", () => {
    const typed = [...area.value];
    if (typed.length > CONTEXT_MOST) area.value = typed.slice(0, CONTEXT_MOST).join("");
    painted();
    save(area.value);
  });
  painted();
  return { area, count, painted };
}

// The meeting whose context was opened from its row: open still when its page is drawn again.
let contextOpen = null;

// The context on a meeting's page: shown and written, closed while a making of it runs or its
// transcript is left unfinished, and made again with it when what was made had another. Its row
// (D89, board 4): the context, opened by it, then the meeting's other settings; their sentences under it.
function contextCard(session, { context, transcript, minutes }, words, settings, sentences) {
  const name = nameOf(session);
  const card = element("section", "settings context");
  card.id = "context";
  card.dataset.name = name;
  card.dataset.changed = String(context.changed);
  // Unfinished, nothing making it: continued with another context, it would be refused.
  card.dataset.unfinished = String(transcript.state === "present" && !transcript.complete);
  const said = element("p", "says");
  said.id = "context-said";
  said.setAttribute("role", "status");
  const changed = element("div", "context-changed");
  changed.id = "context-changed";
  // It names what it starts (D113): the transcript, and the summary when one follows it — the meeting
  // has one, or Settings make it —, as the shell says each time it reads the context.
  const redo = element("button", "open");
  const naming = (shown) => (redo.textContent = words.context_redo[shown.minutes_after ? 1 : 0]);
  naming(context);
  redo.type = "button";
  redo.id = "context-redo";
  redo.addEventListener("click", async () => {
    waiting(redo, true);
    await window.__TAURI__.core.invoke("context_redo", { name }).catch((why) => {
      said.textContent = fill(words.not_saved, why);
    });
    await readMakings();
    waiting(redo, false);
  });
  changed.append(element("p", "says", words.context_changed[minutes.state === "present" ? 1 : 0]), redo);
  const toggle = element("button", "setting context-toggle");
  toggle.type = "button";
  toggle.id = "context-toggle";
  toggle.setAttribute("aria-controls", "context-panel");
  const named = element("span", "name", fill(words.labelled, words.context_title));
  const value = element("b", "value");
  toggle.append(icon("message-square-text"), named, value);
  const valued = (text) => {
    named.hidden = text.trim() === "";
    value.textContent = text.trim() || words.context_add;
  };
  const panel = element("div", "context-panel");
  panel.id = "context-panel";
  const opened = (open) => {
    panel.hidden = !open;
    toggle.setAttribute("aria-expanded", String(open));
    contextOpen = open ? name : null;
  };
  const save = async (text) => {
    valued(text);
    // Written now: what it could not read back is no longer the last word.
    delete card.dataset.unread;
    try {
      const shown = await window.__TAURI__.core.invoke("context_saved", { name, text });
      card.dataset.changed = String(shown.changed);
      naming(shown);
      said.textContent = "";
    } catch {
      said.textContent = words.context_not_saved;
    }
    paintContext();
  };
  const { area, count } = contextField(words, context.text, save, "context-text");
  area.rows = 3;
  area.placeholder = words.context_example;
  const label = element("label", "context-label", words.context_label);
  label.htmlFor = area.id;
  // Made again from its audio: a meeting whose audio was freed is not offered it.
  card.dataset.redo = String(session.audio > 0);
  toggle.addEventListener("click", () => {
    opened(panel.hidden);
    if (!panel.hidden) area.focus();
  });
  valued(context.text);
  opened(contextOpen === name);
  const row = element("div", "settings-row");
  row.append(toggle, ...settings);
  panel.append(label, area, count, element("p", "says", words.context_why));
  card.append(row, ...sentences, said, changed, panel);
  return card;
}

// The context's field as the makings stand: closed and said while one of its meeting runs.
function paintContext() {
  const card = document.getElementById("context");
  if (!card) return;
  const name = card.dataset.name;
  const making = minutesState(name)?.kind === "making" || transcriptState(name)?.kind === "making";
  const closed = making ? words.context_making : card.dataset.unfinished === "true" ? words.context_unfinished : null;
  const area = document.getElementById("context-text");
  // A making of it ended: whether what is there now was made with this context is read again.
  if (card.dataset.making === "true" && !making) {
    window.__TAURI__.core
      .invoke("context_shown", { name })
      .then((shown) => {
        card.dataset.changed = String(shown.changed);
        const redo = document.getElementById("context-redo");
        if (redo) redo.textContent = words.context_redo[shown.minutes_after ? 1 : 0];
        delete card.dataset.unread;
        paintContext();
      })
      // Its flag and its text stay as they were, and that they could not be read back is said.
      .catch((why) => {
        card.dataset.unread = fill(words.context_not_read, why);
        paintContext();
      });
  }
  card.dataset.making = String(making);
  area.readOnly = closed !== null;
  const said = document.getElementById("context-said");
  if (closed !== null) said.textContent = closed;
  else if (card.dataset.unread) said.textContent = card.dataset.unread;
  else if (said.textContent !== words.context_not_saved) said.textContent = "";
  said.hidden = said.textContent === "";
  document.getElementById("context-changed").hidden = closed !== null || card.dataset.changed !== "true" || card.dataset.redo !== "true";
}

// A time running, as the bar says it (D90): hours, minutes and seconds.
const elapsed = (seconds) => {
  const whole = Math.floor(seconds);
  return [Math.floor(whole / 3600), Math.floor((whole % 3600) / 60), whole % 60].map((n) => String(n).padStart(2, "0")).join(":");
};

// A level meter's drawing (#1179, the decider's « B · Trace » of 2026-11-25): the last readings, 0 to 1, as bars, the
// newest at the right, one bar a reading — read ten times a second, a bar is a tenth of a second, and the
// trace moves left a bar a reading, as Voice Memos and Otter draw a recording. Who asks for less motion is drawn a
// level held a second, in three steps — silent, heard, loud —, the same on every bar: nothing scrolls (D69). Pure:
// tests/meter.rs runs it.
function levelShown(was, reading, now, still, count) {
  const percent = Math.round(Math.min(1, Math.max(0, reading)) * 100);
  if (still) {
    if (was?.held && now - was.at < 1000) return was;
    return { bars: Array(count).fill(percent < 20 ? 0 : percent < 75 ? 50 : 90), held: true, at: now };
  }
  const before = was && !was.held ? was.bars : Array(count).fill(0);
  return { bars: [...before.slice(before.length - count + 1), percent], held: false, at: now };
}

// A meter's track, the meter a screen reader reads, from 0 to 100; its bars drawn in it (`paintLevel`): four seconds
// of them, a track showing the newest it has room for (the style's `.trace`).
const TRACE = 40;
function levelTrack(classes, label) {
  const track = element("span", `trace ${classes}`);
  track.setAttribute("role", "meter");
  track.setAttribute("aria-label", label);
  track.setAttribute("aria-valuemin", "0");
  track.setAttribute("aria-valuemax", "100");
  track.setAttribute("aria-valuenow", "0");
  track.append(...Array.from({ length: TRACE }, () => document.createElement("i")));
  return track;
}

// Draws a reading on a meter's track: its bars, and its value the reading itself.
const levelsDrawn = new WeakMap();
const lessMotion = matchMedia("(prefers-reduced-motion: reduce)");
function paintLevel(track, reading) {
  drawLevel(track, levelShown(levelsDrawn.get(track), reading, performance.now(), lessMotion.matches, track.children.length), reading);
}
function drawLevel(track, shown, reading) {
  levelsDrawn.set(track, shown);
  for (const [index, bar] of [...track.children].entries()) bar.style.height = `${shown.bars[index]}%`;
  track.setAttribute("aria-valuenow", String(Math.round(Math.min(1, Math.max(0, reading)) * 100)));
}

// The page of the meeting being recorded (#1076): both sides' levels as the bar's meters, and its time,
// read from what the bar reads ten times a second (`recording_now`). Neither is a live region — a screen
// reader would say them ten times a second —, a meter's value is read when it is reached, as the bar's.
// The poll ends when the page leaves, and when the recording ends: the page is then drawn again, its range back.
let pageLive = null;
// The last reading of the meeting being recorded (#1133): a new drawing of its page starts from it, never from
// « 00:00:00 » and empty meters until the next reading.
let pageLast = null;
// Its meters' tracks, whose traces a new drawing goes on from (#1179).
let pageTracks = [];
function pageLevels(name, time, words) {
  const row = element("div", "page-levels");
  const tracks = [words.small_mic, words.small_meeting].map((label, index) => {
    // Each side's label right under its own meter (#1112): a label sits with what it names.
    const column = element("div", "page-meter");
    const track = levelTrack(index === 0 ? "track page-level" : "track page-level others", label);
    column.append(track, element("span", index === 0 ? "here" : "remote", index === 0 ? words.here_described : words.remote_described));
    row.append(column);
    return track;
  });
  const paint = (now) => {
    time.textContent = elapsed(now.seconds);
    for (const [index, track] of tracks.entries()) paintLevel(track, now.levels[index]);
    // Said confidential or no longer, by the page's switch or the bar's (#1204): its header's lock as the folder says.
    const holds = row.closest("#meeting")?.querySelector(".meeting-title .holds");
    const locked = holds?.querySelector(".chip.locked");
    if (now.confidential && holds && !locked) holds.prepend(lockedChip(words));
    if (!now.confidential) locked?.remove();
  };
  // Drawn again, its traces go on from the ones it replaces, never from empty (#1133).
  if (pageLast?.meeting === name) {
    time.textContent = elapsed(pageLast.seconds);
    for (const [index, track] of tracks.entries()) {
      const was = pageTracks[index] && levelsDrawn.get(pageTracks[index]);
      if (was) drawLevel(track, was, pageLast.levels[index]);
      else paintLevel(track, pageLast.levels[index]);
    }
  }
  pageTracks = tracks;
  // The poll ends with the page or the recording, and its reading with it: a meeting deleted frees its name within
  // the minute (capture's `new_folder`), and a recording under that name must not start from this one's.
  const end = () => {
    clearInterval(pageLive);
    pageLast = null;
    pageTracks = [];
  };
  clearInterval(pageLive);
  pageLive = setInterval(async () => {
    if (!document.contains(time)) return end();
    const now = await window.__TAURI__.core.invoke("recording_now").catch(() => null);
    if (now === null || !document.contains(time)) return;
    if (!now.recording || now.meeting !== name) {
      end();
      return refresh();
    }
    pageLast = now;
    paint(now);
  }, 100);
  return row;
}

const clock = (seconds) => {
  const whole = Math.floor(seconds);
  const parts = [Math.floor(whole / 3600), Math.floor((whole % 3600) / 60), whole % 60];
  const shown = parts[0] > 0 ? parts : parts.slice(1);
  return shown.map((part) => String(part).padStart(2, "0")).join(":");
};

// Who spoke, over the meeting: each track's segments, where they fall in it.
function relief(segments, total, words) {
  // Each segment a bar on its side's lane, yours above, the others' below (#1077): a turn is a bar on one
  // lane and a gap on the other, as a call's player draws who speaks when (Gong). Shares in bins drew two
  // parallel lines on a long meeting, someone always speaking.
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("viewBox", "0 0 1100 88");
  svg.setAttribute("preserveAspectRatio", "none");
  svg.setAttribute("role", "img");
  svg.setAttribute("aria-label", words.who_spoke);
  svg.classList.add("relief");
  const lanes = { local: 14, remote: 50 };
  for (const segment of segments) {
    if (!(segment.track in lanes) || !(total > 0)) continue;
    const x = (Math.max(0, segment.start) / total) * 1100;
    const bar = document.createElementNS("http://www.w3.org/2000/svg", "rect");
    bar.setAttribute("x", x.toFixed(1));
    bar.setAttribute("width", Math.max(1, (Math.min(total, segment.end) / total) * 1100 - x).toFixed(1));
    bar.setAttribute("y", lanes[segment.track]);
    bar.setAttribute("height", 24);
    bar.setAttribute("class", segment.track);
    svg.append(bar);
  }
  return svg;
}

// A meeting's waveform per voice (#1201, #1179 B), as Descript, Riverside and Voice Memos draw a recording: each
// voice's loudness (each stretch's RMS, `peaks`), yours above the line and the meeting's below, its bars grouped to the drawing's width and
// laid over the meeting's `total` as the playhead and a click are; each voice scaled to its own loudest, so that a
// quiet microphone still shows when it speaks. From the peaks the shell keeps (`peaks_of`); none, who spoke stays
// drawn from the transcript (`relief`).
function waveform(peaks, total) {
  const wave = element("div", "wave");
  const bins = (peaks.local ?? peaks.remote).length;
  const most = (values) => (values ? Math.max(1, ...values) : 1);
  const [here, remote] = [most(peaks.local), most(peaks.remote)];
  const tip = element("span", "tip");
  tip.hidden = true;
  const draw = () => {
    const width = wave.getBoundingClientRect().width;
    // Bars of 4 px and gaps of 2 px, as the mock-up; never more than the bins read.
    const count = Math.max(1, Math.min(bins, Math.floor((width + 2) / 6)));
    if (!width || wave.querySelectorAll(".col").length === count) return;
    const columns = [];
    for (let index = 0; index < count; index++) {
      const [from, to] = [index, index + 1].map((edge) => ((edge / count) * total * bins) / peaks.seconds);
      const loud = (values, top) => {
        if (!values) return 0;
        const part = values.slice(Math.floor(from), Math.max(Math.floor(from) + 1, Math.ceil(to)));
        return part.length ? Math.max(...part) / top : 0;
      };
      const column = element("span", "col");
      column.dataset.at = String((index / count) * total);
      const [up, down] = [element("i", "up"), element("i", "down")];
      up.style.height = `${Math.max(2, Math.round(loud(peaks.local, here) * 34))}px`;
      down.style.height = `${Math.max(2, Math.round(loud(peaks.remote, remote) * 34))}px`;
      column.append(up, down);
      columns.push(column);
    }
    wave.replaceChildren(...columns, tip);
    wave.played(wave.at ?? 0);
  };
  // What was played, whole: called as the playhead moves.
  wave.played = (time) => {
    wave.at = time;
    for (const column of wave.querySelectorAll(".col")) column.classList.toggle("played", Number(column.dataset.at) < time);
  };
  wave.addEventListener("pointermove", (event) => {
    const box = wave.getBoundingClientRect();
    const x = Math.min(Math.max(0, event.clientX - box.left), box.width);
    tip.style.left = `${x}px`;
    tip.textContent = clock((x / box.width) * total);
    tip.hidden = false;
  });
  wave.addEventListener("pointerleave", () => (tip.hidden = true));
  new ResizeObserver(() => requestAnimationFrame(draw)).observe(wave);
  return wave;
}

// A provider's host, as a model's line names it: `https://openrouter.ai/api/v1` is openrouter.ai.
function via(provider) {
  try {
    return new URL(provider).host;
  } catch {
    return provider;
  }
}

// What a meeting lacks, offered on its page and made on the user's word: once asked, said so.
function offer(card, label, id, command, session, words) {
  const button = element("button", "primary", label);
  button.type = "button";
  button.id = id;
  button.addEventListener("click", async () => {
    waiting(button, true);
    try {
      await window.__TAURI__.core.invoke(command, { name: nameOf(session) });
      // Its state is said where it is made (D25).
      button.remove();
      readMakings();
    } catch (why) {
      button.replaceWith(element("p", "note warn", fill(words.not_saved, why)));
    }
  });
  card.append(button);
  return button;
}

// « Avancé » (D91, board 6), closed by default: the meeting's own models — the transcription's and the
// minutes' —, each on Settings' choice first, said as such; the same in the bar and in *Refaire…*.
// `offered` is models_offered's answer; `chosen(models)` keeps a choice, and refused, the lists go
// back to what was kept. A confidential meeting is offered no service (D60).
function modelsAdvanced(words, prefix, offered, chosen) {
  const box = element("div", "models-advanced");
  const toggle = element("button", "disclose-advanced");
  toggle.type = "button";
  toggle.id = `${prefix}-advanced`;
  toggle.setAttribute("aria-controls", `${prefix}-advanced-panel`);
  toggle.append(icon("chevron-right"), element("span", "", words.bar_advanced));
  const panel = element("div", "models-advanced-panel");
  panel.id = `${prefix}-advanced-panel`;
  const unfold = (now) => {
    toggle.setAttribute("aria-expanded", String(now));
    toggle.firstChild.replaceWith(icon(now ? "chevron-down" : "chevron-right"));
    panel.hidden = !now;
  };
  toggle.addEventListener("click", () => unfold(toggle.getAttribute("aria-expanded") !== "true"));
  unfold(false);
  const here = (provider) => provider === offered.ollama_address;
  const writerSaid = (provider, model) => `${modelSaid(model)}${here(provider) ? ` — ${words.on_computer}` : fill(words.written_via, via(provider))}`;
  // Who hears and who writes as Settings name them: what « Celui des Réglages » means.
  const settingsHear =
    offered.engine === "speechanalyzer" ? "SpeechAnalyzer"
    : offered.engine === "service" && offered.transcription_model ? `${offered.transcription_model}${fill(words.written_via, via(offered.transcription_provider ?? ""))}`
    : spoken(offered.model);
  // Settings' entry short: the model alone, where a service is named only when it sends the meeting away.
  const settingsWrite =
    offered.minutes_provider && offered.minutes_model ? `${modelSaid(offered.minutes_model)}${here(offered.minutes_provider) ? "" : fill(words.written_via, via(offered.minutes_provider))}`
    : words.models_none;
  // Settings' own choice, as an entry of its list would be: offered once, as Settings' — unless it is the
  // meeting's own, which stays shown as chosen.
  const settingsHearing =
    offered.engine === "speechanalyzer" ? JSON.stringify({ engine: "speechanalyzer" })
    : offered.engine === "service" && offered.transcription_model ? JSON.stringify({ engine: "service", provider: offered.transcription_provider, model: offered.transcription_model })
    : JSON.stringify({ engine: "whisper", model: offered.model });
  const settingsWriting = offered.minutes_provider && offered.minutes_model ? JSON.stringify({ provider: offered.minutes_provider, model: offered.minutes_model }) : "";
  const ownHear = offered.chosen.transcription ? JSON.stringify(offered.chosen.transcription) : "";
  const ownWrite = offered.chosen.minutes ? JSON.stringify({ provider: offered.chosen.minutes.provider, model: offered.chosen.minutes.model }) : "";
  const hearing = [["", fill(words.models_settings, settingsHear)]];
  const offer = (list, value, said, settings, own) => {
    if (value !== settings || value === own) list.push([value, said]);
  };
  for (const model of offered.whisper) offer(hearing, JSON.stringify({ engine: "whisper", model }), spoken(model), settingsHearing, ownHear);
  if (offered.apple) offer(hearing, JSON.stringify({ engine: "speechanalyzer" }), "SpeechAnalyzer", settingsHearing, ownHear);
  if (offered.service) {
    const [provider, model] = offered.service;
    offer(hearing, JSON.stringify({ engine: "service", provider, model }), `${model}${fill(words.written_via, via(provider))}`, settingsHearing, ownHear);
  }
  const writing = [["", fill(words.models_settings, settingsWrite)]];
  for (const model of offered.ollama) {
    offer(writing, JSON.stringify({ provider: offered.ollama_address, model }), writerSaid(offered.ollama_address, model), settingsWriting, ownWrite);
  }
  const list = (id, label, options, value, gone) => {
    const row = element("label", "models-row");
    row.htmlFor = id;
    const pick = element("select");
    pick.id = id;
    // What the meeting chose and is no longer offered — gone, or a service it may no longer have — is
    // shown as chosen, said gone below: never silently another.
    const all = value && !options.some(([known]) => known === value) && !gone ? [...options, [value, JSON.parse(value).model ?? JSON.parse(value).engine]] : options;
    for (const [known, said] of all) {
      const option = element("option", "", said);
      option.value = known;
      pick.append(option);
    }
    pick.value = gone ? "" : value;
    // A name longer than the list is wide: said whole under it — a native list never wraps.
    const whole = element("span", "says whole");
    whole.id = `${id}-whole`;
    whole.hidden = true;
    row.append(element("span", "name", label), pick, whole);
    return [row, pick];
  };
  const measure = document.createElement("canvas").getContext("2d");
  // Said for the list's width as it is drawn: each change of it — « Avancé » opened, the window resized
  // — measures again (ResizeObserver), and each choice (`changed`). Painted at the next frame, never in
  // the observer's own turn: the line it shows or hides changes the bar's height, which the bar's own
  // observer answers by resizing the window — in the same turn, the browser reports a loop as an error.
  const sized = new ResizeObserver((entries) => {
    const picks = entries.map(({ target }) => target);
    requestAnimationFrame(() => picks.filter((pick) => pick.isConnected).forEach((pick) => paintWhole(pick)));
  });
  const paintWhole = (pick) => {
    const whole = pick.parentElement?.querySelector(".whole");
    if (!whole || pick.clientWidth === 0) return;
    const style = getComputedStyle(pick);
    measure.font = `${style.fontSize} ${style.fontFamily}`;
    const text = pick.options[pick.selectedIndex]?.textContent ?? "";
    // The list's own room: its padding and its arrow, about 36 px.
    const cut = Math.ceil(measure.measureText(text).width) + 36 > pick.clientWidth;
    whole.textContent = cut ? text : "";
    whole.hidden = !cut;
  };
  const gone = new Set(offered.gone);
  let [hearRow, hearPick] = list(`${prefix}-transcription-model`, words.models_transcription, hearing, ownHear, gone.has("transcription"));
  let [writeRow, writePick] = list(`${prefix}-minutes-model`, words.models_minutes, writing, ownWrite, gone.has("minutes"));
  const said = element("p", "note warn");
  said.id = `${prefix}-models-said`;
  said.setAttribute("role", "status");
  const goneSaid = [
    offered.unreadable ?? "",
    gone.has("transcription") ? words.models_gone_transcription : "",
    gone.has("minutes") ? words.models_gone_minutes : "",
    offered.unused.length > 0 ? words.models_unused_confidential : "",
  ].filter(Boolean).join(" ");
  said.textContent = goneSaid;
  said.hidden = goneSaid === "";
  let kept = [hearPick.value, writePick.value];
  const models = () => ({
    transcription: hearPick.value ? JSON.parse(hearPick.value) : null,
    minutes: writePick.value ? JSON.parse(writePick.value) : null,
  });
  const changed = async () => {
    try {
      await chosen(models());
      kept = [hearPick.value, writePick.value];
      said.textContent = "";
      said.hidden = true;
    } catch (why) {
      hearPick = putBack(hearPick, kept[0]);
      writePick = putBack(writePick, kept[1]);
      for (const pick of [hearPick, writePick]) {
        pick.addEventListener("change", changed);
        sized.observe(pick);
      }
      said.textContent = fill(words.models_not_kept, why);
      said.hidden = false;
    }
    paintWhole(hearPick);
    paintWhole(writePick);
  };
  hearPick.addEventListener("change", changed);
  writePick.addEventListener("change", changed);
  sized.observe(hearPick);
  sized.observe(writePick);
  // What a choice here changes, said before the lists.
  panel.append(element("p", "says", words.models_own_note), hearRow, writeRow, said);
  box.append(toggle, panel);
  // Each service set up with its key (D96): every model it lists, grouped under its name, as people
  // say them — never Settings' own again, nor one already offered.
  const grouped = (pick, offeredServices, valueOf, saidOf, settings) => {
    const known = new Set([...pick.options].map((option) => option.value));
    for (const service of offeredServices) {
      const group = element("optgroup");
      group.label = service.name;
      for (const model of service.models) {
        const value = valueOf(service.address, model);
        if (known.has(value) || value === settings) continue;
        const option = element("option", "", saidOf(service.address, model));
        option.value = value;
        group.append(option);
        known.add(value);
      }
      if (group.childElementCount > 0) pick.append(group);
    }
    paintWhole(pick);
  };
  const services = (offeredServices) => {
    grouped(writePick, offeredServices.minutes, (provider, model) => JSON.stringify({ provider, model }), writerSaid, settingsWriting);
    grouped(hearPick, offeredServices.transcription, (provider, model) => JSON.stringify({ engine: "service", provider, model }),
      (provider, model) => `${modelSaid(model)}${fill(words.written_via, via(provider))}`, settingsHearing);
  };
  // The model a step will be made by, as people say it — Settings' when none is chosen —, or null when
  // there is none (D96: the button that starts names it).
  const named = (step) => {
    const chosen = models()[step];
    if (step === "minutes") {
      const model = chosen ? chosen.model : offered.minutes_provider && offered.minutes_model;
      return model ? modelSaid(model) : null;
    }
    const heard = chosen ?? (offered.engine === "speechanalyzer" ? { engine: "speechanalyzer" }
      : offered.engine === "service" && offered.transcription_model ? { engine: "service", model: offered.transcription_model }
      : offered.model ? { engine: "whisper", model: offered.model } : null);
    if (!heard) return null;
    return heard.engine === "speechanalyzer" ? "SpeechAnalyzer" : heard.engine === "service" ? modelSaid(heard.model) : spoken(heard.model);
  };
  return { box, toggle, panel, models, unfold, services, named, get picks() { return [hearPick, writePick]; } };
}

// A panel of *Refaire…* closes on *Annuler* or Escape: nothing changed, the focus back on *Refaire…* (D68).
function closes(panel, go, button, words) {
  const cancel = element("button", "open", words.redo_cancel);
  cancel.type = "button";
  cancel.id = `${panel.id}-cancel`;
  const close = () => {
    panel.replaceWith(button);
    button.focus();
  };
  cancel.addEventListener("click", close);
  // From the panel, or from nowhere — its title clicked; a field elsewhere keeps its own Escape.
  const onEscape = (event) => {
    if (!panel.isConnected) return document.removeEventListener("keydown", onEscape);
    if (event.key !== "Escape") return;
    const at = document.activeElement;
    if (at === document.body || panel.contains(at)) close();
  };
  document.addEventListener("keydown", onEscape);
  const row = element("div", "redo-actions");
  go.replaceWith(row);
  row.append(go, cancel);
  return panel;
}

// *Refaire…* (D11's sixth board, D91): made again by the meeting's own models, Settings' first; its
// « Avancé », closed, chooses them — kept for the meeting once Refaire is pressed, never on Annuler.
function redoWith(offered, session, words, button, id, title, notes, again, step, doing) {
  const panel = element("section", "redo");
  panel.id = id;
  panel.append(element("h3", "", title), element("p", "says", words.redo_with));
  const advanced = modelsAdvanced(words, id, offered, async () => {});
  // The services set up, read as the panel opens: their models join the list when they answer (D96).
  if (!offered.confidential) {
    window.__TAURI__.core.invoke("services_offered", { name: nameOf(session) }).then(advanced.services, () => {});
  }
  const go = element("button", "primary");
  go.type = "button";
  go.id = id.replace(/-panel$/, "-do");
  // The one button that starts names the making and its model, as chosen (D96); with none, the making.
  const name = () => {
    const model = advanced.named(step);
    go.textContent = model ? fill(doing, model) : title;
  };
  name();
  for (const pick of advanced.picks) pick.addEventListener("change", name);
  go.addEventListener("click", async () => {
    waiting(go, true);
    try {
      const chosen = advanced.models();
      const own = { transcription: offered.chosen.transcription ?? null, minutes: offered.chosen.minutes ?? null };
      if (JSON.stringify(chosen) !== JSON.stringify(own)) {
        await window.__TAURI__.core.invoke("meeting_models_chosen", { name: nameOf(session), chosen });
        // Another choice kept: the page's lists read it again (D93).
        pageOffered = null;
      }
      await again();
      // Asked: the panel goes, the making is said where it is made (D25).
      panel.replaceWith(button);
      readMakings();
    } catch (why) {
      // Not started: said why and what to do, the button kept for another try (#771).
      wrong.textContent = fill(words.redo_not_done, why);
      wrong.hidden = false;
      waiting(go, false);
    }
  });
  const wrong = element("p", "note warn");
  wrong.hidden = true;
  panel.append(advanced.box, ...notes, wrong, go);
  return closes(panel, go, button, words);
}

function redoPanel(offered, session, words, button) {
  return redoWith(offered, session, words, button, "redo-panel", words.redo_title, [element("p", "says", words.redo_note)], () =>
    window.__TAURI__.core.invoke("minutes_redo", { name: nameOf(session), provider: null, model: null }), "minutes", words.redo_minutes_do);
}

function redoTranscriptPanel(offered, session, words, button, corrected) {
  const note = element("p", "says", words.redo_transcript_note);
  note.id = "redo-transcript-note";
  const notes = [note];
  // Made again, the transcript alone: the summary there is kept, said so (the decider, 2026-11-18, D113).
  if (session.minutes) {
    const kept = element("p", "says", words.redo_keeps_minutes);
    kept.id = "redo-keeps-minutes";
    notes.push(kept);
  }
  // Made again, the transcript is another file: a person's corrections are not in it (ADR-0014).
  if (corrected) {
    const lost = element("p", "note warn", words.redo_loses_corrections);
    lost.id = "redo-loses-corrections";
    notes.push(lost);
  }
  return redoWith(offered, session, words, button, "redo-transcript-panel", words.redo_transcript_title, notes, () =>
    window.__TAURI__.core.invoke("transcript_redo_own", { name: nameOf(session) }), "transcription", words.redo_transcript_do);
}

// D93, criterion 5: what a meeting's page offers to make it by — models_offered's answer, read once a
// page's opening by the first line that offers a choice, never at each drawing; shared by its two
// lines, which keep the meeting's own choice whole between them. Dropped once another meeting is
// shown, once it is said confidential or no longer (`at`), and once Refaire… keeps another choice.
// Said so while the page is open, its lines are not drawn again (D101): `again` reads it anew for them.
let pageOffered = null;

function pageModels(session) {
  const at = `${nameOf(session)}\n${session.confidential}`;
  if (pageOffered?.at === at) return pageOffered;
  const page = { at, name: nameOf(session), own: null, gone: [], asked: null, readers: new Set() };
  page.offered = () => (page.asked ??= window.__TAURI__.core.invoke("models_offered", { name: page.name }).then((offered) => {
    page.own = { transcription: offered.chosen.transcription ?? null, minutes: offered.chosen.minutes ?? null };
    page.gone = offered.gone;
    return offered;
  }));
  page.again = () => {
    page.asked = null;
    page.readers.forEach((read) => read());
  };
  pageOffered = page;
  return page;
}

// A step's choice as one value, whatever order its fields came in.
function modelValue(step, chosen) {
  if (step === "minutes") return JSON.stringify({ provider: chosen.provider, model: chosen.model });
  if (chosen.engine === "service") return JSON.stringify({ engine: "service", provider: chosen.provider, model: chosen.model });
  return JSON.stringify(chosen.engine === "whisper" ? { engine: "whisper", model: chosen.model } : { engine: chosen.engine });
}

// A version's line, its model's name a list in place (D93): the sentence around it as written.
const IN_LINE = "\u{E000}";
function lineWith(said, pick, kind = "maker") {
  const line = element("p", kind);
  const [before, after] = said.split(IN_LINE);
  line.append(before);
  if (after === undefined) return line;
  line.append(pick);
  // A list drawn as the language's setting's (#1094): its value, Lucide's chevron after it.
  if (pick.tagName === "SELECT") {
    const chevron = icon("chevron-down");
    chevron.classList.add("chevron");
    line.append(chevron);
  }
  line.append(after);
  return line;
}

// D93, criterion 5: a step's model, a list on the page itself — the one that made the version shown
// (`version`: what its line says of it, and which offered model it is), or for a step waiting, the
// one it will be made by. The same models as « Avancé » but Settings' entry (D91): a confidential
// meeting is offered none of a service. A choice starts nothing: kept as the meeting's own; `go`
// starts the making, naming it — for a version, shown (`box`) only while another model is chosen.
function modelOnPage(page, words, step, version, go, doing, fallback, box = go) {
  let pick = element("select", "page-model");
  pick.id = `page-${step}-model`;
  pick.setAttribute("aria-label", step === "minutes" ? words.models_minutes : words.models_transcription);
  // Until the models are read: the version's own alone.
  const first = element("option", "", version ? version.said : words.models_none);
  first.value = "";
  pick.append(first);
  const said = element("p", "note warn");
  said.setAttribute("role", "status");
  said.hidden = true;
  let made = "";
  let kept = "";
  const saidOf = (offered, value) => {
    const chosen = JSON.parse(value);
    if (step === "minutes") return modelSaid(chosen.model) + (chosen.provider === offered.ollama_address ? "" : fill(words.written_via, via(chosen.provider)));
    if (chosen.engine === "service") return `${chosen.model}${fill(words.written_via, via(chosen.provider))}`;
    return chosen.engine === "whisper" ? spoken(chosen.model) : "SpeechAnalyzer";
  };
  const name = () => {
    const chosen = pick.value ? JSON.parse(pick.value) : null;
    const model = chosen && (chosen.engine === "whisper" ? spoken(chosen.model) : chosen.engine === "speechanalyzer" ? "SpeechAnalyzer" : modelSaid(chosen.model));
    go.textContent = model ? fill(doing, model) : fallback;
    if (version) box.hidden = pick.value === made;
    pick.title = pick.options[pick.selectedIndex]?.textContent ?? "";
    fitToChoice(pick);
  };
  if (version) box.hidden = true;
  const listed = (offered) => {
    pick.replaceChildren(first);
    const values = [];
    if (step === "minutes") {
      values.push(...offered.ollama.map((model) => ({ provider: offered.ollama_address, model })));
      if (offered.minutes_provider && offered.minutes_model) values.push({ provider: offered.minutes_provider, model: offered.minutes_model });
    } else {
      values.push(...offered.whisper.map((model) => ({ engine: "whisper", model })));
      if (offered.apple) values.push({ engine: "speechanalyzer" });
      if (offered.service) values.push({ engine: "service", provider: offered.service[0], model: offered.service[1] });
    }
    // The meeting's own, offered or not — unless gone, Settings' then standing in, said by the page (D91).
    const own = page.own[step] && !page.gone.includes(step) ? modelValue(step, page.own[step]) : "";
    const options = [...new Set([...values.map((value) => modelValue(step, value)), ...(own ? [own] : [])])];
    made = version ? (options.find((value) => version.is(JSON.parse(value), offered)) ?? "") : "";
    const settings = !version && offered.minutes_provider && offered.minutes_model ? modelValue("minutes", { provider: offered.minutes_provider, model: offered.minutes_model }) : "";
    kept = own || made || settings;
    // The version's model no longer offered stays first, as its line says it: never silently another.
    const placeholder = version ? !made : !kept;
    if (!placeholder) first.remove();
    for (const value of options) {
      const option = element("option", "", saidOf(offered, value));
      option.value = value;
      pick.append(option);
    }
    pick.value = kept;
    name();
  };
  const read = () => page.offered().then(listed, (why) => {
    said.textContent = fill(words.models_not_read, why);
    said.hidden = false;
  });
  // Read again when `again` asks; a line no longer on the page, dropped instead.
  const reader = () => (pick.isConnected ? read() : page.readers.delete(reader));
  page.readers.add(reader);
  read();
  const changed = async () => {
    if (!page.own) return;
    const chosen = { ...page.own, [step]: pick.value ? JSON.parse(pick.value) : null };
    try {
      await window.__TAURI__.core.invoke("meeting_models_chosen", { name: page.name, chosen });
      page.own = chosen;
      page.gone = page.gone.filter((gone) => gone !== step);
      kept = pick.value;
      said.hidden = true;
    } catch (why) {
      // Put back as #926 puts the others: a fresh list, so the very model refused can be chosen again (#843).
      pick = putBack(pick, kept);
      pick.addEventListener("change", changed);
      said.textContent = fill(words.models_not_kept, why);
      said.hidden = false;
    }
    name();
  };
  pick.addEventListener("change", changed);
  name();
  return [pick, said];
}

// A list in a sentence as wide as the name it shows, never as its longest option (#1231): a native list takes its
// longest option's width, « m » then followed by a gap as wide as « Whisper large v3 turbo ». Narrower than its
// line, the CSS's max-width cuts it with an ellipsis. Measured once on the page, and again once its fonts are in.
const fitting = document.createElement("canvas").getContext("2d");
function fitToChoice(pick, frames = 10) {
  if (!pick.isConnected) {
    if (frames > 0) requestAnimationFrame(() => fitToChoice(pick, frames - 1));
    return;
  }
  const style = getComputedStyle(pick);
  fitting.font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
  const text = pick.options[pick.selectedIndex]?.textContent ?? "";
  const room = ["paddingLeft", "paddingRight", "borderLeftWidth", "borderRightWidth"].reduce((sum, side) => sum + (parseFloat(style[side]) || 0), 0);
  pick.style.width = `${Math.ceil(fitting.measureText(text).width + room)}px`;
  if (document.fonts.status !== "loaded") document.fonts.ready.then(() => fitToChoice(pick, 0));
}

// The one button beside a version's list that starts its making again with the model chosen (D93).
function startsOnPage(id, words, command) {
  const go = element("button", "primary");
  go.type = "button";
  go.id = id;
  go.addEventListener("click", async () => {
    waiting(go, true);
    try {
      await command();
      // Asked: its state is said where it is made (D25).
      (go.closest(".page-redo") ?? go).hidden = true;
      readMakings();
    } catch (why) {
      go.replaceWith(element("p", "note warn", fill(words.not_saved, why)));
    }
    waiting(go, false);
  });
  return go;
}

// The last version a redo replaced (D113): brought back by one button that names its time and model;
// the one left becomes the previous, so the same button brings it back.
function previousBack(words, session, step, previous, model) {
  const back = element("button", "open");
  back.type = "button";
  back.id = `${step}-previous`;
  const was = [madeOn(words, previous.made), model].filter(Boolean).join(" · ");
  back.textContent = was ? fill(words.previous_back, was) : words.previous_back.replace(/\s*\(\{\}\)$/, "");
  const help = element("p", "says", words.previous_help);
  help.id = `${step}-previous-help`;
  const failed = element("p", "note warn");
  failed.hidden = true;
  back.addEventListener("click", async () => {
    waiting(back, true);
    try {
      await window.__TAURI__.core.invoke("previous_back", { name: nameOf(session), step });
      await refresh();
    } catch (why) {
      failed.textContent = fill(words.previous_failed, why);
      failed.hidden = false;
      waiting(back, false);
    }
  });
  const box = element("div", "previous");
  box.append(back, help, failed);
  return box;
}

// A meeting's summary sent to a destination the user set up (D110, PDR-0007), on their gesture each time:
// « Envoyer… » among the summary's actions only when it may be — a summary, a destination, not confidential —;
// its panel, each destination by its name; before a destination's first send, what leaves and where, its
// name and never its address; the outcome said in place, with what to do; each send listed under it. A send
// that failed is said again from the record whenever the page is drawn — « perhaps arrived » most of all, read
// on coming back from the channel —, as long as that destination is not sent to again. A confidential meeting's
// send is asked once more every time, in one sentence, the focus on « Annuler » (D119, PDR-0007 §4 amended).
function sendPart(card, top, actions, session, words) {
  const name = nameOf(session);
  const said = element("p", "says send-said");
  said.id = "send-said";
  said.setAttribute("role", "status");
  const failed = element("div", "says send-last");
  failed.id = "send-last";
  const sends = element("ul", "sends");
  sends.id = "sends";
  top.after(said, failed, sends);
  const when = (at) => new Date(at).toLocaleString(document.documentElement.lang || undefined, { dateStyle: "medium", timeStyle: "short" });
  const listSends = (shown) => sends.replaceChildren(...shown.sends.map((sent) => element("li", "", fill(words.send_listed, sent.destination, when(sent.at)))));
  const outcome = (result, destination) => {
    if (result.outcome === "sent") {
      const at = new Date().toLocaleTimeString(document.documentElement.lang || undefined, { hour: "2-digit", minute: "2-digit" });
      return fill(words.send_done, destination, at);
    }
    const by = {
      "address.refused": fill(words.send_address_refused, destination),
      "meeting.unreadable": words.send_unreadable,
      "meeting.confidential": words.send_confidential,
      "meeting.no_summary": words.send_no_summary,
      "summary.too_large": fill(words.send_too_large, destination),
      "network.unreachable": words.send_offline,
      "destination.silent": fill(words.send_silent, destination),
      "destination.gone": fill(words.send_gone, destination),
      "destination.busy": result.retry_after ? fill(words.send_busy, destination, result.retry_after) : fill(words.send_busy_later, destination),
      "destination.refused": fill(words.send_refused, destination),
      "destination.failed": fill(words.send_failed, destination),
    };
    return by[result.code] ?? words.send_not_sent;
  };
  // Each destination's last send for this meeting, when it failed; the one just said in place, not twice.
  const sayFailed = (shown, but) =>
    failed.replaceChildren(
      ...shown.last.filter((sent) => sent.outcome !== "sent" && sent.destination !== but).map((sent) => element("p", "", outcome(sent, sent.destination))),
    );
  (async () => {
    const shown = await window.__TAURI__.core.invoke("sending_of", { name }).catch(() => null);
    if (!shown) return;
    listSends(shown);
    sayFailed(shown);
    if (!shown.offered) return;
    let offered = shown.destinations;
    let secret = shown.confidential;
    // One destination: named where « Envoyer… » is read, its tooltip and name (#1239); several, the panel names them.
    const sendName = () => (offered.length === 1 ? `${fill(words.send_to, offered[0].name)}…` : words.send_open);
    const open = iconButton("send", sendName(), "send-open");
    open.setAttribute("aria-expanded", "false");
    open.setAttribute("aria-controls", "send-panel");
    actions.prepend(open);
    const close = () => {
      document.getElementById("send-panel")?.remove();
      open.setAttribute("aria-expanded", "false");
      open.focus();
    };
    const send = async (destination, confirmed = false) => {
      document.getElementById("send-panel")?.remove();
      open.setAttribute("aria-expanded", "false");
      open.disabled = true;
      said.textContent = fill(words.send_sending, destination);
      try {
        const result = await window.__TAURI__.core.invoke("summary_sent", { name, destination, confirmed });
        said.textContent = outcome(result, destination);
        offered = result.sending.destinations;
        open.title = sendName();
        open.setAttribute("aria-label", open.title);
        secret = result.sending.confidential;
        if (confidentialShown?.name === name) confidentialShown.paint(result.confidential);
        listSends(result.sending);
        sayFailed(result.sending, destination);
        if (!result.sending.offered) open.remove();
      } catch {
        said.textContent = words.send_not_sent;
      } finally {
        open.disabled = false;
        if (open.isConnected) open.focus();
      }
    };
    open.addEventListener("click", () => {
      if (document.getElementById("send-panel")) return close();
      const panel = element("div", "send-panel");
      panel.id = "send-panel";
      panel.setAttribute("role", "group");
      panel.setAttribute("aria-label", words.send_open);
      for (const destination of offered) {
        const to = element("button", "send-to", fill(words.send_to, destination.name));
        to.prepend(kindMark(destination.kind));
        to.type = "button";
        to.dataset.destination = destination.name;
        to.addEventListener("click", () => {
          // Said confidential since the page was drawn: its switch is the one read now.
          secret = document.getElementById("confidential-switch")?.checked ?? secret;
          if (!destination.first && !secret) return send(destination.name);
          const firstSaid = [];
          // A confidential meeting: asked once more, every time, never remembered (D119).
          if (secret) {
            const ask = element("p", "note send-confidential", fill(words.send_confidential_ask, destination.name));
            ask.id = "send-confidential";
            firstSaid.push(ask);
          }
          // Before its first send: what leaves, and where, by its name (PDR-0007 §3).
          if (destination.first) {
            const notice = element("p", "note send-notice", fill(words.send_notice, destination.name));
            notice.id = "send-notice";
            firstSaid.push(notice);
          }
          const confirm = element("button", "primary", words.send_confirm);
          confirm.type = "button";
          confirm.id = "send-confirm";
          // Its name read with what is asked, what leaves, where, and that it cannot be taken back.
          confirm.setAttribute("aria-describedby", firstSaid.map((line) => line.id).join(" "));
          const confirmed = secret;
          confirm.addEventListener("click", () => send(destination.name, confirmed));
          const cancel = element("button", "", words.send_cancel);
          cancel.type = "button";
          cancel.id = "send-cancel";
          cancel.addEventListener("click", close);
          const choices = element("div", "send-choices");
          choices.append(confirm, cancel);
          panel.replaceChildren(...firstSaid, choices);
          // A confidential meeting's question: the focus on the choice that sends nothing.
          (secret ? cancel : confirm).focus();
        });
        panel.append(to);
      }
      panel.addEventListener("keydown", (event) => {
        if (event.key === "Escape") {
          event.stopPropagation();
          close();
        }
      });
      said.before(panel);
      open.setAttribute("aria-expanded", "true");
      panel.querySelector("button")?.focus();
    });
  })();
}

function minutesCard(read, words, session, transcript, onPage, previous, otherVersion = false) {
  const card = element("article", "card minutes");
  // Its title, and Refaire… beside it: the state below them never sits under a button.
  const top = element("div", "top");
  top.append(element("h2", "label", words.minutes_title));
  card.append(top);
  // What is being made for them, or why they were not (D25).
  const state = element("p", "making-state");
  state.id = "minutes-state";
  state.dataset.name = nameOf(session);
  state.dataset.kept = String(read.state === "present");
  state.dataset.made = String(makings?.made[nameOf(session)] ?? 0);
  state.setAttribute("role", "status");
  state.hidden = true;
  card.append(state);
  // Whether its summary went to the export folder too (D109): said when it did not, painted below.
  const exported = element("p", "note warn");
  exported.id = "export-said";
  exported.dataset.name = nameOf(session);
  exported.dataset.confidential = String(!!session.confidential);
  exported.setAttribute("role", "status");
  exported.hidden = true;
  card.append(exported);
  if (read.state !== "present") {
    card.append(element("p", "none", read.state === "absent" ? words.no_minutes : words.unreadable));
    if (read.state === "absent" && transcript.state === "present" && transcript.complete) {
      const ask = offer(card, words.make_minutes, "make-minutes", "minutes_asked", session, words);
      // The model it will be made by, a choice beside it, which the button names (D93).
      const [pick, said] = modelOnPage(onPage, words, "minutes", null, ask, words.make_minutes_with, words.make_minutes);
      ask.before(lineWith(IN_LINE, pick, "make-with"), said);
    }
    return card;
  }
  // Refaire… (D11's sixth board): another model, this time only; the minutes stay until the new
  // ones are whole.
  const redo = iconButton("rotate-ccw", words.redo, "redo-minutes");
  redo.addEventListener("click", async () => {
    const offered = await window.__TAURI__.core.invoke("models_offered", { name: nameOf(session) }).catch((why) => {
      redo.after(element("p", "note warn", fill(words.models_not_read, why)));
      return null;
    });
    if (offered === null) return;
    const panel = redoPanel(offered, session, words, redo);
    redo.replaceWith(panel);
    panel.querySelector("button:enabled").focus();
  });
  // Copied in one click, as written, to paste where the user writes (D45): said in place, a moment. As
  // rich text and as Markdown together, so that a document or a chat keeps its headings and lists, and
  // a plain field takes the Markdown (D109). Without ClipboardItem, the Markdown alone, as before.
  const copy = iconButton("copy", words.copy_minutes, "copy-minutes");
  const copied = element("span", "says copied");
  copied.id = "minutes-copied";
  copied.setAttribute("role", "status");
  let said = 0;
  copy.addEventListener("click", async () => {
    const mine = ++said;
    try {
      if (typeof ClipboardItem === "function" && navigator.clipboard.write) {
        await navigator.clipboard.write([new ClipboardItem({
          "text/html": new Blob([read.body], { type: "text/html" }),
          "text/plain": new Blob([read.markdown], { type: "text/plain" }),
        })]);
      } else {
        await navigator.clipboard.writeText(read.markdown);
      }
      copied.textContent = words.minutes_copied;
      setTimeout(() => mine === said && (copied.textContent = ""), 2000);
    } catch {
      copied.textContent = words.minutes_not_copied;
    }
  });
  const actions = element("span", "minutes-actions");
  actions.append(copied, copy, redo);
  top.append(actions);
  sendPart(card, top, actions, session, words);
  // When they were made first, then who wrote them, the model as people say it (D93); what is not
  // known is left unsaid.
  const when = madeOn(words, read.made);
  // Written on this computer (`local`, minutes/v2): said so, never the contract's word (#1080).
  const from = read.provider === "local" ? `, ${words.on_computer}` : read.provider ? fill(words.written_via, read.provider) : "";
  const by = read.model ? modelSaid(read.model) + from : "";
  // Its model a list in its place (D93, criterion 5); none named, the line alone.
  // The model a list: written on this computer said after it, as without a list (#1090); a service's list says its host.
  const placed = read.provider === "local" ? from : "";
  const maker = (when
    ? fill(words.minutes_of, when) + (by ? ` · ${IN_LINE}${placed}` : "")
    : read.model ? fill(words.written_by, IN_LINE) + placed : fill(words.written_by, "—") + from) + tookAndCost(read, words);
  if (by) {
    const go = startsOnPage("page-minutes-do", words, () =>
      window.__TAURI__.core.invoke("minutes_redo", { name: nameOf(session), provider: null, model: null }));
    // Another model chosen: above its button, which made the summary shown — the line names the choice (D93).
    const box = element("div", "page-redo");
    const was = element("p", "says", fill(words.minutes_made_with, by));
    was.id = "page-minutes-was";
    box.append(was, go);
    // The version's model: the one its line names, made here unless a host says otherwise.
    // Its list names the model alone when the line says « sur cet ordinateur » after it (#1231).
    const version = { said: placed ? modelSaid(read.model) : by, is: (chosen, offered) => chosen.model === read.model
      && (read.provider && read.provider !== "local" ? via(chosen.provider) === read.provider : chosen.provider === offered.ollama_address) };
    const [pick, said] = modelOnPage(onPage, words, "minutes", version, go, words.redo_minutes_do, words.redo_title, box);
    card.append(lineWith(maker, pick), said, box);
  } else card.append(element("p", "maker", maker));
  // A language model wrote them, said where they are read (Store Policies 11.16, #1092).
  const written = element("p", "note", words.minutes_ai);
  written.id = "minutes-ai";
  card.append(written);
  // Made on the other version of the transcript, not the one shown (#1070): said, never passed off as its own.
  if (otherVersion) {
    const other = element("p", "note", words.minutes_other_version);
    other.id = "minutes-other-version";
    card.append(other);
  }
  if (previous) card.append(previousBack(words, session, "minutes", previous, previous.model ? modelSaid(previous.model) : ""));
  // The transcript corrected after they were written (D62): said, and made again only on a click.
  const corrected = transcript.state === "present" ? Math.max(0, ...transcript.segments.map((segment) => Date.parse(segment.edited ?? "") || 0)) : 0;
  if (corrected > (Date.parse(read.made ?? "") || 0)) {
    const since = element("div", "corrected-since");
    since.id = "corrected-since";
    const again = element("button", "open", words.redo_corrections);
    again.type = "button";
    again.id = "redo-corrections";
    again.addEventListener("click", async () => {
      waiting(again, true);
      await window.__TAURI__.core.invoke("minutes_redo", { name: nameOf(session), provider: null, model: null }).catch((why) => {
        since.append(element("p", "note warn", fill(words.not_saved, why)));
      });
      readMakings();
    });
    since.append(element("p", "note", words.corrected_since), again);
    card.append(since);
  }
  // The minutes as written, rendered by the shell: raw HTML in them is text there, a link its words.
  const body = element("div", "body");
  body.innerHTML = read.body;
  // A section made with nothing under it says so (#1234): a title alone reads as a summary cut short.
  for (const title of body.querySelectorAll("h2"))
    if (!title.nextElementSibling || title.nextElementSibling.tagName === "H2") title.after(element("p", "empty-section", words.section_empty));
  const mentioned = actionRows(body, words, read.actions);
  lineTimes(body, session, transcript, words);
  card.append(body);
  datesOffer(card, mentioned, session, transcript, words);
  return card;
}

// « Dates mentionnées » (D115): the dates the actions' by when name, read by the shell against the meeting's day,
// offered to the user's calendar. Offered only when one names a date. Nothing is ticked, and nothing leaves this
// computer before an item is ticked and a destination chosen; a confidential meeting is offered no link (PDR-0007).
// Granola, Otter and Fireflies list actions and leave their dates as words; Gmail and Apple Mail detect a date
// in a message and offer to make an event of it, one at a time: here the same, the date said in full, its passage
// beside it, the user choosing.
function datesOffer(card, mentioned, session, transcript, words) {
  if (mentioned.length === 0) return;
  // Among the summary's actions, as « Envoyer… » and « Copier » are (#1213, D89): a calendar, its tooltip and name the
  // words, before « Copier »; never a bar of its own under the summary.
  const button = iconButton("calendar", words.dates_open, "dates-open");
  button.hidden = true;
  const actions = card.querySelector(".minutes-actions");
  const copy = actions?.querySelector("#copy-minutes");
  if (copy) copy.before(button);
  else (actions ?? card).append(button);
  window.__TAURI__.core
    .invoke("dates_resolved", { name: nameOf(session), whens: mentioned.map((m) => m.when), day: null })
    .then((resolved) => {
      // Its count beside the calendar (#1239), as a mail app counts what waits: how many dates, at a glance; its name
      // and tooltip say both — « Dates mentionnées (2) ».
      const count = resolved?.dates.filter(Boolean).length ?? 0;
      button.hidden = count === 0;
      if (count === 0) return;
      const said = `${words.dates_open} (${count})`;
      button.title = said;
      button.setAttribute("aria-label", said);
      button.classList.add("counted");
      button.append(element("span", "count", String(count)));
    })
    .catch(() => {});
  button.addEventListener("click", () => datesPanel(mentioned, session, transcript, words, button));
}

// What was said where an action was set: the segments from its second, two at most.
function passageAt(transcript, second) {
  if (second === null || transcript.state !== "present") return null;
  const said = transcript.segments.filter((s) => s.end > second).slice(0, 2).map((s) => s.text.trim());
  return said.length ? said.join("\n") : null;
}

function datesPanel(mentioned, session, transcript, words, back) {
  const name = nameOf(session);
  const dialog = element("dialog", "check-sound dates");
  dialog.id = "dates";
  dialog.setAttribute("aria-labelledby", "dates-title");
  const head = element("div", "check-head");
  const title = element("h2", "", words.dates_open);
  title.id = "dates-title";
  const close = iconButton("x", words.check_close, "dates-close");
  head.append(title, close);
  const day = element("input");
  day.type = "date";
  day.id = "dates-day";
  const dayLabel = element("label", "dates-day", words.dates_day);
  dayLabel.htmlFor = "dates-day";
  const list = element("ul", "dates-list");
  list.id = "dates-list";
  const google = element("button", "primary", words.dates_google);
  google.type = "button";
  google.id = "dates-google";
  google.disabled = true;
  // Read when it opens: the switch may have changed since the page was drawn; the shell refuses it anyway.
  google.hidden = document.getElementById("confidential-switch")?.checked === true;
  const ics = element("button", "open", words.dates_ics);
  ics.type = "button";
  ics.id = "dates-ics";
  ics.disabled = true;
  const saved = element("p", "note");
  saved.id = "dates-saved";
  saved.setAttribute("role", "status");
  const said = element("p", "note warn");
  said.id = "dates-said";
  said.setAttribute("role", "status");
  said.hidden = true;
  const goes = element("div", "dates-goes");
  goes.append(ics, google);
  dayLabel.append(day);
  dialog.append(head, element("p", "note", words.dates_says), dayLabel, list, goes, saved, said);
  const full = new Intl.DateTimeFormat(document.documentElement.lang, { weekday: "long", day: "numeric", month: "long", year: "numeric", timeZone: "UTC" });
  let shown = [];
  const ticked = () => shown.filter((item) => item.box.checked);
  const draw = async (asked) => {
    const resolved = await window.__TAURI__.core.invoke("dates_resolved", { name, whens: mentioned.map((m) => m.when), day: asked }).catch(() => null);
    if (!resolved) return;
    day.value = resolved.day;
    shown = [];
    list.replaceChildren();
    for (const [index, date] of resolved.dates.entries()) {
      if (!date) continue;
      const action = mentioned[index];
      const item = element("li", "dates-item");
      const box = element("input");
      box.type = "checkbox";
      box.addEventListener("change", () => (google.disabled = ics.disabled = ticked().length === 0));
      const words_ = element("label", "dates-what");
      const when = element("span", "dates-when", full.format(new Date(`${date}T00:00:00Z`)));
      words_.append(box, when, element("span", "", action.who ? `${action.what} — ${action.who}` : action.what));
      const passage = passageAt(transcript, action.second);
      item.append(words_);
      if (passage) item.append(element("blockquote", "dates-passage", passage));
      list.append(item);
      shown.push({ box, date, action, passage });
    }
    google.disabled = ics.disabled = true;
  };
  day.addEventListener("change", () => day.value && draw(day.value));
  ics.addEventListener("click", async () => {
    said.hidden = true;
    saved.textContent = "";
    const to = await window.__TAURI__.core.invoke("dates_file_chosen", { name }).catch(() => null);
    if (!to) return;
    const dates = ticked().map(({ date, action, passage }) => ({ id: `a${action.place}`, date, what: action.what, who: action.who, passage }));
    try {
      await window.__TAURI__.core.invoke("dates_ics", { name, to, dates });
      saved.textContent = fill(words.dates_saved, to.split(/[\\/]/).pop());
    } catch (why) {
      said.textContent = fill(words.not_saved, why);
      said.hidden = false;
    }
  });
  google.addEventListener("click", async () => {
    said.hidden = true;
    const meeting = document.querySelector("#meeting h1")?.textContent ?? name;
    for (const { date, action, passage } of ticked()) {
      const item = { date, what: action.what, who: action.who, passage, meeting };
      try {
        await window.__TAURI__.core.invoke("date_google", { name, item });
      } catch (why) {
        said.textContent = why;
        said.hidden = false;
        return;
      }
    }
  });
  const end = () => {
    dialog.close();
    dialog.remove();
    back.focus();
  };
  close.addEventListener("click", end);
  dialog.addEventListener("cancel", (event) => {
    event.preventDefault();
    end();
  });
  document.body.append(dialog);
  dialog.showModal();
  draw(null);
}

// A line's time (D75), the shell's own mark: a small link that opens its passage and plays from that
// second. Nothing to play, it opens the passage and says that alone; nothing to open either, it
// stays quiet text: the time as said, no link that leads nowhere. Once, on the gesture: the page
// does not follow what plays (D26).
function lineTimes(body, session, transcript, words) {
  const plays = player?.name === nameOf(session);
  const passages = transcript.state === "present" && transcript.segments.length > 0;
  for (const time of body.querySelectorAll("time")) {
    const second = Number(time.dataset.second);
    // As the transcript and the player write it, past the hour too.
    time.textContent = clock(second);
    if (!plays && !passages) continue;
    const link = element("button", "line-time", clock(second));
    link.type = "button";
    link.setAttribute("aria-label", fill(plays ? words.time_listen : words.time_passage, clock(second)));
    link.title = link.getAttribute("aria-label");
    link.addEventListener("click", () => {
      const lines = document.querySelector("#meeting .lines");
      // A segment holds its second from its start cut to whole, as the contract counts it.
      const all = lines ? [...lines.children] : [];
      const line = all.findLast((one) => Math.floor(Number(one.dataset.start)) <= second) ?? all[0];
      if (line?.hidden) {
        const search = document.getElementById("search");
        search.value = "";
        search.dispatchEvent(new Event("input"));
      }
      roam(lines, line);
      // In view already, the page stays as it is; else its passage comes with what is around it.
      const box = line?.getBoundingClientRect();
      if (box && (box.top < 0 || box.bottom > innerHeight)) line.scrollIntoView({ block: "center" });
      if (player?.name === nameOf(session)) {
        document.getElementById("not-played").textContent = "";
        player.seek(second);
        player.play();
      } else {
        // No playback to mark it: marked here, the line asked for alone.
        for (const one of all) one.classList.toggle("now", one === line);
      }
    });
    time.replaceWith(link);
  }
}

// The minutes' actions as rows (D46): « what — who — by when », as `minutes/v1` writes them, shown
// Qui · Quoi · Pour quand, who in its colour, by when in monospace, the cells named for assistive
// technology alone; an item that does not split in three is shown as written, in its place. A
// line's time is none of the three (`minutes/v3`). `at` is the actions' section, counted from one, as
// the shell read it: v1 to v4's second, v5's named by its id (D111); none, no rows.
function actionRows(body, words, at) {
  // Every second-level title, as the shell's reader counts them.
  const titles = body.querySelectorAll("h2");
  // Each row split in three whose by when is said, for « Dates mentionnées » (D115).
  const mentioned = [];
  if (!at || titles.length < at) return mentioned;
  const next = titles[at] ?? null;
  let list = titles[at - 1].nextElementSibling;
  while (list && list !== next && !["UL", "OL"].includes(list.tagName)) list = list.nextElementSibling;
  if (!list || list === next) return mentioned;
  const table = element("table", "actions");
  const head = element("thead");
  const named = element("tr");
  for (const cell of words.actions_cells) named.append(element("th", "", cell));
  head.append(named);
  const rows = element("tbody");
  const unsettled = (text) => [words.unsettled, "non précisé", "not specified"].includes(text.toLowerCase());
  for (const item of list.children) {
    const time = item.querySelector("time");
    const held = time?.parentElement;
    time?.remove();
    const parts = item.querySelector("ul, ol") ? [] : item.textContent.trim().split(" — ").map((part) => part.trim());
    const row = element("tr");
    if (parts.length === 3 && parts.every((part) => part !== "")) {
      const [what, who, when] = parts;
      // `place`, the action's row in the minutes from one: the `.ics`'s UID keeps it (D115).
      if (!unsettled(when)) mentioned.push({ what, who: unsettled(who) ? null : who, when, second: time ? Number(time.dataset.second) : null, place: rows.children.length + 1 });
      for (const [text, kind] of [[who, "who"], [what, "what"], [when, "when"]]) {
        // Short, on one line; long, back to the line between words, never pushing the card wider.
        const long = kind !== "what" && text.length > 18 ? " long" : "";
        const cell = element("td", (unsettled(text) ? `${kind} unsettled` : kind) + long, text);
        if (kind === "what" && time) cell.append(" ", time);
        row.append(cell);
      }
    } else {
      const cell = element("td", "written");
      cell.colSpan = 3;
      if (time) held.append(time);
      cell.append(...item.childNodes);
      row.append(cell);
    }
    rows.append(row);
  }
  table.append(head, rows);
  list.replaceWith(table);
  return mentioned;
}

const folded = (text) => text.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();

// The word sought marked where it is found, accents and case aside as the search reads (#1078): `<mark>`, the
// HTML element for a run of text highlighted for its relevance to what the user is doing. Marks before are
// taken away first; nothing sought, none.
function marking(root, wanted) {
  for (const old of root.querySelectorAll("mark")) old.replaceWith(old.textContent);
  root.normalize();
  if (!wanted) return;
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
    acceptNode: (node) => (node.parentElement.closest("svg, select, button") ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT),
  });
  const texts = [];
  while (walker.nextNode()) texts.push(walker.currentNode);
  for (const node of texts) {
    // Each folded character points back to where its own character starts in the text.
    let flat = "";
    const at = [];
    for (let i = 0; i < node.data.length; ) {
      const char = String.fromCodePoint(node.data.codePointAt(i));
      for (const one of folded(char)) {
        flat += one;
        at.push(i);
      }
      i += char.length;
    }
    at.push(node.data.length);
    const found = [];
    for (let from = flat.indexOf(wanted); from !== -1; from = flat.indexOf(wanted, from + wanted.length)) found.push([at[from], at[from + wanted.length]]);
    let rest = node;
    for (const [start, end] of found.reverse()) {
      const tail = rest.splitText(start);
      tail.splitText(end - start);
      const mark = document.createElement("mark");
      tail.replaceWith(mark);
      mark.append(tail);
    }
  }
}

// The one line of a transcript in the order of Tab: this one, the others reached by the arrows.
function roam(lines, line) {
  if (!line || line.tabIndex === -1 && !line.hasAttribute("role")) return;
  for (const other of lines.children) {
    if (!other.hasAttribute("role")) continue;
    other.tabIndex = other === line ? 0 : -1;
    // Its pencil follows it in the order of Tab: one line, one pencil reached by Tab (D62).
    const pencil = other.querySelector(".edit");
    if (pencil) pencil.tabIndex = other.tabIndex;
  }
}

// When a version was made, in the system's language and local time: the day without its year, then
// the hour — "13 nov. à 09:42", "Nov 13 at 9:42 AM" (D93, #1075). An instant not read, nothing.
function madeOn(words, made) {
  const at = made && dayOf({ instant: Date.parse(made) / 1000 || null });
  if (!at) return "";
  const day = new Intl.DateTimeFormat(words.locale, { day: "numeric", month: "short" }).format(at);
  return `${day} ${words.at} ${new Intl.DateTimeFormat(words.locale, hours(words)).format(at)}`;
}

// What a version's making took, after its model (D97): how long, then what the service said it cost
// (D51) — each only when its file says it; the shell gives a cost and its currency both or neither.
// A time in whole seconds, at least one, then minutes and seconds, then hours and minutes:
// "en 3 s", "en 2 min 05 s", "en 1 h 02 min".
function tookAndCost(read, words) {
  let said = "";
  if (read.made_in !== null && read.made_in !== undefined) {
    const s = Math.max(1, Math.round(read.made_in));
    const two = (n) => String(n).padStart(2, "0");
    const took = s < 60 ? `${s} s` : s < 3600 ? `${Math.floor(s / 60)} min ${two(s % 60)} s` : `${Math.floor(s / 3600)} h ${two(Math.floor(s / 60) % 60)} min`;
    said += ` · ${fill(words.made_in, took)}`;
  }
  if (read.cost !== null && read.cost !== undefined) said += ` · ${new Intl.NumberFormat(words.locale, { style: "currency", currency: read.currency, maximumSignificantDigits: 2 }).format(read.cost)}`;
  return said;
}

// Who said a line, as the page names them: the track — by the name the user gave its side (#1205), else the
// page's word —, and the voice within it when one was told.
function whoSaid(segment, words, names) {
  const side = segment.track === "local" ? names?.me ?? words.here : names?.others ?? words.remote;
  return segment.voice === null ? side : `${side} · ${fill(words.voice, segment.voice)}`;
}

// The button that opens the speakers' names, and its panel under the card's top (#1205): Enter saves, Escape or
// « Annuler » closes it as it was; a name refused or not kept is said under the fields, what was typed kept.
function speakersButton(card, names, read, words, session) {
  const open = iconButton("user-pen", words.speakers_open, "speakers-open");
  open.setAttribute("aria-expanded", "false");
  open.setAttribute("aria-controls", "speakers-panel");
  const close = () => {
    document.getElementById("speakers-panel")?.remove();
    open.setAttribute("aria-expanded", "false");
    open.focus();
  };
  open.addEventListener("click", () => {
    if (document.getElementById("speakers-panel")) return close();
    const panel = element("form", "fields speakers-panel");
    panel.id = "speakers-panel";
    panel.setAttribute("aria-label", words.speakers_open);
    // Settings' own fields (guidedField): labelled by their side, the page's word as the placeholder.
    const field = (id, label, value, placeholder) => {
      const [row, input] = guidedField(id, label, "text", value, false);
      input.maxLength = 60;
      input.placeholder = placeholder;
      input.autocomplete = "off";
      return [row, input];
    };
    const [meRow, me] = field("speaker-me", words.here_described, names.me, words.here);
    const [othersRow, others] = field("speaker-others", words.remote_described, names.others, words.remote);
    const hint = element("p", "note", words.speakers_hint);
    hint.id = "speakers-hint";
    me.setAttribute("aria-describedby", "speakers-hint");
    others.setAttribute("aria-describedby", "speakers-hint");
    const said = element("p", "note warn", "");
    said.id = "speakers-said";
    said.setAttribute("role", "alert");
    said.hidden = true;
    const save = element("button", "primary", words.speakers_save);
    save.type = "submit";
    save.id = "speakers-save";
    const cancel = element("button", "", words.speakers_cancel);
    cancel.type = "button";
    cancel.id = "speakers-cancel";
    cancel.addEventListener("click", close);
    const choices = element("div", "speakers-choices");
    choices.append(save, cancel);
    panel.append(meRow, othersRow, hint, said, choices);
    panel.addEventListener("keydown", (event) => {
      if (event.key !== "Escape") return;
      event.stopPropagation();
      close();
    });
    panel.addEventListener("submit", async (event) => {
      event.preventDefault();
      save.disabled = true;
      try {
        const kept = await window.__TAURI__.core.invoke("speakers_named", { name: nameOf(session), me: me.value, others: others.value });
        names.me = kept.me;
        names.others = kept.others;
        // Every line, and what a copy takes, names them now.
        const lines = card.querySelectorAll(".lines li");
        read.segments.forEach((segment, index) => {
          const who = lines[index]?.querySelector(".who");
          if (who) who.textContent = whoSaid(segment, words, names);
        });
        close();
      } catch {
        said.textContent = words.speakers_not_saved;
        said.hidden = false;
      } finally {
        save.disabled = false;
      }
    });
    card.querySelector(".top").after(panel);
    open.setAttribute("aria-expanded", "true");
    me.focus();
  });
  return open;
}

function transcriptCard(read, words, session, onPage, previous, speakers) {
  // The sides' names as kept (#1205): none where the meeting cannot hold them; renamed in place below.
  const names = speakers ? { ...speakers } : null;
  const card = element("aside", "card transcript");
  card.setAttribute("aria-label", words.transcript_title);
  const top = element("div", "top");
  top.append(element("h2", "label", words.transcript_title));
  card.append(top);
  // What is being made for it, or why it was not (D25).
  const state = element("p", "making-state");
  state.id = "transcript-state";
  state.dataset.name = nameOf(session);
  state.dataset.made = String(makings?.transcripts_made[nameOf(session)] ?? 0);
  state.setAttribute("role", "status");
  state.hidden = true;
  card.append(state);
  if (read.state !== "present") {
    card.append(element("p", "none", read.state === "absent" ? words.no_transcript : words.unreadable));
    if (read.state === "absent" && session.audio > 0 && session.state !== "recording") {
      offer(card, words.make_transcript, "make-transcript", "transcript_asked", session, words);
    }
    return card;
  }
  const search = element("input", "search");
  search.type = "search";
  search.id = "search";
  search.placeholder = words.search;
  search.setAttribute("aria-label", words.search);
  // Its icon in the field, as the sidebar's search (D89).
  const finding = element("div", "finding");
  finding.append(icon("search"), search);
  // Refaire…: another Whisper model of this computer, this time only; beside the search (board 4).
  const redo = iconButton("rotate-ccw", words.redo, "redo-transcript");
  redo.addEventListener("click", async () => {
    const offered = await window.__TAURI__.core.invoke("models_offered", { name: nameOf(session) }).catch((why) => {
      redo.after(element("p", "note warn", fill(words.models_not_read, why)));
      return null;
    });
    if (offered === null) return;
    const panel = redoTranscriptPanel(offered, session, words, redo, read.segments.some((segment) => segment.edited));
    redo.replaceWith(panel);
    panel.querySelector("button:enabled").focus();
  });
  // Copied in one click, each line with when and who said it, as the page names them (D109); said in
  // place, a moment, as the minutes' copy is.
  const copy = iconButton("copy", words.copy_transcript, "copy-transcript");
  const copied = element("span", "says copied");
  copied.id = "transcript-copied";
  copied.setAttribute("role", "status");
  let said = 0;
  copy.addEventListener("click", async () => {
    const mine = ++said;
    const lines = read.segments.map((segment) => fill(words.transcript_copied_line, clock(segment.start), whoSaid(segment, words, names), segment.text));
    try {
      await navigator.clipboard.writeText(`${lines.join("\n")}\n`);
      copied.textContent = words.minutes_copied;
      setTimeout(() => mine === said && (copied.textContent = ""), 2000);
    } catch {
      copied.textContent = words.transcript_not_copied;
    }
  });
  top.append(finding, copied, copy, redo);
  // Nommer les intervenants… (#1205), once complete: the two sides' names in two labelled fields, as Otter and
  // Zoom rename a speaker from the transcript; kept with the meeting, every line following at once.
  if (names && read.complete) top.append(speakersButton(card, names, read, words, session));
  const heard = element("div", "made");
  // A service names its host as who heard (D43): said as the minutes' maker is, never « on this
  // computer » — a host of one word (`http://transcriber/v1`) is a service too. This computer's
  // engines name themselves: whisper.cpp, SpeechAnalyzer; none named, nothing is claimed.
  const service = read.engine && !["whisper.cpp", "SpeechAnalyzer"].includes(read.engine);
  // Made whole at a time it says (D93): that time first, then its model, as the minutes say theirs;
  // a transcript that does not say it, its model alone, as before.
  const when = madeOn(words, read.made);
  const by = service ? (read.model || "—") + fill(words.written_via, read.engine) : read.model ? spoken(read.model) : read.engine || "";
  // Its model a list in its place once it is whole (D93, criterion 5); none named, the line alone.
  const choice = by && read.complete;
  const heardSaid = when
    ? fill(words.transcribed_on, when) + (by ? ` · ${choice ? IN_LINE : by}` : "")
    : fill(service ? words.written_by : words.heard_by, choice ? IN_LINE : by || "—");
  if (choice) {
    const go = startsOnPage("page-transcription-do", words, () => window.__TAURI__.core.invoke("transcript_redo_own", { name: nameOf(session) }));
    const box = element("div", "page-redo");
    const was = element("p", "says", fill(words.transcript_made_with, by));
    was.id = "page-transcription-was";
    box.append(was, go);
    // Made again, the transcript is another file: a person's corrections are not in it (ADR-0014).
    if (read.segments.some((segment) => segment.edited)) box.append(element("p", "note warn", words.redo_loses_corrections));
    const version = { said: by, is: (chosen) => read.engine === "SpeechAnalyzer" ? chosen.engine === "speechanalyzer"
      : service ? chosen.engine === "service" && chosen.model === read.model && via(chosen.provider) === read.engine
      : chosen.engine === "whisper" && chosen.model === read.model };
    const [pick, said] = modelOnPage(onPage, words, "transcription", version, go, words.redo_transcript_do, words.redo_transcript_title, box);
    heard.append(lineWith(heardSaid + tookAndCost(read, words), pick), said, box);
  } else heard.append(element("p", "maker", heardSaid + tookAndCost(read, words)));
  card.append(heard);
  // Its model said as its own line says it: a service's by its host, this computer's by its name.
  if (previous) {
    const elsewhere = previous.provider && !["whisper.cpp", "SpeechAnalyzer"].includes(previous.provider);
    const model = elsewhere ? (previous.model || "—") + fill(words.written_via, previous.provider) : previous.model ? spoken(previous.model) : previous.provider || "";
    card.append(previousBack(words, session, "transcript", previous, model));
  }
  if (!read.complete) card.append(element("p", "note", words.being_made));
  if (read.unread > 0) card.append(element("p", "note", fill(words.unread, read.unread)));
  const lines = element("ol", "lines");
  // What a correction met, said where a screen reader hears it.
  const said_ = element("p", "says");
  said_.id = "correction-said";
  said_.setAttribute("role", "status");
  card.append(said_);
  for (const segment of read.segments) {
    const line = element("li");
    line.dataset.track = segment.track;
    line.dataset.start = String(segment.start);
    // A click on a line, or Enter on it, plays the meeting from it (D26).
    const from = () => {
      if (!player || player.name !== nameOf(session)) return;
      player.seek(segment.start);
      player.play();
    };
    line.addEventListener("click", () => {
      roam(lines, line);
      from();
    });
    // By the keyboard: one line in the order of Tab, the arrows to the others; Enter or Space
    // plays from it, as a button's keys do.
    if (player?.name === nameOf(session)) {
      line.setAttribute("role", "button");
      line.tabIndex = lines.children.length === 0 ? 0 : -1;
      line.addEventListener("keydown", (event) => {
        // The next line shown: one a search hides is passed over.
        let next = { ArrowDown: line.nextElementSibling, ArrowUp: line.previousElementSibling }[event.key];
        while (next?.hidden) next = event.key === "ArrowDown" ? next.nextElementSibling : next.previousElementSibling;
        if (next) {
          event.preventDefault();
          roam(lines, next);
          next.focus();
        }
        if (event.key !== "Enter" && event.key !== " ") return;
        event.preventDefault();
        if (!event.repeat) from();
      });
    }
    const who = whoSaid(segment, words, names);
    const said = element("div", "said");
    const text = element("span", "text", segment.text);
    said.append(element("span", `who ${segment.track}`, who), text);
    if (segment.edited) said.append(element("span", "edited", words.corrected));
    line.append(element("span", "at", clock(segment.start)), said);
    // Corrected by whoever listens, once complete (D62): the line becomes a field; Enter or leaving
    // it keeps what is typed, Escape leaves it as it was.
    if (read.complete) {
      const pencil = element("button", "edit");
      pencil.append(icon("pencil"));
      pencil.type = "button";
      // Its own id, the line's track and start — never its place —: found again once drawn anew (D101).
      pencil.id = `correct-${segment.track}-${segment.start}`;
      pencil.dataset.name = nameOf(session);
      pencil.setAttribute("aria-label", fill(words.correct_line, clock(segment.start)));
      pencil.title = pencil.getAttribute("aria-label");
      pencil.tabIndex = line.hasAttribute("role") ? line.tabIndex : 0;
      // Its own keys: Enter on it opens the field, never plays the line it sits in.
      pencil.addEventListener("keydown", (event) => event.stopPropagation());
      pencil.addEventListener("click", (event) => {
        event.stopPropagation();
        const field = element("input", "correcting");
        field.type = "text";
        field.value = text.textContent;
        field.setAttribute("aria-label", pencil.getAttribute("aria-label"));
        // A line break pasted is a space: a field of one line would drop it, gluing two words.
        field.addEventListener("paste", (event) => {
          const pasted = event.clipboardData?.getData("text/plain");
          if (!pasted || !/[\r\n]/.test(pasted)) return;
          event.preventDefault();
          field.setRangeText(pasted.replace(/[\r\n]+/g, " "), field.selectionStart, field.selectionEnd, "end");
        });
        let done = false;
        const leave = (kept) => {
          done = true;
          if (kept !== null) text.textContent = kept;
          field.replaceWith(text);
          pencil.hidden = false;
        };
        const keep = async () => {
          if (done) return;
          if (field.value === text.textContent) return leave(null);
          if (field.value.replace(/[\r\n]+/g, " ").trim() === "") {
            said_.textContent = words.correction_empty;
            return field.focus();
          }
          done = true;
          try {
            await window.__TAURI__.core.invoke("transcript_corrected", { name: nameOf(session), track: segment.track, start: segment.start, text: field.value });
            said_.textContent = "";
            if (!said.querySelector(".edited")) said.append(element("span", "edited", words.corrected));
            leave(field.value.replace(/[\r\n]+/g, " ").trim());
          } catch (why) {
            said_.textContent = fill(words.correction_failed, why);
            leave(null);
          }
        };
        for (const kind of ["click", "keydown"]) field.addEventListener(kind, (event) => event.stopPropagation());
        field.addEventListener("keydown", (event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            keep().then(() => pencil.focus());
          } else if (event.key === "Escape") {
            event.preventDefault();
            said_.textContent = "";
            leave(null);
            pencil.focus();
          }
        });
        field.addEventListener("blur", () => keep());
        text.replaceWith(field);
        pencil.hidden = true;
        field.focus();
      });
      line.append(pencil);
    }
    lines.append(line);
  }
  const nothing = element("p", "none", words.no_match);
  nothing.hidden = true;
  search.addEventListener("input", () => {
    const wanted = folded(search.value.trim());
    let shown = 0;
    for (const line of lines.children) {
      line.hidden = wanted !== "" && !folded(line.querySelector(".text").textContent).includes(wanted);
      marking(line.querySelector(".text"), line.hidden ? "" : wanted);
      shown += line.hidden ? 0 : 1;
    }
    nothing.hidden = shown > 0;
    // The line in the order of Tab hidden: the first one shown takes its place.
    const stop = [...lines.children].find((line) => line.tabIndex === 0);
    if (stop?.hidden) roam(lines, [...lines.children].find((line) => !line.hidden));
  });
  card.append(lines, nothing);
  return card;
}

// The meeting being listened to (D26): kept while its page is drawn again, so that a window
// coming forward never stops what plays. Stopped when another page is shown.
let player = null;

// Plays a meeting from the loopback server: its tracks as one sound, each where the session's
// clock puts it (ADR-0009). One media element, so the page keeps no clock of its own: its time
// is the session's.
function listen(name, url) {
  if (player?.name === name) return player;
  player?.stop();
  const audio = new Audio(url);
  audio.preload = "metadata";
  audio.className = "track";
  const shown = { name, audio, listeners: new Set(), failures: new Set(), ticking: 0 };
  const tell = () => {
    for (const listener of shown.listeners) listener(audio.currentTime);
  };
  const failed = (error) => {
    // A play cut short by a pause is no failure.
    if (error?.name === "AbortError") return;
    clearInterval(shown.ticking);
    audio.pause();
    for (const failure of shown.failures) failure();
  };
  shown.time = () => audio.currentTime;
  shown.playing = () => !audio.paused;
  // After a failure, the sound is asked for again before it is played.
  shown.play = () => {
    if (audio.error) audio.load();
    return audio.play().catch(failed);
  };
  shown.pause = () => audio.pause();
  // Never at its very end, where WebKitGTK's player stops answering: a tenth of a second before.
  // A seek asked while another is still going on is kept and made once that one ends: WebKitGTK's
  // player drops it otherwise, and the slider stays where it was (#369).
  shown.wanted = null;
  shown.seek = (at) => {
    shown.wanted = Math.max(0, Math.min(at, (audio.duration || at + 1) - 0.1));
    if (!audio.seeking) audio.currentTime = shown.wanted;
    tell();
  };
  audio.addEventListener("seeked", () => {
    if (shown.wanted !== null && Math.abs(audio.currentTime - shown.wanted) > 0.05) audio.currentTime = shown.wanted;
    else shown.wanted = null;
    tell();
  });
  shown.stop = () => {
    clearInterval(shown.ticking);
    audio.pause();
    audio.remove();
  };
  // The playhead and the line said, ten times a second while it plays; once more at each change.
  // A failure is read at each tick too: WebKitGTK sets a network error on a range refused
  // without the event, and its player waits for good.
  audio.addEventListener("play", () => {
    clearInterval(shown.ticking);
    shown.ticking = setInterval(() => (audio.error ? failed(audio.error) : tell()), 100);
    tell();
  });
  for (const changed of ["pause", "ended", "seeked", "loadedmetadata"]) {
    audio.addEventListener(changed, () => {
      if (changed !== "seeked" && changed !== "loadedmetadata") clearInterval(shown.ticking);
      tell();
    });
  }
  audio.addEventListener("error", failed);
  // Kept in the page, never shown: the browser plays an element it holds.
  document.body.append(audio);
  player = shown;
  return shown;
}

// The player of a meeting's page: Écouter / Pause, where the playback is, and the drawing of who
// spoke as its control — a click, or the arrows on it, put the playback there (D26).
function playerBar(session, listened, total, words, drawing) {
  if (!listened) {
    // Its audio freed while it played: nothing plays on.
    if (player?.name === nameOf(session)) {
      player.stop();
      player = null;
    }
    // Being recorded, no track written yet: nothing to say, and nothing deleted (#1054).
    if (session.audio === 0 && session.state === "recording") return "";
    // Freed, said so; there and not playable, said so too — never one for the other.
    const freed = element("p", "note", session.audio > 0 ? words.not_playable : words.no_player);
    freed.id = "no-player";
    return freed;
  }
  const shown = listen(nameOf(session), listened);
  const bar = element("div", "player");
  const toggle = element("button", "open", "");
  toggle.type = "button";
  toggle.id = "play";
  const at = element("span", "at", "");
  at.id = "played";
  // A track the window could not play, said where a screen reader hears it.
  const says = element("p", "note", "");
  says.id = "not-played";
  says.setAttribute("role", "status");
  const head = element("span", "playhead");
  drawing.append(head);
  drawing.tabIndex = 0;
  drawing.setAttribute("role", "slider");
  drawing.setAttribute("aria-label", words.playback);
  drawing.setAttribute("aria-valuemin", "0");
  // The slider runs to the sound's end, once known: it may outlast the session's recorded length.
  const span = () => Math.max(total, shown.audio.duration || 0);
  const paint = (time) => {
    // Written only when it changes: playing, this runs ten times a second, and a press on words
    // written again under the pointer is no click in WebKitGTK.
    const said = shown.playing() ? words.pause : words.play;
    if (toggle.textContent !== said) toggle.textContent = said;
    at.textContent = clock(time);
    head.style.left = `${Math.min(100, (time / total) * 100)}%`;
    drawing.querySelector(".wave")?.played(time);
    drawing.setAttribute("aria-valuemax", String(Number(span().toFixed(1))));
    drawing.setAttribute("aria-valuenow", String(Number(time.toFixed(1))));
    drawing.setAttribute("aria-valuetext", `${clock(time)} / ${clock(span())}`);
  };
  shown.listeners.clear();
  shown.listeners.add(paint);
  shown.failures.clear();
  shown.failures.add(() => (says.textContent = words.not_played));
  toggle.addEventListener("click", () => {
    if (shown.playing()) return shown.pause();
    says.textContent = "";
    shown.play();
  });
  drawing.addEventListener("click", (event) => {
    const box = drawing.getBoundingClientRect();
    shown.seek(((event.clientX - box.left) / box.width) * total);
  });
  // A slider's keys: five seconds back or on, the start, the end.
  drawing.addEventListener("keydown", (event) => {
    // From where the last key asked, not from where the player still is.
    const at = shown.wanted ?? shown.time();
    const to = { ArrowLeft: at - 5, ArrowDown: at - 5, ArrowRight: at + 5, ArrowUp: at + 5, PageDown: at - 30, PageUp: at + 30, Home: 0, End: span() }[
      event.key
    ];
    if (to === undefined) return;
    event.preventDefault();
    shown.seek(to);
  });
  paint(shown.time());
  bar.append(toggle, at, says);
  return bar;
}

// The space bar plays and pauses the meeting shown, wherever the focus is but in a field or on
// a control of its own; held down, it does not toggle again and again.
document.addEventListener("keydown", (event) => {
  if (event.key !== " " || !player || document.getElementById("meeting")?.hidden) return;
  if (event.target.closest("input, textarea, select, button, a, [role=button]")) return;
  event.preventDefault();
  if (event.repeat) return;
  player.playing() ? player.pause() : player.play();
});

// A meeting to the trash from its page (D100): the question shows while `trashAsking` names it — a
// drawing meanwhile draws it again as it was —; once it went, the list focuses `trashedFocus`.
let trashAsking = null;
let trashedFocus = null;

// The question, in place: what the meeting holds, that the system's trash restores it, « Annuler »
// first; confirmed, sent as « Faire de la place » sends it — the whole folder to the system's trash.
function trashQuestion(session, transcript, minutes, words, trash) {
  const name = nameOf(session);
  const ask = element("section", "card trash-ask");
  ask.id = "trash-ask";
  ask.hidden = trashAsking !== name;
  ask.setAttribute("role", "group");
  ask.setAttribute("aria-labelledby", "trash-ask-title");
  const title = element("p", "name", words.trash_ask);
  title.id = "trash-ask-title";
  const held = [session.audio > 0, transcript.state === "present", minutes.state === "present"]
    .map((there, at) => (there ? words.trash_parts[at] : null))
    .filter(Boolean);
  const holds = element("p", "says", held.length ? fill(words.trash_holds, new Intl.ListFormat(words.locale, { type: "conjunction" }).format(held)) : "");
  const restore = element("p", "says", words.trash_restore);
  const cancel = element("button", "open", words.trash_cancel);
  cancel.type = "button";
  cancel.id = "trash-cancel";
  // Never red: red says a recording, and nothing else (look.rs).
  const confirm = element("button", "open", words.trash_confirm);
  confirm.type = "button";
  confirm.id = "trash-confirm";
  const said = element("p", "note warn");
  said.id = "trash-said";
  said.setAttribute("role", "status");
  const close = () => {
    trashAsking = null;
    ask.hidden = true;
    said.textContent = "";
    trash.setAttribute("aria-expanded", "false");
    trash.focus();
  };
  cancel.addEventListener("click", close);
  ask.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    event.preventDefault();
    event.stopPropagation();
    close();
  });
  confirm.addEventListener("click", async () => {
    // Never disabled while it holds the focus: WebKit would let it go (#886).
    if (confirm.getAttribute("aria-disabled") === "true") return;
    waiting(confirm, true);
    try {
      await window.__TAURI__.core.invoke("freed", { names: [name], whole: true });
    } catch (why) {
      // Being recorded or made, or the trash refused: said, nothing moved.
      said.textContent = fill(words.trash_refused, why);
      waiting(confirm, false);
      return;
    }
    trashAsking = null;
    // The list next, the focus on the meeting after it — or before it, the last one —, or on the list.
    const links = [...document.querySelectorAll("#groups .session")];
    const at = links.findIndex((link) => link.getAttribute("href") === `#/meeting/${encodeURIComponent(name)}`);
    const next = at < 0 ? null : (links[at + 1] ?? links[at - 1] ?? null);
    trashedFocus = next ? next.getAttribute("href") : "";
    location.hash = "#/";
  });
  const choices = element("div", "trash-choices");
  choices.append(cancel, confirm);
  ask.append(title, holds, restore, choices, said);
  return ask;
}

// After a drawing of the list: the focus a meeting gone to the trash asked for (D100).
// A making cancelled, its card saying it stopped: the page drawn anew offers to make it again, and the
// focus goes there once the drawing has given back its own place (D92, D101).
function cancelledFocused() {
  if (cancelledFrom === null) return;
  const said = document.getElementById(cancelledFrom.id);
  if (!said || said.dataset.name !== cancelledFrom.name || !said.classList.contains("stopped")) return;
  const offered = document.getElementById(said.id === "minutes-state" ? "make-minutes" : "make-transcript");
  if (!offered || offered.hidden) return;
  offered.focus();
  cancelledFrom = null;
}

function trashedFocused() {
  if (trashedFocus === null || !(location.hash === "" || location.hash === "#/")) return;
  const to = trashedFocus;
  trashedFocus = null;
  const link = to ? document.querySelector(`#groups .session[href="${CSS.escape(to)}"]`) : null;
  const list = document.getElementById("list");
  if (link) link.focus();
  else if (list) {
    list.tabIndex = -1;
    list.focus();
  }
}

function showMeeting({ session, transcript, minutes, developer, listen: listened, context, confidential, language, languages, meeting_language, apple, system_language, models_gone, previous_minutes, previous_transcript, minutes_other_version, speakers }, words) {
  const page = document.getElementById("meeting");
  const top = element("div", "meeting-top");
  const open = iconButton("folder-open", words.open_folder, "open-folder");
  open.addEventListener("click", () => window.__TAURI__.core.invoke("open_folder", { name: nameOf(session) }).catch((why) => {
    open.replaceWith(element("p", "note warn", why));
  }));
  // To the system's trash from its page (D100): asked in place first, « Annuler » focused.
  const trash = iconButton("trash-2", words.trash_meeting, "trash-meeting");
  const ask = trashQuestion(session, transcript, minutes, words, trash);
  trash.setAttribute("aria-controls", ask.id);
  trash.setAttribute("aria-expanded", String(!ask.hidden));
  trash.addEventListener("click", () => {
    trashAsking = nameOf(session);
    ask.hidden = false;
    trash.setAttribute("aria-expanded", "true");
    document.getElementById("trash-cancel")?.focus();
  });
  const actions = element("div", "actions");
  // The rarer actions, behind « Plus d’actions » (D89, board 4): the name, as its heading clicked.
  const rarer = [["rename-item", words.rename_meeting, () => document.getElementById("rename")?.click()]];
  // In developer mode: this meeting's logs, to send to whoever looks into it — nothing of the
  // meeting is in them (D29).
  if (developer) {
    // What became of it, said where a screen reader hears it.
    const copied = element("span", "says", "");
    copied.setAttribute("role", "status");
    rarer.push(["copy-diagnostic", words.copy_diagnostic, async () => {
      try {
        const lines = await window.__TAURI__.core.invoke("diagnostic", { name: nameOf(session) });
        await navigator.clipboard.writeText(lines);
        copied.textContent = lines ? words.diagnostic_copied : words.diagnostic_empty;
      } catch {
        copied.textContent = words.diagnostic_not_copied;
      }
    }]);
    actions.append(copied);
  }
  actions.append(open, trash, moreActions(rarer, words));
  // The way back is the pane's, under 840 px alone (D61).
  top.append(actions);
  // The meeting's header (D89, board 4): a bar of 56 px, its name, how long and how heavy beside
  // it, its actions at their right.
  const head = element("div", "meeting-head");
  const title = element("section", "meeting-title");
  const named = element("div", "named");
  // Named by its minutes (D48): the date above the name.
  if (session.title) named.append(element("span", "on", heading(session, words)));
  named.append(meetingName(session, words));
  const lasts = element("span", "lasts");
  if (session.seconds !== null) lasts.append(element("span", "length", length(session.seconds)));
  if (session.audio > 0) lasts.append(element("span", "weighs", fill(words.audio_weighs, size(session.audio, words))));
  const held = holdings(session, words, true);
  // While it records, said confidential (#1204, D60): the lock and its word first, beside « Enregistrement », as the
  // bar shows it; at rest, the page's own switch says it.
  if (session.confidential && session.state === "recording") held.prepend(lockedChip(words));
  title.append(named, lasts, held);
  head.append(title, top);
  const said = transcript.state === "present" ? transcript.segments : [];
  const total = session.seconds || said.reduce((end, s) => Math.max(end, s.end), 0) || 1;
  const drawn = element("section", "card drawn");
  const legend = element("div", "legend");
  // While it records (#1076): the time running in place of the range, read as the bar reads it.
  const live = session.state === "recording";
  // A legend names what is drawn: none before a transcript (#1116); while it records, the labels stand under the meters.
  if (transcript.state === "present") legend.append(element("span", "here", words.here_described), element("span", "remote", words.remote_described));
  if (!live) legend.append(element("span", "range", `${clock(0)} – ${clock(total)}`));
  const drawing = element("div", "drawing");
  if (live) {
    // Large in the who-spoke area while it records (#1112), as Voice Memos and Granola centre it, the meters under it.
    const time = element("span", "live-time", elapsed(0));
    time.id = "page-time";
    const now = element("div", "live-now");
    now.append(time, pageLevels(nameOf(session), time, words));
    drawing.append(now);
  } else if (transcript.state === "present") {
    drawing.append(relief(said, total, words));
    // Its sound's own shape once the shell has read it (#1201): who spoke, from the transcript, until then — and for
    // good when no track reads.
    if (session.audio > 0)
      window.__TAURI__.core
        .invoke("peaks_of", { name: nameOf(session) })
        .then((peaks) => {
          const turns = drawing.querySelector(".relief");
          if (peaks && turns?.isConnected) turns.replaceWith(waveform(peaks, total));
        })
        .catch(() => {});
  }
  else {
    // No transcript yet (#1116, the decider): one line says when who spoke will appear, no blank canvas. Outside the
    // drawing (#1134), which is the player's slider once the sound plays: a slider's children are presentational
    // (WAI-ARIA 1.2). The drawing is then a bare track, as Voice Memos' and Otter's before their transcript.
    const later = element("p", "note", words.who_spoke_later);
    later.id = "who-spoke-later";
    drawn.append(later);
    drawing.classList.add("bare");
  }
  // While it records, the legend's labels stand under their meters; at rest, the legend in its place — none while
  // nothing is drawn to name or to scale (#1134): the header says the meeting's length.
  drawn.append(drawing);
  if (!live && transcript.state === "present") drawn.append(legend);
  // No player while it records (#1130), as Voice Memos and Granola play a meeting once it stops: the drawing,
  // made its slider, would make the meters inside it presentational (WAI-ARIA 1.2), and a press on the time a seek.
  if (!live) drawn.append(playerBar(session, listened, total, words, drawing));
  const columns = element("div", "columns");
  // The models each line offers, read once for both (D93).
  const onPage = pageModels(session);
  columns.append(minutesCard(minutes, words, session, transcript, onPage, previous_minutes, minutes_other_version), transcriptCard(transcript, words, session, onPage, previous_transcript, speakers));
  // Shown again when the window comes forward: a search in progress in this meeting is kept.
  const searching = page.dataset.name === nameOf(session) ? (document.getElementById("search")?.value ?? "") : "";
  // A name being typed is kept the same way, with its refusal and where the caret was (D70).
  const naming = document.querySelector("#naming:not([data-saving])");
  const namingRefused = document.getElementById("naming-said")?.textContent ?? "";
  const same = page.dataset.name === nameOf(session);
  const typing = same && naming ? { value: naming.value, caret: naming.selectionStart, end: naming.selectionEnd, focused: document.activeElement === naming } : null;
  page.dataset.name = nameOf(session);
  const [secret, left, secretSaid, secretDescribed] = confidentialCard(session, confidential, words);
  const [spoken, spokenSaid] = languageRow(
    (chosen) => window.__TAURI__.core.invoke("language_set", { name: nameOf(session), language: chosen }),
    language, languages, words, meeting_language, apple, system_language, "meeting-language", "language-said", nameOf(session),
  );
  // A model the meeting chose and cannot have now (D91): Settings' stands in, said, as Auto for a microphone.
  const gone = element("p", "note warn");
  gone.id = "models-gone";
  gone.textContent = models_gone.map((step) => (step === "transcription" ? words.models_gone_transcription : words.models_gone_minutes)).join(" ");
  gone.hidden = models_gone.length === 0;
  const settings = contextCard(session, { context, transcript, minutes }, words, [spoken, secret], [left, secretSaid, spokenSaid, secretDescribed, gone]);
  // The trash's question in its own card, under the head, as wide as the page's cards (D100).
  // Its making, one bar across both steps under its head (D92), painted with the makings.
  const making = element("section", "card making");
  making.id = "making";
  making.dataset.name = nameOf(session);
  making.hidden = true;
  // The same meeting drawn again — its transcript written, say —: its step's live region is the same
  // node, never one said anew, and the bar is painted before anything can show it missing.
  const step = (same && document.getElementById("making-step")) || element("span", "what");
  step.id = "making-step";
  step.setAttribute("aria-live", "polite");
  const cancel = element("span", "cancel");
  making.append(step, element("span", "by"), element("span", "left"), cancel, element("div", "segments"));
  page.replaceChildren(head, making, ask, settings, drawn, columns);
  paintBar();
  // The line being said, marked while it plays; the page stays where the user left it (D26, the
  // decider, 2026-10-24: following the line was intrusive).
  if (player?.name === nameOf(session)) {
    const lines = [...page.querySelectorAll(".lines li")];
    let marked = null;
    // The player it follows, not the page's: a stopped one still says its last pause, after the
    // page let it go.
    const followed = player;
    followed.listeners.add((at) => {
      const now = lines.findLast((line) => Number(line.dataset.start) <= at) ?? null;
      if (now === marked) return;
      marked?.classList.remove("now");
      now?.classList.add("now");
      marked = now;
    });
    followed.listeners.forEach((listener) => listener(followed.time()));
  }
  const search = document.getElementById("search");
  if (search && searching) {
    search.value = searching;
    search.dispatchEvent(new Event("input"));
  }
  if (typing) {
    const button = document.getElementById("rename");
    // Typed in, it takes the focus back; the user gone elsewhere, they are not brought back.
    nameField(button.parentElement, button, session, words, typing.value, namingRefused, typing.focused);
    if (typing.focused) document.getElementById("naming").setSelectionRange(typing.caret, typing.end);
  }
}

// A name refused while its field stays open, until the page is left.
let nameRefused = null;
// What the service's button last said, and whether it is still asking: Settings are drawn anew under
// it by a model come or the window coming back, the answer still to arrive (D72).
let serviceAnswer = null;

// A sentence in the pane, until the next address.
function paneSays(sentence) {
  const said = document.getElementById("pane-said");
  said.textContent = sentence;
  said.hidden = false;
}

// The meeting's heading (D70): clicked, a field. Enter or leaving it keeps what is typed, emptied
// its minutes' title or its date comes back; Escape leaves it as it was.
function meetingName(session, words) {
  const heading_ = element("h1");
  const button = element("button", "rename", session.title ?? heading(session, words));
  button.type = "button";
  button.id = "rename";
  button.title = words.rename_meeting;
  button.addEventListener("click", () => nameField(heading_, button, session, words, session.title ?? ""));
  heading_.append(button);
  return heading_;
}

function nameField(heading_, button, session, words, typed, refused = "", focused = true) {
  const field = element("input", "naming");
  field.type = "text";
  field.id = "naming";
  field.maxLength = 80;
  field.value = typed;
  field.placeholder = words.name_meeting;
  field.setAttribute("aria-label", words.name_meeting);
  const said = element("p", "says");
  said.id = "naming-said";
  said.setAttribute("role", "status");
  said.textContent = refused;
  said.hidden = refused === "";
  let done = false;
  const leave = () => {
    done = true;
    nameRefused = null;
    said.remove();
    field.replaceWith(button);
  };
  const keep = async () => {
    if (done) return;
    if (field.value.trim() === (session.title ?? "")) return leave();
    done = true;
    // Being saved, no longer being typed: a reading of the page meanwhile does not open it again.
    field.dataset.saving = "";
    // An earlier refusal is no longer the last word, nor a sentence for the next address.
    nameRefused = null;
    const asked = location.hash;
    try {
      await window.__TAURI__.core.invoke("title_saved", { name: nameOf(session), title: field.value });
      button.textContent = field.value.trim() || heading(session, words);
      leave();
      await refresh();
    } catch (why) {
      done = false;
      delete field.dataset.saving;
      // Its page left or drawn anew meanwhile: said where it stays, never to no one.
      if (!field.isConnected || location.hash !== asked) return paneSays(fill(words.name_not_saved, why));
      said.textContent = fill(words.name_not_saved, why);
      said.hidden = false;
      // The page left next, by the click that took the focus from here: the pane says it then.
      nameRefused = said.textContent;
    }
  };
  field.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      keep().then(() => document.getElementById("rename")?.focus());
    } else if (event.key === "Escape") {
      event.preventDefault();
      leave();
      button.focus();
    }
  });
  // Another window come forward leaves the field where it was: only a click or a tab elsewhere keeps it.
  field.addEventListener("blur", () => {
    if (document.activeElement !== field) keep();
  });
  button.replaceWith(field);
  heading_.after(said);
  if (!focused) return;
  field.focus();
  field.select();
}

// The settings' page: each change saved at once; nothing offered over a file that cannot be read.
function choice(name, value, [label, says], checked, disabled) {
  const option = element("label", "choice");
  const input = element("input");
  input.type = "radio";
  input.name = name;
  input.value = value;
  // Its own id, from what it chooses — never its place —: found again once drawn anew (D101).
  input.id = `${name}-${value}`;
  input.checked = checked;
  input.disabled = disabled;
  const text = element("span", "said");
  text.append(element("span", "name", label), element("span", "says", says));
  option.append(input, text);
  return option;
}

// Who writes the minutes: picked from what Ollama holds, an online service, or the user's own
// address; a key goes to the system's keyring, and the page never receives one back.
let testing = false;
// Settings' « Avancé » unfolded: kept while Settings are drawn again, as a choice there redraws them.
let generalAdvanced = false;
// « Modèles à télécharger » unfolded by the user, kept over the page's drawings (#782).
let offeredOpen = false;
// The form adding a destination unfolded (D110): folded at first, no address asked in the common
// case (E5); kept while Settings are drawn again, as a destination added redraws them.
let destinationOpen = false;
// Each guided path's own (D95), the summary's and the transcription's: « Avancé » unfolded, kept
// while Settings are drawn again; each service's last answer to « Vérifier la clé », by its place
// in the list — what it refused, or the models it listed —, and where and which service chosen,
// not yet saved: kept while Settings are drawn again by a model come or the window's width, until
// the user acts (ux.md: an error stays until corrected). Another address forgets these two.
const guided = {};
for (const path of ["minutes", "transcription"]) guided[path] = { advanced: null, answers: new Map(), chosen: {} };

// The guided path's pieces (D95), the same for the transcription and the summary.
function guidedField(id, text, type, value, locked) {
  const wrap = element("label", "field");
  wrap.htmlFor = id;
  const input = element("input");
  input.id = id;
  input.type = type;
  input.value = value || "";
  input.defaultValue = input.value;
  input.disabled = locked;
  wrap.append(element("span", "name", text), input);
  return [wrap, input];
}

function guidedList(id, text, locked) {
  const wrap = element("label", "field");
  wrap.htmlFor = id;
  const pick = element("select");
  pick.id = id;
  pick.disabled = locked;
  wrap.append(element("span", "name", text), pick);
  return [wrap, pick];
}

function guidedOption(pick, value, text) {
  const one = element("option", "", text);
  one.value = value;
  pick.append(one);
}

function guidedButton(id, text, primary, locked) {
  const one = element("button", primary ? "primary" : "open", text);
  one.type = "button";
  one.id = id;
  one.disabled = locked;
  return one;
}

// First: on this computer, or a service — one choice, named by the step's title; the way chosen
// kept as the path's own. `way` is the one shown checked.
function guidedWhere(path, words, way, locked) {
  const group = element("div", "choices");
  group.id = `${path}-where`;
  group.setAttribute("role", "radiogroup");
  group.setAttribute("aria-labelledby", `${path}-title`);
  const here = choice(`${path}-where`, "here", [words.welcome_on_this_computer, ""], way === "here", locked);
  const away = choice(`${path}-where`, "online", [words.where_service, ""], way === "online", locked);
  here.querySelector("input").id = `${path}-here`;
  away.querySelector("input").id = `${path}-online`;
  group.append(here, away);
  return [group, here.querySelector("input"), away.querySelector("input")];
}

// « Vérifier la clé »: the service's models asked with the key typed — or the one kept —, its
// answer kept as the path's own, said on the page as it is now: drawn again meanwhile, or another
// service chosen, the page says its own.
// `keyFor()`: what the key field is for now — the service chosen, or under « Avancé » an address on
// another host (`keyFor`'s `host:…`). Returns the key typed for one of them, and what to call when the
// address changes.
function guidedKeyTested(path, words, locked, { servicePick, key, keyTest, keySaid, modelsPick }, ask, shownSaved, keyFor) {
  // A key typed and not yet saved: kept across a drawing for what it was typed for, as the model chosen
  // with it — in the page's memory alone, never written nor logged, forgotten with the choices at
  // another address (#948). A key leaves only for the host it was typed for: another service or
  // another host shows its own, or none (#964; OWASP ASVS, a secret sent to its recipient alone).
  const keys = () => guided[path].chosen.keys ?? {};
  const shown = () => {
    key.value = keys()[keyFor()] ?? "";
  };
  shown();
  key.addEventListener("input", () => {
    guided[path].chosen.keys = { ...keys(), [keyFor()]: key.value };
  });
  servicePick.addEventListener("change", shown);
  const typed = (id) => (id === keyFor() ? key.value : keys()[id]) || null;
  // Saved: no longer needed in the page; the field says a key is kept (#964).
  const forget = (id) => {
    const { [id]: _, ...rest } = keys();
    guided[path].chosen.keys = rest;
  };
  keyTest.addEventListener("click", async () => {
    waiting(keyTest, true);
    keySaid.textContent = words.testing;
    const asked = servicePick.value;
    let models = null;
    let said = "";
    try {
      models = await ask(asked, typed(`service:${asked}`));
    } catch (why) {
      said = why;
    }
    guided[path].answers.set(asked, { said, models });
    const now = document.getElementById(`${path}-service`);
    if (now !== servicePick) return now?.dispatchEvent(new Event("change"));
    waiting(keyTest, false);
    if (servicePick.value !== asked) return;
    shownSaved();
    if (models !== null) modelsPick.focus();
  });
  return { shown, typed, forget };
}

// What a key is kept for: the service chosen, or an address of one's own on another host (#964).
function keyHost(address) {
  try {
    return new URL(address).host;
  } catch {
    return address;
  }
}
function keyFor(address, serviceAddress, index) {
  const own = address.trim();
  return own && keyHost(own) !== keyHost(serviceAddress) ? `host:${keyHost(own)}` : `service:${index}`;
}

// « Avancé », folded unless the path keeps it unfolded or `own` is saved: what it holds hidden.
function guidedAdvanced(path, words, own, contents) {
  const toggle = element("button", "disclose-advanced");
  toggle.type = "button";
  toggle.id = `${path}-advanced`;
  toggle.setAttribute("aria-controls", `${path}-advanced-panel`);
  toggle.append(icon("chevron-right"), element("span", "", words.advanced_title));
  const panel = element("div", "fields");
  panel.id = `${path}-advanced-panel`;
  panel.append(...contents);
  const unfold = (now) => {
    guided[path].advanced = now;
    toggle.setAttribute("aria-expanded", String(now));
    toggle.firstChild.replaceWith(icon(now ? "chevron-down" : "chevron-right"));
    panel.hidden = !now;
  };
  toggle.addEventListener("click", () => unfold(!guided[path].advanced));
  // Folded or unfolded by the user, kept across a drawing; else unfolded when what is saved is of its own (#948).
  unfold(guided[path].advanced ?? own);
  return [toggle, panel];
}

// A model chosen in a list that waits for « Enregistrer », not yet saved: kept across a drawing, as the
// way and the service above it are — Settings are drawn anew from what is saved whenever another
// control is (#948). `key`: which list of the step, read as it changes; else the one saved.
function chosenKept(path, list, key, saved) {
  const values = [...list.options].map((option) => option.value);
  const pending = guided[path].chosen.models?.[key];
  const kept = values.includes(pending) ? pending : values.includes(saved) ? saved : null;
  if (kept !== null) list.value = kept;
}
function chosenKeeps(path, list, key) {
  list.addEventListener("change", () => {
    guided[path].chosen.models = { ...guided[path].chosen.models, [key()]: list.value };
  });
}

function minutesRow(minutes, words, locked) {
  const box = element("div", "step");
  const title = element("span", "name", words.minutes_step);
  title.id = "minutes-title";
  const online = minutes.online.findIndex((service) => service.address === minutes.provider);
  const savedHere = !!minutes.provider && minutes.provider === minutes.ollama_address;
  const own = !!minutes.provider && !savedHere && online < 0;
  // First (D95): on this computer, or a service — one choice; an address of one's own under « Avancé ».
  const [way, hereRadio, awayRadio] = guidedWhere("minutes", words, guided.minutes.chosen.way ?? (savedHere ? "here" : minutes.provider ? "online" : null), locked);
  const where = element("p", "status");
  where.id = "minutes-status";
  const fields = element("div", "fields");
  const field = (id, text, type, value) => guidedField(id, text, type, value, locked);
  const listed = (id, text) => guidedList(id, text, locked);
  const option = guidedOption;
  const button = (id, text, primary) => guidedButton(id, text, primary, locked);
  // Saved or not, the page is drawn again from what the shell holds; a failure stays said.
  const save = async (provider, model, key, keyId) => {
    let failed = null;
    try {
      const notKept = await window.__TAURI__.core.invoke("minutes_saved", { provider, model, key });
      if (notKept !== null) failed = fill(words.key_not_kept, notKept);
    } catch (why) {
      failed = fill(words.not_saved, why);
    }
    if (failed === null && keyId) keyed.forget(keyId);
    notSaved = failed;
    await refresh();
    savedSaid(failed === null);
  };
  // Chosen, not yet saved: who writes them meanwhile is said — going through a list, with the
  // keyboard's arrows too, never changes who does (D25).
  const unsaved = element("p", "says");
  unsaved.id = "minutes-unsaved";
  unsaved.textContent = minutes.provider && minutes.model ? fill(words.minutes_unsaved, modelSaid(minutes.model)) : words.minutes_unsaved_none;
  unsaved.hidden = true;
  // On this computer: Ollama's models, each said as people say it.
  const [hereWrap, herePick] = listed("minutes", words.model_name);
  for (const name of minutes.ollama) option(herePick, name, name === minutes.recommended ? fill(words.model_recommended, modelSaid(name)) : modelSaid(name));
  chosenKept("minutes", herePick, "here", savedHere ? minutes.model : null);
  chosenKeeps("minutes", herePick, () => "here");
  const hereSave = button("minutes-save", words.save, false);
  hereSave.addEventListener("click", () => save(minutes.ollama_address, herePick.value, null));
  herePick.addEventListener("change", () => (unsaved.hidden = savedHere && herePick.value === minutes.model));
  // A service: named, its key tested at once — its models asked with it —, then one of them (D95).
  const [serviceWrap, servicePick] = listed("minutes-service", words.service_name);
  minutes.online.forEach((service, index) => option(servicePick, String(index), service.name));
  servicePick.value = guided.minutes.chosen.service ?? String(Math.max(online, 0));
  serviceMarked(servicePick, () => minutes.online[Number(servicePick.value)]?.name);
  const [keyWrap, key] = field("minutes-key", words.key, "password", "");
  key.autocomplete = "off";
  const keyTest = button("minutes-key-test", words.key_test, false);
  const keySaid = element("p", "says");
  keySaid.id = "minutes-key-said";
  keySaid.setAttribute("role", "status");
  const [modelsWrap, modelsPick] = listed("minutes-models", words.model_name);
  const onlineSave = button("minutes-online-save", words.save, true);
  const service = () => minutes.online[Number(servicePick.value)];
  chosenKeeps("minutes", modelsPick, () => servicePick.value);
  // One « Enregistrer » at a time, as the transcription's (#860): the service's, « Avancé » folded;
  // unfolded, its own (#968).
  let listedShown = false;
  const paintSave = () => {
    onlineSave.hidden = !listedShown || (guided.minutes.advanced ?? own);
  };
  // The models listed with its key, the one saved chosen.
  const shownListed = (models) => {
    modelsPick.replaceChildren();
    for (const id of models) option(modelsPick, id, modelSaid(id));
    chosenKept("minutes", modelsPick, servicePick.value, minutes.model);
    modelsWrap.hidden = models.length === 0;
    listedShown = models.length > 0;
    paintSave();
  };
  // This service's last answer, said again; else the model saved with it — the others come with its key.
  const shownSaved = () => {
    const saved = online >= 0 && Number(servicePick.value) === online && minutes.model;
    key.placeholder = saved && minutes.key_kept ? words.key_kept : "";
    const answer = guided.minutes.answers.get(servicePick.value);
    keySaid.textContent = answer?.said ?? "";
    if (answer?.models) return shownListed(answer.models);
    modelsPick.replaceChildren();
    if (saved) option(modelsPick, minutes.model, modelSaid(minutes.model));
    modelsWrap.hidden = !saved;
    listedShown = !!saved;
    paintSave();
  };
  shownSaved();
  servicePick.addEventListener("change", () => {
    guided.minutes.chosen.service = servicePick.value;
    shownSaved();
    address.placeholder = service().address;
  });
  // « Avancé »: an address of one's own, OpenAI's way of asking, its model named by hand.
  const [addressWrap, address] = field("minutes-address", words.address, "url", own ? minutes.provider : "");
  const [modelWrap, model] = field("minutes-model", words.model_name, "text", own ? minutes.model : "");
  const ownSave = button("minutes-own-save", words.save, false);
  // Its address left empty: the service chosen above, with a model it does not list — a new release,
  // an id of its own — named by hand.
  address.placeholder = service().address;
  // « Avancé » folded, its address is not what the key is typed for: the service's is (#963).
  const keyNow = () => keyFor((guided.minutes.advanced ?? own) ? address.value : "", service().address, servicePick.value);
  const keyed = guidedKeyTested("minutes", words, locked, { servicePick, key, keyTest, keySaid, modelsPick },
    (index, typed) => window.__TAURI__.core.invoke("minutes_models", { provider: minutes.online[Number(index)].address, key: typed }), shownSaved, keyNow);
  address.addEventListener("input", keyed.shown);
  const serviceKeyId = () => `service:${servicePick.value}`;
  onlineSave.addEventListener("click", () => save(service().address, modelsPick.value, keyed.typed(serviceKeyId()), serviceKeyId()));
  ownSave.addEventListener("click", () => save(address.value.trim() || service().address, model.value, keyed.typed(keyNow()), keyNow()));
  const [toggle, advanced] = guidedAdvanced("minutes", words, own, [addressWrap, modelWrap, ownSave]);
  toggle.addEventListener("click", () => {
    paintSave();
    keyed.shown();
  });
  const away = element("div", "fields");
  away.append(serviceWrap, keyWrap, keyTest, keySaid, modelsWrap, onlineSave, toggle, advanced);
  const here = element("div", "fields");
  // No model in Ollama, or no Ollama: said so, nothing to save.
  if (minutes.ollama.length > 0) here.append(hereWrap, hereSave);
  // Said once: Ollama saved here says it already, below the choice.
  else if (!savedHere) here.append(element("p", "says", words.ollama_silent));
  const draw = () => {
    const isHere = hereRadio.checked;
    const isAway = awayRadio.checked;
    here.hidden = !isHere;
    away.hidden = !isAway;
    unsaved.hidden = !(isHere && minutes.ollama.length > 0 && !(savedHere && herePick.value === minutes.model));
    // Where the text goes, as saved: here, nothing leaves; else the service's host.
    where.textContent = "";
    where.className = "status";
    if (savedHere || (own && minutes.provider_local)) {
      where.className = "status here";
      where.textContent = words.on_this_computer;
    } else if (minutes.provider) {
      let host = minutes.provider;
      try {
        host = new URL(minutes.provider).host;
      } catch {}
      where.className = "status online";
      where.replaceChildren(words.where_online, element("code", "tag", host));
    }
  };
  for (const radio of [hereRadio, awayRadio]) {
    radio.addEventListener("change", () => {
      guided.minutes.chosen.way = radio.value;
      draw();
    });
  }
  draw();
  fields.append(here, away);
  box.append(title, way, where);
  // This computer's Ollama saved: a model it does not have named; not answering, or with no model
  // at all, said so — never "the model is missing" on a list that could not be read.
  if (savedHere && minutes.model && !minutes.ollama.includes(minutes.model)) {
    const answered = minutes.ollama.length > 0;
    const said = element("p", "says warn", answered ? fill(words.ollama_lacks, minutes.model) : words.ollama_silent);
    said.id = answered ? "minutes-lacking" : "minutes-silent";
    box.append(said);
  }
  box.append(fields, unsaved);
  // Tester: the choice saved, asked whether it answers — one sentence of Kitokey's, said first.
  // One test at a time, the page drawn again meanwhile or not.
  if (minutes.provider && minutes.model) {
    const test = element("div", "row");
    const button = element("button", "open", words.test);
    button.type = "button";
    button.id = "minutes-test";
    button.disabled = locked || testing;
    const tested = element("p", "says");
    tested.id = "minutes-tested";
    tested.setAttribute("aria-live", "polite");
    let host = minutes.provider;
    try {
      host = new URL(minutes.provider).host;
    } catch {}
    tested.textContent = testing ? words.testing : minutes.provider === minutes.ollama_address ? words.test_sends_here : fill(words.test_sends, host);
    button.dataset.locked = String(locked);
    button.addEventListener("click", async () => {
      testing = true;
      waiting(button, true);
      tested.textContent = words.testing;
      let said;
      try {
        await window.__TAURI__.core.invoke("minutes_tested");
        said = words.test_answered;
      } catch (why) {
        said = why;
      }
      testing = false;
      // Said on the page as it is now: it may have been drawn again meanwhile.
      const now = document.getElementById("minutes-tested");
      if (now) now.textContent = said;
      const again = document.getElementById("minutes-test");
      if (again) {
        waiting(again, false);
        again.disabled = again.dataset.locked === "true";
      }
    });
    test.append(button, tested);
    box.append(test);
  }
  return box;
}

// A Whisper model's file named as people say it: `ggml-large-v3-turbo-q8_0.bin` is Whisper large
// v3 turbo. A file of another kind, or outside Kitokey's folder, keeps its own name.
// A provider's model named as people say it: `anthropic/claude-sonnet` is Claude Sonnet; its
// version and size stay as written (`mistral:7b` is Mistral 7b).
function modelSaid(model) {
  // What people say of a model: the last part of its id, without what only machines read — a
  // file's extension, a tag "latest", a date of release, a quantisation (q4_K_M, fp16, qat…).
  let name = model.split("/").filter(Boolean).pop() ?? model;
  name = name.replace(/\.(gguf|bin)$/i, "").replace(/:latest$/i, "");
  name = name.replace(/[-_:.](?:i?q\d(?:_[a-z0-9]+)*|b?fp?16|fp?32|int[48])$/i, "");
  name = name.replace(/[-_:](?:\d{8}|\d{4}-\d{2}-\d{2}|\d{2}-\d{4})$/, "");
  const machine = /^(?:i?q\d(?:_[a-z0-9]+)*|b?fp?16|fp?32|int[48]|qat|k|m|s|l)$/i;
  const words = name.split(/[-_:\s]+/).filter(Boolean);
  while (words.length > 1 && machine.test(words.at(-1))) words.pop();
  if (words.length === 0 || (words.length === 1 && machine.test(words[0]))) return model;
  const said = [];
  for (const word of words) {
    // A version written with dashes — claude-sonnet-4-5 — is said with its point.
    if (/^\d{1,2}$/.test(word) && /^\d{1,2}(\.\d{1,2})*$/.test(said.at(-1) ?? "")) said[said.length - 1] += `.${word}`;
    else said.push(word.toLowerCase() === "gpt" ? "GPT" : /^[a-z]/.test(word) ? word[0].toUpperCase() + word.slice(1) : word);
  }
  return said.join(" ");
}

function spoken(name) {
  const file = name.match(/^ggml-(.+?)(?:[-.](?:q\d+_\d+|f16|f32))?(?:\.bin)?$/i);
  return file ? `Whisper ${file[1].replace(/[-_]/g, " ")}` : name;
}

function showSettings(shown, minutes, words) {
  const page = document.getElementById("settings");
  const locked = shown.unreadable !== null;
  // Saved or not, the page is drawn again from what the shell holds; a failure stays said.
  const save = async (change) => {
    let failed = null;
    try {
      await window.__TAURI__.core.invoke("settings_saved", { change });
    } catch (why) {
      failed = fill(words.not_saved, why);
    }
    notSaved = failed;
    await refresh();
    savedSaid(failed === null);
  };
  const parts = [];
  if (locked) {
    const unreadable = element("p", "card note warn", shown.unreadable);
    unreadable.id = "settings-unreadable";
    parts.push(unreadable);
  }
  const failed = element("p", "card note warn");
  failed.id = "not-saved";
  // What the last save could not do, said on every drawing of the page until the next save (#655).
  failed.textContent = notSaved ?? "";
  failed.hidden = notSaved === null;
  parts.push(failed);
  // What is left of the trial, on a trial licence alone (D83): said first, nothing to change.
  if (shown.trial !== null) {
    const trial = element("section", "card group state");
    trial.id = "trial-left";
    const trialSaid = element("div", "said");
    trialSaid.append(element("span", "name", words.trial), element("span", "says", shown.trial));
    trial.append(trialSaid);
    parts.push(trial);
  }

  const after = element("section", "card group");
  const afterTitle = element("h2", "", words.after_title);
  afterTitle.id = "after-title";
  // The radios alone in the group named by the heading: the folder and the sound kept follow them.
  const made = element("div", "choices");
  made.setAttribute("role", "radiogroup");
  made.setAttribute("aria-labelledby", "after-title");
  after.append(afterTitle, made);
  for (const [value, said] of [["nothing", words.after_nothing], ["transcript", words.after_transcript], ["minutes", words.after_minutes]]) {
    const one = choice("after", value, said, shown.after === value, locked);
    one.querySelector("input").addEventListener("change", () => save({ after: value }));
    made.append(one);
  }
  parts.push(after);

  // Where a summary can be sent (D110, PDR-0007), a part of « Après un enregistrement » — Settings keep
  // their three groups (D95): each destination by its name, its kind, and its address as its host and
  // the end of its path — never whole; one added is judged before it is kept.
  const sending = element("div", "sending");
  {
    sending.append(element("h3", "", words.destinations_title), element("p", "says", words.destinations_said));
    const kinds = { webhook: 0, discord: 1, slack: 2 };
    const kept = element("ul", "lines");
    kept.id = "destinations";
    const none = element("p", "says", shown.destinations.length ? "" : words.destinations_none);
    none.id = "destinations-said";
    shown.destinations.forEach((destination, index) => {
      const row = element("li", "row");
      const rowSaid = element("span", "said");
      rowSaid.append(element("span", "name", destination.name), element("span", "says", `${words.destination_kinds[kinds[destination.kind]]} · ${destination.shown}`));
      const forget = element("button", "open", words.destination_forget);
      forget.type = "button";
      forget.id = `destination-forget-${index}`;
      forget.disabled = locked;
      forget.addEventListener("click", () =>
        window.__TAURI__.core.invoke("destination_forgotten", { name: destination.name }).then(refresh, (why) => {
          none.textContent = fill(words.not_saved, why);
        }),
      );
      row.append(kindMark(destination.kind), rowSaid, forget);
      kept.append(row);
    });
    // The add form apart from the list, its fields the Settings' own (guidedField, guidedList); the
    // address on a whole line, long enough to read back what was pasted.
    const adding = element("div", "fields adding");
    adding.id = "destination-adding";
    // Its kind a tile each (#1238), as a desktop offers a few kinds: its mark or the page's icon, its name; a radio group
    // the arrows move in, its first chosen at first, as the list's first was.
    // On a whole line, as the address: the three tiles side by side.
    const kindField = element("div", "field kinds whole");
    const kindName = element("span", "name", words.destination_fields[0]);
    kindName.id = "destination-kind-name";
    const tiles = element("div", "tiles");
    tiles.setAttribute("role", "radiogroup");
    tiles.setAttribute("aria-labelledby", "destination-kind-name");
    for (const [value, index] of Object.entries(kinds)) {
      const tile = element("label", "tile");
      const input = element("input");
      input.type = "radio";
      input.name = "destination-kind";
      input.value = value;
      input.id = `destination-kind-${value}`;
      input.checked = index === 0;
      input.disabled = locked;
      tile.append(input, kindMark(value), element("span", "", words.destination_kinds[index]));
      tiles.append(tile);
    }
    kindField.append(kindName, tiles);
    const kind = { get value() { return tiles.querySelector("input:checked")?.value ?? "webhook"; }, focus: () => tiles.querySelector("input:checked")?.focus() };
    const [namedField, named] = guidedField("destination-name", words.destination_fields[1], "text", "", locked);
    named.maxLength = 40;
    named.autocomplete = "off";
    const [addressField, address] = guidedField("destination-address", words.destination_fields[2], "url", "", locked);
    addressField.classList.add("whole");
    address.autocomplete = "off";
    address.spellcheck = false;
    address.setAttribute("aria-describedby", "destination-address-said");
    const addressSaid = element("p", "says", words.destination_address_said);
    addressSaid.id = "destination-address-said";
    const add = element("button", "open", words.destination_add);
    add.type = "button";
    add.id = "destination-add";
    add.disabled = locked;
    const result = element("p", "says");
    result.id = "destination-said";
    result.setAttribute("role", "status");
    add.addEventListener("click", () => {
      add.disabled = true;
      window.__TAURI__.core
        .invoke("destination_added", { kind: kind.value, name: named.value, address: address.value })
        .then(
          () => {
            // Kept: the address leaves the page at once; the list, drawn again, names it.
            address.value = "";
            return refresh().then(() => {
              const said = document.getElementById("destination-said");
              if (said) said.textContent = fill(words.destination_added, named.value.trim());
            });
          },
          (why) => {
            result.textContent = fill(words.destination_not_added, why);
            add.disabled = locked;
          },
        );
    });
    adding.append(
      kindField,
      namedField,
      addressField,
      addressSaid,
      add,
      result,
    );
    // Folded at first: a disclosure (ARIA APG), as « Avancé »; unfolded by a click, the focus goes to
    // its first field. Escape does not fold it: a half-filled form is not lost by a key.
    const opening = element("button", "disclose-advanced");
    opening.type = "button";
    opening.id = "destination-open";
    opening.setAttribute("aria-controls", "destination-adding");
    opening.append(icon("chevron-right"), element("span", "", words.destination_open));
    const unfoldAdding = (now) => {
      destinationOpen = now;
      opening.setAttribute("aria-expanded", String(now));
      opening.firstChild.replaceWith(icon(now ? "chevron-down" : "chevron-right"));
      adding.hidden = !now;
    };
    opening.addEventListener("click", () => {
      unfoldAdding(!destinationOpen);
      if (destinationOpen) kind.focus();
    });
    unfoldAdding(destinationOpen);
    sending.append(kept, none, opening, adding);
  }

  const models = element("section", "card group");
  models.append(element("h2", "", words.models_title), element("p", "says", words.models_intro));
  // The first run's assistant again, the current setting chosen (D58).
  const redo = element("div", "row");
  const redoSaid = element("span", "said");
  redoSaid.append(element("span", "name", words.welcome_again[0]), element("span", "says", words.welcome_again[1]));
  const redoButton = element("button", "open", words.welcome_again[0]);
  redoButton.type = "button";
  redoButton.id = "welcome-again";
  redoButton.disabled = locked;
  redoButton.addEventListener("click", () => (location.hash = "#/welcome/again"));
  redo.append(redoSaid, redoButton);
  const step = element("div", "step");
  // Who hears (D95): on this computer — a Whisper model, or Apple's SpeechAnalyzer — or a service
  // named, its key tested at once, then one of its models; an address of one's own under « Avancé ».
  const title = element("span", "name", words.transcription);
  title.id = "transcription-title";
  // The services that hear, offered by name as the minutes' are (D63); any other, an address of the user's.
  const presetOf = (text) => shown.hearing.findIndex((hearing) => hearing.address === text);
  const preset = shown.engine === "service" ? presetOf(shown.transcription_provider) : -1;
  const own = shown.engine === "service" && preset < 0;
  const [way, hereRadio, awayRadio] = guidedWhere("transcription", words, guided.transcription.chosen.way ?? (shown.engine === "service" ? "online" : "here"), locked);
  const row = element("div", "row");
  const label = element("label", "name", words.model_name);
  label.htmlFor = "model";
  const pick = element("select");
  pick.id = "model";
  pick.disabled = locked;
  // The model chosen, not here yet: said so in the choice, never shown as another (D37).
  if (!shown.models.some((model) => model.path === shown.model)) {
    const missing = element("option", "", fill(words.model_chosen_missing, spoken(shown.model.split("/").pop())));
    missing.disabled = true;
    missing.selected = true;
    pick.append(missing);
    // Cut by the list on a narrow window (#868): whole as its tooltip too, and said under it by `#model-state`.
    pick.title = missing.textContent;
  }
  for (const model of shown.models) {
    const option = element("option", "", `${spoken(model.name)} — ${fill(words.installed, size(model.bytes, words))}`);
    option.value = model.path;
    option.selected = model.path === shown.model;
    pick.append(option);
  }
  // Apple's SpeechAnalyzer among who hears, where this Mac offers it (D42): asked away from the
  // page; its model fetched by macOS on the click alone, then it is who hears.
  const speech = element("div", "speech");
  speech.id = "speech";
  const speechSaid = element("p", "says");
  speechSaid.setAttribute("role", "status");
  const fetchApple = element("button", "open", words.speech_fetch);
  fetchApple.type = "button";
  fetchApple.id = "speech-fetch";
  fetchApple.hidden = true;
  fetchApple.addEventListener("click", async () => {
    waiting(fetchApple, true);
    speechSaid.textContent = words.speech_fetching;
    const fetched = await window.__TAURI__.core.invoke("speech_installed").then(() => true, () => false);
    // Chosen only while it is still the user's choice: Settings may have been drawn again meanwhile.
    if (fetched && document.getElementById("model")?.value === "speechanalyzer") return save({ engine: "speechanalyzer" });
    if (fetched) return;
    waiting(fetchApple, false);
    speechSaid.textContent = words.speech_failed;
    // Not fetched: who hears is still the Whisper model, and the choice says so.
    if (shown.models.some((model) => model.path === shown.model)) pick.value = shown.model;
  });
  speech.append(fetchApple, speechSaid);
  let appleHere = false;
  window.__TAURI__.core.invoke("speech_shown").then((status) => {
    if (!status?.offered) return;
    appleHere = status.installed;
    const option = element("option", "", status.installed ? words.speech_option : `${words.speech_option} — ${words.speech_missing}`);
    option.value = "speechanalyzer";
    option.selected = shown.engine === "speechanalyzer";
    pick.append(option);
  }).catch(() => {});
  // On this computer, chosen at once; while a service hears, with « Enregistrer » — going through
  // the list, with the keyboard's arrows too, never changes who hears (D25).
  const hereSave = guidedButton("transcription-here-save", words.save, false, locked);
  hereSave.hidden = shown.engine !== "service";
  const chooseHere = () => {
    if (pick.value !== "speechanalyzer") return save({ model: pick.value });
    if (appleHere) return save({ engine: "speechanalyzer" });
    // Not chosen until its model is on the Mac: the gesture comes first, never the download.
    fetchApple.hidden = false;
    fetchApple.focus();
  };
  pick.addEventListener("change", () => {
    if (shown.engine !== "service") chooseHere();
  });
  hereSave.addEventListener("click", chooseHere);
  // A service (D43): nothing sent of a meeting before the user agreed to its host; a host on this
  // computer needs no agreement.
  const hostOf = (text) => { try { return new URL(text).host; } catch { return ""; } };
  // This computer's own host needs no agreement: nothing of the meeting leaves it.
  // The whole host, its port aside: `localhost.example` or `127.example` is elsewhere. The shell
  // judges again (`providers::local`); this only spares the agreement where none is needed.
  const isHere = (host) => /^(localhost|127(\.\d{1,3}){3}|\[::1\])(:\d+)?$/i.test(host);
  const agreement = (id) => {
    const wrap = element("label", "row");
    const box = element("input");
    box.type = "checkbox";
    box.id = id;
    // A box ticked is no text being typed: the page still comes back when the window does.
    box.defaultValue = box.value;
    box.disabled = locked;
    const said = element("span", "says");
    wrap.append(box, said);
    // Shown where the form asks for it, and only for a host elsewhere: none is owed on this computer (#859).
    let wanted = true;
    let here = false;
    const paint = () => { wrap.hidden = !wanted || here; };
    // Ticked only for the host already agreed to.
    const hostIs = (host) => {
      said.textContent = fill(words.service_agree, host || "—");
      box.checked = !!shown.transcription_agreed && shown.transcription_agreed === host;
      here = isHere(host);
      paint();
    };
    const shownIs = (now) => { wanted = now; paint(); };
    return [wrap, box, hostIs, shownIs];
  };
  const [serviceWrap, servicePick] = guidedList("transcription-service", words.service_name, locked);
  shown.hearing.forEach((hearing, index) => guidedOption(servicePick, String(index), hearing.name));
  servicePick.value = guided.transcription.chosen.service ?? String(Math.max(preset, 0));
  serviceMarked(servicePick, () => shown.hearing[Number(servicePick.value)]?.name);
  const hearing = () => shown.hearing[Number(servicePick.value)];
  const [keyWrap, serviceKey] = guidedField("transcription-key", words.service_key, "password", "", locked);
  serviceKey.autocomplete = "off";
  const keyTest = guidedButton("transcription-key-test", words.key_test, false, locked);
  const keySaid = element("p", "says");
  keySaid.id = "transcription-key-said";
  keySaid.setAttribute("role", "status");
  const [modelsWrap, serviceModels] = guidedList("transcription-models", words.service_model, locked);
  chosenKeeps("transcription", serviceModels, () => servicePick.value);
  const [agreeWrap, agreeBox, agreeHost, agreeShown] = agreement("transcription-agreed");
  const saveService = guidedButton("transcription-save", words.service_save, true, locked);
  // One « Enregistrer » and one agreement at a time (#860): the service's, « Avancé » folded; unfolded,
  // its own, which saves what the page shows.
  let listedShown = false;
  const paintSaves = () => {
    const open = guided.transcription.advanced;
    saveService.hidden = !listedShown || open;
    agreeShown(listedShown && !open);
  };
  // The models listed with its key, the one saved chosen.
  const shownListed = (models) => {
    serviceModels.replaceChildren();
    for (const id of models) guidedOption(serviceModels, id, modelSaid(id));
    chosenKept("transcription", serviceModels, servicePick.value, shown.transcription_model);
    modelsWrap.hidden = models.length === 0;
    listedShown = models.length > 0;
    paintSaves();
  };
  // This service's last answer, said again; else the model saved with it — the others come with its key.
  const shownSaved = () => {
    const saved = preset >= 0 && Number(servicePick.value) === preset && shown.transcription_model;
    agreeHost(hostOf(hearing().address));
    const answer = guided.transcription.answers.get(servicePick.value);
    keySaid.textContent = answer?.said ?? "";
    if (answer?.models) return shownListed(answer.models);
    serviceModels.replaceChildren();
    if (saved) guidedOption(serviceModels, shown.transcription_model, modelSaid(shown.transcription_model));
    modelsWrap.hidden = !saved;
    listedShown = !!saved;
    paintSaves();
  };
  shownSaved();
  // « Avancé »: an address of one's own, OpenAI's way of asking, its model named by hand.
  const [addressWrap, address] = guidedField("transcription-provider", words.service_address, "url", own ? shown.transcription_provider : "", locked);
  const [modelWrap, serviceModel] = guidedField("transcription-model", words.service_model, "text", own ? shown.transcription_model : "", locked);
  const [ownAgreeWrap, ownAgreeBox, ownAgreeHost] = agreement("transcription-own-agreed");
  // Its address left empty: the service chosen above, with a model it does not list — a new release,
  // an id of its own — named by hand; the field shows that address as it stands in.
  const ownAddress = () => address.value.trim() || hearing().address;
  const standsIn = () => {
    address.placeholder = hearing().address;
    ownAgreeHost(hostOf(ownAddress()));
  };
  standsIn();
  // « Avancé » folded, its address is not what the key is typed for: the service's is (#963).
  const keyNow = () => keyFor((guided.transcription.advanced ?? own) ? address.value : "", hearing().address, servicePick.value);
  const keyed = guidedKeyTested("transcription", words, locked, { servicePick, key: serviceKey, keyTest, keySaid, modelsPick: serviceModels },
    (index, typed) => window.__TAURI__.core.invoke("transcription_models", { index: Number(index), key: typed }), shownSaved, keyNow);
  address.addEventListener("input", () => {
    ownAgreeHost(hostOf(ownAddress()));
    ownAgreeBox.checked = false;
    keyed.shown();
  });
  const ownSave = guidedButton("transcription-own-save", words.service_save, false, locked);
  const [hearToggle, hearAdvanced] = guidedAdvanced("transcription", words, own, [addressWrap, modelWrap, ownAgreeWrap, ownSave]);
  hearToggle.addEventListener("click", () => {
    paintSaves();
    keyed.shown();
  });
  paintSaves();
  // What « Avancé »'s « Enregistrer » saves: its model typed, or else — its address left empty, the
  // service chosen above — the model chosen in that service's list (#860).
  const ownModel = () => serviceModel.value.trim() || (address.value.trim() ? "" : serviceModels.value);
  const serviceSaid = element("p", "says");
  serviceSaid.id = "transcription-said";
  serviceSaid.setAttribute("role", "status");
  const saveHearing = async (provider, model, key, keyId, box, addressField, modelField) => {
    const { invoke } = window.__TAURI__.core;
    if (!/^https?:$/.test((() => { try { return new URL(provider.trim()).protocol; } catch { return ""; } })()) || !hostOf(provider.trim())) {
      serviceSaid.textContent = words.service_address_bad;
      return addressField?.focus();
    }
    if (!model.trim()) {
      serviceSaid.textContent = words.service_model_named;
      return modelField.focus();
    }
    if (!box.checked && !isHere(hostOf(provider))) {
      serviceSaid.textContent = fill(words.service_agree_first, hostOf(provider) || "—");
      return box.focus();
    }
    const saved = await invoke("transcription_saved", {
      provider, model, key: key || null, agreed: box.checked,
    }).then((notKept) => {
      if (notKept !== null) return fill(words.key_not_kept, notKept);
      // Saved: the key no longer kept in the page (#964).
      keyed.forget(keyId);
      // This computer's: said so, never as an address and a port (#775).
      return isHere(hostOf(provider)) ? words.service_saved_here : fill(words.service_saved, hostOf(provider));
    }, (why) => fill(words.not_saved, why));
    // A Tester still asking stays: its answer comes after this, the button its own.
    serviceAnswer ??= { said: "", asking: false };
    serviceAnswer.said = saved;
    serviceSaid.textContent = saved;
  };
  const serviceKeyId = () => `service:${servicePick.value}`;
  saveService.addEventListener("click", () => saveHearing(hearing().address, serviceModels.value, keyed.typed(serviceKeyId()), serviceKeyId(), agreeBox, null, serviceModels));
  ownSave.addEventListener("click", () => saveHearing(ownAddress(), ownModel(), keyed.typed(keyNow()), keyNow(), ownAgreeBox, address, serviceModel));
  const testService = element("button", "open", words.service_test);
  testService.type = "button";
  testService.id = "transcription-test";
  testService.disabled = locked;
  // Drawn anew while it asks: waiting still, never `disabled` (#886).
  waiting(testService, serviceAnswer?.asking === true);
  serviceSaid.textContent = serviceAnswer?.said ?? "";
  testService.addEventListener("click", async () => {
    const mine = (serviceAnswer = { said: "", asking: true });
    waiting(testService, true);
    serviceSaid.textContent = "";
    // The service asked, as saved: an answer for one no longer saved is no answer for what is.
    const asking = async () => {
      const { transcription_provider, transcription_model } = await window.__TAURI__.core.invoke("settings_shown");
      return JSON.stringify([transcription_provider, transcription_model]);
    };
    try {
      const asked = await asking();
      let answer;
      try {
        await window.__TAURI__.core.invoke("transcription_tested");
        answer = words.service_tested;
      } catch (why) {
        answer = why;
      }
      if ((await asking()) === asked) mine.said = answer;
    } catch (why) {
      mine.said = why;
    }
    mine.asking = false;
    // Said on the page as it is now; another address or another service: nothing to say any more.
    if (serviceAnswer !== mine) return;
    const said = document.getElementById("transcription-said");
    if (said) said.textContent = mine.said;
    const button = document.getElementById("transcription-test");
    if (button) {
      waiting(button, false);
      button.disabled = locked;
    }
  });
  // Another service chosen: what was said of the last goes at once, its button given back.
  servicePick.addEventListener("change", () => {
    guided.transcription.chosen.service = servicePick.value;
    serviceAnswer = null;
    serviceSaid.textContent = "";
    waiting(testService, false);
    shownSaved();
    standsIn();
  });
  const service = element("div", "fields");
  service.id = "transcription-away";
  service.append(serviceWrap, keyWrap, keyTest, keySaid, modelsWrap, agreeWrap, saveService, hearToggle, hearAdvanced, testService, serviceSaid);
  row.append(label, pick, hereSave);
  const here = element("p", "status here", words.on_this_computer);
  here.append(element("code", "tag", words.builtin));
  // The language meetings are spoken in, unless one says its own (D65): detected by default.
  const spokenRow = element("div", "row");
  const spokenLabel = element("label", "name", words.meetings_language);
  spokenLabel.htmlFor = "meetings-language";
  const spokenPick = element("select");
  spokenPick.id = "meetings-language";
  spokenPick.disabled = locked;
  const spokenAdd = (value, text) => {
    const option = element("option", "", text);
    option.value = value;
    spokenPick.append(option);
  };
  // SpeechAnalyzer hears French and English and detects none: those two, or the system's (D64).
  const apple = shown.engine === "speechanalyzer";
  const heard = shown.languages.filter(([code]) => !apple || code === "fr" || code === "en");
  spokenAdd("", apple ? words.language_as_system : words.language_detected);
  for (const [code, said] of heard) spokenAdd(code, said);
  // One it does not hear, chosen while Whisper heard: kept, said as heard (D65).
  const spokenChosen = shown.meeting_language;
  const spokenName = shown.languages.find(([code]) => code === spokenChosen)?.[1] ?? spokenChosen;
  if (spokenChosen && !heard.some(([code]) => code === spokenChosen)) {
    spokenAdd(spokenChosen, apple ? fill(words.language_heard_as, spokenName, words.language_as_system.toLowerCase()) : spokenName);
  }
  spokenPick.value = shown.meeting_language ?? "";
  spokenPick.addEventListener("change", () => save({ meeting_language: spokenPick.value }));
  spokenRow.append(spokenLabel, spokenPick);
  // The transcription model's state, under its choice: there and ready; else said, and offered, by
  // paintModel — a line of this step, no card of its own (D95, #782).
  const chosen = shown.models.find((model) => model.path === shown.model);
  const state = element("div", "row state");
  const stateSaid = element("div", "said");
  stateSaid.append(
    element("span", "name", words.model_state),
    element("span", "says", chosen ? fill(words.model_here, size(chosen.bytes, words)) : ""),
  );
  state.append(stateSaid);
  if (chosen) state.append(readySaid(words.model_ready));
  else state.dataset.missing = "true";
  state.id = "model-state";
  paintModel(state);
  const hereBox = element("div");
  hereBox.id = "transcription-on-computer";
  hereBox.append(row, state, speech, here);
  const drawWhere = () => {
    hereBox.hidden = !hereRadio.checked;
    service.hidden = !awayRadio.checked;
  };
  for (const radio of [hereRadio, awayRadio]) {
    radio.addEventListener("change", () => {
      guided.transcription.chosen.way = radio.value;
      drawWhere();
    });
  }
  drawWhere();
  step.append(title, way, hereBox, service, spokenRow);
  // The short list of models to download, each said by what it is good for (D37).
  const offered = element("ul", "offered");
  offered.id = "offered";
  offered.setAttribute("aria-label", words.offered_title);
  for (const model of shown.offered) {
    const item = element("li", "row");
    item.dataset.offered = model.file;
    item.dataset.here = String(model.here);
    const said_ = element("div", "said");
    // Its host shown before anything is fetched from it (security rule 1, D40).
    said_.append(element("span", "name", `${spoken(model.file)} · ${size(model.bytes, words)}, ${model.host}`));
    if (model.good) said_.append(element("span", "says", model.good));
    item.append(said_, element("span", "state"));
    offered.append(item);
  }
  // The minutes' recommended model, fetched by Ollama — never Ollama itself (D37).
  const pulled = element("li", "row");
  pulled.dataset.offered = minutes.recommended;
  pulled.dataset.here = String(minutes.ollama.includes(minutes.recommended));
  pulled.dataset.by = minutes.ollama_answers ? "ollama" : "absent";
  const pulledSaid = element("div", "said");
  pulledSaid.append(element("span", "name", `${minutes.recommended} · ${words.offered_ollama_size}`), element("span", "says", words.offered_ollama_good));
  pulled.append(pulledSaid, element("span", "state"));
  offered.append(pulled);
  // The list read again from where it is kept, only on this click, its host shown (D40): read,
  // the page is drawn with it; refused, the list above stays, and it is said — signed by the
  // decider's key alone, built in since D56.
  const refreshed = element("p", "says");
  refreshed.id = "offered-refreshed";
  refreshed.setAttribute("role", "status");
  const refresh_ = element("button", "open", fill(words.offered_refresh, shown.registry_host));
  refresh_.type = "button";
  refresh_.id = "offered-refresh";
  refresh_.addEventListener("click", async () => {
    waiting(refresh_, true);
    refreshed.textContent = "";
    const read = await window.__TAURI__.core.invoke("registry_refresh").then(() => true, () => false);
    if (read) return refresh();
    waiting(refresh_, false);
    refreshed.textContent = words.offered_refresh_failed;
  });
  // Folded at first (#782): a disclosure (ARIA APG), as « Avancé », left as the user leaves it when
  // the page is drawn again — a download running in it goes on either way.
  const offeredToggle = element("button", "disclose-advanced");
  offeredToggle.type = "button";
  offeredToggle.id = "offered-toggle";
  offeredToggle.setAttribute("aria-controls", "offered");
  offeredToggle.append(icon("chevron-right"), element("span", "", words.offered_title));
  const offeredPanel = [offered, refresh_, refreshed];
  const unfoldOffered = (now) => {
    offeredOpen = now;
    offeredToggle.setAttribute("aria-expanded", String(now));
    offeredToggle.firstChild.replaceWith(icon(now ? "chevron-down" : "chevron-right"));
    for (const part of offeredPanel) part.hidden = !now;
  };
  offeredToggle.addEventListener("click", () => unfoldOffered(!offeredOpen));
  unfoldOffered(offeredOpen);
  step.append(offeredToggle, ...offeredPanel);
  paintOffered(offered);
  models.append(step, minutesRow(minutes, words, locked), element("p", "says", words.models_note));
  parts.push(models);

  const more = element("section", "card group");
  more.append(element("h2", "", words.general_title));
  const remind = element("label", "row");
  const box = element("input");
  box.type = "checkbox";
  box.id = "reminder";
  box.setAttribute("role", "switch");
  box.checked = shown.reminder;
  box.disabled = locked;
  box.addEventListener("change", () => save({ reminder: box.checked }));
  const remindSaid = element("span", "said");
  remindSaid.append(element("span", "name", words.reminder[0]), element("span", "says", words.reminder[1]));
  remind.append(remindSaid, box);
  // When another application takes the microphone (D30), as board 4 has it: offer to record,
  // or do nothing.
  const offer = element("div", "row");
  const offerSaid = element("label", "said");
  offerSaid.htmlFor = "offer";
  offerSaid.append(element("span", "name", words.offer[0]), element("span", "says", words.offer[1]));
  const offerPick = element("select");
  offerPick.id = "offer";
  offerPick.disabled = locked;
  for (const [value, text] of [["offer", words.offer_choices[0]], ["nothing", words.offer_choices[1]]]) {
    const option = element("option", "", text);
    option.value = value;
    offerPick.append(option);
  }
  offerPick.value = shown.offer ? "offer" : "nothing";
  offerPick.addEventListener("change", () => save({ offer: offerPick.value === "offer" }));
  offer.append(offerSaid, offerPick);
  // Started with the user's session (D32): the desktop's own entry, turned on or off here.
  const start = element("label", "row");
  const startBox = element("input");
  startBox.type = "checkbox";
  startBox.id = "autostart";
  startBox.setAttribute("role", "switch");
  startBox.checked = shown.autostart;
  // On Windows, held by the user or a policy (D84): shown as Windows holds it, never moved here.
  const held = { user: words.autostart_held[0], policy: words.autostart_held[1] }[shown.autostart_held];
  startBox.disabled = locked || held !== undefined;
  startBox.addEventListener("change", () => save({ autostart: startBox.checked }));
  const startSaid = element("span", "said");
  startSaid.append(element("span", "name", words.autostart[0]), element("span", "says", held ?? words.autostart[1]));
  start.append(startSaid, startBox);
  const keep = element("div", "row");
  const keepSaid = element("label", "said");
  keepSaid.htmlFor = "keep";
  keepSaid.append(element("span", "name", words.keep_audio[0]), element("span", "says", words.keep_audio[1]));
  const keepPick = element("select");
  keepPick.id = "keep";
  keepPick.disabled = locked;
  const days = new Set([7, 30, 90, 365]);
  if (shown.keep_audio !== "always") days.add(shown.keep_audio.days);
  const options = [
    ["always", words.keep_always],
    ...[...days].sort((a, b) => a - b).map((n) => [String(n), fill(words.keep_days, n)]),
    ["other", words.keep_other],
  ];
  for (const [value, text] of options) {
    const option = element("option", "", text);
    option.value = value;
    option.selected = shown.keep_audio === "always" ? value === "always" : value === String(shown.keep_audio.days);
    keepPick.append(option);
  }
  // Any number of days the file allows, typed when the list has not the one wanted.
  const other = element("input");
  other.id = "keep-other";
  other.type = "number";
  other.min = "1";
  other.max = "36500";
  other.hidden = true;
  other.disabled = locked;
  other.setAttribute("aria-label", words.keep_other);
  other.addEventListener("change", () => {
    const days = Number(other.value);
    if (other.value !== "" && Number.isInteger(days) && days >= 1 && days <= 36500) return save({ keep_audio: { days } });
    const said = document.getElementById("not-saved");
    said.textContent = fill(words.not_saved, fill(words.keep_not_days, other.value || "—"));
    said.hidden = false;
  });
  other.addEventListener("keydown", (event) => {
    if (event.key === "Enter") other.blur();
  });
  keepPick.addEventListener("change", () => {
    if (keepPick.value === "other") {
      other.hidden = false;
      other.focus();
      return;
    }
    save({ keep_audio: keepPick.value === "always" ? "always" : { days: Number(keepPick.value) } });
  });
  const keepPicks = element("div", "picks");
  keepPicks.append(keepPick, other);
  keep.append(keepSaid, keepPicks);
  // Where the sessions go: typed, or chosen in the system's own dialog.
  const where = element("div", "row folder");
  const whereSaid = element("label", "said");
  whereSaid.htmlFor = "sessions";
  whereSaid.append(element("span", "name", words.sessions_folder[0]), element("span", "says", words.sessions_folder[1]));
  const path = element("input");
  path.id = "sessions";
  path.type = "text";
  path.value = shown.sessions_said;
  path.disabled = locked;
  path.spellcheck = false;
  path.hidden = true;
  // Shown as the board says it — Dossier personnel / Kitokey —; a click makes it a path to type.
  const said = shown.sessions_said.replace(/^~(?=\/|$)/, words.home).split("/").filter(Boolean).join(" / ");
  const folderShown = element("button", "path", shown.sessions_said.startsWith("/") ? `/ ${said}` : said);
  folderShown.type = "button";
  whereSaid.id = "sessions-label";
  folderShown.setAttribute("aria-labelledby", "sessions-label sessions-shown");
  folderShown.id = "sessions-shown";
  folderShown.disabled = locked;
  folderShown.addEventListener("click", () => {
    folderShown.hidden = true;
    path.hidden = false;
    path.focus();
    path.select();
  });
  path.addEventListener("change", () => save({ sessions: path.value.trim() }));
  // Enter says the folder is typed: the field lets go, which is its change.
  path.addEventListener("keydown", (event) => {
    if (event.key === "Enter") path.blur();
  });
  const choose = element("button", "open", words.choose);
  choose.type = "button";
  choose.id = "choose-sessions";
  choose.disabled = locked;
  choose.addEventListener("click", async () => {
    const chosen = await window.__TAURI__.core.invoke("choose_folder");
    if (chosen !== null) save({ sessions: chosen });
  });
  where.append(whereSaid, folderShown, path, choose);
  // Every new summary written to a folder too (D109), typed or chosen as the meetings' folder is;
  // none, nowhere else. « Ne plus exporter » forgets it.
  const exporting = element("div", "row folder");
  const exportSaid = element("label", "said");
  exportSaid.htmlFor = "export-folder";
  exportSaid.id = "export-label";
  const exportHelp = element("span", "says", words.export_folder[1]);
  exportHelp.id = "export-help";
  exportSaid.append(element("span", "name", words.export_folder[0]), exportHelp);
  const exportPath = element("input");
  exportPath.id = "export-folder";
  exportPath.type = "text";
  exportPath.value = shown.export_said ?? "";
  exportPath.disabled = locked;
  exportPath.spellcheck = false;
  exportPath.hidden = true;
  const exportAs = (shown.export_said ?? "").replace(/^~(?=\/|$)/, words.home).split("/").filter(Boolean).join(" / ");
  const exportShown = element(
    "button",
    "path",
    shown.export_said === null ? words.export_none : shown.export_said.startsWith("/") ? `/ ${exportAs}` : exportAs,
  );
  exportShown.type = "button";
  exportShown.id = "export-shown";
  exportShown.setAttribute("aria-labelledby", "export-label export-shown");
  exportShown.disabled = locked;
  exportShown.addEventListener("click", () => {
    exportShown.hidden = true;
    exportPath.hidden = false;
    exportPath.focus();
    exportPath.select();
  });
  exportPath.addEventListener("change", () => save({ export: exportPath.value.trim() }));
  exportPath.addEventListener("keydown", (event) => {
    if (event.key === "Enter") exportPath.blur();
  });
  const exportChoose = element("button", "open", words.choose);
  exportChoose.type = "button";
  exportChoose.id = "choose-export";
  exportChoose.disabled = locked;
  exportChoose.addEventListener("click", async () => {
    const chosen = await window.__TAURI__.core.invoke("choose_folder");
    if (chosen !== null) save({ export: chosen });
  });
  const exportStop = element("button", "open", words.export_stop);
  exportStop.type = "button";
  exportStop.id = "export-stop";
  exportStop.disabled = locked;
  exportStop.hidden = shown.export_said === null;
  exportStop.addEventListener("click", () => save({ export: "" }));
  exporting.append(exportSaid, exportShown, exportPath, exportChoose, exportStop);
  // The shortcut: the desktop's to grant and to change; the page says which keys, and where.
  const keys = element("div", "row");
  const keysSaid = element("span", "said");
  keysSaid.append(element("span", "name", words.shortcut[0]), element("span", "says", `${words.shortcut[1]} ${words.shortcut_where}`));
  const granted = element("kbd", "keys", shown.shortcut ?? words.shortcut_none);
  granted.id = "shortcut";
  keys.append(keysSaid, granted);
  if (shown.shortcut_settings) {
    const open = element("button", "open", words.shortcut_open);
    open.type = "button";
    open.id = "shortcut-settings";
    open.addEventListener("click", () =>
      window.__TAURI__.core.invoke("shortcut_settings").catch((why) => {
        keysSaid.lastChild.textContent = why;
      }),
    );
    keys.append(open);
  }
  // The language Kitokey speaks (D64): the system's by default, each other named in its own.
  const speaks = element("div", "row");
  const speaksSaid = element("label", "said");
  speaksSaid.htmlFor = "app-language";
  speaksSaid.append(element("span", "name", words.app_language[0]), element("span", "says", words.app_language[1]));
  const speaksPick = element("select");
  speaksPick.id = "app-language";
  speaksPick.disabled = locked;
  for (const [value, text] of [["", words.app_language_system], ...shown.languages.filter(([code]) => code === "fr" || code === "en")]) {
    const option = element("option", "", text);
    option.value = value;
    speaksPick.append(option);
  }
  speaksPick.value = shown.language ?? "";
  speaksPick.addEventListener("change", () => {
    // Drawn again in the language chosen: the choice keeps the focus.
    focusLanguage = true;
    save({ language: speaksPick.value });
  });
  speaks.append(speaksSaid, speaksPick);
  // The appearance (D98): the system's by default, or light or dark whatever the system says —
  // taken at once here, by the bar at its next look.
  const looks = element("div", "row");
  const looksSaid = element("label", "said");
  looksSaid.htmlFor = "appearance";
  looksSaid.append(element("span", "name", words.appearance[0]), element("span", "says", words.appearance[1]));
  const looksPick = element("select");
  looksPick.id = "appearance";
  looksPick.disabled = locked;
  ["system", "light", "dark"].forEach((value, at) => {
    const option = element("option", "", words.appearance_choices[at]);
    option.value = value;
    looksPick.append(option);
  });
  looksPick.value = shown.appearance;
  looksPick.addEventListener("change", () => {
    takeAppearance(looksPick.value);
    save({ appearance: looksPick.value });
  });
  looks.append(looksSaid, looksPick);
  // « Vérifier le son » (D88): the microphone and the meeting's sound, before a recording — the panel
  // the bar opens too.
  const check = element("div", "row");
  const checkSaid = element("span", "said");
  checkSaid.append(element("span", "name", words.check_sound), element("span", "says", words.check_sound_says));
  const checkButton = element("button", "open with-icon");
  checkButton.type = "button";
  checkButton.id = "check-sound-open";
  checkButton.append(icon("audio-lines"), element("span", "", words.check_sound));
  checkButton.addEventListener("click", () => checkSound(words, checkButton));
  check.append(checkSaid, checkButton);
  // Outside Windows' package, no startup task: nothing offered (D84).
  more.append(speaks, looks, remind, check, offer, ...(shown.autostart_offered ? [start] : []), keys, redo);
  // What becomes of a recording: where it goes and how long its sound stays, after what is made.
  after.append(where, exporting, keep, sending);

  // Last, for whoever looks into a failure: the developer mode (D29), folded at the bottom of
  // « Général » (D95) — a disclosure (ARIA APG), as the redo panel's « Avancé ».
  const toggle = element("button", "disclose-advanced");
  toggle.type = "button";
  toggle.id = "general-advanced";
  toggle.setAttribute("aria-controls", "general-advanced-panel");
  toggle.append(icon("chevron-right"), element("span", "", words.advanced_title));
  const advanced = element("div", "general-advanced");
  advanced.id = "general-advanced-panel";
  const unfold = (now) => {
    generalAdvanced = now;
    toggle.setAttribute("aria-expanded", String(now));
    toggle.firstChild.replaceWith(icon(now ? "chevron-down" : "chevron-right"));
    advanced.hidden = !now;
  };
  toggle.addEventListener("click", () => unfold(!generalAdvanced));
  unfold(generalAdvanced);
  const developer = element("label", "row");
  const developerBox = element("input");
  developerBox.type = "checkbox";
  developerBox.id = "developer";
  developerBox.setAttribute("role", "switch");
  developerBox.checked = shown.developer;
  developerBox.disabled = locked;
  developerBox.addEventListener("change", () => save({ developer: developerBox.checked }));
  const developerSaid = element("span", "said");
  developerSaid.append(element("span", "name", words.developer[0]), element("span", "says", words.developer[1]));
  developer.append(developerSaid, developerBox);
  advanced.append(developer);
  if (shown.developer) {
    // The logs' level, read at Kitokey's next start by the shell and its programs.
    const level = element("div", "row");
    const levelSaid = element("label", "said");
    levelSaid.htmlFor = "log-level";
    levelSaid.append(element("span", "name", words.log_level[0]), element("span", "says", words.log_level[1]));
    const levelPick = element("select");
    levelPick.id = "log-level";
    levelPick.disabled = locked;
    for (const [value, text] of [["info", words.log_info], ["debug", words.log_debug]]) {
      const option = element("option", "", text);
      option.value = value;
      option.selected = shown.log === value;
      levelPick.append(option);
    }
    levelPick.addEventListener("change", () => save({ log: levelPick.value }));
    level.append(levelSaid, levelPick);
    const logs = element("div", "row");
    const logsSaid = element("span", "said");
    logsSaid.setAttribute("role", "status");
    logsSaid.append(element("span", "name", words.logs[0]), element("span", "says", words.logs[1]));
    const openLogs = element("button", "open", words.open_logs);
    openLogs.type = "button";
    openLogs.id = "open-logs";
    openLogs.addEventListener("click", () =>
      window.__TAURI__.core.invoke("open_logs").catch(() => {
        logsSaid.lastChild.textContent = words.logs_not_opened;
      }),
    );
    logs.append(logsSaid, openLogs);
    advanced.append(level, logs);
  }
  // Its last line, for support: the version, as `--version` says it (#1089).
  const version = element("p", "says", `${words.version_said} ${shown.version}`);
  version.id = "app-version";
  more.append(toggle, advanced, version);
  parts.push(more);
  page.replaceChildren(...parts);
  if (focusLanguage) {
    focusLanguage = false;
    document.getElementById("app-language")?.focus();
  }
  // From the assistant's « Choisir mon service »: the minutes' choice, in view and focused.
  if (focusMinutes) {
    focusMinutes = false;
    document.getElementById("minutes-online")?.click();
    const minutesPick = document.getElementById("minutes-service");
    minutesPick?.scrollIntoView({ block: "center" });
    minutesPick?.focus();
  }
}

// Before a meeting first leaves for a host (D11's fifth board): what leaves, what stays, to
// whom; the user's word — once, or for this service —; or the minutes written on this computer.
// Nothing here is sent before a click.
function showAgreement(name, agreement, words, session) {
  const page = document.getElementById("agree");
  const card = element("section", "card agreement");
  card.append(element("p", "eyebrow", words.agree_eyebrow), element("h1", "", fill(words.agree_title, session ? session.title ?? heading(session, words) : name)), element("p", "says", words.agree_lead));
  const goes = element("ul", "goes");
  const item = (mark, [title, says], ...values) => {
    const line = element("li", mark);
    const said = element("div", "");
    said.append(element("strong", "", title), element("span", "says", fill(says, ...values)));
    const sign = element("span", "sign");
    // What stays, Confidentielle's lock: one meaning everywhere, it stays on this computer (#1114, the decider).
    sign.append(icon(mark === "leaves" ? "arrow-right" : "lock"));
    line.append(sign, said);
    return line;
  };
  goes.append(item("leaves", words.agree_leaves, Math.max(1, Math.round(agreement.seconds / 60)), agreement.words.toLocaleString(document.documentElement.lang)));
  if (agreement.context_words !== null) goes.append(item("leaves", words.agree_context, agreement.context_words.toLocaleString(document.documentElement.lang)));
  goes.append(item("stays", words.agree_stays));
  const to = element("dl", "recipient");
  to.append(element("dt", "", words.agree_to), element("dd", "", agreement.host), element("dt", "", words.agree_model), element("dd", "", agreement.model));
  const remember = element("label", "remember");
  const tick = element("input");
  tick.type = "checkbox";
  tick.id = "agree-remember";
  remember.append(tick, element("span", "", words.agree_remember));
  const failed = element("p", "note warn");
  failed.id = "not-saved";
  failed.hidden = true;
  const act = async (command, args) => {
    try {
      await window.__TAURI__.core.invoke(command, { name, ...args });
      location.hash = `#/meeting/${encodeURIComponent(name)}`;
    } catch (why) {
      failed.textContent = fill(words.not_saved, why);
      failed.hidden = false;
    }
  };
  const choices = element("div", "choices");
  const send = element("button", "primary", words.agree_send);
  send.type = "button";
  send.id = "agree-send";
  send.addEventListener("click", () => act("agreed", { host: agreement.host, remember: tick.checked }));
  const not = element("a", "open", words.agree_not);
  not.href = "#/";
  not.id = "agree-not";
  choices.append(send, not);
  card.append(goes, to, remember, choices, failed);
  if (agreement.ollama.length > 0) {
    const instead = element("button", "link", words.agree_local);
    instead.type = "button";
    instead.id = "agree-local-open";
    const here = element("div", "choices");
    here.hidden = true;
    const pick = element("select");
    pick.id = "agree-local";
    for (const model of agreement.ollama) {
      const option = element("option", "", model);
      option.value = model;
      pick.append(option);
    }
    const write = element("button", "open", words.agree_local_write);
    write.type = "button";
    write.id = "agree-local-write";
    write.addEventListener("click", () => act("local_instead", { model: pick.value }));
    here.append(pick, write);
    instead.addEventListener("click", () => {
      here.hidden = false;
      pick.focus();
    });
    card.append(instead, here);
  }
  page.replaceChildren(card);
  send.focus();
}

// « Vérifier le son » (D88, board 7): a modal panel over the page that opened it. capture listens only
// while it shows: each side's device and an 8 px meter, a headset's sentence, a test sound and what was
// heard of it — never while a call holds a microphone —, and a recording started from here. Closed by ×,
// Escape or the recording, the listening ends with it, and the focus goes back to `back` — or, `back`
// hidden meanwhile, to what `lost` gives it (the bar's state's own gesture).
let checking = null;
async function checkSound(words, back, lost) {
  if (checking) return;
  const dialog = element("dialog", "check-sound");
  dialog.id = "check-sound";
  dialog.setAttribute("aria-labelledby", "check-sound-title");
  const head = element("div", "check-head");
  const title = element("h2", "", words.check_sound);
  title.id = "check-sound-title";
  const close = iconButton("x", words.check_close, "check-sound-close");
  head.append(barMark(), title, close);
  const side = (glyph, name, id, kind) => {
    const source = element("div", `check-source ${kind}`);
    const top = element("div", "check-top");
    const device = element("span", "check-device", "");
    device.id = `check-device-${id}`;
    top.append(icon(glyph), element("span", "check-name", name), device);
    const track = levelTrack("check-track", name);
    track.id = `check-level-${id}`;
    source.append(top, track);
    return [source, device, track];
  };
  const [me, meDevice, meFill] = side("mic", words.small_mic, "me", "near");
  const [others, othersDevice, othersFill] = side("volume-2", words.small_meeting, "others", "far");
  const said = (glyph, id, kind) => {
    const line = element("p", `check-said ${kind}`);
    line.id = id;
    line.hidden = true;
    line.append(icon(glyph), element("span", ""));
    return line;
  };
  const bluetooth = said("bluetooth", "check-bluetooth", "info");
  bluetooth.lastChild.textContent = words.check_bluetooth;
  const call = said("info", "check-call", "info");
  call.lastChild.textContent = words.check_call;
  const heard = said("circle-check", "check-heard", "okay");
  heard.lastChild.textContent = words.check_heard;
  const missed = said("info", "check-missed", "info");
  missed.lastChild.textContent = words.check_not_heard;
  const failed = said("info", "check-failed", "info");
  failed.lastChild.textContent = words.check_failed;
  for (const line of [heard, missed, failed]) line.setAttribute("role", "status");
  const play = element("button", "open with-icon");
  play.type = "button";
  play.id = "check-play";
  play.append(icon("volume-2"), element("span", "", words.check_play));
  const record = element("button", "primary with-icon");
  record.type = "button";
  record.id = "check-record";
  record.append(element("span", "dot"), element("span", "", words.record_start));
  const actions = element("div", "check-actions");
  actions.append(play, record);
  const note = element("p", "check-note", words.check_open_note);
  dialog.append(head, me, others, bluetooth, call, heard, missed, failed, actions, note);
  document.body.append(dialog);
  let testing = false;
  let poll = null;
  // This panel's own: a late answer of a panel closed before cannot close the one opened after it.
  const end = async (then) => {
    if (checking !== end) return;
    checking = null;
    clearInterval(poll);
    await window.__TAURI__.core.invoke("sound_check_close").catch(() => {});
    if (dialog.open) dialog.close();
    dialog.remove();
    if (then) await then();
    // Its button, as the page is now: drawn again meanwhile, it is found by its id.
    else (document.getElementById(back?.id) ?? back)?.focus();
    lost?.();
  };
  checking = end;
  close.addEventListener("click", () => end());
  // Escape closes the panel as a dialog does — and the listening with it.
  dialog.addEventListener("cancel", (event) => {
    event.preventDefault();
    end();
  });
  record.addEventListener("click", () => end(() => window.__TAURI__.core.invoke("record")));
  // Its window hidden or its page left: the panel goes, and the microphone with it.
  const hidden = () => {
    if (checking !== end) return document.removeEventListener("visibilitychange", hidden);
    if (document.visibilityState === "hidden") end();
  };
  document.addEventListener("visibilitychange", hidden);
  // Not offered before the shell's first answer says no call holds a microphone (`paint`).
  play.disabled = true;
  play.addEventListener("click", async () => {
    testing = true;
    play.disabled = true;
    heard.hidden = missed.hidden = true;
    // The shell refuses it while a call holds a microphone: then nothing plays.
    const { marked } = await window.__TAURI__.core.invoke("sound_check_mark").catch(() => ({ marked: false }));
    if (!marked) {
      testing = false;
      return;
    }
    await testSound();
    // What the meeting's side heard of it, a moment after it ended: the system's latency.
    setTimeout(async () => {
      const now = await window.__TAURI__.core.invoke("sound_check_now").catch(() => null);
      testing = false;
      if (!now?.open || checking !== end) return;
      heard.hidden = now.heard.test !== true;
      missed.hidden = now.heard.test === true;
      paint(now);
    }, 600);
  });
  const paint = ({ open, heard: listened, call: held }) => {
    // Its listening ended by the shell — a recording started elsewhere, the panel given up —: it goes.
    if (!open) return end();
    for (const [index, device, fill] of [[0, meDevice, meFill], [1, othersDevice, othersFill]]) {
      device.textContent = listened.unavailable[index] ? words.check_unavailable : listened.devices[index] ?? "";
      paintLevel(fill, listened.levels[index]);
    }
    // A call holds the microphone: its sentence alone — the headset stays in its call profile after.
    bluetooth.hidden = !listened.headset || held;
    failed.hidden = !listened.ended;
    // A call holds a microphone: no sound played, it would go into the call (D88).
    call.hidden = !held;
    play.disabled = testing || held || listened.ended;
    if (held) heard.hidden = missed.hidden = true;
  };
  dialog.showModal();
  const { opened } = await window.__TAURI__.core.invoke("sound_check_open").catch(() => ({ opened: false }));
  // Closed while it opened — a close sent before the shell counted this opening finds nothing to
  // stop —: what opened is closed now, and nothing is asked again.
  if (checking !== end) {
    if (opened) await window.__TAURI__.core.invoke("sound_check_close").catch(() => {});
    return;
  }
  if (!opened) {
    failed.lastChild.textContent = words.check_recording;
    failed.hidden = false;
    play.disabled = record.disabled = true;
    return;
  }
  poll = setInterval(async () => {
    const now = await window.__TAURI__.core.invoke("sound_check_now").catch(() => null);
    if (now && checking === end) paint(now);
  }, 100);
}

// The test sound (D88): two short notes on the system's default output, drawn here — no file, no
// library —; its context closed once played, so that no stream of Kitokey's stays open.
async function testSound() {
  const context = new AudioContext();
  const gain = context.createGain();
  gain.connect(context.destination);
  const start = context.currentTime + 0.05;
  for (const [frequency, at] of [[660, 0], [880, 0.25]]) {
    const note = context.createOscillator();
    note.frequency.value = frequency;
    note.connect(gain);
    note.start(start + at);
    note.stop(start + at + 0.22);
  }
  gain.gain.setValueAtTime(0, start);
  gain.gain.linearRampToValueAtTime(0.3, start + 0.02);
  gain.gain.setValueAtTime(0.3, start + 0.45);
  gain.gain.linearRampToValueAtTime(0, start + 0.5);
  await new Promise((done) => setTimeout(done, 650));
  await context.close();
}

// The mark, alone in the accent, the bar's handle (D90; the decider, 2026-11-08): its dot red and
// larger while recording, its stem cut further, as the tray's icon draws it.
function barMark() {
  const ns = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(ns, "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("class", "mark");
  svg.setAttribute("aria-hidden", "true");
  for (const [tag, attributes] of [
    ["path", { class: "rest", d: "M4.9 8.019A2.9 2.9 0 0 0 8.1 8.019V20A1.6 1.6 0 0 1 4.9 20Z" }],
    ["path", { class: "live", d: "M4.9 9.313A3.5 3.5 0 0 0 8.1 9.313V20A1.6 1.6 0 0 1 4.9 20Z" }],
    ["circle", { class: "dot", cx: 6.5, cy: 5.6, r: 1.6 }],
    ["path", { class: "arms", d: "M6.5 14.2 17 4M9.8 11 17.3 20" }],
  ]) {
    const shape = document.createElementNS(ns, tag);
    for (const [key, value] of Object.entries(attributes)) shape.setAttribute(key, value);
    svg.append(shape);
  }
  return svg;
}

// The recording bar (D90, replacing D35's small window): one row of 44 px — the mark as its handle,
// ● at rest or, recording, the time and ■ —, its options under it once opened; the window as tall
// as it shows. The recording asked ten times a second.
function showSmall(words) {
  const page = document.getElementById("small");
  document.body.classList.add("small-window");
  const invoke = (...asked) => window.__TAURI__.core.invoke(...asked);
  const full = (where) => invoke("full_window", { page: where ?? null });
  // As tall as it shows — « Vérifier le son »'s panel too, while it shows in place of the options.
  const fit = () => {
    const panel = document.getElementById("check-sound");
    const height = Math.max(page.getBoundingClientRect().height, panel?.open ? panel.getBoundingClientRect().height : 0);
    invoke("bar_fit", { height: Math.ceil(height) }).catch(() => {});
  };
  const fits = new ResizeObserver(fit);
  fits.observe(page);
  const toFull = iconButton("maximize-2", words.small_full, "small-full");
  toFull.addEventListener("click", () => full());
  const handle = element("span", "handle");
  handle.append(barMark());
  // ● and ■ (D90's second round): icons alone, their words the tooltip — with the shortcut — and the name.
  let keys = null;
  const named = (button, said) => {
    button.setAttribute("aria-label", said);
    button.title = keys ? `${said} (${keys})` : said;
  };
  // Its new state at once, disabled until the recording's own says otherwise (D68). Answered, a press
  // the shell took for the same one, or a start that failed, gives the button back.
  let unanswered = false;
  const asking = async (button, said, wanted, back) => {
    button.disabled = true;
    named(button, said);
    unanswered = true;
    await window.__TAURI__.core.invoke("record");
    unanswered = false;
    const header = await window.__TAURI__.core.invoke("header");
    if (header.recording !== wanted) {
      button.disabled = false;
      named(button, back);
    }
  };
  const start = element("button", "record");
  start.type = "button";
  start.id = "small-record";
  start.append(element("span", "record-dot"));
  named(start, words.record_start);
  start.addEventListener("click", () => asking(start, words.record_starting, true, words.record_start));
  const stop = element("button", "danger");
  stop.type = "button";
  stop.id = "small-stop";
  stop.append(element("span", "stop-square"));
  named(stop, words.record_stop);
  stop.addEventListener("click", () => asking(stop, words.record_stopping, false, words.record_stop));
  const time = element("span", "time", "00:00:00");
  // The context typed before the start (D91, board 5): in the row at rest, the shell keeps it for the
  // next recording until it is cleared — its ✕, or emptied —, said under the row while one is kept.
  // Enter in it starts the recording. Recording, the options' field is the meeting's own (D59).
  const nextField = element("span", "next-context");
  const nextText = element("input", "next-context-text");
  nextText.type = "text";
  nextText.id = "small-next-context";
  nextText.autocomplete = "off";
  nextText.placeholder = words.bar_context;
  nextText.setAttribute("aria-label", words.context_label);
  nextText.setAttribute("aria-describedby", "small-context-kept");
  const clear = iconButton("x", words.bar_context_clear, "small-context-clear");
  nextField.append(nextText, clear);
  const keptNext = element("p", "says", words.bar_context_kept);
  keptNext.id = "small-context-kept";
  const paintNext = () => {
    clear.hidden = nextText.value === "";
    keptNext.hidden = recording !== false || nextText.value === "";
  };
  // Each change kept in turn: Enter starts once the last one is.
  let keeping = Promise.resolve();
  const keepNext = () => {
    const text = nextText.value;
    paintNext();
    keeping = keeping
      .then(() => invoke("next_context_kept", { text }))
      // Refused once a recording runs, the field gone with the rest: nothing to say then.
      .then(() => smallSaid("context", null), () => recording === false && smallSaid("context", words.context_not_saved));
  };
  // Never more than the shell takes, counted as it counts: in characters (D59).
  nextText.addEventListener("input", () => {
    const typed = [...nextText.value];
    if (typed.length > CONTEXT_MOST) nextText.value = typed.slice(0, CONTEXT_MOST).join("");
    keepNext();
  });
  nextText.addEventListener("keydown", async (event) => {
    if (event.key !== "Enter" || event.isComposing || start.hidden || start.disabled) return;
    event.preventDefault();
    await keeping;
    start.click();
  });
  clear.addEventListener("click", () => {
    nextText.value = "";
    keepNext();
    nextText.focus();
  });
  // « Plus d’options » (D90): what is rarer, under the row; Escape closes it.
  const more = iconButton("chevron-down", words.bar_more, "small-more");
  more.setAttribute("aria-controls", "small-options");
  // « Masquer la barre » (D90): hidden, a recording goes on; the tray brings it back, nothing else.
  const hide = iconButton("minus", words.bar_hide, "small-hide");
  hide.addEventListener("click", () => invoke("bar_hide"));
  const row = element("div", "bar-row");
  const space = element("span", "bar-space");
  row.append(handle, nextField, time, space, start, stop, more, toFull, hide);
  // No title bar (D90, board 8): the bar is moved by its handle, or its row anywhere but a control.
  row.addEventListener("mousedown", (event) => {
    if (event.button === 0 && !event.target.closest("button, input, textarea, select")) invoke("bar_drag");
  });
  const rest = element("div", "small-rest");
  // The focus on something shown, or else the state's own gesture — ■ recording, ● at rest (#754): after
  // each change of state, and once « Vérifier le son »'s panel has closed, whichever ends first. The
  // options never fold under a focus of theirs but at a change of state: Escape and their chevron leave
  // it on « Plus d’options ».
  const refocus = () => {
    const at = document.activeElement;
    if (!at || at === document.body || at.getClientRects().length === 0) (recording ? stop : start).focus();
  };
  let open = false;
  const unfoldOptions = (now, back) => {
    open = now;
    options.hidden = !open;
    more.setAttribute("aria-expanded", String(open));
    more.title = open ? words.bar_less : words.bar_more;
    more.setAttribute("aria-label", more.title);
    more.firstChild.replaceWith(icon(open ? "chevron-up" : "chevron-down"));
    if (back) more.focus();
  };
  more.addEventListener("click", () => unfoldOptions(!open, false));
  // Which microphone the next recording takes (D74): the name, opened on Auto and what capture lists,
  // over what is under it, as a menu is. Nothing is offered where capture lists none. Read when it
  // opens, never watched: a device arriving opens nothing.
  const microphone = element("div", "small-microphone");
  microphone.hidden = true;
  const pick = element("button", "open disclose");
  pick.type = "button";
  pick.id = "small-microphone";
  pick.setAttribute("aria-expanded", "false");
  pick.setAttribute("aria-controls", "small-microphones");
  const pickName = element("span", "named");
  pick.append(pickName, element("span", "chevron", "▾"));
  const picks = element("div", "small-microphones");
  picks.id = "small-microphones";
  picks.setAttribute("role", "radiogroup");
  picks.setAttribute("aria-label", words.small_microphones);
  picks.hidden = true;
  microphone.append(pick, picks);
  let known = { chosen: null, list: [], unread: false };
  const paintPick = () => {
    const name = fill(words.small_microphone, known.chosen?.description ?? words.microphone_auto);
    pickName.textContent = name;
    pick.title = name;
  };
  const focusPick = () => picks.querySelector("input:checked")?.focus();
  const drawPicks = () => {
    const absent = known.chosen !== null && !known.list.some((one) => one.name === known.chosen.name);
    const each = [
      [null, words.microphone_auto, words.microphone_auto_says],
      ...known.list.map((one) => [one, one.description, ""]),
      ...(absent ? [[known.chosen, known.chosen.description, words.microphone_absent]] : []),
    ];
    picks.replaceChildren(...each.map(([one, label, says]) => {
      const option = choice("small-microphone-choice", one?.name ?? "", [label, says], (one?.name ?? null) === (known.chosen?.name ?? null), false);
      option.querySelector("input").addEventListener("change", () => choosePick(one));
      return option;
    }));
    if (known.unread) {
      const said = element("p", "says", words.microphones_unread);
      said.setAttribute("role", "status");
      picks.append(said);
    }
  };
  // Each choice made, and each the shell has answered: a list asked before a choice, or while one is
  // not yet kept, answers with the choice as it was then — never taken (#750).
  let choicesMade = 0;
  let choicesKept = 0;
  // The choice the shell last kept, as an optimistic update keeps it: a refusal goes back to it (#839).
  let confirmed = null;
  const choosePick = async (one) => {
    const mine = ++choicesMade;
    known = { ...known, chosen: one };
    paintPick();
    try {
      await invoke("microphone_chosen", { name: one?.name ?? null, description: one?.description ?? null });
      confirmed = one;
      smallSaid("microphone", null);
    } catch {
      // Put back only while it is the last choice made — a later one stands, refused or not yet —, and
      // to what the shell kept, never to a choice it was still to answer (#839).
      if (mine === choicesMade) {
        known = { ...known, chosen: confirmed };
        paintPick();
        drawPicks();
      }
      smallSaid("microphone", words.microphone_not_kept);
    } finally {
      choicesKept += 1;
    }
  };
  const closePicks = (back) => {
    if (picks.hidden) return;
    picks.hidden = true;
    pick.setAttribute("aria-expanded", "false");
    pick.lastChild.textContent = "▾";
    if (back) pick.focus();
  };
  const readPicks = async () => {
    const asked = choicesMade;
    const settled = choicesKept === choicesMade;
    const read = await invoke("microphones_listed").catch(() => null);
    if (read === null) return;
    const same = JSON.stringify(read.list) === JSON.stringify(known.list) && read.unread === known.unread;
    // The list and whether it read are taken; the choice, only if none was made since it was asked,
    // nor was waiting for the shell when it was.
    const taken = asked === choicesMade && settled;
    if (taken) confirmed = read.chosen;
    known = { ...read, chosen: taken ? read.chosen : known.chosen };
    microphone.hidden = known.list.length === 0 && !known.unread && known.chosen === null;
    paintPick();
    if (!picks.hidden && !same) {
      drawPicks();
      focusPick();
    }
  };
  pick.addEventListener("click", () => {
    if (!picks.hidden) return closePicks(true);
    picks.hidden = false;
    pick.setAttribute("aria-expanded", "true");
    pick.lastChild.textContent = "▴";
    drawPicks();
    focusPick();
    readPicks();
  });
  // As a menu button's list, or a select: the arrows, Home and End move the focus alone; Enter, Space or
  // a click chooses, and closes; Escape closes, the choice as it was.
  picks.addEventListener("keydown", (event) => {
    const inputs = [...picks.querySelectorAll("input")];
    const at = inputs.indexOf(document.activeElement);
    const to = { ArrowDown: at + 1, ArrowRight: at + 1, ArrowUp: at - 1, ArrowLeft: at - 1, Home: 0, End: inputs.length - 1 }[event.key];
    if (to !== undefined) {
      event.preventDefault();
      inputs[Math.max(0, Math.min(inputs.length - 1, to))]?.focus();
      return;
    }
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      if (at >= 0 && !inputs[at].checked) inputs[at].click();
      closePicks(true);
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      closePicks(true);
    }
  });
  // The row the focus is on, seen whole: the list scrolls to it.
  picks.addEventListener("focusin", (event) => event.target.closest(".choice")?.scrollIntoView({ block: "nearest" }));
  // A press keeps the focus here until its click has chosen; a click chooses and closes, as a menu does.
  picks.addEventListener("mousedown", (event) => event.preventDefault());
  picks.addEventListener("click", (event) => {
    if (event.detail > 0 && event.target.closest(".choice")) setTimeout(() => closePicks(true));
  });
  microphone.addEventListener("focusout", (event) => {
    if (!microphone.contains(event.relatedTarget)) closePicks(false);
  });
  document.addEventListener("click", (event) => {
    if (!microphone.contains(event.target)) closePicks(false);
  });
  // Recording, the microphone says the one recorded; a choice made then is for the next recording.
  const micNext = element("p", "says");
  micNext.id = "small-microphone-next";
  micNext.hidden = true;
  // « Vérifier le son » (D88), at rest alone: D88's panel in the bar, in place of the options. Never
  // while recording — capture does not listen then, and the row's meters say each side.
  const check = element("div", "small-check");
  const checkButton = element("button", "open with-icon");
  checkButton.type = "button";
  checkButton.id = "small-check";
  checkButton.append(icon("audio-lines"), element("span", "", words.check_sound));
  checkButton.addEventListener("click", () => {
    checkSound(words, checkButton, refocus);
    const panel = document.getElementById("check-sound");
    if (!panel) return;
    fits.observe(panel);
    panel.addEventListener("close", fit);
  });
  check.append(checkButton, element("span", "says", words.check_sound_says));
  rest.append(microphone, check);
  // « Avancé » (D91), closed by default: the models of the next recording at rest — taken by it once it
  // starts —, of the one recording while it runs. Read anew each time it opens.
  const advanced = element("div", "small-advanced");
  // What is drawn, and from what: drawn again only when what it offers changed, never under the user.
  let advancedDrawn = { from: null, models: null };
  let shownConfidential = null;
  // Chosen at rest, « Avancé » closed: said under the row, naming them short, as the context kept is
  // (D91); given with « Avancé » to a screen reader while it is shown.
  const modelsKept = element("p", "says");
  modelsKept.id = "small-models-kept";
  modelsKept.hidden = true;
  const keptNamed = (chosen) => {
    const hear = chosen.transcription;
    const named = [
      !hear ? null : hear.engine === "whisper" ? spoken(hear.model).replace(/^Whisper /, "") : hear.engine === "service" ? hear.model : "SpeechAnalyzer",
      chosen.minutes ? modelSaid(chosen.minutes.model) : null,
    ].filter(Boolean);
    return named.length === 0 ? "" : fill(named.length === 1 ? words.bar_model_kept : words.bar_models_kept, named.join(" · "));
  };
  // Whether models are kept for the next recording, as the shell says it — read again at each change
  // of state and after each choice at rest, never kept apart from it.
  let nextChosen = false;
  const readKept = async () => {
    const offered = await invoke("models_offered", { name: null }).catch((why) => {
      smallSaid("models", fill(words.models_not_read, why));
      return null;
    });
    nextChosen = offered !== null && Boolean(offered.chosen.transcription || offered.chosen.minutes);
    if (offered !== null) modelsKept.textContent = keptNamed(offered.chosen);
    paintModelsKept();
  };
  const paintModelsKept = () => {
    const open = !options.hidden && advancedDrawn.models?.toggle.getAttribute("aria-expanded") === "true";
    modelsKept.hidden = recording !== false || !nextChosen || open;
    const toggle = advancedDrawn.models?.toggle;
    if (toggle && modelsKept.hidden) toggle.removeAttribute("aria-describedby");
    else if (toggle) toggle.setAttribute("aria-describedby", modelsKept.id);
  };
  // Each ask numbered: an answer come after a later ask's is neither drawn nor said — the later one
  // is (#815). Ollama answering late, an older answer, come last, closed « Avancé » under the user.
  let advancedAsked = 0;
  // `closing`: a close's ask, made only to drop an opening's answer — the user asked nothing to be read,
  // so its failure is never said (#1142).
  const advancedOf = async (opened, closing = false) => {
    const name = meeting.name;
    const asked = ++advancedAsked;
    const offered = await invoke("models_offered", { name }).catch((why) => {
      if (asked === advancedAsked && !closing) smallSaid("models", fill(words.models_not_read, why));
      return null;
    });
    if (offered === null || asked !== advancedAsked) return;
    smallSaid("models", null);
    // Asked for another meeting than the bar's now — a start or a stop meanwhile —: the later ask draws.
    if (meeting.name !== name) return;
    const from = JSON.stringify([name, offered]);
    // The same: left as it is, opened when asked — never closed under the user.
    if (from === advancedDrawn.from && advancedDrawn.models) {
      if (opened) advancedDrawn.models.unfold(true);
      return;
    }
    const drawn = modelsAdvanced(words, "small", offered, async (chosen) => {
      if (name !== null) return invoke("meeting_models_chosen", { name, chosen });
      await invoke("next_models_kept", { chosen });
      readKept();
    });
    drawn.unfold(opened);
    // A close is an ask too: an opening's answer come after it is no longer the latest, and never opens
    // « Avancé » again under the user (#1142).
    drawn.toggle.addEventListener("click", () => {
      if (drawn.toggle.getAttribute("aria-expanded") === "true") advancedOf(true).then(() => document.getElementById("small-advanced")?.focus());
      else advancedOf(false, true);
    });
    // The control the focus is on stays the one focused once drawn again (showSmall's rule).
    const focused = advanced.contains(document.activeElement) ? document.activeElement.id : null;
    advanced.replaceChildren(drawn.box);
    advancedDrawn = { from, models: drawn };
    paintModelsKept();
    if (focused) document.getElementById(focused)?.focus();
  };

  // The meeting's language (D99): the page's row, of the next meeting at rest — taken by it once it
  // starts —, of the one recording while it runs. Read anew at each change of meeting.
  const spokenBox = element("div", "small-language");
  const languageOf = async () => {
    const name = meeting.name;
    const shown = await invoke("bar_language", { name }).catch((why) => {
      smallSaid("language", fill(words.language_not_saved, why));
      return null;
    });
    if (shown === null || meeting.name !== name) return;
    smallSaid("language", null);
    const save = name !== null ? (chosen) => invoke("language_set", { name, language: chosen }) : (chosen) => invoke("next_language_kept", { language: chosen });
    const [row, said] = languageRow(save, shown.language, shown.languages, words, shown.meeting_language, shown.apple, shown.system_language,
      "small-language", "small-language-said");
    // The control the focus is on stays the one focused once drawn again (showSmall's rule).
    const focused = spokenBox.contains(document.activeElement) ? document.activeElement.id : null;
    spokenBox.replaceChildren(row, said);
    if (focused) document.getElementById(focused)?.focus();
  };

  const live = element("div", "small-live");
  // A thin meter for each side in the row (D90, board 5), named by its side and, in its tooltip, by
  // what it is recorded from (D33). Its health in a grey line under the row: a side silent for two
  // minutes, a microphone that sends nothing, a side with no device — never as an error (D49).
  const checks = [];
  const meter = (label, id, device, glyphs, says) => {
    const row = element("span", "meter");
    row.append(icon(glyphs[0]));
    const bar = levelTrack("track", label);
    bar.id = id;
    const named = element("span", "device unseen");
    named.id = device;
    row.append(bar, named);
    const sentence = element("p", "says check-says");
    const said = element("span");
    sentence.append(icon(glyphs[1]), said);
    sentence.setAttribute("role", "status");
    sentence.hidden = true;
    checks.push({ row, sentence, said, says, glyphs, quiet: false });
    return row;
  };
  // The reminder where the user looks, the recording's first seconds, its announcement one click
  // away (D50). Gone, its room stays: Arrêter never moves.
  const announce = element("div", "small-announce");
  announce.id = "small-announce";
  // Not offered before the first reading gives its sentence: a click would copy nothing.
  announce.hidden = true;
  const announced = element("button", "link", words.copy_announcement);
  announced.type = "button";
  announced.id = "small-announce-copy";
  const said_copied = element("span", "says copied");
  said_copied.id = "small-announce-copied";
  said_copied.setAttribute("role", "status");
  announce.append(element("span", "says", words.small_announce), announced, said_copied);
  let announcement = null;
  announced.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(announcement);
      said_copied.textContent = words.minutes_copied;
    } catch {
      said_copied.textContent = words.announcement_not_copied;
    }
    setTimeout(() => (said_copied.textContent = ""), 2000);
  });
  const meters = element("span", "meters");
  meters.append(
    meter(words.small_mic, "level-me", "device-me", ["mic", "mic-off"], { silent: words.bar_silent_me, nothing: words.small_nothing, waiting: words.small_waiting_me }),
    meter(words.small_meeting, "level-others", "device-others", ["volume-2", "volume-x"], { silent: words.bar_silent_others, waiting: words.small_waiting_others }),
  );
  meters.hidden = true;
  time.after(meters);
  // Said confidential (#1204, D60): a lock in the row while it records, named for a screen reader and in its tooltip,
  // as a row's mark in the list — the bar has no room for its word, the switch under « Plus d’options » says it whole.
  const locked = element("span", "bar-locked");
  locked.id = "small-locked";
  locked.setAttribute("role", "img");
  locked.setAttribute("aria-label", words.confidential_name);
  locked.title = words.confidential_name;
  locked.append(lock());
  locked.hidden = true;
  meters.after(locked);
  const note = element("p", "says", words.small_note);
  note.id = "small-note";
  note.hidden = true;
  // The meeting's context (D59), in the options while recording: kept as it is typed, once the
  // meeting's folder holds its session.
  const meeting = { name: null, saved: null, typed: null, refused: null };
  // What could not be kept: it outlasts the recording (D67).
  const unkept = new Map();
  const unkeptSaid = element("p", "note warn small-said");
  unkeptSaid.id = "small-said";
  unkeptSaid.setAttribute("role", "status");
  unkeptSaid.hidden = true;
  const smallSaid = (what, sentence) => {
    if (sentence === null) unkept.delete(what);
    else unkept.set(what, sentence);
    unkeptSaid.textContent = [...unkept.values()].join("\n");
    unkeptSaid.hidden = unkept.size === 0;
  };
  advancedOf(false);
  languageOf();
  readKept();
  const keep = async (last = false) => {
    const { name, typed } = meeting;
    if (name === null || typed === null || typed === meeting.saved) return;
    // Refused: tried again when it changes, and when the recording stops; each try is a line in the log.
    if (typed === meeting.refused && !last) return;
    try {
      await invoke("context_saved", { name, text: typed });
      meeting.saved = typed;
      smallSaid("context", null);
    } catch {
      meeting.refused = typed;
      smallSaid("context", words.context_not_saved);
    }
  };
  const field = contextField(words, "", (text) => {
    meeting.typed = text;
    keep();
  }, "small-context");
  field.area.rows = 2;
  field.area.placeholder = words.context_label;
  field.area.setAttribute("aria-label", words.context_label);
  const opened = element("div", "small-context");
  opened.append(field.area, field.count);
  // Said confidential (D60), under the microphone in the options; kept as it is turned, once the
  // meeting's folder holds its session.
  const secret = confidentialSwitch(words, "small-confidential");
  const mark = { wanted: null, refused: 0 };
  const marked = async () => {
    if (meeting.name === null || mark.wanted === null) return;
    const wanted = mark.wanted;
    try {
      await invoke("confidential_set", { name: meeting.name, on: wanted });
      if (mark.wanted === wanted) mark.wanted = null;
      smallSaid("mark", null);
      secret.row.classList.toggle("on", secret.box.checked);
    } catch {
      // Said, its lock off: on with nothing kept would promise what is not so. Asked again every
      // five seconds, not every tick: each try is a line in the log.
      mark.refused = Date.now();
      secret.row.classList.remove("on");
      smallSaid("mark", words.confidential_not_saved);
    }
  };
  secret.box.addEventListener("change", () => {
    mark.wanted = secret.box.checked;
    mark.refused = 0;
    secret.row.classList.toggle("on", secret.box.checked);
    marked();
  });
  live.append(opened, micNext, secret.row, note);
  live.hidden = true;
  // At rest, the meeting last stopped and its stage (D92): one line under the row, cut by an ellipsis
  // where it is longer than the bar, the whole of it its tooltip; « Ouvrir » beside it once it is made,
  // stopped or failed, never focused by itself. Between two steps, the stage said stays. Said by a
  // polite live region of its own, unseen and always in the page — a region there before its first
  // words says them —, whose text changes only with the stage: never said again as the makings are read.
  const lastLive = element("span", "unseen");
  lastLive.id = "small-last-live";
  lastLive.setAttribute("aria-live", "polite");
  const lastLine = element("p", "says small-last");
  lastLine.id = "small-last";
  lastLine.hidden = true;
  const lastSaid = element("span");
  lastSaid.id = "small-last-said";
  const lastOpen = element("button", "link", words.bar_last_open);
  lastOpen.type = "button";
  lastOpen.id = "small-last-open";
  lastOpen.hidden = true;
  lastOpen.addEventListener("click", () => full(`#/meeting/${encodeURIComponent(lastOpen.dataset.name)}`));
  lastLine.append(lastSaid, " ", lastOpen);
  const lastStage = (last) => {
    const heard = transcriptState(last.name);
    const minutes = minutesState(last.name);
    if (!last.ended) {
      if (heard && heard.kind !== "failed" && heard.kind !== "stopped") return { said: words.bar_last_hearing };
      if (minutes && minutes.kind !== "failed" && minutes.kind !== "stopped") return { said: words.bar_last_summary };
      return null;
    }
    if (heard?.kind === "failed") return { said: words.bar_last_failed, why: words.transcript_not_made, open: true };
    if (minutes?.kind === "failed") {
      return { said: words.bar_last_failed, why: minutes.said ? `${words.minutes_not_made} ${minutes.said}` : words.minutes_not_made, open: true };
    }
    if (heard?.kind === "stopped" || minutes?.kind === "stopped") return { said: words.bar_last_stopped, open: true };
    if (minutes) return { said: words.bar_last_summary };
    // Written, but not to the export folder (D109): said as its page says it.
    if (makings?.not_exported.includes(last.name)) return { said: words.bar_last_failed, why: words.export_failed, open: true };
    return { said: words.bar_last_ready, open: true };
  };
  // Painted with each reading of the makings, and as a recording starts or ends (`recording`, below).
  let lastShown = null;
  paintLast = () => {
    const last = recording === false ? lastStopped : null;
    const stage = last && lastStage(last);
    if (stage) lastShown = { name: last.name, text: fill(stage.said, last.said, stage.why ?? ""), open: !!stage.open };
    else if (!last || lastShown?.name !== last.name) lastShown = null;
    const text = lastShown?.text ?? "";
    if (lastLive.textContent !== text) lastLive.textContent = text;
    lastSaid.textContent = text;
    lastSaid.title = text;
    lastOpen.hidden = !lastShown?.open;
    lastOpen.dataset.name = lastShown?.name ?? "";
    lastLine.hidden = text === "";
  };
  const lines = element("div", "bar-lines");
  lines.append(lastLine, keptNext, modelsKept, announce, ...checks.map((check) => check.sentence), unkeptSaid);
  const options = element("div", "bar-options");
  options.id = "small-options";
  options.append(rest, live, advanced, spokenBox);
  options.hidden = true;
  // Escape anywhere in the bar folds them (board 5): the focus back on « Plus d’options » from inside
  // them, where it is from the row — a field's text left as it is.
  page.addEventListener("keydown", (event) => {
    if (event.key !== "Escape" || event.defaultPrevented || !open) return;
    event.preventDefault();
    unfoldOptions(false, options.contains(event.target));
  });
  page.replaceChildren(row, lines, options, lastLive);

  // The shortcut, in ●'s and ■'s tooltips, as the desktop grants it.
  const paintKeys = async () => {
    const header = await invoke("header").catch(() => null);
    // Chosen in Settings meanwhile: the bar takes it too (D98).
    if (header !== null) takeAppearance(header.appearance);
    if (header === null || header.shortcut === keys) return;
    keys = header.shortcut;
    named(start, start.getAttribute("aria-label"));
    named(stop, stop.getAttribute("aria-label"));
  };

  let recording = null;
  // Each reading numbered, as « Avancé »'s asks are (#815): readings every 100 ms may overlap, and one
  // answered after a later one is older than what is drawn — never drawn over it (#1166).
  let tickAsked = 0;
  let tickDrawn = 0;
  const tick = async () => {
    const asked = ++tickAsked;
    const now = await invoke("recording_now").catch(() => null);
    if (now === null || asked < tickDrawn) return;
    tickDrawn = asked;
    if (now.recording !== recording) {
      const first = recording === null;
      // What was typed last, kept before the field is emptied for the next recording.
      if (recording) {
        await keep(true);
        await marked();
        // Refused to the end: at rest no switch is left, and what remains is said as it is.
        smallSaid("mark", mark.wanted === true ? words.small_not_confidential : null);
      }
      // A new recording starts clean; one that ended keeps saying what it could not keep.
      if (now.recording) {
        smallSaid("context", null);
        smallSaid("mark", null);
      }
      closePicks(false);
      // Back at rest, and the first time: what capture lists now.
      if (!now.recording) readPicks();
      recording = now.recording;
      paintLast();
      start.disabled = false;
      named(start, words.record_start);
      stop.disabled = false;
      named(stop, words.record_stop);
      start.hidden = nextField.hidden = recording;
      stop.hidden = time.hidden = meters.hidden = space.hidden = !recording;
      // At rest, what the shell keeps for the next recording; taken by this one, recording (D91).
      if (!recording) nextText.value = (await invoke("next_context").catch(() => ({ text: "" }))).text;
      paintNext();
      // At rest, the bar is its row: the recording's lines go with it (the ticks stop at the stop).
      if (!recording) {
        for (const check of checks) check.sentence.hidden = true;
        announce.hidden = true;
      }
      handle.classList.toggle("live", recording);
      const dot = handle.querySelector(".dot");
      dot.setAttribute("r", recording ? "2.2" : "1.6");
      dot.setAttribute("cy", recording ? "6.2" : "5.6");
      // Each recording its own context: nothing of the one before.
      Object.assign(meeting, { name: null, saved: null, typed: null, refused: null });
      // « Avancé » and what is said kept: of the next recording again, as the shell has them.
      advancedOf(false);
      languageOf();
      readKept();
      mark.wanted = null;
      secret.box.checked = false;
      secret.row.classList.remove("on");
      field.area.value = "";
      field.painted();
      rest.hidden = recording;
      live.hidden = !recording;
      // At rest, no lock: the next recording says its own.
      if (!recording) locked.hidden = true;
      // A recording starts its traces from empty, never from the last one's last two seconds.
      if (recording) for (const id of ["level-me", "level-others"]) drawLevel(document.getElementById(id), levelShown(undefined, 0, 0, false, TRACE), 0);
      // The microphone in each state's options: recording, after the context, as board 6 has it.
      if (recording) opened.after(microphone);
      else rest.prepend(microphone);
      // No list where capture lists none: nothing to change, nothing said of it.
      micNext.hidden = !recording || microphone.hidden;
      // Each state opens closed: the row alone.
      advancedDrawn.models?.unfold(false);
      unfoldOptions(false, false);
      if (!recording) paintKeys();
      // Opened at rest, the recording's gesture has the focus; after, a focus left on nothing — ● or ■
      // gone with its state, a control of the options hidden with them — follows to the state's own
      // gesture, as the full window's button gives it back (`paintRecord`).
      if (first && !recording) start.focus();
      else if (!first) refocus();
    }
    paintModelsKept();
    if (!recording) {
      // A start that failed late: the shell has let it go, and no recording will come to give it back.
      if (start.disabled && !unanswered && !(await window.__TAURI__.core.invoke("header")).recording) {
        start.disabled = false;
        named(start, words.record_start);
      }
      return;
    }
    const recorded = now.devices[0] ? fill(words.bar_microphone_recorded, now.devices[0]) : words.bar_microphone_next;
    if (micNext.textContent !== recorded) micNext.textContent = recorded;
    const whole = Math.floor(now.seconds);
    time.textContent = [Math.floor(whole / 3600), Math.floor((whole % 3600) / 60), whole % 60].map((n) => String(n).padStart(2, "0")).join(":");
    for (const [index, id] of ["level-me", "level-others"].entries()) paintLevel(document.getElementById(id), now.levels[index]);
    // Written when it changes alone: a tick that rewrites text under a click loses it in WebKitGTK.
    for (const [index, check] of checks.entries()) {
      const device = now.devices[index] ?? "";
      const named = check.row.querySelector(".device");
      if (named.textContent !== device) named.textContent = device;
      if (check.row.title !== device) check.row.title = device;
      // Silent two minutes, said; a device's state at once; the microphone chosen gone says more (D74).
      const state = now.checks[index];
      const silent = state === "silent" && (now.silent_for[index] ?? 0) >= SILENT_SAID;
      const says = index === 0 && now.microphone_gone ? words.small_microphone_gone : state === "silent" && !silent ? "" : check.says[state] ?? "";
      if (check.said.textContent !== says) check.said.textContent = says;
      check.sentence.hidden = says === "" || !recording;
      const quiet = silent || state === "nothing" || state === "waiting";
      if (check.quiet !== quiet) {
        check.quiet = quiet;
        check.row.classList.toggle("quiet", quiet);
        check.row.firstChild.replaceWith(icon(check.glyphs[Number(quiet)]));
      }
    }
    announcement = now.announcement;
    announce.hidden = announcement === null || !recording;
    announce.classList.toggle("gone", now.seconds >= ANNOUNCED_FOR);
    stop.disabled ||= !now.recording;
    note.hidden = !now.transcript_follows;
    // The meeting's folder there: what it holds already is shown, what was typed before is kept.
    if (now.meeting !== meeting.name) {
      const another = meeting.name !== null;
      meeting.name = now.meeting;
      if (another) meeting.typed = null;
      // « Avancé » now speaks of this meeting — or, stopped, of the next one —, closed as board 6 draws it.
      advancedOf(false);
      languageOf();
      if (now.meeting !== null) {
        const there = await invoke("context_shown", { name: now.meeting }).catch((why) => {
          smallSaid("context-read", fill(words.context_not_read, why));
          return null;
        });
        if (there !== null) smallSaid("context-read", null);
        meeting.saved = there?.text ?? "";
        if (meeting.typed === null && meeting.name === now.meeting) {
          field.area.value = meeting.saved;
          field.painted();
        }
      }
    }
    keep();
    // What the folder says, unless a turn of the switch is still to be kept.
    if (mark.wanted === null && now.meeting !== null && secret.box.checked !== now.confidential) {
      secret.box.checked = now.confidential;
      secret.row.classList.toggle("on", now.confidential);
    }
    if (Date.now() - mark.refused >= 5000) marked();
    // The lock as the switch stands: the folder's word, or a turn not yet kept.
    locked.hidden = !(recording && secret.box.checked);
    // Said confidential, or no longer: « Avancé » offers otherwise — no service then (D60) —, drawn
    // again as it was, open or closed.
    if (now.meeting !== null && now.confidential !== shownConfidential) {
      shownConfidential = now.confidential;
      advancedOf(advancedDrawn.models?.toggle.getAttribute("aria-expanded") === "true");
    }
    if (now.meeting === null) shownConfidential = null;
  };
  setInterval(tick, 100);
  setInterval(() => recording === false && paintKeys(), 1500);
  tick();
}

// Freeing space (D11's third board): the audio alone or the whole meeting, the heaviest first,
// what is freed said before; everything to the system's trash, on the user's word.
function showFree(listing, busy, words) {
  const page = document.getElementById("free");
  let whole = page.dataset.whole === "true";
  const ticked = new Set(JSON.parse(page.dataset.ticked || "[]"));
  const draw = () => {
    page.dataset.whole = String(whole);
    page.dataset.ticked = JSON.stringify([...ticked]);
    const weight = (s) => (whole ? s.audio + s.text : s.audio);
    const free = listing.sessions.filter((s) => weight(s) > 0 && !busy.includes(nameOf(s)));
    const kept = listing.sessions.filter((s) => busy.includes(nameOf(s)));
    free.sort((a, b) => weight(b) - weight(a));
    for (const name of [...ticked]) if (!free.some((s) => nameOf(s) === name)) ticked.delete(name);
    // As the board heads it: the way back, and the page's name.
    const top = element("div", "page-top");
    top.append(element("h1", "", words.free));
    const parts = [top, element("p", "eyebrow", words.free_what)];
    const ways = element("div", "ways");
    ways.setAttribute("role", "radiogroup");
    for (const [value, said] of [[false, words.free_audio], [true, words.free_whole]]) {
      const one = choice("free-what", String(value), said, whole === value, false);
      one.querySelector("input").addEventListener("change", () => {
        whole = value;
        draw();
      });
      ways.append(one);
    }
    parts.push(ways);
    const head = element("div", "row");
    const older = element("button", "open", words.free_older);
    older.type = "button";
    older.id = "free-older";
    older.addEventListener("click", () => {
      const month = Date.now() / 1000 - 30 * 86400;
      for (const s of free) if (s.instant !== null && s.instant < month) ticked.add(nameOf(s));
      draw();
    });
    head.append(element("p", "eyebrow", words.free_heaviest), older);
    parts.push(head);
    const list = element("ol", "free-list");
    for (const s of free) {
      const item = element("li");
      const row = element("label", "card freeable");
      const box = element("input");
      box.type = "checkbox";
      box.name = "freeable";
      box.value = nameOf(s);
      box.checked = ticked.has(nameOf(s));
      box.addEventListener("change", () => {
        if (box.checked) ticked.add(nameOf(s));
        else ticked.delete(nameOf(s));
        draw();
      });
      const said = element("span", "said");
      // Named (D70): the date stays beside the length, what tells two of one name apart.
      const lasted = s.seconds !== null ? length(s.seconds) : "";
      said.append(element("span", "name", s.title ?? heading(s, words)), element("span", "length", s.title ? `${heading(s, words)} · ${lasted}` : lasted));
      row.append(box, said, element("span", "weighs", size(weight(s), words)));
      item.append(row);
      list.append(item);
    }
    for (const s of kept) {
      const item = element("li");
      const row = element("div", "card freeable busy");
      const said = element("span", "said");
      said.append(element("span", "name", s.title ?? heading(s, words)), element("span", "length", s.title ? `${heading(s, words)} · ${words.free_busy}` : words.free_busy));
      row.append(said);
      item.append(row);
      list.append(item);
    }
    parts.push(list);
    const chosen = free.filter((s) => ticked.has(nameOf(s)));
    const freed = chosen.reduce((sum, s) => sum + weight(s), 0);
    const total = element("section", "card free-total");
    const sum = element("div", "said");
    const named = element("div", "total");
    named.append(element("strong", "", size(freed, words)), element("span", "", chosen.length === 1 ? words.free_one : fill(words.free_total, chosen.length)));
    sum.append(named, element("p", "says", words.free_note));
    if (!whole) sum.append(element("p", "says", words.free_audio_note));
    const go = element("button", "open", whole ? words.free_do_whole : words.free_do_audio);
    go.type = "button";
    go.id = "free-do";
    go.disabled = chosen.length === 0;
    const said = element("p", "note");
    said.id = "free-said";
    said.setAttribute("aria-live", "polite");
    go.addEventListener("click", async () => {
      go.disabled = true;
      try {
        await window.__TAURI__.core.invoke("freed", { names: chosen.map(nameOf), whole });
        ticked.clear();
        page.dataset.ticked = "[]";
        page.dataset.said = words.free_done;
      } catch (why) {
        page.dataset.said = fill(words.not_saved, why);
      }
      await refresh();
    });
    total.append(sum, go);
    said.textContent = page.dataset.said ?? "";
    delete page.dataset.said;
    parts.push(total, said);
    // Each gesture draws the page again: the focus goes back where the keyboard left it.
    const focused = page.contains(document.activeElement) ? document.activeElement : null;
    const refocus = focused && (focused.id ? `#${CSS.escape(focused.id)}` : focused.name ? `input[name="${CSS.escape(focused.name)}"][value="${CSS.escape(focused.value)}"]` : null);
    page.replaceChildren(...parts);
    if (refocus) page.querySelector(refocus)?.focus();
  };
  draw();
}

// The first run's assistant (D58): one question — two when the minutes may go online —, then the
// setting said plainly. Private unless the user says otherwise; from Settings (`#/welcome/again`)
// the current setting chosen, and nothing saved before the last screen's confirmation.
// Settings open at the minutes once, after « Choisir mon service ».
let focusMinutes = false;
// The language just chosen in Settings (D64): focused again once the page is drawn in it.
let focusLanguage = false;

async function showWelcome(again, words) {
  const page = document.getElementById("welcome");
  const { invoke } = window.__TAURI__.core;
  const [shown, minutesFirst] = await Promise.all([invoke("settings_shown"), invoke("minutes_shown")]);
  // Asked again by « Réessayer » when Ollama does not answer (#781).
  let minutes = minutesFirst;
  // Apple's SpeechAnalyzer, asked aside: only the summary needs it.
  const speech = invoke("speech_shown").catch(() => null);
  const local = !minutes.provider || minutes.provider_local;
  // From Settings, the setting as it stands; at the first run, nothing chosen for the user.
  const state = { step: "question", private: again && minutes.provider ? (local ? "often" : "rarely") : null, writer: again && minutes.provider && !local ? "service" : null };
  const connected = () => state.private === "rarely" && state.writer === "service";
  const ollamaModel = minutes.provider === minutes.ollama_address && minutes.model ? minutes.model : minutes.recommended;
  const engine = shown.engine === "speechanalyzer" ? "speechanalyzer" : "whisper";
  const modelFile = shown.model.split(/[\\/]/).pop();
  const modelHere = shown.models.some((model) => model.path === shown.model);
  // The model the tray fetches is the list's first, the default: one the user named is theirs.
  const fetchable = !modelHere && shown.offered[0]?.file === modelFile ? shown.offered[0] : null;

  const card = (id) => {
    const made = element("section", "card welcome");
    made.id = id;
    const top = element("div", "welcome-top");
    top.append(document.querySelector("#sidebar .brand").cloneNode(true));
    made.append(top);
    return [made, top];
  };
  const heading = (text) => {
    const h1 = element("h1", "", text);
    h1.id = "welcome-heading";
    h1.tabIndex = -1;
    return h1;
  };
  const button = (id, className, text, act) => {
    const made = element("button", className, text);
    made.type = "button";
    made.id = id;
    made.addEventListener("click", act);
    return made;
  };
  const group = (name, labelled, options, chosen) => {
    const radios = element("div", "welcome-choices");
    radios.setAttribute("role", "radiogroup");
    radios.setAttribute("aria-labelledby", labelled);
    for (const [value, said] of options) {
      const one = choice(name, value, said, chosen === value, false);
      one.querySelector("input").addEventListener("change", () => (state[name] = value));
      radios.append(one);
    }
    return radios;
  };
  const said = element("p", "says welcome-said");
  said.id = "welcome-said";
  said.setAttribute("role", "status");
  const go = (step) => {
    state.step = step;
    draw();
  };
  // Confirmed: what the summary said, saved with Settings' own commands; refused, said, nothing more.
  const save = async () => {
    try {
      if (!connected() && (minutes.provider !== minutes.ollama_address || minutes.model !== ollamaModel)) {
        await invoke("minutes_saved", { provider: minutes.ollama_address, model: ollamaModel, key: null });
      }
      // Private: the transcription comes back to this computer, the engine alone. Connected
      // changes nothing by itself: the service is chosen in Settings.
      if (!connected() && shown.engine !== engine) await invoke("settings_saved", { change: { engine } });
      return true;
    } catch (why) {
      said.textContent = fill(words.not_saved, why);
      return false;
    }
  };
  const done = (hash = "#/") => {
    location.hash = hash;
  };

  const question = () => {
    const [made] = card("welcome-question");
    const examples = element("p", "says", words.welcome_examples);
    examples.id = "welcome-examples";
    const radios = group("private", "welcome-heading", [["often", words.welcome_often], ["rarely", words.welcome_rarely], ["unsure", words.welcome_unsure]], state.private);
    const actions = element("div", "welcome-actions");
    actions.append(
      again
        ? button("welcome-skip", "link", words.welcome_cancel, () => done("#/settings"))
        : button("welcome-skip", "link", words.welcome_skip, async () => {
          state.private = "often";
          if (await save()) done();
        }),
      // Nothing chosen is the most discreet: private.
      button("welcome-next", "primary", words.welcome_next, () => go(state.private === "rarely" ? "writer" : "summary")),
    );
    radios.setAttribute("aria-describedby", "welcome-examples");
    made.append(element("p", "says welcome-eyebrow", words.welcome_title), heading(words.welcome_question), examples, radios, actions, said);
    return made;
  };

  const writer = () => {
    const [made] = card("welcome-writer");
    const title = heading(words.welcome_writer);
    const radios = group("writer", "welcome-heading", [["here", words.welcome_here], ["service", words.welcome_service]], state.writer);
    const actions = element("div", "welcome-actions");
    actions.append(button("welcome-back", "link", words.welcome_back, () => go("question")), button("welcome-next", "primary", words.welcome_next, () => go("summary")));
    made.append(title, radios, actions, said);
    return made;
  };

  const summary = async () => {
    const status = await speech;
    const [made, top] = card("welcome-summary");
    top.append(button("welcome-back", "link", words.welcome_back, () => go(state.private === "rarely" ? "writer" : "question")));
    const online = connected();
    const [name, line] = online ? words.welcome_connected : words.welcome_private;
    const mode = element("div", `welcome-mode ${online ? "connected" : "private"}`);
    mode.append(heading(name), element("p", "", line));
    const facts = element("dl", "welcome-facts");
    const fact = (term, main, secondary) => {
      const value = element("dd");
      value.append(element("span", "name", main));
      if (secondary) value.append(element("span", "says", secondary));
      facts.append(element("dt", "", term), value);
    };
    const [transcription, minutesRow_, reminder] = words.welcome_rows;
    // Connected leaves who hears as it is: a service already chosen in Settings is said as one.
    let hearer = shown.transcription_provider ?? "";
    try {
      hearer = new URL(hearer).host;
    } catch {}
    if (online && shown.engine === "service") fact(transcription, words.service_option, hearer);
    else fact(transcription, words.welcome_on_this_computer, engine === "speechanalyzer" ? words.speech_option : spoken(modelFile));
    let host = "";
    try {
      host = new URL(minutes.provider).host;
    } catch {}
    // Online, chosen already (from Settings): its host; else the services Settings offer.
    const offered = new Intl.ListFormat(words.locale, { type: "disjunction" }).format(minutes.online.map((service) => service.name));
    if (online) fact(minutesRow_, local ? words.welcome_online : words.welcome_online_chosen, local ? offered : host);
    else fact(minutesRow_, words.welcome_on_this_computer, fill(words.welcome_ollama_by, modelSaid(ollamaModel)));
    fact(reminder, shown.reminder ? words.welcome_reminder_on : words.welcome_reminder_off);
    made.append(mode, facts);
    // The minutes wait for Ollama: said, with where it comes from — Kitokey never installs it.
    if (!online && !minutes.ollama_answers) {
      // Asked again on a click, as a meeting's page does: Ollama started meanwhile, the summary said anew (#781).
      const again = button("welcome-ollama-again", "open", words.try_again, async () => {
        waiting(again, true);
        // Read at the click, as the pages read it, not the `invoke` the welcome took at its start.
        minutes = await window.__TAURI__.core.invoke("minutes_shown").catch((why) => {
          pageFailed(why);
          return minutes;
        });
        await draw();
      });
      const wait = element("p", "says welcome-wait", words.welcome_ollama_silent);
      wait.append(" ", again);
      made.append(wait);
    } else if (!online && !minutes.ollama.includes(ollamaModel)) made.append(element("p", "says welcome-wait", fill(words.welcome_ollama_lacks, ollamaModel)));
    const actions = element("div", "welcome-actions");
    // The model still to fetch: « Plus tard » says the meetings will wait for it (#781).
    const waits = (engine === "speechanalyzer" && status?.offered && !status.installed) || (engine === "whisper" && fetchable);
    const later = button("welcome-later", "link", waits ? words.welcome_later_waits : words.welcome_later, async () => {
      if (await save()) done();
    });
    let primary;
    let from = null;
    if (online) {
      primary = button("welcome-go", "primary", words.welcome_choose, async () => {
        if (!(await save())) return;
        focusMinutes = true;
        done("#/settings");
      });
    } else if (engine === "speechanalyzer" && status?.offered && !status.installed) {
      // Apple's model, fetched by macOS on this click, as Settings fetch it (D42).
      primary = button("welcome-go", "primary", words.speech_fetch, async () => {
        if (!(await save())) return;
        invoke("speech_installed").catch((why) => pageFailed(why));
        done();
      });
    } else if (engine === "whisper" && fetchable) {
      // The tray's own fetch, its size and host shown first (D36); how far is said where Settings
      // and the tray say it.
      primary = button("welcome-go", "primary", fill(words.welcome_fetch, size(fetchable.bytes, words)), async () => {
        if (!(await save())) return;
        await invoke("model_fetch").catch(() => {});
        done();
      });
      from = element("p", "says welcome-from", fill(words.welcome_fetch_from, fetchable.host));
      from.id = "welcome-host";
    } else {
      primary = button("welcome-go", "primary", words.welcome_done, async () => {
        if (await save()) done();
      });
    }
    actions.append(later, primary);
    made.append(actions);
    if (from) made.append(from);
    made.append(said);
    return made;
  };

  const draw = async () => {
    said.textContent = "";
    const made = state.step === "question" ? question() : state.step === "writer" ? writer() : await summary();
    page.replaceChildren(made);
    made.querySelector("#welcome-heading").focus();
  };
  await draw();
}

let words = null;

// What the last save on the settings' page could not do, for the page drawn after it.
let notSaved = null;

// The latest drawing asked for: one begun before it, its answers come later, draws nothing.
let drawing = 0;

// Under 840 px, one pane at a time (D61): the list's address is the list alone.
const narrow = matchMedia("(max-width: 839px)");
// Only the list's address shows another thing across the break (its pane beside it, D61): a meeting's
// page stays as it is, a panel open in it too, its choice and the focus (#777).
narrow.addEventListener("change", () => {
  if (location.hash === "" || location.hash === "#/") refresh();
});

// The address changed: what is shown acts as it stands, so it goes at once, as a page does when it
// navigates; the drawing fills it. Only a meeting's own address keeps it, reached from beside the
// list. Compared as written: an address that does not decode is another, its drawing's failure says so.
function leavePage() {
  const page = document.getElementById("meeting");
  document.getElementById("agree").replaceChildren();
  document.getElementById("free").replaceChildren();
  document.getElementById("settings").replaceChildren();
  document.getElementById("settings-link").setAttribute("aria-current", "false");
  if (location.hash === `#/meeting/${encodeURIComponent(page.dataset.name ?? "")}`) return;
  page.replaceChildren();
  delete page.dataset.name;
  delete page.dataset.at;
  document.querySelector("#groups .session[aria-current]")?.removeAttribute("aria-current");
}
// A drawing that failed: said in the pane, at any width, with a way to try again.
let pageNotShown = false;
function notShown(said) {
  pageNotShown = said;
  document.getElementById("pane-again").hidden = !said;
}
// What the meeting's page was drawn from, to know whether it changed since.
let drawnMeeting = null;

// Every window's look (D98): the system's, or the one chosen whatever the system says.
function takeAppearance(appearance) {
  if (appearance === "light" || appearance === "dark") document.documentElement.dataset.theme = appearance;
  else delete document.documentElement.dataset.theme;
}

// Where the user is on a page (D101), as WAI-ARIA's practices keep it across a change: the control
// focused — or the one just changed, a click in WebKit focusing none —, the boxes holding it, how far
// the page is scrolled. The address it was read at: given back once that page is drawn again there,
// never on another page.
let drawnAddress = null;
let changedLast = null;
for (const type of ["change", "input"]) {
  document.addEventListener(type, (event) => {
    const changed = event.target;
    if (!(changed instanceof Element) || !changed.id || !changed.closest("#meeting, #settings")) return;
    changedLast = { id: changed.id, at: Date.now() };
    // Changed by a click WebKit gave no focus to — a box, a radio, a list —: the control acted on has it,
    // as a key would have given it, wherever the focus was — on nothing, or left on another control, which
    // a drawing would then give it back to (#944: the decider's #autostart). Never a field of text: its
    // change comes as it is left, the focus passing by the page on its way elsewhere (meeting_renamed,
    // « leaving the field keeps it »).
    const clicked = changed.matches("input[type=checkbox], input[type=radio], select");
    if (clicked && document.activeElement !== changed) changed.focus({ preventScroll: true });
  }, true);
}
// The places of the drawings under way: the user moving the focus meanwhile moves where each one gives it
// back (#906) — read when the drawing began, a place the user has since left is no place to return to.
const placesPending = new Set();
document.addEventListener("focusin", (event) => {
  if (placesPending.size === 0 || !(event.target instanceof Element) || !event.target.closest("#meeting, #settings")) return;
  for (const place of placesPending) Object.assign(place, placeOf());
}, true);
// A list's menu is a window of its own (GTK): opening it takes the window's focus away, closing it gives
// it back — no coming back to the window, nothing to draw anew. Drawn then, the list is replaced under the
// choice being made, before its change: the choice not yet saved lost, the focus with it (#944). The menu
// opening: the window's focus going while a list has the page's, or a moment after a list left it for nowhere.
let listLeft = 0;
let inMenu = false;
document.addEventListener("focusout", (event) => {
  if (event.relatedTarget === null && event.target instanceof HTMLSelectElement) listLeft = Date.now();
}, true);
window.addEventListener("blur", () => {
  inMenu = document.activeElement instanceof HTMLSelectElement || Date.now() - listLeft < 1000;
});
const FOCUSABLE = "button:not([disabled]), select:not([disabled]), input:not([disabled]), textarea:not([disabled]), a[href], summary, [tabindex]:not([tabindex='-1'])";
function placeOf() {
  const place = { address: drawnAddress === location.hash ? location.hash : null, y: scrollY, id: null, near: [], before: [] };
  const at = document.activeElement;
  const focused = at && at !== document.body && at.id && at.closest("#meeting, #settings") ? at : null;
  const changed = changedLast && Date.now() - changedLast.at < 5000 ? document.getElementById(changedLast.id) : null;
  const control = focused ?? changed;
  if (!control) return place;
  place.id = control.id;
  // Its boxes with an id, nearest first: where a control the change removed was.
  for (let up = control.parentElement; up && up.id !== "meeting" && up.id !== "settings"; up = up.parentElement) if (up.id) place.near.push(up.id);
  // The controls with an id before it in the page, nearest first: where the focus goes when it is gone and no
  // box of its own is found (WAI-ARIA APG: content removed, the focus to its logical predecessor; #944).
  const page = control.closest("#meeting, #settings");
  const controls = page ? [...page.querySelectorAll(FOCUSABLE)].filter((one) => one.id) : [];
  place.before = controls.slice(0, Math.max(controls.indexOf(control), 0)).slice(-3).reverse().map((one) => one.id);
  return place;
}
// Drawn again at the same address: the focus back on the same control — or, removed, on the first one
// in the nearest box left that holds one, else on the control before it —, the page where it was. A focus
// the user moved meanwhile stays.
function placeBack(place) {
  if (place.address === null || place.address !== location.hash) return;
  if (place.id !== null && (document.activeElement === null || document.activeElement === document.body)) {
    const same = document.getElementById(place.id);
    // Disabled, a control takes no focus: as good as gone.
    let target = same?.getClientRects().length && !same.disabled ? same : null;
    for (const id of place.near) {
      if (target) break;
      target = [...(document.getElementById(id)?.querySelectorAll(FOCUSABLE) ?? [])].find((control) => control.getClientRects().length) ?? null;
    }
    // No box of its own holds one: the control before it in the page (« Garder … jours » gone, its list).
    target ??= place.before.map((id) => document.getElementById(id)).find((one) => one?.getClientRects().length && !one.disabled) ?? null;
    // None left to take it: the nearest box still there, focused itself — never the page's start.
    target ??= place.near.map((id) => document.getElementById(id)).find((box) => box?.getClientRects().length) ?? null;
    // A box — the one focused before, drawn anew, too — takes the focus by script alone.
    if (target && !target.matches(FOCUSABLE) && !target.hasAttribute("tabindex")) target.tabIndex = -1;
    target?.focus({ preventScroll: true });
  }
  if (Math.round(scrollY) !== Math.round(place.y)) window.scrollTo(0, place.y);
}

// `again`: the list's address drawn once more after a page gone — never more than once.
// Whoever asked: a drawing still the current one that fails says so; one superseded answers for
// nothing. The failure goes on to `page_failed`, with its stack.
async function refresh(again = false) {
  const mine = ++drawing;
  const place = placeOf();
  placesPending.add(place);
  try {
    const drawn = await draw(mine, again);
    // Followed no more: the focus `placeBack` gives is its own, never a move of the user's.
    placesPending.delete(place);
    if (mine === drawing) {
      placeBack(place);
      drawnAddress = location.hash;
      trashedFocused();
      cancelledFocused();
    }
    return drawn;
  } catch (why) {
    if (mine === drawing) {
      // Nothing left to stop it with.
      if (document.getElementById("meeting").childElementCount === 0) {
        player?.stop();
        player = null;
      }
      if (words) {
        paneSays(words.page_not_shown);
        document.getElementById("pane-again").textContent = words.try_again;
        notShown(true);
      }
    }
    throw why;
  } finally {
    placesPending.delete(place);
  }
}

async function draw(mine, again) {
  // The small window is its own: it draws its view once, and asks for nothing else.
  if (location.hash === "#/small") {
    if (!document.body.classList.contains("small-window")) {
      const listing = await window.__TAURI__.core.invoke("listed");
      words = listing.words;
      takeAppearance(listing.appearance);
      document.documentElement.lang = words.locale;
      document.getElementById("small").hidden = false;
      document.getElementById("frame").hidden = true;
      showSmall(words);
    }
    return;
  }
  // What the last save could not do is said until the next save, or until Settings is left (#655):
  // dropped as soon as another page is asked, before anything is awaited — a drawing the next one
  // overtakes stops at its first answer, and would leave it to be said again.
  if (location.hash !== "#/settings") notSaved = null;
  const listing = await window.__TAURI__.core.invoke("listed");
  if (mine !== drawing) return;
  words = listing.words;
  takeAppearance(listing.appearance);
  show(listing);
  const wanted = location.hash.match(/^#\/meeting\/(.+)$/);
  // The list's address: the pane shows the last meeting beside it (D61); under 840 px, the list alone.
  const listed = location.hash === "" || location.hash === "#/";
  document.body.dataset.at = listed ? "list" : "page";
  const beside = listed && !narrow.matches && listing.sessions.length > 0;
  const name = wanted ? decodeURIComponent(wanted[1]) : beside ? nameOf(listing.sessions[0]) : null;
  // Its page opened: the bar no longer says the meeting last stopped (D92).
  if (wanted && !document.hidden) window.__TAURI__.core.invoke("meeting_seen", { name }).catch(() => {});
  // A refused language is said until its meeting's own page is left (#841).
  if (languageRefused && !(wanted && decodeURIComponent(wanted[1]) === languageRefused.name)) languageRefused = null;
  // Another meeting shown, or none: its models are read again when it is (D93).
  if (pageOffered && pageOffered.name !== name) pageOffered = null;
  // Another page, or another meeting: what was played stops — and the list alone under 840 px.
  if (player && player.name !== name) {
    player.stop();
    player = null;
  }
  const settings = location.hash === "#/settings";
  // The first run's assistant, alone in the window: no header, no other page (D58).
  const welcoming = /^#\/welcome(\/again)?$/.test(location.hash);
  document.getElementById("frame").hidden = welcoming;
  const welcome = document.getElementById("welcome");
  welcome.hidden = !welcoming;
  if (welcoming) {
    for (const other of ["meeting", "settings", "agree", "free"]) document.getElementById(other).hidden = true;
    // Drawn once while it is shown: the page read again meanwhile — a model come, a fetch ended —
    // never takes the answers from under the user.
    if (welcome.dataset.shown === location.hash) return;
    welcome.dataset.shown = location.hash;
    return showWelcome(location.hash === "#/welcome/again", words);
  }
  welcome.replaceChildren();
  delete welcome.dataset.shown;
  const freeing = location.hash === "#/free";
  document.getElementById("free").hidden = !freeing;
  if (freeing) {
    const busy = await window.__TAURI__.core.invoke("busy_meetings");
    if (mine !== drawing) return;
    showFree(listing, busy, words);
  }
  else document.getElementById("free").replaceChildren();
  const agreeing = location.hash.match(/^#\/agree\/(.+)$/);
  let agreement = null;
  if (agreeing) {
    try {
      agreement = await window.__TAURI__.core.invoke("agreement_shown", { name: decodeURIComponent(agreeing[1]) });
    } catch (why) {
      // Said in the pane: the list drawn again keeps it, until the next address.
      const said = document.getElementById("pane-said");
      said.textContent = fill(words.agreement_refused, why);
      said.hidden = false;
      // No hashchange follows a replaced address: the list's address is drawn here (D61).
      history.replaceState(null, "", "#/");
      if (!again) return refresh(true);
    }
  }
  let opened = null;
  let seen = makings;
  // Numbered as readMakings' readings are: the snapshot is kept only if no later reading answered
  // meanwhile — never the making drawn back over the stop that followed it (D92).
  let asked = makingsAsked;
  // Settings too: their models' states are drawn with the page, not at the next reading (D37).
  if (name || settings) {
    // What is being made, read before the meeting: the counts its page keeps are never ahead of
    // what it shows, never behind what was made before it was read — drawn again only for what
    // was made since.
    asked = ++makingsAsked;
    seen = await window.__TAURI__.core.invoke("makings").catch(() => makings);
  }
  if (name) {
    // A meeting no longer there, or a name that is none, goes back to the list.
    try {
      opened = await window.__TAURI__.core.invoke("opened", { name });
    } catch (why) {
      // A reading since then answers for its own address.
      if (mine !== drawing) return;
      // Gone while it played: it stops with its page.
      player?.stop();
      player = null;
      history.replaceState(null, "", "#/");
      // No hashchange follows a replaced address: said now, a name refused on that page first.
      paneSays(nameRefused ?? fill(words.meeting_not_opened, why));
      nameRefused = null;
      if (!again) return refresh(true);
    }
  }
  if (mine !== drawing) return;
  if (pageNotShown) document.getElementById("pane-said").hidden = true;
  notShown(false);
  // The settings unreadable, the folder listed is not the user's: said, never « no meeting » (#770).
  document.getElementById("empty").hidden = !listed || listing.sessions.length > 0 || listing.settings_unread;
  // Put in its live region when it comes, taken out when it goes — never drawn again meanwhile, which
  // a screen reader would say again at each listing.
  // Its sentence what is true: the defaults listed, or the last settings read held (#770).
  const live = document.getElementById("settings-banner-live");
  // Hidden on Réglages, never emptied: a live region says a change of its words, not of its
  // visibility (WAI-ARIA) — back on the list, the same banner, not said again.
  live.hidden = settings;
  const sentence = !listing.settings_unread ? null : listing.settings_kept ? words.settings_banner_kept : words.settings_banner;
  if (sentence !== (document.getElementById("settings-banner-said")?.textContent ?? null)) {
    const banner = element("div", "banner");
    banner.id = "settings-banner";
    const said = element("p", "", sentence);
    said.id = "settings-banner-said";
    const link = element("a", "link", words.settings);
    link.id = "settings-banner-link";
    link.href = "#/settings";
    banner.append(said, link);
    live.replaceChildren(...(sentence ? [banner] : []));
  }
  // No meeting yet: under 840 px, the sidebar and what to do, one under the other.
  document.body.classList.toggle("no-meeting", listing.sessions.length === 0);
  document.getElementById("agree").hidden = agreement === null;
  // Shown or emptied: never a question left hidden under another page.
  if (agreement) {
    const asked = decodeURIComponent(agreeing[1]);
    showAgreement(asked, agreement, words, listing.sessions.find((session) => nameOf(session) === asked));
  }
  else document.getElementById("agree").replaceChildren();
  document.getElementById("meeting").hidden = opened === null;
  // Not shown, not kept: a page hidden under another is never found in its place.
  if (opened === null) {
    document.getElementById("meeting").replaceChildren();
    delete document.getElementById("meeting").dataset.name;
  }
  document.getElementById("settings").hidden = !settings;
  document.getElementById("settings-link").setAttribute("aria-current", settings ? "page" : "false");
  // The meeting shown, marked in the list.
  const shownAt = opened ? `#/meeting/${encodeURIComponent(name)}` : null;
  for (const link of document.querySelectorAll("#groups .session")) {
    if (link.getAttribute("href") === shownAt) link.setAttribute("aria-current", "page");
    else link.removeAttribute("aria-current");
  }
  // Drawn with what was read before it, never with a newer reading taken meanwhile.
  // The meeting shown beside the list, then reached at its own address — its row clicked, or
  // the small window's *Lire* —, read again and unchanged: the same page, not drawn again under
  // the user (D61). Anything changed meanwhile, or any other reading, draws it anew.
  const page = document.getElementById("meeting");
  const address = listed ? "#/" : location.hash;
  const fresh = opened ? JSON.stringify(opened) : null;
  const kept = opened && page.dataset.name === name && page.dataset.at === "#/" && address !== "#/" && fresh === drawnMeeting;
  if (opened && !kept) {
    page.classList.toggle("arriving", page.childElementCount === 0);
    if (asked === makingsAsked) makings = seen;
    showMeeting(opened, words);
    drawnMeeting = fresh;
  }
  if (opened) page.dataset.at = address;
  else delete page.dataset.at;
  paintMakings();
  if (settings) {
    const [shown, minutes] = await Promise.all([
      window.__TAURI__.core.invoke("settings_shown"),
      window.__TAURI__.core.invoke("minutes_shown"),
    ]);
    if (mine !== drawing) return;
    if (asked === makingsAsked) makings = seen;
    showSettings(shown, minutes, words);
  }
}

document.getElementById("small-link").addEventListener("click", () => window.__TAURI__.core.invoke("small_window"));
document.getElementById("find").addEventListener("input", sought);
// The sidebar folded, or brought back (D61): kept in the settings; the focus follows a button gone.
async function fold(folded) {
  document.body.classList.toggle("folded", folded);
  const gone = document.getElementById(folded ? "sidebar" : "unfold");
  if (gone.contains(document.activeElement)) document.getElementById(folded ? "unfold" : "fold").focus();
  // Not kept: said; the sidebar stays as asked until the next start.
  const said = document.getElementById("pane-said");
  try {
    await window.__TAURI__.core.invoke("settings_saved", { change: { folded } });
    said.hidden = true;
  } catch (why) {
    said.textContent = fill(words.fold_not_kept, why);
    said.hidden = false;
  }
}
document.getElementById("fold").addEventListener("click", () => fold(true));
document.getElementById("unfold").addEventListener("click", () => fold(false));
// Ctrl+B, ⌘B on a Mac, outside a text field: the convention of sidebars that fold.
document.addEventListener("keydown", (event) => {
  // The key where B is, whatever the layout writes there; held down, it folds once.
  if (!(event.ctrlKey || event.metaKey) || event.altKey || event.shiftKey || event.code !== "KeyB" || event.repeat) return;
  const typing = event.target.closest?.("textarea, select, [contenteditable], input:not([type=checkbox], [type=radio], [type=button])");
  if (typing || narrow.matches || document.getElementById("frame").hidden || document.body.classList.contains("small-window")) return;
  event.preventDefault();
  fold(!document.body.classList.contains("folded"));
});
// ↑ and ↓ on the list: the meeting above or below shown in the pane, the focus on its row (D61).
document.getElementById("groups").addEventListener("keydown", (event) => {
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
  const rows = [...document.querySelectorAll("#groups li:not([hidden]) .session")];
  const at = rows.indexOf(event.target.closest(".session"));
  const next = rows[at + (event.key === "ArrowDown" ? 1 : -1)];
  if (at < 0 || !next) return;
  event.preventDefault();
  next.focus();
  location.hash = next.getAttribute("href");
});
// The empty pane's own gesture (D61's board 5): the sidebar's, offered where the eye is.
document.getElementById("empty-record").addEventListener("click", () => document.getElementById("record").click());
document.getElementById("empty-again").addEventListener("click", () => refresh());
// The purchase asked, then the licence read again by the shell: bought, nothing is said any more.
// One purchase at a time: « Acheter » is disabled until the Store has answered (D83).
document.getElementById("buy").addEventListener("click", async () => {
  const buy = document.getElementById("buy");
  if (buy.disabled) return;
  buy.disabled = true;
  try {
    paintRefused((await window.__TAURI__.core.invoke("buy")).refused);
  } finally {
    buy.disabled = false;
  }
});
document.getElementById("pane-again").addEventListener("click", () => refresh());
// Its new state at once, the command answering after it; a second click meanwhile asks nothing.
// Answered, the shell's word: a press it took for the same one, or a start that failed, shows (D68).
document.getElementById("record").addEventListener("click", async () => {
  if (recordAsked !== null) return;
  recordAsked = !document.getElementById("record").classList.contains("live");
  paintRecord(recordAsked, false);
  try {
    await window.__TAURI__.core.invoke("record");
  } finally {
    recordAsked = null;
  }
  const { recording, stopping, refused } = await window.__TAURI__.core.invoke("header");
  paintRecord(recording, stopping);
  paintRefused(refused);
  // The recording says it started, or stopped, a moment later: then the header follows.
  setTimeout(refresh, 1200);
});
// The header follows a recording started or stopped elsewhere — the tray, the shortcut — while
// the window is shown: asked lightly, every second and a half.
setInterval(async () => {
  if (document.hidden || words === null) return;
  const asked = await window.__TAURI__.core.invoke("header").catch(() => null);
  if (asked === null) return;
  lastStopped = asked.last ?? null;
  readMakings();
  const { recording, stopping, shortcut, refused } = asked;
  takeAppearance(asked.appearance);
  // The shortcut, granted or changed meanwhile: in the recording's tooltip and the empty state (D89).
  recordKeys = shortcut ?? null;
  paintRecord(recording, stopping);
  paintRefused(refused);
  const keys = document.getElementById("empty-keys");
  keys.textContent = shortcut ?? "";
  keys.hidden = !shortcut;
}, 1500);
refresh();
// Coming back to the window draws it anew — unless who writes the minutes is being chosen there
// and not yet saved: that choice stays.
window.addEventListener("focus", () => {
  // The assistant too: its answers are not drawn over.
  const choosing = !document.getElementById("welcome").hidden
    || document.querySelector("#settings:not([hidden]) #minutes-unsaved:not([hidden])")
    || [...document.querySelectorAll("#settings:not([hidden]) .fields input")].some((input) => input.value !== input.defaultValue)
    // The speakers being named (#1205): what is typed stays.
    || document.getElementById("speakers-panel");
  // Back from a list's menu, not from elsewhere: nothing drawn anew (#944).
  const menu = inMenu;
  inMenu = false;
  if (!choosing && !menu) refresh();
});
window.addEventListener("hashchange", () => {
  // Another address: the page still shown, or its agreement, acts on its own meeting: no press and
  // no focus there until what was asked is drawn — never one sent for a meeting left.
  leavePage();
  document.getElementById("pane-said").hidden = true;
  notShown(false);
  serviceAnswer = null;
  for (const path of Object.values(guided)) {
    path.answers = new Map();
    path.chosen = {};
    path.advanced = null;
  }
  if (nameRefused) paneSays(nameRefused);
  nameRefused = null;
  // Another address: the assistant, shown again, is drawn anew — even if the drawing that left it was cut short.
  delete document.getElementById("welcome").dataset.shown;
  refresh();
  window.scrollTo(0, 0);
});
