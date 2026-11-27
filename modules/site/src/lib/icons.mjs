// The page's pictograms (24 × 24, stroke = currentColor), drawn for it, as one sprite. No brand is drawn
// here: a brand's mark is its owner's own file, unaltered, in public/img/brands/ (BRANDS.md); Slack and
// Google Calendar are named in words.
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
  unlock: '<rect x="5" y="10.5" width="14" height="10" rx="2.5"/><path d="M8 10.5V7.5a4 4 0 0 1 7.6-1.8"/><circle cx="12" cy="15.5" r="1.2" fill="currentColor" stroke="none"/>',
  webhook: '<circle cx="6.5" cy="17" r="2.6"/><circle cx="17.5" cy="17" r="2.6"/><circle cx="12" cy="6.5" r="2.6"/><path d="M10.6 8.8 7.8 14.6M13.4 8.8l2.8 5.8M9.1 17h5.8"/>',
  code: '<path d="m9 7-5 5 5 5M15 7l5 5-5 5"/>',
};
export const ico = (name, cls = "ico") =>
  `<svg class="${cls}" viewBox="0 0 24 24" aria-hidden="true" focusable="false"><use href="#i-${name}"/></svg>`;
export const sprite = Object.entries(ICONS).map(([k, v]) =>
  `<symbol id="i-${k}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">${v}</symbol>`).join("");
