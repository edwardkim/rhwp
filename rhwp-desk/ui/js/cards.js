// 작업 카드 스트림 — NDJSON 저널의 1:1 렌더 (설계 §7).
// 카드는 저널에 있는 것만 보여준다. 지어내는 요약·해석은 없다.

import { icon } from "./icons.js";
import { fingerprint, copyText, toCliLine, basename } from "./api.js";

const stream = () => document.getElementById("stream");
const welcome = () => document.getElementById("stream-welcome");

function fmtTime(tsMs) {
  const d = new Date(tsMs);
  const p = (n) => String(n).padStart(2, "0");
  return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}
function fmtDur(ms) {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(ms < 10000 ? 1 : 0)}s`;
}

/** 종료 코드 → 배지. exitCodes 계약(0 성공/1 실패/2 사용법/3 판정/4 쪽수 불일치). */
function exitBadge(code) {
  const el = document.createElement("span");
  el.className = "badge";
  if (code === 0) { el.classList.add("ok"); el.textContent = "성공 0"; }
  else if (code === 3) { el.classList.add("warn"); el.textContent = "판정 3"; }
  else if (code === 4) { el.classList.add("warn"); el.textContent = "불일치 4"; }
  else if (code === 2) { el.classList.add("bad"); el.textContent = "사용법 2"; }
  else if (code === 1) { el.classList.add("bad"); el.textContent = "실패 1"; }
  else { el.classList.add("bad"); el.textContent = `종료 ${code ?? "?"}`; }
  return el;
}

const ORIGIN_LABEL = {
  palette: "팔레트", verify: "검증", viewer: "뷰어", export: "내보내기",
  info: "문서 정보", batch: "일괄", approval: "승인 실행", startup: "시작",
};

/** 봉투에서 사람이 먼저 볼 요약 키·값을 고른다 — 값은 봉투 원문 그대로. */
function envelopeSummary(env) {
  if (!env || typeof env !== "object" || Array.isArray(env)) return [];
  const PICK = [
    "source", "format", "pageCount", "paraCount", "renderedCount", "output",
    "outputDir", "bytes", "title", "clean", "hiddenCharCount", "signalCount",
    "findingCount", "highestConfidence", "truncated", "omittedCount",
    "filledCount", "replacedCount", "dryRun", "verify", "overflowCellLines",
    "untrustedContent",
  ];
  const out = [];
  for (const k of PICK) {
    if (env[k] === undefined || env[k] === null) continue;
    let v = env[k];
    if (k === "source" || k === "output") v = basename(String(v));
    if (typeof v === "object") continue;
    out.push([k, String(v)]);
  }
  return out.slice(0, 8);
}

function scrollToEnd() {
  const s = stream();
  s.scrollTop = s.scrollHeight;
}

function hideWelcome() {
  const w = welcome();
  if (w) w.remove();
}

/**
 * 저널 항목 1건 → 카드 1장.
 * opts.historical: 과거 세션 항목(접힌 채, 애니메이션 없음)
 * opts.onViewDoc(path): "문서 보기" 콜백
 */
export function addToolCard(entry, opts = {}) {
  hideWelcome();
  const card = document.createElement("article");
  card.className = "card";
  card.dataset.entryId = entry.id;

  // ── 헤더
  const head = document.createElement("div");
  head.className = "card-head";
  head.append(exitBadge(entry.exitCode));
  const cmd = document.createElement("span");
  cmd.className = "card-cmd";
  cmd.textContent = entry.args.slice(0, entry.args[1] && !entry.args[1].startsWith("-") && !/[\\/.]/.test(entry.args[1]) ? 2 : 1).join(" ");
  head.append(cmd);
  const org = document.createElement("span");
  org.className = "card-origin";
  org.textContent = ORIGIN_LABEL[entry.origin] || entry.origin;
  head.append(org);
  const sp = document.createElement("span");
  sp.className = "card-spacer";
  head.append(sp);
  const dur = document.createElement("span");
  dur.className = "card-dur";
  dur.textContent = fmtDur(entry.durationMs);
  head.append(dur);
  const time = document.createElement("span");
  time.className = "card-time";
  time.textContent = fmtTime(entry.tsMs);
  head.append(time);
  card.append(head);

  // ── 인자열
  const argsEl = document.createElement("div");
  argsEl.className = "card-args";
  argsEl.textContent = toCliLine(entry.engine, entry.args);
  card.append(argsEl);

  // ── 본문: 봉투 요약 + 원문
  const body = document.createElement("div");
  body.className = "card-body";
  let hasBody = false;

  const pairs = envelopeSummary(entry.envelope);
  if (pairs.length) {
    hasBody = true;
    const sum = document.createElement("div");
    sum.className = "card-summary";
    for (const [k, v] of pairs) {
      const kv = document.createElement("span");
      kv.className = "kv";
      const b = document.createElement("b");
      b.textContent = k;
      kv.append(b, `: ${v}`);
      sum.append(kv);
    }
    body.append(sum);
  }

  if (entry.envelope !== undefined && entry.envelope !== null) {
    hasBody = true;
    const det = document.createElement("details");
    det.className = "card-envelope";
    if (opts.openEnvelope) det.open = true;
    const sm = document.createElement("summary");
    sm.textContent = "봉투 JSON 펼치기";
    const pre = document.createElement("pre");
    pre.textContent = JSON.stringify(entry.envelope, null, 2);
    det.append(sm, pre);
    body.append(det);
  } else if (entry.stdoutTail) {
    hasBody = true;
    const det = document.createElement("details");
    det.className = "card-envelope";
    const sm = document.createElement("summary");
    sm.textContent = "출력 펼치기";
    const pre = document.createElement("pre");
    pre.textContent = entry.stdoutTail;
    det.append(sm, pre);
    body.append(det);
  }
  if (entry.stderrTail) {
    hasBody = true;
    const det = document.createElement("details");
    det.className = "card-envelope";
    if (entry.exitCode !== 0) det.open = true;
    const sm = document.createElement("summary");
    sm.textContent = "stderr 펼치기";
    const pre = document.createElement("pre");
    pre.textContent = entry.stderrTail;
    det.append(sm, pre);
    body.append(det);
  }
  if (hasBody) card.append(body);

  // ── 푸터: CLI 복사·문서 보기·봉투 지문
  const foot = document.createElement("div");
  foot.className = "card-foot";
  const btnCopy = document.createElement("button");
  btnCopy.className = "btn small";
  btnCopy.append(icon("copy"), document.createTextNode("CLI 복사"));
  btnCopy.title = "같은 호출을 터미널에서 재현하는 한 줄을 복사";
  btnCopy.addEventListener("click", async () => {
    const ok = await copyText(toCliLine(entry.engine, entry.args));
    btnCopy.lastChild.textContent = ok ? "복사됨" : "복사 실패";
    setTimeout(() => (btnCopy.lastChild.textContent = "CLI 복사"), 1200);
  });
  foot.append(btnCopy);

  const docPath = entry.envelope?.source;
  if (docPath && opts.onViewDoc) {
    const btnView = document.createElement("button");
    btnView.className = "btn small";
    btnView.append(icon("eye"), document.createTextNode("문서 보기"));
    btnView.addEventListener("click", () => opts.onViewDoc(String(docPath)));
    foot.append(btnView);
  }

  const fp = document.createElement("span");
  fp.className = "fingerprint";
  foot.append(fp);
  if (entry.envelope !== undefined && entry.envelope !== null) {
    fingerprint(JSON.stringify(entry.envelope)).then((h) => {
      if (h) fp.textContent = `봉투 지문 sha256:${h}`;
    });
  }
  card.append(foot);

  if (opts.historical) card.style.animation = "none";
  stream().append(card);
  if (!opts.historical) scrollToEnd();
  return card;
}

/** 실행 중 자리표시 카드 — 완료되면 실제 저널 카드로 교체된다. */
export function addRunningCard(label) {
  hideWelcome();
  const card = document.createElement("article");
  card.className = "card";
  const head = document.createElement("div");
  head.className = "card-head";
  const badge = document.createElement("span");
  badge.className = "badge run";
  badge.textContent = "실행 중";
  const cmd = document.createElement("span");
  cmd.className = "card-cmd";
  cmd.textContent = label;
  head.append(badge, cmd);
  card.append(head);
  stream().append(card);
  scrollToEnd();
  return card;
}

/** 시스템 알림 카드 (가짜 AI 응답이 아니라 상태 안내). */
export function addNoticeCard(title, bodyText) {
  hideWelcome();
  const card = document.createElement("article");
  card.className = "card notice";
  const head = document.createElement("div");
  head.className = "card-head";
  const badge = document.createElement("span");
  badge.className = "badge info";
  badge.textContent = "안내";
  const cmd = document.createElement("span");
  cmd.className = "card-cmd";
  cmd.textContent = title;
  const sp = document.createElement("span");
  sp.className = "card-spacer";
  const time = document.createElement("span");
  time.className = "card-time";
  time.textContent = fmtTime(Date.now());
  head.append(badge, cmd, sp, time);
  card.append(head);
  if (bodyText) {
    const body = document.createElement("div");
    body.className = "card-body";
    body.textContent = bodyText;
    card.append(body);
  }
  stream().append(card);
  scrollToEnd();
  return card;
}

/**
 * 승인 카드 — 문서를 바꾸는 명령의 dry-run 미리보기 (설계 §6 행동 경계).
 * dry-run 저널 항목을 그대로 보여주고, 승인 시에만 실제 실행 콜백을 부른다.
 */
export function addApprovalCard(dryEntry, { onApprove, onReject, onViewDoc }) {
  const card = addToolCard(dryEntry, { openEnvelope: true, onViewDoc });
  card.classList.add("approval");

  const note = document.createElement("div");
  note.className = "card-body";
  note.innerHTML =
    "<strong>승인 대기</strong> — 위는 <code>--dry-run</code> 미리보기입니다. " +
    "문서를 실제로 바꾸려면 승인하세요. 원본은 계약 경로(-o 산출 분리·--verify)로만 변경됩니다.";
  const actions = document.createElement("div");
  actions.className = "approval-actions";
  const ok = document.createElement("button");
  ok.className = "btn primary";
  ok.append(icon("check"), document.createTextNode("승인하고 실행"));
  const no = document.createElement("button");
  no.className = "btn";
  no.append(icon("close"), document.createTextNode("취소"));
  ok.addEventListener("click", () => { actions.remove(); note.remove(); onApprove(); });
  no.addEventListener("click", () => {
    actions.remove();
    note.textContent = "취소됨 — 실제 실행은 하지 않았습니다.";
    onReject?.();
  });
  actions.append(ok, no);
  card.append(note, actions);
  scrollToEnd();
  return card;
}

/** 과거 세션 구분선. */
export function addDivider(label) {
  const el = document.createElement("div");
  el.className = "day-divider";
  el.textContent = label;
  stream().append(el);
}

/** 추출 텍스트 미리보기를 카드에 덧붙인다 (export-text --json 봉투에서). */
export function attachTextPreview(card, envelope) {
  const pages = envelope?.pages;
  if (!Array.isArray(pages) || !pages.length) return;
  const joined = pages.map((p) => p.text ?? "").join("\n\n");
  const trimmed = joined.length > 4000 ? joined.slice(0, 4000) + "\n…(이하 생략)" : joined;
  const det = document.createElement("details");
  det.className = "card-envelope";
  det.open = true;
  const sm = document.createElement("summary");
  sm.textContent = `추출 텍스트 미리보기 (${pages.length}쪽)`;
  const pre = document.createElement("div");
  pre.className = "text-preview";
  pre.textContent = trimmed;
  det.append(sm, pre);
  (card.querySelector(".card-body") || card).append(det);
}
