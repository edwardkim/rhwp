// 도구 온톨로지 — 백엔드 tool_ontology 그래프를 받아 카드 아래에
// "다음 작업 제안" 칩을 붙인다. LLM 추론 없이도(모델 미연결이어도) 동작한다.

import { invoke } from "./api.js";

const LABEL = {
  hwp_info: "문서 정보", hwp_digest: "요약", hwp_export_text: "텍스트 추출",
  hwp_export_structure: "구조 추출", hwp_search: "검색", hwp_extract_data: "데이터 추출",
  hwp_fields: "필드 목록", hwp_explain: "설명", hwp_inspect_hidden_text: "은닉 텍스트 검사",
  hwp_inspect_injection: "주입 신호 검사", hwp_inspect_unicode: "유니코드 기만 검사",
  hwp_inspect_watermark: "워터마크 검사",
  hwp_export_pdf: "PDF 내보내기", hwp_export_svg: "SVG 내보내기", hwp_export_markdown: "마크다운 내보내기",
  hwp_thumbnail: "썸네일", hwp_export_tables: "표 추출", hwp_table_to_csv: "표 CSV 변환",
  hwp_csv_to_table: "CSV로 표 채우기", hwp_chart_to_csv: "차트 CSV 변환",
  hwp_csv_to_chart: "CSV로 차트 채우기",
  hwp_replace_text: "텍스트 치환", hwp_fill_fields: "필드 채우기", hwp_set_cell: "셀 채우기",
  hwp_set_checkbox: "체크박스 설정", hwp_insert_image: "도장/서명 삽입",
};

let graphPromise = null;

/** 그래프는 세션당 한 번만 백엔드에서 받아온다 — 정적 데이터라 매번 물을 필요 없다. */
function loadGraph() {
  if (!graphPromise) graphPromise = invoke("tool_ontology").catch(() => null);
  return graphPromise;
}

/** 최상위 CLI 명령이 곧 도구 이름과 1:1인 경우(entry.command === 이 키). */
const CLI_TO_TOOL = {
  info: "hwp_info", digest: "hwp_digest", "export-text": "hwp_export_text",
  "export-structure": "hwp_export_structure", search: "hwp_search",
  "extract-data": "hwp_extract_data", fields: "hwp_fields", explain: "hwp_explain",
  "export-pdf": "hwp_export_pdf", "export-svg": "hwp_export_svg",
  "export-markdown": "hwp_export_markdown", thumbnail: "hwp_thumbnail",
  "export-tables": "hwp_export_tables", "table-to-csv": "hwp_table_to_csv",
  "csv-to-table": "hwp_csv_to_table", "chart-to-csv": "hwp_chart_to_csv",
  "csv-to-chart": "hwp_csv_to_chart",
};

/**
 * "edit" 서브커맨드 전용 매핑 — journal.rs 는 command 를 args[0] 으로만 채우므로
 * fill-fields/replace-text/set-cell/insert-image 는 entry.command 가 전부 "edit"다.
 * args[1](서브커맨드 이름)로 따로 구분해야 한다. 이걸 CLI_TO_TOOL 에 "fill-fields" 같은
 * 키로 넣어봤자 entry.command("edit")와 절대 안 맞아 죽은 코드였다 — 여기서 고친다.
 * 예외: hwp_set_checkbox 는 내부적으로 replace-text(□→☑ 치환)라 args[1] 만으로는
 * hwp_replace_text 와 구분이 안 된다. 어느 쪽으로 표시되든 다음 제안(도장 삽입 등)은
 * 같아서 실사용에 영향은 없다.
 */
const EDIT_SUB_TO_TOOL = {
  "fill-fields": "hwp_fill_fields", "replace-text": "hwp_replace_text",
  "set-cell": "hwp_set_cell", "insert-image": "hwp_insert_image",
};

const TOOL_TO_CLI = Object.fromEntries([
  ...Object.entries(CLI_TO_TOOL).map(([cli, tool]) => [tool, cli]),
  ...Object.entries(EDIT_SUB_TO_TOOL).map(([sub, tool]) => [tool, sub]),
]);
TOOL_TO_CLI.hwp_inspect_hidden_text = "inspect hidden-text";
TOOL_TO_CLI.hwp_inspect_injection = "inspect injection";
TOOL_TO_CLI.hwp_inspect_unicode = "inspect unicode";
TOOL_TO_CLI.hwp_inspect_watermark = "inspect watermark";
TOOL_TO_CLI.hwp_set_checkbox = "set-checkbox"; // edit 팔레트 검색어 — 실제 CLI 서브커맨드는 replace-text

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
    if (axis === "watermark") return "hwp_inspect_watermark";
    return null;
  }
  if (cmd === "edit") return EDIT_SUB_TO_TOOL[entry.args?.[1]] || null;
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
