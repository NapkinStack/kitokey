// The page's pictograms (24 × 24, stroke = currentColor), drawn for it, as one sprite; and the systems'
// marks. The channels' brands are not drawn here: Discord's symbol is its own file, unaltered
// (BRANDS.md); Slack and Google Calendar are named in words, as their terms ask (2026-11-26).
export const ICONS = {
  lock: '<rect x="5" y="10.5" width="14" height="10" rx="2.5"/><path d="M8 10.5V7.5a4 4 0 0 1 8 0v3"/><circle cx="12" cy="15.5" r="1.2" fill="currentColor" stroke="none"/>',
  mic: '<rect x="9" y="3" width="6" height="11" rx="3"/><path d="M5.5 11a6.5 6.5 0 0 0 13 0M12 17.5V21M8.5 21h7"/>',
  transcript: '<path d="M5 4.5h14a1.5 1.5 0 0 1 1.5 1.5v9a1.5 1.5 0 0 1-1.5 1.5H11l-4.5 3.5V16.5H5A1.5 1.5 0 0 1 3.5 15V6A1.5 1.5 0 0 1 5 4.5z"/><path d="M7.5 8.5h9M7.5 12h6"/>',
  summary: '<path d="M7 3h7l4 4v13a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1z"/><path d="M14 3v4h4M9 14l2 2 4-4"/>',
  calendar: '<rect x="4" y="5.5" width="16" height="15" rx="2.5"/><path d="M4 10h16M8.5 3.5v4M15.5 3.5v4"/><circle cx="12" cy="15" r="1.3" fill="currentColor" stroke="none"/>',
  send: '<path d="M21 3 10 14M21 3l-7 18-4-7-7-4z"/>',
  search: '<circle cx="11" cy="11" r="6.5"/><path d="m16 16 4.5 4.5"/>',
  export: '<path d="M12 3v12M7.5 10.5 12 15l4.5-4.5M5 19.5h14"/>',
  play: '<circle cx="12" cy="12" r="9"/><path d="M10 8.5v7l6-3.5z" fill="currentColor" stroke="none"/>',
  webhook: '<circle cx="6.5" cy="17" r="2.6"/><circle cx="17.5" cy="17" r="2.6"/><circle cx="12" cy="6.5" r="2.6"/><path d="M10.6 8.8 7.8 14.6M13.4 8.8l2.8 5.8M9.1 17h5.8"/>',
  code: '<path d="m9 7-5 5 5 5M15 7l5 5-5 5"/>',
};
export const BRANDS = {
  fedora: '<circle cx="12" cy="12" r="11" fill="#3C6EB4"/><path d="M12.2 18.5V10a3 3 0 0 1 3-3M8.8 12.3h6.4" fill="none" stroke="#fff" stroke-width="2.3" stroke-linecap="round"/>',
  windows: '<g fill="#0078D4"><rect x="2.5" y="2.5" width="9" height="9" rx=".6"/><rect x="12.5" y="2.5" width="9" height="9" rx=".6"/><rect x="2.5" y="12.5" width="9" height="9" rx=".6"/><rect x="12.5" y="12.5" width="9" height="9" rx=".6"/></g>',
  ubuntu: '<circle cx="12" cy="12" r="11" fill="#E95420"/><circle cx="12" cy="12" r="5.2" fill="none" stroke="#fff" stroke-width="1.8"/><circle cx="6.4" cy="12" r="1.9" fill="#fff" stroke="#E95420" stroke-width="1"/><circle cx="14.8" cy="7.1" r="1.9" fill="#fff" stroke="#E95420" stroke-width="1"/><circle cx="14.8" cy="16.9" r="1.9" fill="#fff" stroke="#E95420" stroke-width="1"/>',
  apple: '<path fill="currentColor" d="M16.5 12.7c0-2.1 1.7-3.1 1.8-3.2-1-1.4-2.5-1.6-3-1.7-1.3-.1-2.5.8-3.1.8-.7 0-1.6-.8-2.7-.7-1.4 0-2.7.8-3.4 2.1-1.5 2.5-.4 6.3 1 8.3.7 1 1.5 2.1 2.6 2.1 1-.1 1.4-.7 2.7-.7 1.2 0 1.6.7 2.7.6 1.1 0 1.8-1 2.5-2 .8-1.1 1.1-2.2 1.1-2.3 0 0-2.2-.8-2.2-3.3zM14.4 6.5c.6-.7 1-1.7.9-2.7-.9 0-1.9.6-2.5 1.3-.5.6-1 1.6-.9 2.6 1 .1 1.9-.5 2.5-1.2z"/>',
};
export const ico = (name, cls = "ico") =>
  `<svg class="${cls}" viewBox="0 0 24 24" aria-hidden="true" focusable="false"><use href="#i-${name}"/></svg>`;
export const brand = (name) =>
  `<svg class="os-logo" viewBox="0 0 24 24" aria-hidden="true" focusable="false">${BRANDS[name]}</svg>`;
export const sprite = Object.entries(ICONS).map(([k, v]) =>
  `<symbol id="i-${k}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">${v}</symbol>`).join("");
