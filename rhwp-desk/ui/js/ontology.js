// 도구 온톨로지 — 백엔드 tool_ontology 그래프를 받아 카드 아래에
// "다음 작업 제안" 칩을 붙인다. LLM 추론 없이도(모델 미연결이어도) 동작한다.

import { invoke } from "./api.js";

const LABEL = {
  hwp_info: "문서 정보", hwp_digest: "요약", hwp_export_text: "텍스트 추출",
  hwp_export_structure: "구조 추출", hwp_search: "검색", hwp_extract_data: "데이터 추출",
  hwp_fields: "필드 목록", hwp_explain: "설명", hwp_inspect_hidden_text: "은닉 텍스트 검사",
  hwp_inspect_injection: "주입 신호 검사", hwp_inspect_unicode: "유니코드 기만 검사",
  hwp_export_pdf: "PDF 내보내기", hwp_export_svg: "SVG 내보내기", hwp_export_markdown: "마크다운 내보내기",
  hwp_thumbnail: "썸네일", hwp_export_tables: "표 추출", hwp_table_to_csv: "표 CSV 변환",
  hwp_replace_text: "텍스트 치환", hwp_fill_fields: "필드 채우기", hwp_set_cell: "셀 채우기",
  hwp_set_checkbox: "체크박스 설정",
};

let graphPromise = null;

/** 그래프는 세션당 한 번만 백엔드에서 받아온다 — 정적 데이터라 매번 물을 필요 없다. */
function loadGraph() {
  if (!graphPromise) graphPromise = invoke("tool_ontology").catch(() => null);
  return graphPromise;
}

/** MCP 도구 이름(hwp_*)이 아니라 CLI 명령 인자로 온 경우를 위한 매핑. */
const CLI_TO_TOOL = {
  info: "hwp_info", digest: "hwp_digest", "export-text": "hwp_export_text",
  "export-structure": "hwp_export_structure", search: "hwp_search",
  "extract-data": "hwp_extract_data", fields: "hwp_fields", explain: "hwp_explain",
  "export-pdf": "hwp_export_pdf", "export-svg": "hwp_export_svg",
  "export-markdown": "hwp_export_markdown", thumbnail: "hwp_thumbnail",
  "export-tables": "hwp_export_tables", "table-to-csv": "hwp_table_to_csv",
  "replace-text": "hwp_replace_text", "fill-fields": "hwp_fill_fields",
  "set-cell": "hwp_set_cell", "set-checkbox": "hwp_set_checkbox",
};

const TOOL_TO_CLI = Object.fromEntries(Object.entries(CLI_TO_TOOL).map(([cli, tool]) => [tool, cli]));
TOOL_TO_CLI.hwp_inspect_hidden_text = "inspect hidden-text";
TOOL_TO_CLI.hwp_inspect_injection = "inspect injection";
TOOL_TO_CLI.hwp_inspect_unicode = "inspect unicode";

/** 온톨로지 도구 이름(hwp_*) → 명령 팔레트 검색어로 쓸 CLI 명령 이름. */
export const cliCommandFor = (toolName) => TOOL_TO_CLI[toolName] || toolName;

/** tool_name이 만든 결과를 이어받을 수 있는 MCP 도구 이름들(hwp_*) — 없으면 빈 배열. */
export async function suggestedTools(toolName) {
  const graph = await loadGraph();
  return graph?.edges?.[toolName] || [];
}

function toolNameFromEntry(entry) {
  const cmd = entry.command || entry.args?.[0];
  if (cmd === "inspect") {
    const axis = entry.args?.[1];
    if (axis === "hidden-text") return "hwp_inspect_hidden_text";
    if (axis === "injection") return "hwp_inspect_injection";
    if (axis === "unicode") return "hwp_inspect_unicode";
    return null;
  }
  return CLI_TO_TOOL[cmd] || null;
}

/**
 * 저널 항목 카드 아래에 "다음 작업 제안" 칩 줄을 붙인다.
 * onPick(nextToolName)이 주어지면 칩 클릭 시 호출한다(팔레트 프리필 등).
 */
export async function attachSuggestions(card, entry, onPick) {
  const tool = toolNameFromEntry(entry);
  if (!tool) return;
  const graph = await loadGraph();
  const next = graph?.edges?.[tool];
  if (!next || !next.length) return;

  const row = document.createElement("div");
  row.className = "ontology-row";
  const label = document.createElement("span");
  label.className = "ontology-label";
  label.textContent = "다음 작업 제안:";
  row.append(label);
  for (const t of next) {
    const chip = document.createElement("button");
    chip.className = "ontology-chip";
    chip.textContent = LABEL[t] || t;
    chip.title = t;
    chip.addEventListener("click", () => onPick?.(t));
    row.append(chip);
  }
  card.append(row);
}
