// 명령 팔레트 (Ctrl+K) — `rhwp capabilities` JSON에서 명령 목록·인자 폼을
// 자동 생성한다. 명령이 늘어도 이 파일은 안 바뀐다(자기서술 계약).

import { icon, mountIcons } from "./icons.js";
import { pickAnyFile, basename } from "./api.js";

const $ = (id) => document.getElementById(id);

// 값을 받는 것이 실측으로 확인된 플래그(v0.8.4 manual). 목록에 없으면
// 기본은 "값 입력란 열어두기" — 비워 두면 플래그 단독으로 나간다.
const BOOLEAN_FLAGS = new Set([
  "--json", "--dry-run", "--verify", "--in-place", "--keep-preview",
  "--ignore-case", "--keep-style", "--include-offpage", "--include-fields",
  "--no-raw", "--bom", "--base64", "--data-uri", "--show-para-marks",
  "--show-control-codes", "--debug-overlay", "--respect-vpos-reset",
  "--font-style", "--embed-fonts", "--annotate-metric-font", "--benchmark",
]);

const CATEGORY_LABEL = {
  query: "조회", export: "내보내기", edit: "편집", diagnostic: "진단",
  batch: "일괄", serve: "서버", internal: "내부",
};

export class Palette {
  /**
   * @param {object} opts
   *  - getCaps(): capabilities 원문 JSON
   *  - getActiveDoc(): 현재 문서 경로 | null
   *  - onRun(args, meta): 실행 요청 (meta.mutating = edit 계열)
   */
  constructor(opts) {
    this.opts = opts;
    this.sel = 0;
    this.filtered = [];
    this.overlay = $("palette");
    this.input = $("palette-input");
    this.list = $("palette-list");
    this.form = $("palette-form");
    this.count = $("palette-count");

    this.input.addEventListener("input", () => this.renderList());
    this.input.addEventListener("keydown", (e) => this.onKey(e));
    this.overlay.addEventListener("mousedown", (e) => {
      if (e.target === this.overlay) this.close();
    });
  }

  commands() {
    const caps = this.opts.getCaps();
    if (!caps?.commands) return [];
    // internal(픽스처 생성기)·serve 는 팔레트 소음이라 뒤로 보낸다. 숨기진 않는다.
    const rank = (c) => (c.category === "internal" || c.category === "serve" ? 1 : 0);
    return [...caps.commands].sort((a, b) => rank(a) - rank(b));
  }

  open() {
    const caps = this.opts.getCaps();
    this.overlay.hidden = false;
    this.form.hidden = true;
    this.list.hidden = false;
    this.input.value = "";
    this.sel = 0;
    $("palette-src").textContent = caps
      ? `capabilities v${caps.version ?? "?"} · 명령 ${caps.commands.length}개 자동 생성`
      : "capabilities 미로드";
    this.renderList();
    setTimeout(() => this.input.focus(), 0);
  }

  close() {
    this.overlay.hidden = true;
  }
  get isOpen() {
    return !this.overlay.hidden;
  }

  renderList() {
    const q = this.input.value.trim().toLowerCase();
    const all = this.commands();
    this.filtered = q
      ? all.filter(
          (c) =>
            c.name.toLowerCase().includes(q) ||
            (c.summary || "").toLowerCase().includes(q) ||
            (c.subcommands || []).some((s) => s.name.includes(q)),
        )
      : all;
    if (this.sel >= this.filtered.length) this.sel = 0;
    this.count.textContent = `${this.filtered.length}/${all.length}`;
    this.list.replaceChildren();
    this.filtered.slice(0, 60).forEach((c, i) => {
      const row = document.createElement("div");
      row.className = "palette-item" + (i === this.sel ? " sel" : "");
      const name = document.createElement("span");
      name.className = "p-name";
      name.textContent = c.name;
      const sum = document.createElement("span");
      sum.className = "p-sum";
      sum.textContent = c.summary || "";
      const cat = document.createElement("span");
      cat.className = "p-cat" + (c.category === "edit" ? " edit" : "");
      cat.textContent = CATEGORY_LABEL[c.category] || c.category || "";
      row.append(name, sum, cat);
      row.addEventListener("click", () => this.pick(c));
      row.addEventListener("mousemove", () => {
        if (this.sel !== i) { this.sel = i; this.highlight(); }
      });
      this.list.append(row);
    });
    if (!this.filtered.length) {
      const p = document.createElement("p");
      p.className = "empty-hint";
      p.style.padding = "14px";
      p.textContent = "일치하는 명령이 없습니다";
      this.list.append(p);
    }
  }

  highlight() {
    [...this.list.children].forEach((el, i) =>
      el.classList.toggle("sel", i === this.sel),
    );
  }

  onKey(e) {
    if (!this.list.hidden) {
      if (e.key === "ArrowDown") { e.preventDefault(); this.sel = Math.min(this.sel + 1, this.filtered.length - 1); this.highlight(); this.list.children[this.sel]?.scrollIntoView({ block: "nearest" }); }
      else if (e.key === "ArrowUp") { e.preventDefault(); this.sel = Math.max(this.sel - 1, 0); this.highlight(); this.list.children[this.sel]?.scrollIntoView({ block: "nearest" }); }
      else if (e.key === "Enter" && this.filtered[this.sel]) { e.preventDefault(); this.pick(this.filtered[this.sel]); }
    }
  }

  /** 2단계: 인자 폼 자동 생성. */
  pick(cmd) {
    this.list.hidden = true;
    this.form.hidden = false;
    this.form.replaceChildren();

    const head = document.createElement("div");
    head.className = "pf-head";
    const back = document.createElement("button");
    back.className = "btn small";
    back.append(icon("left"), document.createTextNode("목록"));
    back.addEventListener("click", () => { this.form.hidden = true; this.list.hidden = false; this.input.focus(); });
    const name = document.createElement("span");
    name.className = "p-name";
    name.textContent = cmd.name;
    head.append(back, name);
    this.form.append(head);

    const sum = document.createElement("p");
    sum.className = "pf-sum";
    sum.textContent = cmd.summary || "";
    this.form.append(sum);

    const mutating = cmd.category === "edit";
    if (mutating) {
      const warn = document.createElement("div");
      warn.className = "pf-warn";
      warn.append(icon("alert"), document.createTextNode(
        "문서를 바꾸는 명령입니다 — 실행하면 먼저 --dry-run 미리보기 카드가 뜨고, 승인해야 실제로 실행됩니다."));
      this.form.append(warn);
    }

    // 하위 명령
    let subSelect = null;
    if (cmd.subcommands?.length) {
      const f = document.createElement("label");
      f.className = "field";
      const l = document.createElement("span");
      l.className = "field-label";
      l.textContent = "하위 명령";
      subSelect = document.createElement("select");
      for (const s of cmd.subcommands) {
        const o = document.createElement("option");
        o.value = s.name;
        o.textContent = s.summary ? `${s.name} — ${s.summary}` : s.name;
        subSelect.append(o);
      }
      const row = document.createElement("div");
      row.className = "field-row";
      row.append(subSelect);
      f.append(l, row);
      this.form.append(f);
    }

    // 대상 문서(위치 인자)
    const df = document.createElement("label");
    df.className = "field";
    df.innerHTML =
      `<span class="field-label">대상 문서 <span style="font-weight:400;color:var(--text-faint)">(위치 인자 — 필요 없는 명령이면 비우세요)</span></span>`;
    const drow = document.createElement("div");
    drow.className = "field-row";
    const dinput = document.createElement("input");
    dinput.type = "text";
    dinput.value = this.opts.getActiveDoc() || "";
    dinput.placeholder = "문서 경로";
    const dpick = document.createElement("button");
    dpick.className = "btn";
    dpick.textContent = "찾기";
    dpick.addEventListener("click", async () => {
      const p = await pickAnyFile();
      if (p) { dinput.value = p; preview(); }
    });
    drow.append(dinput, dpick);
    df.append(drow);
    this.form.append(df);

    // 플래그 그리드
    const rows = [];
    if (cmd.flags?.length) {
      const fl = document.createElement("div");
      fl.className = "field";
      fl.innerHTML = `<span class="field-label">플래그</span>
        <span class="field-help">체크하면 포함됩니다. 값이 필요한 플래그는 옆 칸에 입력 — 비우면 플래그만 나갑니다.</span>`;
      const grid = document.createElement("div");
      grid.className = "flag-grid";
      for (const flag of cmd.flags) {
        const row = document.createElement("label");
        row.className = "flag-row";
        const cb = document.createElement("input");
        cb.type = "checkbox";
        if (flag === "--json" && cmd.json) cb.checked = true;
        const nm = document.createElement("span");
        nm.className = "f-name";
        nm.textContent = flag;
        row.append(cb, nm);
        let val = null;
        if (!BOOLEAN_FLAGS.has(flag)) {
          val = document.createElement("input");
          val.type = "text";
          val.placeholder = "값";
          val.addEventListener("input", () => { if (val.value) cb.checked = true; preview(); });
          row.append(val);
        }
        cb.addEventListener("change", preview);
        rows.push({ flag, cb, val });
        grid.append(row);
      }
      fl.append(grid);
      this.form.append(fl);
    }

    // 추가 인자(자유 입력)
    const xf = document.createElement("label");
    xf.className = "field";
    xf.innerHTML = `<span class="field-label">추가 인자 <span style="font-weight:400;color:var(--text-faint)">(그대로 뒤에 붙습니다)</span></span>`;
    const xinput = document.createElement("input");
    xinput.type = "text";
    xinput.placeholder = '예: --threshold-pt 0.5';
    const xrow = document.createElement("div");
    xrow.className = "field-row";
    xrow.append(xinput);
    xf.append(xrow);
    this.form.append(xf);

    // 미리보기 + 실행
    const pv = document.createElement("div");
    pv.className = "pf-preview";
    this.form.append(pv);

    // 함수 선언(호이스팅) — 위 플래그 행 리스너 등록 시점에도 참조 가능해야 한다.
    function buildArgs() {
      const args = [cmd.name];
      if (subSelect) args.push(subSelect.value);
      if (dinput.value.trim()) args.push(dinput.value.trim());
      for (const { flag, cb, val } of rows) {
        if (!cb.checked) continue;
        args.push(flag);
        if (val && val.value.trim()) args.push(val.value.trim());
      }
      if (xinput.value.trim()) {
        for (const tok of xinput.value.trim().match(/(?:[^\s"]+|"[^"]*")+/g) || [])
          args.push(tok.replaceAll('"', ""));
      }
      return args;
    }
    function preview() {
      pv.textContent = "rhwp " + buildArgs().map((a) => (/\s/.test(a) ? `"${a}"` : a)).join(" ");
    }
    dinput.addEventListener("input", preview);
    xinput.addEventListener("input", preview);
    subSelect?.addEventListener("change", preview);
    preview();

    const actions = document.createElement("div");
    actions.className = "pf-actions";
    const run = document.createElement("button");
    run.className = "btn primary";
    run.append(icon("play"), document.createTextNode(mutating ? "미리보기 실행" : "실행"));
    run.addEventListener("click", () => {
      this.close();
      this.opts.onRun(buildArgs(), { mutating, name: cmd.name, docLabel: basename(dinput.value.trim()) });
    });
    actions.append(run);
    this.form.append(actions);
    mountIcons(this.form);
    // 목록에서 Enter 로 넘어온 직후 다시 Enter 로 실행할 수 있게 실행 버튼에 포커스.
    run.focus();
  }
}
