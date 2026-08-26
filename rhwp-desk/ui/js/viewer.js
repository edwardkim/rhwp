// 보조 문서 패널 — 카드에서 "문서 보기"로 열리는 존재.
// rhwp-desk 는 뷰어가 아니다: 렌더도 계약 경로(export-svg --json)의 산출을
// 그대로 보여줄 뿐, 편집 UI 는 없다(편집은 명령으로만).

import { renderPage, basename } from "./api.js";

const $ = (id) => document.getElementById(id);

export class Viewer {
  constructor({ getEngine, onError, onInfo }) {
    this.getEngine = getEngine;
    this.onError = onError;
    this.onInfo = onInfo; // 문서 info 봉투 조회 콜백(있으면 하단에 표시)
    this.doc = null;
    this.page = 1;
    this.pageCount = 0;
    this.zoom = 1;
    this.fitWidth = true;
    this.cache = new Map(); // `${doc}#${page}` -> svg

    $("viewer-close").addEventListener("click", () => this.hide());
    $("pg-prev").addEventListener("click", () => this.go(this.page - 1));
    $("pg-next").addEventListener("click", () => this.go(this.page + 1));
    $("pg-input").addEventListener("change", (e) => this.go(parseInt(e.target.value, 10) || 1));
    $("zoom-in").addEventListener("click", () => this.setZoom(this.zoom * 1.2));
    $("zoom-out").addEventListener("click", () => this.setZoom(this.zoom / 1.2));
    $("zoom-fit").addEventListener("click", () => { this.fitWidth = true; this.applyZoom(); });
    new ResizeObserver(() => { if (this.fitWidth) this.applyZoom(); })
      .observe($("viewer-body"));
  }

  get visible() { return !$("viewer-pane").hidden; }

  show() { $("viewer-pane").hidden = false; }
  hide() { $("viewer-pane").hidden = true; }
  toggle() { $("viewer-pane").hidden = !$("viewer-pane").hidden; }

  /** 문서를 패널에 연다. page 생략 시 1쪽. */
  async open(docPath, page = 1) {
    this.doc = docPath;
    this.page = page;
    this.show();
    $("viewer-title").textContent = basename(docPath);
    await this.load();
  }

  async go(page) {
    if (!this.doc) return;
    const p = Math.min(Math.max(1, page), this.pageCount || page);
    if (p === this.page && $("svg-host").firstChild) {
      $("pg-input").value = p;
      return;
    }
    this.page = p;
    await this.load();
  }

  async load() {
    const host = $("svg-host");
    const key = `${this.doc}#${this.page}`;
    $("pg-input").value = this.page;
    try {
      let svg = this.cache.get(key);
      if (!svg) {
        host.style.opacity = ".4";
        const res = await renderPage(this.getEngine(), this.doc, this.page);
        svg = res.svg;
        this.pageCount = res.pageCount;
        this.cache.set(key, svg);
        if (this.cache.size > 24) this.cache.delete(this.cache.keys().next().value);
      }
      host.innerHTML = svg;
      host.style.opacity = "1";
      $("pg-total").textContent = this.pageCount || "?";
      const el = host.querySelector("svg");
      if (el) {
        // 원본 크기 기억(줌 기준)
        const w = parseFloat(el.getAttribute("width")) || el.viewBox?.baseVal?.width || 794;
        const h = parseFloat(el.getAttribute("height")) || el.viewBox?.baseVal?.height || 1123;
        this.natural = { w, h };
        if (!el.getAttribute("viewBox")) el.setAttribute("viewBox", `0 0 ${w} ${h}`);
        this.applyZoom();
      }
      this.renderInfo();
    } catch (e) {
      host.style.opacity = "1";
      this.onError?.(String(e));
    }
  }

  setZoom(z) {
    this.fitWidth = false;
    this.zoom = Math.min(4, Math.max(0.2, z));
    this.applyZoom();
  }

  applyZoom() {
    const el = $("svg-host").querySelector("svg");
    if (!el || !this.natural) return;
    if (this.fitWidth) {
      const avail = $("viewer-body").clientWidth - 36;
      this.zoom = Math.max(0.2, avail / this.natural.w);
    }
    el.setAttribute("width", Math.round(this.natural.w * this.zoom));
    el.setAttribute("height", Math.round(this.natural.h * this.zoom));
    $("zoom-label").textContent = `${Math.round(this.zoom * 100)}%`;
  }

  /** 문서 정보(info 봉투)를 하단 스트립에. 봉투가 없으면 비운다. */
  renderInfo() {
    const box = $("viewer-info");
    const env = this.onInfo?.(this.doc);
    box.replaceChildren();
    if (!env) return;
    const add = (label, val) => {
      if (val === undefined || val === null || val === "") return;
      const kv = document.createElement("span");
      kv.className = "kv";
      const b = document.createElement("b");
      b.textContent = label;
      kv.append(b, `: ${val}`);
      box.append(kv);
    };
    add("형식", env.format);
    add("쪽수", env.pageCount);
    add("문단", env.paraCount);
    add("구역", env.sections);
    add("크기", env.sizeBytes ? `${(env.sizeBytes / 1024).toFixed(0)}KB` : null);
    add("제목", env.title);
    if (Array.isArray(env.fonts) && env.fonts.length) {
      add("글꼴", [...new Set(env.fonts)].slice(0, 6).join(", "));
    }
  }

  /** 문서가 바뀌었을 수 있을 때(편집 후) 캐시를 비운다. */
  invalidate(docPath) {
    for (const k of [...this.cache.keys()]) {
      if (k.startsWith(docPath + "#")) this.cache.delete(k);
    }
  }
}
