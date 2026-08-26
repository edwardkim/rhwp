// 인라인 SVG 아이콘 — 외부 CDN 없이 오프라인 동작 (설계 원칙).
// stroke: currentColor 라 테마 색을 그대로 따른다.

const S = (body, vb = "0 0 24 24") =>
  `<svg viewBox="${vb}" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" xmlns="http://www.w3.org/2000/svg">${body}</svg>`;

export const ICONS = {
  logo: S(`<rect x="3" y="3" width="18" height="18" rx="4"/><path d="M8 12h8M8 8h8M8 16h5"/><circle cx="17.5" cy="16" r="1.4" fill="currentColor" stroke="none"/>`),
  "logo-big": S(`<rect x="3" y="3" width="18" height="18" rx="4"/><path d="M8 12h8M8 8h8M8 16h5"/><circle cx="17.5" cy="16" r="1.4" fill="currentColor" stroke="none"/>`),
  search: S(`<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>`),
  doc: S(`<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/>`),
  folder: S(`<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>`),
  panel: S(`<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M14 4v16"/>`),
  moon: S(`<path d="M21 13A8.5 8.5 0 0 1 11 3a8.5 8.5 0 1 0 10 10z"/>`),
  sun: S(`<circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M4.9 4.9l1.4 1.4m11.4 11.4 1.4 1.4M2 12h2m16 0h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>`),
  gear: S(`<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .34 1.87l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.7 1.7 0 0 0-1.87-.34 1.7 1.7 0 0 0-1 1.55V21a2 2 0 1 1-4 0v-.09a1.7 1.7 0 0 0-1-1.55 1.7 1.7 0 0 0-1.87.34l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.7 1.7 0 0 0 .34-1.87 1.7 1.7 0 0 0-1.55-1H3a2 2 0 1 1 0-4h.09a1.7 1.7 0 0 0 1.55-1 1.7 1.7 0 0 0-.34-1.87l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.7 1.7 0 0 0 1.87.34h0a1.7 1.7 0 0 0 1-1.55V3a2 2 0 1 1 4 0v.09a1.7 1.7 0 0 0 1 1.55h0a1.7 1.7 0 0 0 1.87-.34l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.7 1.7 0 0 0-.34 1.87v0a1.7 1.7 0 0 0 1.55 1H21a2 2 0 1 1 0 4h-.09a1.7 1.7 0 0 0-1.55 1z"/>`),
  queue: S(`<path d="M4 6h16M4 12h16M4 18h10"/><circle cx="19" cy="18" r="2"/>`),
  journal: S(`<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20V4H6.5A2.5 2.5 0 0 0 4 6.5z"/><path d="M9 8h7M9 11h5"/>`),
  close: S(`<path d="M18 6 6 18M6 6l12 12"/>`),
  left: S(`<path d="m15 18-6-6 6-6"/>`),
  right: S(`<path d="m9 18 6-6-6-6"/>`),
  minus: S(`<path d="M5 12h14"/>`),
  plus: S(`<path d="M12 5v14M5 12h14"/>`),
  fit: S(`<path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"/>`),
  copy: S(`<rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>`),
  send: S(`<path d="m22 2-7 20-4-9-9-4z"/><path d="M22 2 11 13"/>`),
  drop: S(`<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="M7 10l5 5 5-5"/><path d="M12 15V3"/>`),
  shield: S(`<path d="M12 22s8-3.5 8-10V5l-8-3-8 3v7c0 6.5 8 10 8 10z"/><path d="m9 12 2 2 4-4"/>`),
  text: S(`<path d="M4 7V5h16v2"/><path d="M12 5v14"/><path d="M9 19h6"/>`),
  pdf: S(`<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/><path d="M9 15h1.5a1.5 1.5 0 0 0 0-3H9v6"/>`),
  info: S(`<circle cx="12" cy="12" r="9"/><path d="M12 8h.01M12 11v5"/>`),
  play: S(`<path d="m6 4 14 8-14 8z"/>`),
  eye: S(`<path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7z"/><circle cx="12" cy="12" r="3"/>`),
  terminal: S(`<rect x="3" y="4" width="18" height="16" rx="2"/><path d="m7 9 3 3-3 3M13 15h4"/>`),
  check: S(`<path d="m5 13 4 4L19 7"/>`),
  alert: S(`<path d="M12 3 2 20h20z"/><path d="M12 9v5M12 17h.01"/>`),
};

/** data-icon 속성이 붙은 모든 요소에 아이콘을 주입한다. */
export function mountIcons(root = document) {
  for (const el of root.querySelectorAll("[data-icon]")) {
    const name = el.getAttribute("data-icon");
    if (ICONS[name] && !el.firstChild) el.innerHTML = ICONS[name];
  }
}

/** 아이콘 span 을 만든다. */
export function icon(name) {
  const span = document.createElement("span");
  span.setAttribute("data-icon", name);
  span.innerHTML = ICONS[name] || "";
  return span;
}
