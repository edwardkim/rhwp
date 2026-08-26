// Tauri 백엔드 호출 래퍼 — withGlobalTauri 전역 API 사용 (빌드 도구 없는 순수 ESM).

const T = window.__TAURI__;

/** Tauri IPC 없이 뜬 경우(개발자도구·프리뷰) 조용히 no-op — 크래시 대신 저하 동작. */
export const invoke = (cmd, args = {}) =>
  T ? T.core.invoke(cmd, args) : Promise.reject(new Error(`Tauri IPC 없음(${cmd})`));
export const listen = (ev, cb) =>
  T ? T.event.listen(ev, cb) : Promise.resolve(() => {});

export const detectEngine = (configured) =>
  invoke("detect_engine", { configured: configured || null });

export const loadCapabilities = (enginePath) =>
  invoke("load_capabilities", { enginePath });

/** 도구 호출 1건 — journal-first. 반환값 = 저널 항목(카드의 원천). */
export const runTool = (enginePath, args, origin) =>
  invoke("run_tool", { enginePath, args, origin });

export const renderPage = (enginePath, file, page) =>
  invoke("render_page", { enginePath, file, page });

export const readJournal = (limit = 200) => invoke("read_journal", { limit });
export const startupArgs = () => invoke("startup_args");
export const pathKind = (path) => invoke("path_kind", { path });
export const listDocuments = (dir) => invoke("list_documents", { dir });

export async function pickDocument() {
  return T.dialog.open({
    multiple: false,
    title: "HWP/HWPX 문서 열기",
    filters: [{ name: "HWP 문서", extensions: ["hwp", "hwpx"] }],
  });
}

export async function pickFolder() {
  return T.dialog.open({ directory: true, title: "일괄 처리할 폴더 선택" });
}

export async function pickAnyFile() {
  return T.dialog.open({ multiple: false, title: "파일 선택" });
}

/** 봉투 지문 — SHA-256 앞 12자리. 카드에서 산출물 동일성 비교용. */
export async function fingerprint(text) {
  try {
    const buf = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text));
    return [...new Uint8Array(buf)]
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("")
      .slice(0, 12);
  } catch {
    return null;
  }
}

export async function copyText(text) {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}

/** 인자열을 복사 가능한 CLI 한 줄로. 공백·따옴표가 있으면 감싼다. */
export function toCliLine(engine, args) {
  const q = (s) => (/[\s"]/.test(s) ? `"${s.replaceAll('"', '\\"')}"` : s);
  const exe = engine ? engine.split(/[\\/]/).pop() : "rhwp";
  return [exe, ...args.map(q)].join(" ");
}

export const basename = (p) => (p || "").split(/[\\/]/).pop();
export const dirname = (p) => {
  const i = Math.max(p.lastIndexOf("\\"), p.lastIndexOf("/"));
  return i > 0 ? p.slice(0, i) : p;
};
export const isDocPath = (p) => /\.(hwp|hwpx)$/i.test(p || "");
