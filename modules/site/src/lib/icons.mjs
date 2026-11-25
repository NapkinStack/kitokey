// The page's pictograms (24 × 24, stroke = currentColor), drawn for it, as one sprite; and the systems' and
// channels' marks. Discord's is Simple Icons' rendition (CC0) in its colour, its official file a launch blocker
// (the coordinator, 2026-11-25). Slack's mark is not drawn: its terms forbid distributing it without a written
// licence, so Slack is named beside the neutral « channel » pictogram until the decider settles it (2026-11-26).
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
  channel: '<path d="M10 4 8 20M16 4l-2 16M4.5 9h15M3.5 15h15"/>',
  webhook: '<circle cx="6.5" cy="17" r="2.6"/><circle cx="17.5" cy="17" r="2.6"/><circle cx="12" cy="6.5" r="2.6"/><path d="M10.6 8.8 7.8 14.6M13.4 8.8l2.8 5.8M9.1 17h5.8"/>',
  code: '<path d="m9 7-5 5 5 5M15 7l5 5-5 5"/>',
};
export const BRANDS = {
  fedora: '<circle cx="12" cy="12" r="11" fill="#3C6EB4"/><path d="M12.2 18.5V10a3 3 0 0 1 3-3M8.8 12.3h6.4" fill="none" stroke="#fff" stroke-width="2.3" stroke-linecap="round"/>',
  windows: '<g fill="#0078D4"><rect x="2.5" y="2.5" width="9" height="9" rx=".6"/><rect x="12.5" y="2.5" width="9" height="9" rx=".6"/><rect x="2.5" y="12.5" width="9" height="9" rx=".6"/><rect x="12.5" y="12.5" width="9" height="9" rx=".6"/></g>',
  ubuntu: '<circle cx="12" cy="12" r="11" fill="#E95420"/><circle cx="12" cy="12" r="5.2" fill="none" stroke="#fff" stroke-width="1.8"/><circle cx="6.4" cy="12" r="1.9" fill="#fff" stroke="#E95420" stroke-width="1"/><circle cx="14.8" cy="7.1" r="1.9" fill="#fff" stroke="#E95420" stroke-width="1"/><circle cx="14.8" cy="16.9" r="1.9" fill="#fff" stroke="#E95420" stroke-width="1"/>',
  discord: '<path fill="#5865F2" d="M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z"/>',
  apple: '<path fill="currentColor" d="M16.5 12.7c0-2.1 1.7-3.1 1.8-3.2-1-1.4-2.5-1.6-3-1.7-1.3-.1-2.5.8-3.1.8-.7 0-1.6-.8-2.7-.7-1.4 0-2.7.8-3.4 2.1-1.5 2.5-.4 6.3 1 8.3.7 1 1.5 2.1 2.6 2.1 1-.1 1.4-.7 2.7-.7 1.2 0 1.6.7 2.7.6 1.1 0 1.8-1 2.5-2 .8-1.1 1.1-2.2 1.1-2.3 0 0-2.2-.8-2.2-3.3zM14.4 6.5c.6-.7 1-1.7.9-2.7-.9 0-1.9.6-2.5 1.3-.5.6-1 1.6-.9 2.6 1 .1 1.9-.5 2.5-1.2z"/>',
};
export const ico = (name, cls = "ico") =>
  `<svg class="${cls}" viewBox="0 0 24 24" aria-hidden="true" focusable="false"><use href="#i-${name}"/></svg>`;
export const brand = (name) =>
  `<svg class="os-logo" viewBox="0 0 24 24" aria-hidden="true" focusable="false">${BRANDS[name]}</svg>`;
export const sprite = Object.entries(ICONS).map(([k, v]) =>
  `<symbol id="i-${k}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">${v}</symbol>`).join("");
