// rhwp-desk 부트스트랩·배선 — 에이전트 관제판.
// 기본 화면은 작업 카드 스트림 + 작업 큐이며, 문서 뷰는 보조 패널이다.

import { mountIcons, icon, ICONS } from "./icons.js";
import * as api from "./api.js";
import * as cards from "./cards.js";
import { Palette } from "./palette.js";
import { Viewer } from "./viewer.js";
import { BatchRunner } from "./batch.js";
import { runAgentTask } from "./agent.js";
import { attachSuggestions, cliCommandFor } from "./ontology.js";

const $ = (id) => document.getElementById(id);
const LS = {
  get: (k, d) => { try { return JSON.parse(localStorage.getItem("rhwpDesk." + k)) ?? d; } catch { return d; } },
  set: (k, v) => localStorage.setItem("rhwpDesk." + k, JSON.stringify(v)),
};

/* ══════════ 전역 상태 ══════════ */
const state = {
  engine: null,          // {path, source, version}
  caps: null,            // capabilities 원문
  mcp: null,             // capabilities --mcp 원문
  tools: { openaiTools: null, byName: new Map() },
  docs: new Map(),       // path -> {path, info: envelope|null, axes: {}}
  activeDoc: null,
  queue: [],             // {id, label, total, done, failed, status, cancelled, detail}
  profiles: LS.get("plannerProfiles", { list: [], activeId: null }),
  sessionKeys: new Map(),// profileId -> 세션 한정 키
  allowBody: false,      // 문서 본문 LLM 전송 — 기본 차단, 세션 한정
  pendingApprovals: new Map(), // id -> {label, path, resolveApprove, resolveReject}
};

/* ══════════ 오늘 완료한 작업 (날짜 경계에서 리셋) ══════════ */
function todayKey() {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}
function bumpDoneToday() {
  const rec = LS.get("doneToday", { date: "", count: 0 });
  const key = todayKey();
  if (rec.date !== key) { rec.date = key; rec.count = 0; }
  rec.count += 1;
  LS.set("doneToday", rec);
  renderDashboard();
}
function getDoneToday() {
  const rec = LS.get("doneToday", { date: "", count: 0 });
  return rec.date === todayKey() ? rec.count : 0;
}

/* ══════════ 토스트 ══════════ */
function toast(msg, kind = "") {
  const el = document.createElement("div");
  el.className = "toast " + kind;
  el.textContent = msg;
  $("toasts").append(el);
  setTimeout(() => el.remove(), 4200);
}

/* ══════════ 작업 큐 ══════════ */
const queue = {
  start(label, total = 0) {
    const q = { id: crypto.randomUUID(), label, total, done: 0, failed: 0, status: "run", cancelled: false, detail: "" };
    state.queue.unshift(q);
    renderQueue();
    return q;
  },
  step(q, detail) { q.detail = detail; renderQueue(); },
  progress(q, done, failed) { q.done = done; if (failed !== undefined) q.failed = failed; renderQueue(); },
  finish(q, ok) { q.status = ok ? "done" : "failed"; q.detail = ""; renderQueue(); },
  isCancelled: (q) => q.cancelled,
};

function renderQueue() {
  const box = $("queue-list");
  box.replaceChildren();
  if (!state.queue.length) {
    box.innerHTML = `<p class="empty-hint">대기 중인 작업이 없습니다</p>`;
    return;
  }
  for (const q of state.queue.slice(0, 12)) {
    const el = document.createElement("div");
    el.className = "queue-item " + (q.status === "done" ? "done" : q.status === "failed" ? "failed" : "");
    const head = document.createElement("div");
    head.className = "q-head";
    const label = document.createElement("span");
    label.className = "q-label";
    label.textContent = q.label;
    label.title = q.label;
    const count = document.createElement("span");
    count.className = "q-count";
    count.textContent = q.status === "run"
      ? (q.total ? `${q.done}/${q.total}` : "진행 중")
      : q.status === "done" ? "완료" : "실패/중단";
    head.append(label, count);
    if (q.status === "run") {
      const cancel = document.createElement("button");
      cancel.className = "btn small";
      cancel.textContent = "취소";
      cancel.addEventListener("click", () => { q.cancelled = true; });
      head.append(cancel);
    }
    el.append(head);
    if (q.detail) {
      const d = document.createElement("div");
      d.className = "q-count";
      d.style.marginTop = "4px";
      d.textContent = q.detail;
      el.append(d);
    }
    const bar = document.createElement("div");
    bar.className = "queue-bar";
    const fill = document.createElement("i");
    fill.style.width = q.status !== "run" ? "100%" : q.total ? `${(q.done / q.total) * 100}%` : "30%";
    bar.append(fill);
    el.append(bar);
    if (q.failed) {
      const f = document.createElement("div");
      f.className = "q-fails";
      f.textContent = `실패 ${q.failed}건 (격리 후 계속)`;
      el.append(f);
    }
    box.append(el);
  }
}

/* ══════════ 문서 관리 ══════════ */
const AXES = ["hidden-text", "injection", "unicode"];

function getDoc(path) {
  if (!state.docs.has(path)) {
    state.docs.set(path, { path, info: null, axes: {} });
  }
  return state.docs.get(path);
}

async function openDocument(path, { activate = true } = {}) {
  if (!api.isDocPath(path)) { toast("HWP/HWPX 문서가 아닙니다: " + api.basename(path), "error"); return; }
  const doc = getDoc(path);
  if (activate) state.activeDoc = path;
  addRecent(path);
  renderDocs();
  if (!doc.info) {
    try {
      const entry = await api.runTool(state.engine.path, ["info", path, "--json"], "info");
      onEntry(entry);
      if (entry.exitCode === 0 && entry.envelope) doc.info = entry.envelope;
    } catch (e) { toast(String(e), "error"); }
    renderDocs();
  }
}

function addRecent(path) {
  const list = LS.get("recent", []).filter((p) => p !== path);
  list.unshift(path);
  LS.set("recent", list.slice(0, 12));
  renderRecent();
}

function axisDot(status) {
  const el = document.createElement("span");
  el.className = "axis " + (status || "");
  return el;
}

function axisStatus(doc, axis) {
  const env = doc.axes[axis];
  if (env === "run") return "run";
  if (!env) return "";
  if (env.clean === true) return "ok";
  const n = env.findingCount ?? env.signalCount ?? env.hiddenCharCount ?? (Array.isArray(env.hiddenText) ? env.hiddenText.length : 0);
  return n > 0 ? "bad" : "ok";
}

function renderDocs() {
  // 사이드바 목록
  const box = $("doc-list");
  box.replaceChildren();
  if (!state.docs.size) {
    box.innerHTML = `<p class="empty-hint">문서를 끌어다 놓거나 [문서 열기]</p>`;
  }
  for (const doc of state.docs.values()) {
    const el = document.createElement("div");
    el.className = "doc-item" + (doc.path === state.activeDoc ? " active" : "");
    const ic = document.createElement("span");
    ic.className = "d-icon";
    ic.innerHTML = ICONS.doc;
    const name = document.createElement("span");
    name.className = "d-name";
    name.textContent = api.basename(doc.path);
    name.title = doc.path;
    const badges = document.createElement("span");
    badges.className = "d-badges";
    for (const a of AXES) badges.append(axisDot(axisStatus(doc, a)));
    el.append(ic, name, badges);
    el.addEventListener("click", () => { state.activeDoc = doc.path; renderDocs(); });
    el.addEventListener("dblclick", () => viewer.open(doc.path));
    box.append(el);
  }
  // 상단 문서 스트립(칩 + 액션)
  const strip = $("doc-strip");
  strip.replaceChildren();
  strip.hidden = !state.docs.size;
  for (const doc of state.docs.values()) {
    strip.append(docChip(doc));
  }
  renderDashboard();
}

function docChip(doc) {
  const chip = document.createElement("div");
  chip.className = "doc-chip" + (doc.path === state.activeDoc ? " active" : "");
  const name = document.createElement("span");
  name.className = "c-name";
  name.textContent = api.basename(doc.path);
  name.title = doc.path;
  name.style.cursor = "pointer";
  name.addEventListener("click", () => { state.activeDoc = doc.path; renderDocs(); });
  chip.append(name);
  if (doc.info) {
    const meta = document.createElement("span");
    meta.className = "c-meta";
    meta.textContent = `${doc.info.format ?? "?"} · ${doc.info.pageCount ?? "?"}쪽`;
    chip.append(meta);
  }
  const axes = document.createElement("span");
  axes.className = "c-axes";
  axes.title = "검증 3축: 은닉 텍스트 · 주입 신호 · 유니코드 기만";
  for (const a of AXES) axes.append(axisDot(axisStatus(doc, a)));
  chip.append(axes);

  const mk = (label, title, fn) => {
    const b = document.createElement("button");
    b.className = "btn";
    b.textContent = label;
    b.title = title;
    b.addEventListener("click", fn);
    return b;
  };
  chip.append(
    mk("검증", "inspect 3축 스윕 — 결과는 카드와 배지로", () => verifyDoc(doc.path)),
    mk("보기", "보조 문서 패널에서 페이지 렌더", () => viewer.open(doc.path)),
    mk("텍스트", "쪽별 TXT 추출 (문서 폴더/rhwp-out)", () => exportText(doc.path)),
    mk("PDF", "PDF 내보내기 (문서 폴더/rhwp-out)", () => exportPdf(doc.path)),
  );
  const close = document.createElement("button");
  close.className = "btn icon-btn";
  close.innerHTML = ICONS.close;
  close.title = "목록에서 제거(파일은 그대로)";
  close.addEventListener("click", () => {
    state.docs.delete(doc.path);
    if (state.activeDoc === doc.path) state.activeDoc = state.docs.keys().next().value ?? null;
    renderDocs();
  });
  chip.append(close);
  return chip;
}

/* ══════════ 관제판(Fleet Dashboard) — 기본 화면 ══════════ */
function docHasAttention(doc) {
  return AXES.some((a) => axisStatus(doc, a) === "bad");
}
function docAttentionReasons(doc) {
  const LABEL = { "hidden-text": "은닉 텍스트", injection: "주입 신호", unicode: "유니코드 기만" };
  return AXES.filter((a) => axisStatus(doc, a) === "bad").map((a) => LABEL[a]);
}
function docIsChecked(doc) {
  return AXES.every((a) => doc.axes[a] && doc.axes[a] !== "run");
}

function renderDashboard() {
  const panel = $("fleet-dashboard");
  if (!panel) return;
  const docs = [...state.docs.values()];
  const checked = docs.filter(docIsChecked);
  const attention = checked.filter(docHasAttention);
  const ok = checked.filter((d) => !docHasAttention(d));
  const pending = [...state.pendingApprovals.values()];

  $("fs-total").textContent = docs.length;
  $("fs-ok").textContent = ok.length;
  $("fs-attention").textContent = attention.length;
  $("fs-pending").textContent = pending.length;
  $("fs-done-today").textContent = getDoneToday();

  // 주의 문서 목록
  const attBox = $("fleet-attention");
  attBox.replaceChildren();
  if (!attention.length) {
    attBox.innerHTML = `<p class="empty-hint">${docs.length ? "검증 통과 — 주의할 문서가 없습니다" : "검증을 돌리면 발견된 문제가 여기 모입니다"}</p>`;
  } else {
    for (const doc of attention) {
      const row = document.createElement("div");
      row.className = "fleet-row";
      const name = document.createElement("span");
      name.className = "fr-name";
      name.textContent = api.basename(doc.path);
      name.title = doc.path;
      const reason = document.createElement("span");
      reason.className = "fr-badge";
      reason.textContent = docAttentionReasons(doc).join(" · ");
      row.append(name, reason);
      row.addEventListener("click", () => { state.activeDoc = doc.path; renderDocs(); viewer.open(doc.path); });
      attBox.append(row);
    }
  }

  // 승인 대기 목록
  const apBox = $("fleet-approvals");
  apBox.replaceChildren();
  if (!pending.length) {
    apBox.innerHTML = `<p class="empty-hint">문서를 바꾸는 작업은 실행 전 여기서 승인합니다</p>`;
  } else {
    for (const [id, p] of state.pendingApprovals) {
      const row = document.createElement("div");
      row.className = "fleet-row";
      const name = document.createElement("span");
      name.className = "fr-name";
      name.textContent = p.label;
      name.title = p.path || p.label;
      const badge = document.createElement("span");
      badge.className = "fr-badge warn";
      badge.textContent = "승인 대기";
      const actions = document.createElement("span");
      actions.className = "fr-actions";
      const ok2 = document.createElement("button");
      ok2.className = "btn small primary"; ok2.textContent = "승인";
      ok2.addEventListener("click", (ev) => { ev.stopPropagation(); p.approve(); });
      const no2 = document.createElement("button");
      no2.className = "btn small"; no2.textContent = "거절";
      no2.addEventListener("click", (ev) => { ev.stopPropagation(); p.reject(); });
      actions.append(ok2, no2);
      row.append(name, badge, actions);
      row.addEventListener("click", () => { const c = document.querySelector(`[data-entry-id="${id}"]`); c?.scrollIntoView({ behavior: "smooth", block: "center" }); });
      apBox.append(row);
    }
  }
  panel.hidden = false;
}

/** 승인 카드를 만들고 관제판 승인 대기 목록에도 등록한다(카드/관제판 어느 쪽에서든 승인·거절 가능). */
function registerApproval(id, label, path, { onApprove, onReject }) {
  let settled = false;
  const settle = (fn) => {
    if (settled) return;
    settled = true;
    state.pendingApprovals.delete(id);
    renderDashboard();
    fn();
  };
  state.pendingApprovals.set(id, {
    label, path,
    approve: () => settle(onApprove),
    reject: () => settle(onReject),
  });
  renderDashboard();
  return {
    settleApprove: () => state.pendingApprovals.get(id)?.approve(),
    settleReject: () => state.pendingApprovals.get(id)?.reject(),
  };
}

function renderRecent() {
  const box = $("recent-list");
  box.replaceChildren();
  const list = LS.get("recent", []);
  if (!list.length) { box.innerHTML = `<p class="empty-hint">아직 없습니다</p>`; return; }
  for (const p of list) {
    const el = document.createElement("div");
    el.className = "doc-item";
    const ic = document.createElement("span");
    ic.className = "d-icon";
    ic.innerHTML = ICONS.journal;
    const name = document.createElement("span");
    name.className = "d-name";
    name.textContent = api.basename(p);
    name.title = p;
    el.append(ic, name);
    el.addEventListener("click", async () => {
      if ((await api.pathKind(p)) === "file") openDocument(p);
      else { toast("파일이 더 이상 없습니다", "error"); LS.set("recent", LS.get("recent", []).filter((x) => x !== p)); renderRecent(); }
    });
    box.append(el);
  }
}

/* ══════════ 카드 공통 ══════════ */
function onEntry(entry, opts = {}) {
  const card = cards.addToolCard(entry, { onViewDoc: (p) => viewer.open(p), ...opts });
  // 관제판 등재 — 봉투에 source가 있으면(batch/agent가 연 문서 포함) 관리 목록에 올린다.
  const src = entry.envelope?.source ? String(entry.envelope.source) : null;
  if (src && api.isDocPath(src)) {
    const doc = getDoc(src);
    // 검증 축 배지 갱신
    if (entry.command === "inspect") {
      const axis = entry.args[1];
      if (AXES.includes(axis)) doc.axes[axis] = entry.envelope;
    }
    renderDocs();
  }
  if (entry.command === "export-text" && entry.envelope) cards.attachTextPreview(card, entry.envelope);
  if (!opts.historical && (entry.exitCode === 0 || entry.exitCode === 3)) bumpDoneToday();
  attachSuggestions(card, entry, (nextTool) => {
    palette.open();
    const input = $("palette-input");
    input.value = cliCommandFor(nextTool);
    input.dispatchEvent(new Event("input"));
  });
  return card;
}

/* ══════════ 검증 스윕 ══════════ */
async function verifyDoc(path) {
  const doc = getDoc(path);
  const axes = [...AXES];
  if (state.caps?.commands?.some((c) => c.name === "layout-anomaly")) axes.push("layout-anomaly");
  const q = queue.start(`검증: ${api.basename(path)}`, axes.length);
  let done = 0, bad = 0;
  for (const axis of axes) {
    if (queue.isCancelled(q)) break;
    doc.axes[axis] = "run";
    renderDocs();
    queue.step(q, `inspect ${axis}`);
    try {
      const args = axis === "layout-anomaly" ? [axis, path, "--json"] : ["inspect", axis, path, "--json"];
      const entry = await api.runTool(state.engine.path, args, "verify");
      onEntry(entry);
      if (entry.exitCode !== 0 && entry.exitCode !== 3) bad++;
      if (!entry.envelope) doc.axes[axis] = undefined;
    } catch (e) {
      doc.axes[axis] = undefined;
      toast(String(e), "error");
      bad++;
    }
    queue.progress(q, ++done, bad);
  }
  renderDocs();
  queue.finish(q, bad === 0);
}

/* ══════════ 내보내기 ══════════ */
async function exportText(path) {
  const outDir = api.dirname(path) + "\\rhwp-out";
  try {
    const entry = await api.runTool(state.engine.path, ["export-text", path, "-o", outDir], "export");
    onEntry(entry);
    if (entry.exitCode === 0) toast("텍스트 추출 완료: " + outDir, "ok");
  } catch (e) { toast(String(e), "error"); }
}
async function exportPdf(path) {
  const out = api.dirname(path) + "\\rhwp-out\\" + api.basename(path).replace(/\.(hwp|hwpx)$/i, "") + ".pdf";
  try {
    const entry = await api.runTool(state.engine.path, ["export-pdf", path, "-o", out, "--json"], "export");
    onEntry(entry);
    if (entry.exitCode === 0) toast("PDF 저장: " + out, "ok");
  } catch (e) { toast(String(e), "error"); }
}

/* ══════════ 뷰어·팔레트·배치 ══════════ */
const viewer = new Viewer({
  getEngine: () => state.engine?.path,
  onError: (e) => toast(e, "error"),
  onInfo: (p) => state.docs.get(p)?.info,
});

const palette = new Palette({
  getCaps: () => state.caps,
  getActiveDoc: () => state.activeDoc,
  onRun: (args, meta) => runPaletteCommand(args, meta),
});

async function runPaletteCommand(args, meta) {
  try {
    if (meta.mutating && !args.includes("--dry-run")) {
      // 행동 경계: 문서를 바꾸는 명령은 dry-run 미리보기 → 승인 카드 → 실행
      const dryEntry = await api.runTool(state.engine.path, [...args, "--dry-run"], "palette");
      const reg = registerApproval(dryEntry.id, `${meta.name || args[0]} — ${api.basename(args[1] || "")}`, dryEntry.envelope?.source, {
        onApprove: async () => {
          try {
            const entry = await api.runTool(state.engine.path, args, "approval");
            onEntry(entry);
            const src = entry.envelope?.source;
            if (src) viewer.invalidate(String(src));
          } catch (e) { toast(String(e), "error"); }
        },
        onReject: () => {},
      });
      cards.addApprovalCard(dryEntry, {
        onApprove: reg.settleApprove,
        onReject: reg.settleReject,
        onViewDoc: (p) => viewer.open(p),
      });
    } else {
      const running = cards.addRunningCard("rhwp " + args.join(" "));
      const entry = await api.runTool(state.engine.path, args, "palette");
      running.remove();
      onEntry(entry);
    }
  } catch (e) { toast(String(e), "error"); }
}

const batch = new BatchRunner({
  enginePath: () => state.engine?.path,
  queue,
  onEntry,
  note: (t, b) => cards.addNoticeCard(t, b),
});

let pendingBatch = null;
async function startBatch(dir) {
  try {
    const prep = await batch.prepare(dir);
    if (!prep.files.length) { toast("폴더에 HWP/HWPX 문서가 없습니다", "error"); return; }
    pendingBatch = prep;
    $("batch-summary").textContent = `${dir} — 문서 ${prep.files.length}건`;
    $("batch-modal").hidden = false;
  } catch (e) { toast(String(e), "error"); }
}

/* ══════════ Planner 연결 ══════════ */
function activeProfile() {
  return state.profiles.list.find((p) => p.id === state.profiles.activeId) || null;
}

function saveProfiles() { LS.set("plannerProfiles", state.profiles); }

function renderMode() {
  const badge = $("privacy-badge");
  const prof = activeProfile();
  badge.classList.remove("offline", "local", "remote");
  if (!prof) {
    badge.classList.add("offline");
    badge.textContent = "오프라인";
    badge.title = "Planner 미연결 — 문서는 이 기계 밖으로 나가지 않습니다. 클릭하면 설정.";
  } else if (/^https?:\/\/(127\.0\.0\.1|localhost|\[::1\])/i.test(prof.baseUrl)) {
    badge.classList.add("local");
    badge.textContent = "로컬 모델";
    badge.title = `${prof.name} (${prof.model}) — 로컬 서버라 문서 데이터가 기계 밖으로 나가지 않습니다.`;
  } else {
    badge.classList.add("remote");
    badge.textContent = "외부 API";
    badge.title = `${prof.name} (${prof.model}) — 외부 서버입니다. 도구 결과의 메타데이터가 전송되며, 본문은 '본문 전송' 토글을 켠 경우에만 나갑니다. 전송 내용은 planner/chat 카드에서 확인하세요.`;
  }
  // 컴포저 상태
  const stateText = $("composer-state-text");
  const dot = $("composer-state").querySelector(".dot");
  dot.className = "dot " + (prof ? "dot-ok" : "dot-gray");
  stateText.textContent = prof ? `${prof.name} · ${prof.model}` : "Planner 미연결";
  $("composer-input").placeholder = prof
    ? "자연어로 지시하세요 — 예: 이 문서 숨긴 텍스트 있는지 보고 요약해줘"
    : "Planner 미연결 — 설정에서 모델을 연결하거나 Ctrl+K 명령 팔레트를 사용하세요";
}

function renderProfiles() {
  const box = $("llm-profiles");
  box.replaceChildren();
  for (const p of state.profiles.list) {
    const el = document.createElement("div");
    el.className = "llm-profile" + (p.id === state.profiles.activeId ? " active" : "");
    const name = document.createElement("span");
    name.className = "lp-name";
    name.textContent = p.name;
    const meta = document.createElement("span");
    meta.className = "lp-meta";
    meta.textContent = `${p.baseUrl} · ${p.model}` + (p.keyStore === "keyring" ? " · 키:자격증명관리자" : p.keyStore === "session" ? " · 키:세션" : "");
    el.append(name, meta);
    const mk = (label, fn, cls = "btn small") => {
      const b = document.createElement("button");
      b.className = cls;
      b.textContent = label;
      b.addEventListener("click", fn);
      return b;
    };
    el.append(
      mk(p.id === state.profiles.activeId ? "사용 중" : "사용", async () => {
        state.profiles.activeId = p.id;
        saveProfiles(); renderProfiles(); renderMode();
        cards.addNoticeCard("Planner 연결됨", `${p.name} (${p.model}) — 이제 아래 입력창에 자연어로 지시할 수 있습니다. 문서를 바꾸는 도구는 항상 승인 카드를 거칩니다.`);
        await ensureTools();
      }),
      mk("테스트", async (ev) => {
        ev.target.textContent = "…";
        const r = await api.invoke("planner_test", {
          baseUrl: p.baseUrl, model: p.model, profileId: p.id,
          sessionKey: state.sessionKeys.get(p.id) ?? null,
        });
        ev.target.textContent = "테스트";
        toast(r.ok ? `연결 성공 (${r.latencyMs}ms)` : r.message, r.ok ? "ok" : "error");
      }),
      mk("삭제", async () => {
        if (p.keyStore === "keyring") { try { await api.invoke("secret_delete", { profileId: p.id }); } catch {} }
        state.profiles.list = state.profiles.list.filter((x) => x.id !== p.id);
        if (state.profiles.activeId === p.id) state.profiles.activeId = null;
        saveProfiles(); renderProfiles(); renderMode();
      }),
    );
    box.append(el);
  }
  if (!state.profiles.list.length) {
    box.innerHTML = `<p class="empty-hint">등록된 엔드포인트가 없습니다 — 아래에서 추가하거나, 로컬 서버가 떠 있으면 자동 탐지 카드가 뜹니다.</p>`;
  }
}

async function probeLocal({ silent = true } = {}) {
  try {
    const found = await api.invoke("probe_local_llm");
    const area = $("llm-probe-area");
    area.replaceChildren();
    for (const srv of found) {
      const card = document.createElement("div");
      card.className = "llm-probe-card";
      const label = document.createElement("span");
      label.textContent = `로컬 모델 서버 발견: ${srv.baseUrl} (모델 ${srv.models.length}개)`;
      const sel = document.createElement("select");
      for (const m of srv.models) {
        const o = document.createElement("option");
        o.value = m; o.textContent = m;
        sel.append(o);
      }
      const btn = document.createElement("button");
      btn.className = "btn small primary";
      btn.textContent = "연결하기";
      btn.addEventListener("click", () => {
        const id = crypto.randomUUID();
        state.profiles.list.push({ id, name: "로컬 서버", baseUrl: srv.baseUrl, model: sel.value, keyStore: "none" });
        state.profiles.activeId = id;
        saveProfiles(); renderProfiles(); renderMode();
        toast("로컬 모델 연결됨: " + sel.value, "ok");
        ensureTools();
      });
      card.append(label, sel, btn);
      area.append(card);
    }
    if (found.length && !activeProfile() && !silent) toast("로컬 모델 서버를 발견했습니다 — 설정에서 연결하세요", "ok");
    if (found.length && !activeProfile()) {
      cards.addNoticeCard("로컬 모델 발견", `${found[0].baseUrl} 에서 모델 ${found[0].models.length}개를 찾았습니다. 설정 → 모델 연결에서 한 번의 클릭으로 연결할 수 있습니다. 로컬 모델은 문서 데이터가 기계 밖으로 나가지 않습니다.`);
    }
    return found;
  } catch { return []; }
}

/** capabilities --mcp 도구를 로드해 Planner 에 넘길 준비. */
const AGENT_TOOL_ALLOWLIST = [
  "hwp_info", "hwp_digest", "hwp_export_text", "hwp_export_structure",
  "hwp_search", "hwp_extract_data", "hwp_fields", "hwp_explain",
  "hwp_inspect_hidden_text", "hwp_inspect_injection", "hwp_inspect_unicode",
  "hwp_export_pdf", "hwp_export_svg", "hwp_export_markdown", "hwp_thumbnail",
  "hwp_export_tables", "hwp_table_to_csv", "hwp_replace_text",
  "hwp_fill_fields", "hwp_set_cell", "hwp_set_checkbox",
];
async function ensureTools() {
  if (state.tools.openaiTools || !state.engine) return;
  try {
    state.mcp = await api.invoke("load_mcp_tools", { enginePath: state.engine.path });
    state.tools.openaiTools = await api.invoke("mcp_to_openai_tools", {
      mcp: state.mcp, allowlist: AGENT_TOOL_ALLOWLIST,
    });
    state.tools.byName = new Map((state.mcp.tools || []).map((t) => [t.name, t]));
  } catch (e) {
    toast("도구 스키마 로드 실패: " + e, "error");
  }
}

/* 승인 카드 → Promise */
function approvalPromise(dryEntry, label) {
  return new Promise((resolve) => {
    const id = dryEntry?.id || crypto.randomUUID();
    const path = dryEntry?.envelope?.source;
    const reg = registerApproval(id, label, path, {
      onApprove: () => resolve(true),
      onReject: () => resolve(false),
    });
    if (dryEntry) {
      cards.addApprovalCard(dryEntry, {
        onApprove: reg.settleApprove,
        onReject: reg.settleReject,
        onViewDoc: (p) => viewer.open(p),
      });
    } else {
      const card = cards.addNoticeCard(`승인 대기 — ${label}`,
        "문서를 바꾸는 도구입니다. dry-run 미리보기를 지원하지 않아 바로 실행 여부만 묻습니다.");
      card.dataset.entryId = id;
      const actions = document.createElement("div");
      actions.className = "approval-actions";
      const ok = document.createElement("button");
      ok.className = "btn primary"; ok.textContent = "승인하고 실행";
      const no = document.createElement("button");
      no.className = "btn"; no.textContent = "거부";
      ok.addEventListener("click", () => { actions.remove(); reg.settleApprove(); });
      no.addEventListener("click", () => { actions.remove(); reg.settleReject(); });
      actions.append(ok, no);
      card.classList.add("approval");
      card.append(actions);
    }
  });
}

function assistantCard(text, meta) {
  const card = cards.addNoticeCard(`Planner 요약 (${meta.model})`, text);
  card.classList.add("assistant-card");
  const tag = document.createElement("div");
  tag.className = "assistant-tag";
  tag.textContent = "모델 출력 — 검증되지 않은 내용입니다. 근거는 위 도구 카드의 봉투를 보세요.";
  card.append(tag);
  return card;
}

async function submitComposer() {
  const input = $("composer-input");
  const text = input.value.trim();
  if (!text) return;
  input.value = "";
  const prof = activeProfile();
  if (!prof) {
    cards.addNoticeCard("Planner 미연결",
      "자연어 지시는 모델 연결 후 가능합니다. 설정(톱니) → 모델 연결에서 로컬 서버나 API 엔드포인트를 등록하세요. 그 전에도 Ctrl+K 명령 팔레트로 모든 작업을 할 수 있습니다.");
    return;
  }
  await ensureTools();
  if (!state.tools.openaiTools) return;
  cards.addNoticeCard("사용자 지시", text);
  runAgentTask(text, {
    profile: () => activeProfile(),
    sessionKey: () => state.sessionKeys.get(activeProfile()?.id) ?? null,
    enginePath: () => state.engine.path,
    tools: state.tools,
    allowBody: () => state.allowBody,
    docPaths: () => [...state.docs.keys()],
    ui: {
      plannerCard: (entry) => onEntry(entry),
      toolCard: (entry) => onEntry(entry),
      assistantCard,
      approval: approvalPromise,
      note: (t, b) => cards.addNoticeCard(t, b),
    },
    queue,
  });
}

/* ══════════ 테마 ══════════ */
function applyTheme(mode) {
  const root = document.documentElement;
  if (mode === "light" || mode === "dark") root.setAttribute("data-theme", mode);
  else root.removeAttribute("data-theme");
  LS.set("theme", mode);
  const dark = mode === "dark" || (mode !== "light" && matchMedia("(prefers-color-scheme: dark)").matches);
  $("btn-theme").innerHTML = "";
  $("btn-theme").append(icon(dark ? "sun" : "moon"));
}

/* ══════════ 엔진 부트 ══════════ */
async function bootEngine() {
  const configured = LS.get("enginePath", "");
  try {
    state.engine = await api.detectEngine(configured || null);
    $("engine-status").querySelector(".dot").className = "dot dot-ok";
    $("engine-version").textContent = `${state.engine.version ?? "rhwp"} (${state.engine.source})`;
    $("engine-path").textContent = state.engine.path;
    $("engine-path").title = state.engine.path;
    state.caps = await api.loadCapabilities(state.engine.path);
    $("firstrun").hidden = true;
    return true;
  } catch (e) {
    $("engine-status").querySelector(".dot").className = "dot dot-bad";
    $("engine-version").textContent = "엔진 없음";
    $("engine-path").textContent = String(e);
    $("firstrun").hidden = false;
    return false;
  }
}

/* ══════════ 저널 복원 ══════════ */
async function restoreJournal() {
  try {
    const entries = await api.readJournal(60);
    if (entries.length) {
      cards.addDivider(`이전 기록 ${entries.length}건 (저널: journal.ndjson)`);
      for (const e of entries) onEntry(e, { historical: true });
      cards.addDivider("여기부터 이번 세션");
    }
  } catch { /* 저널 없음은 정상 */ }
}

/* ══════════ 이벤트 배선 ══════════ */
function wire() {
  $("btn-open").addEventListener("click", async () => {
    const p = await api.pickDocument();
    if (p) openDocument(p);
  });
  $("btn-batch").addEventListener("click", async () => {
    const dir = await api.pickFolder();
    if (dir) startBatch(dir);
  });
  $("fleet-add-folder").addEventListener("click", async () => {
    const dir = await api.pickFolder();
    if (dir) startBatch(dir);
  });
  for (const stat of document.querySelectorAll(".fleet-stat")) {
    stat.addEventListener("click", () => {
      const f = stat.dataset.filter;
      if (f === "attention") $("fleet-attention").scrollIntoView({ behavior: "smooth", block: "nearest" });
      else if (f === "pending") $("fleet-approvals").scrollIntoView({ behavior: "smooth", block: "nearest" });
      else $("doc-strip")?.scrollIntoView({ behavior: "smooth", block: "nearest" });
    });
  }
  $("btn-palette").addEventListener("click", () => palette.open());
  $("btn-viewer-toggle").addEventListener("click", () => {
    if (viewer.visible) viewer.hide();
    else if (state.activeDoc) viewer.open(state.activeDoc);
    else viewer.show();
  });
  $("btn-theme").addEventListener("click", () => {
    const cur = LS.get("theme", "system");
    applyTheme(cur === "dark" ? "light" : cur === "light" ? "system" : "dark");
    $("setting-theme").value = LS.get("theme", "system");
  });
  $("btn-settings").addEventListener("click", () => { $("settings").hidden = false; probeLocal(); });
  $("privacy-badge").addEventListener("click", () => { $("settings").hidden = false; probeLocal(); });
  $("settings-close").addEventListener("click", () => { $("settings").hidden = true; });
  $("batch-close").addEventListener("click", () => { $("batch-modal").hidden = true; });
  for (const btn of document.querySelectorAll(".batch-mode")) {
    btn.addEventListener("click", () => {
      $("batch-modal").hidden = true;
      if (pendingBatch) batch.run(pendingBatch.dir, pendingBatch.files, btn.dataset.mode);
    });
  }

  // 설정: 엔진 경로
  $("setting-engine").value = LS.get("enginePath", "");
  $("setting-engine-apply").addEventListener("click", async () => {
    LS.set("enginePath", $("setting-engine").value.trim());
    const ok = await bootEngine();
    toast(ok ? "엔진 연결: " + state.engine.version : "엔진을 찾지 못했습니다", ok ? "ok" : "error");
  });
  $("setting-theme").value = LS.get("theme", "system");
  $("setting-theme").addEventListener("change", (e) => applyTheme(e.target.value));

  // 첫 실행 화면
  $("firstrun-apply").addEventListener("click", async () => {
    LS.set("enginePath", $("firstrun-engine").value.trim());
    $("setting-engine").value = $("firstrun-engine").value.trim();
    if (!(await bootEngine())) $("firstrun-error").textContent = "해당 경로에서 실행 파일을 확인하지 못했습니다.";
  });
  $("firstrun-retry").addEventListener("click", async () => {
    if (!(await bootEngine())) $("firstrun-error").textContent = "여전히 찾지 못했습니다. 경로를 직접 지정하세요.";
  });

  // Planner 추가 폼
  $("llm-fetch-models").addEventListener("click", async () => {
    const base = $("llm-base").value.trim();
    if (!base) return;
    try {
      const models = await api.invoke("planner_list_models", {
        baseUrl: base, profileId: null, sessionKey: $("llm-key").value || null,
      });
      const dl = $("llm-model-list");
      dl.replaceChildren();
      for (const m of models) {
        const o = document.createElement("option");
        o.value = m;
        dl.append(o);
      }
      if (models.length && !$("llm-model").value) $("llm-model").value = models[0];
      toast(`모델 ${models.length}개 확인`, "ok");
    } catch (e) { toast(String(e), "error"); }
  });
  $("llm-test").addEventListener("click", async () => {
    const r = await api.invoke("planner_test", {
      baseUrl: $("llm-base").value.trim(), model: $("llm-model").value.trim(),
      profileId: null, sessionKey: $("llm-key").value || null,
    });
    $("llm-test-result").textContent = r.ok ? `성공 — ${r.message}` : `실패 — ${r.message}`;
    $("llm-test-result").style.color = r.ok ? "var(--ok)" : "var(--bad)";
  });
  $("llm-save").addEventListener("click", async () => {
    const name = $("llm-name").value.trim() || "엔드포인트";
    const baseUrl = $("llm-base").value.trim();
    const model = $("llm-model").value.trim();
    if (!baseUrl || !model) { toast("Base URL과 모델명을 입력하세요", "error"); return; }
    const id = crypto.randomUUID();
    const key = $("llm-key").value;
    let keyStore = "none";
    if (key) {
      if ($("llm-key-session").checked) {
        state.sessionKeys.set(id, key);
        keyStore = "session";
      } else {
        try {
          await api.invoke("secret_set", { profileId: id, key });
          keyStore = "keyring";
        } catch (e) {
          state.sessionKeys.set(id, key);
          keyStore = "session";
          toast("자격 증명 관리자 저장 실패 — 이 세션에만 유지합니다: " + e, "error");
        }
      }
    }
    state.profiles.list.push({ id, name, baseUrl, model, keyStore });
    state.profiles.activeId = id;
    saveProfiles(); renderProfiles(); renderMode();
    $("llm-key").value = "";
    toast("프로필 저장됨: " + name, "ok");
    ensureTools();
  });

  // 컴포저
  $("composer-send").addEventListener("click", submitComposer);
  $("composer-input").addEventListener("keydown", (e) => { if (e.key === "Enter") submitComposer(); });
  $("body-toggle").addEventListener("click", () => {
    state.allowBody = !state.allowBody;
    $("body-toggle").textContent = "본문 전송: " + (state.allowBody ? "허용" : "차단");
    $("body-toggle").style.color = state.allowBody ? "var(--warn)" : "";
    cards.addNoticeCard(
      state.allowBody ? "본문 전송 허용됨 (이 세션 한정)" : "본문 전송 차단됨",
      state.allowBody
        ? "이제 도구 결과의 문서 본문 텍스트가 Planner 모델로 전송될 수 있습니다. 전송된 내용은 planner/chat 카드의 request에서 그대로 확인할 수 있습니다."
        : "도구 결과의 본문성 문자열은 [본문 차단]으로 가려져 전송됩니다. 수치·메타데이터만 나갑니다.",
    );
  });

  // 키보드
  document.addEventListener("keydown", (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      palette.isOpen ? palette.close() : palette.open();
    } else if (e.key === "Escape") {
      palette.close();
      $("settings").hidden = true;
      $("batch-modal").hidden = true;
    }
  });

  // 드래그앤드롭 (Tauri 네이티브 이벤트)
  api.listen("tauri://drag-enter", () => { $("drop-veil").hidden = false; });
  api.listen("tauri://drag-leave", () => { $("drop-veil").hidden = true; });
  api.listen("tauri://drag-drop", async (ev) => {
    $("drop-veil").hidden = true;
    const paths = ev.payload?.paths || [];
    for (const p of paths) {
      const kind = await api.pathKind(p);
      if (kind === "dir") startBatch(p);
      else if (kind === "file" && api.isDocPath(p)) openDocument(p);
      else toast("지원하지 않는 항목: " + api.basename(p), "error");
    }
  });
}

/* ══════════ 시작 ══════════ */
// 조용한 실패 방지 — 처리되지 않은 오류는 토스트로 드러낸다.
window.addEventListener("error", (e) => toast("오류: " + (e.message || e.error), "error"));
window.addEventListener("unhandledrejection", (e) => toast("오류: " + e.reason, "error"));

async function boot() {
  mountIcons();
  applyTheme(LS.get("theme", "system"));
  wire();
  renderRecent();
  renderQueue();
  renderProfiles();
  renderMode();
  renderDashboard();

  const ok = await bootEngine();
  await restoreJournal();
  probeLocal();

  if (ok) {
    // 파일 연결/명령행 인자
    try {
      const args = await api.startupArgs();
      for (const a of args) {
        if (api.isDocPath(a) && (await api.pathKind(a)) === "file") await openDocument(a);
      }
      // 파일 연결/더블클릭으로 열렸을 때는 보조 문서 패널도 바로 연다 (M0 뷰어 역할).
      if (state.activeDoc) await viewer.open(state.activeDoc);
      if (args.includes("--autorun-info") && state.activeDoc) {
        await runPaletteCommand(["info", state.activeDoc, "--json"], { mutating: false, name: "info" });
      }
    } catch { /* 무시 */ }
  }
}

boot();
