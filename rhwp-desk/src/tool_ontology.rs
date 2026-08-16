// 에이전트 도구 온톨로지 — 도구 사이의 생산/소비 관계를 데이터로 표현한다.
//
// 지금까지 rhwp-desk 의 Planner(LLM)는 도구 설명문만 보고 "표를 찾았으니 다음은
// 셀을 채운다" 같은 연결을 매번 스스로 추론해야 했다. 이 모듈은 그 연결을
// 하드코딩된 그래프로 미리 선언해, LLM 추론 없이도(모델 미연결 상태에서도)
// "방금 이 결과가 나왔으니 다음은 이 도구들이 자연스럽다"를 결정론적으로 계산한다.
//
// 각 도구는 무엇을 만들어내는지(produces)와 무엇을 필요로 하는지(consumes)를
// 태그로 선언한다. A가 만드는 태그를 B가 필요로 하면 A → B 간선이 생긴다.

use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ToolNode {
    pub tool: &'static str,
    pub produces: &'static [&'static str],
    pub consumes: &'static [&'static str],
}

/// AGENT_TOOL_ALLOWLIST(ui/js/main.js)와 이름을 맞춘 정적 레지스트리.
/// 새 MCP 도구를 화이트리스트에 추가할 때 여기도 함께 등재한다.
const REGISTRY: &[ToolNode] = &[
    ToolNode {
        tool: "hwp_info",
        produces: &["doc-meta"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_digest",
        produces: &["doc-summary"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_export_text",
        produces: &["text-content"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_export_structure",
        produces: &["doc-structure"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_search",
        produces: &["search-hits"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_extract_data",
        produces: &["extracted-data"],
        consumes: &["search-hits"],
    },
    ToolNode {
        tool: "hwp_fields",
        produces: &["field-names"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_explain",
        produces: &["explanation"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_inspect_hidden_text",
        produces: &["finding:hidden-text"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_inspect_injection",
        produces: &["finding:injection"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_inspect_unicode",
        produces: &["finding:unicode"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_export_pdf",
        produces: &["pdf-file"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_export_svg",
        produces: &["svg-file"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_export_markdown",
        produces: &["markdown-file"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_thumbnail",
        produces: &["thumbnail-image"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_export_tables",
        produces: &["table-data"],
        consumes: &[],
    },
    ToolNode {
        tool: "hwp_table_to_csv",
        produces: &["csv-file"],
        consumes: &["table-data"],
    },
    ToolNode {
        tool: "hwp_replace_text",
        produces: &["doc-mutation"],
        consumes: &["text-content"],
    },
    ToolNode {
        tool: "hwp_fill_fields",
        produces: &["doc-mutation"],
        consumes: &["field-names"],
    },
    ToolNode {
        tool: "hwp_set_cell",
        produces: &["doc-mutation"],
        consumes: &["table-data"],
    },
    ToolNode {
        tool: "hwp_set_checkbox",
        produces: &["doc-mutation"],
        consumes: &["field-names"],
    },
];

/// tool_name이 방금 만들어낸 결과를 이어받을 수 있는 도구 이름들 — 등재 순서대로.
pub fn suggest_next(tool_name: &str) -> Vec<&'static str> {
    let Some(src) = REGISTRY.iter().find(|n| n.tool == tool_name) else {
        return Vec::new();
    };
    REGISTRY
        .iter()
        .filter(|n| n.tool != tool_name)
        .filter(|n| n.consumes.iter().any(|c| src.produces.contains(c)))
        .map(|n| n.tool)
        .collect()
}

#[derive(Serialize)]
pub struct OntologyGraph {
    pub nodes: Vec<ToolNode>,
    /// tool -> 다음으로 이어지는 도구 이름 목록. 프런트가 매번 재계산하지 않도록 미리 채워 보낸다.
    pub edges: BTreeMap<&'static str, Vec<&'static str>>,
}

/// 전체 그래프 — UI가 카드마다 다시 계산하지 않도록 통짜로 넘긴다.
pub fn graph() -> OntologyGraph {
    let edges = REGISTRY
        .iter()
        .map(|n| (n.tool, suggest_next(n.tool)))
        .collect();
    OntologyGraph {
        nodes: REGISTRY.to_vec(),
        edges,
    }
}

#[tauri::command]
pub fn tool_ontology() -> OntologyGraph {
    graph()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 표를_찾으면_셀_채우기와_csv_내보내기로_이어진다() {
        let next = suggest_next("hwp_export_tables");
        assert!(next.contains(&"hwp_set_cell"));
        assert!(next.contains(&"hwp_table_to_csv"));
    }

    #[test]
    fn 필드_목록은_채우기와_체크박스_두_경로로_이어진다() {
        let next = suggest_next("hwp_fields");
        assert!(next.contains(&"hwp_fill_fields"));
        assert!(next.contains(&"hwp_set_checkbox"));
    }

    #[test]
    fn 검증_축_3종은_말단이라_다음_제안이_없다() {
        assert!(suggest_next("hwp_inspect_hidden_text").is_empty());
        assert!(suggest_next("hwp_inspect_injection").is_empty());
        assert!(suggest_next("hwp_inspect_unicode").is_empty());
    }

    #[test]
    fn 산출_파일류_도구는_모두_말단이다() {
        for t in [
            "hwp_export_pdf",
            "hwp_export_svg",
            "hwp_export_markdown",
            "hwp_thumbnail",
        ] {
            assert!(suggest_next(t).is_empty(), "{t} 는 말단이어야 한다");
        }
    }

    #[test]
    fn 모르는_도구_이름은_빈_목록이다() {
        assert!(suggest_next("hwp_없는_도구").is_empty());
    }

    #[test]
    fn 자기_자신은_제안에_포함되지_않는다() {
        for node in REGISTRY {
            assert!(!suggest_next(node.tool).contains(&node.tool));
        }
    }

    #[test]
    fn 그래프_노드_수는_레지스트리와_같다() {
        assert_eq!(graph().nodes.len(), REGISTRY.len());
    }
}
