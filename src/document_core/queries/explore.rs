//! [#gym] `explore` 명령의 어포던스 라우터 — "이 문서로 무엇을 할 수 있는가".
//!
//! `explain` 이 문서가 **무엇인지**(형식·쪽수·표·누름틀)를 서술한다면, `explore` 는
//! 이 문서에 대해 **무엇을 할 수 있는지**를 라우팅한다. 처음 보는 문서 앞에서
//! 에이전트가 "70개 명령 중 무엇이 이 문서에 적용되는가"를 매번 뒤지지 않도록,
//! 문서를 한 번 분석해 **적용 가능한 행동만** 골라 순위 매긴 메뉴로 돌려준다.
//!
//! ## 새 판정 로직이 아니다
//!
//! 어포던스는 **기존 조회가 이미 센 값**에서 유도한다 — `table_extract::extract_tables`,
//! `field_query::collect_all_fields`, `structure::build_structure`,
//! `chart_extract::collect_charts`, `explain::count_notes`, `injection_scan`,
//! `hidden_text`. 이 모듈은 탐지기를 다시 구현하지 않고, 그 개수를 [`DocFacts`] 로
//! 받아 **순위 매긴 메뉴**([`build_menu`])로 옮길 뿐이다. `explain` 이 네 조회의
//! 값을 사람 문장으로 옮기는 것과 같은 결정론적 템플릿 조립이다.
//!
//! ## 정직한 휴리스틱
//!
//! 메뉴는 **제안**이지 완전성 보장이 아니다. "표가 있으니 표 명령을 써 볼 수 있다"고
//! 안내할 뿐, 그 표가 원하는 표인지·다른 숨은 행동이 없는지는 판정하지 않는다.
//! 증거(`why`)는 문서 원문이 아니라 **엔진이 센 개수·형식 레이블**이라 출처상
//! 문서 파생 문자열을 싣지 않는다(봉투 표지 `untrustedContent:false`). 명령
//! 템플릿의 경로 자리는 실제 경로가 아니라 `<file>` 자리표시자를 쓴다 — 소비자가
//! 자기 경로로 치환한다.

/// "긴 문서" 판정 임계 — 이 쪽수 이상이면 통독 대신 요약·절 청킹을 권한다.
///
/// 10쪽은 한 화면에 담기지 않아 컨텍스트를 아껴 읽어야 하는 실무 하한이다.
pub const LONG_DOC_PAGES: u32 = 10;

/// 봉투에 함께 싣는 정직성 고지 — 메뉴가 제안이지 보장이 아님을 명시한다.
pub const HONESTY_NOTE: &str = "정직한 휴리스틱 안내다 — 이 문서에 적용 가능한 rhwp 행동을 \
개수 근거와 함께 제안할 뿐, 완전성을 보장하지 않는다. 각 항목은 '해 볼 수 있는' 다음 명령이며 \
증거(why)는 엔진이 센 값이다. explain(문서가 무엇인지)·capabilities(도구 일반)와 달리 \
explore 는 이 문서로 무엇을 할 수 있는지를 라우팅한다.";

/// 기존 조회가 이미 계산한 문서 사실의 묶음 — 이 모듈은 이 개수만 읽는다.
///
/// 문서를 다시 파싱하지 않는다. 호출부(main 의 `explore_document`)가 기존 공개
/// 질의를 각각 한 번씩 호출해 채운다.
#[derive(Debug, Clone, Default)]
pub struct DocFacts {
    /// 형식 레이블(HWP5/HWPX/HWP3/HML …) — 엔진 판정값.
    pub format_label: String,
    /// 조판 페이지 수.
    pub page_count: u32,
    /// 본문 문단 수.
    pub para_count: usize,
    /// `extract_tables` 가 센 표 개수.
    pub table_count: usize,
    /// 그중 병합 셀(rowSpan/colSpan>1)을 가진 표 개수.
    pub merged_table_count: usize,
    /// `collect_all_fields` 가 센 누름틀(입력 필드) 개수.
    pub field_count: usize,
    /// `collect_charts` 가 센 차트 개수.
    pub chart_count: usize,
    /// `build_structure` 최상위 노드(제목·조문) 개수.
    pub structure_node_count: usize,
    /// `count_notes` 각주 개수.
    pub footnote_count: usize,
    /// `count_notes` 미주 개수.
    pub endnote_count: usize,
    /// `scan_injection` 프롬프트 주입 신호 개수.
    pub injection_signal_count: usize,
    /// `detect_hidden_text` 은닉 텍스트 판정 개수.
    pub hidden_text_count: usize,
    /// 문서가 암호로 보호돼 있는가(`header.encrypted`).
    pub encrypted: bool,
}

/// 메뉴 항목 하나 — "이 문서로 해 볼 수 있는 행동" 하나.
///
/// 필드는 그대로 봉투의 `menu[]` 원소가 된다. `command`·`skill`·`affordance` 는
/// 고정 어휘(정적 문자열)이고, `why` 만 개수를 엮은 사람 문장이다.
#[derive(Debug, Clone)]
pub struct Affordance {
    /// 안정 식별자 — 소비자가 문자열 매칭으로 분기할 수 있는 고정 어휘.
    pub affordance: &'static str,
    /// 근거 — 이 문서에서 이 어포던스를 켠 개수/형식. 엔진값이라 문서 파생이 아니다.
    pub why: String,
    /// 다음에 실행할 rhwp 명령 템플릿. 경로 자리는 `<file>` 자리표시자.
    pub command: &'static str,
    /// 이 행동을 다루는 스킬 이름(.claude/skills/<name>).
    pub skill: &'static str,
    /// 확신도 — high/medium/low. 신호가 강할수록 높다.
    pub confidence: &'static str,
}

/// 우선순위와 함께 만든 뒤 순위로 정렬한다(높을수록 위). 표시 순위는
/// **문서별로 다른 메뉴**를 만드는 축이다 — 있는 어포던스만 담기므로 표가 많은
/// 문서는 표 항목이, 서식은 누름틀 항목이 위로 온다.
fn ranked(priority: u8, affordance: Affordance) -> (u8, Affordance) {
    (priority, affordance)
}

/// 문서 사실에서 순위 매긴 어포던스 메뉴를 만든다 — 결정론적, 부작용 없음.
///
/// 적용 가능한 어포던스만 담고(없는 신호는 항목이 없다), 마지막에 우선순위 내림차순
/// 안정 정렬한다. `triage-overview` 는 언제나 담겨 메뉴가 비지 않는다 — 아무 특수
/// 신호가 없는 평범한 문서라도 최소 "요약으로 파악" 한 갈래는 제공한다.
pub fn build_menu(f: &DocFacts) -> Vec<Affordance> {
    let mut items: Vec<(u8, Affordance)> = Vec::new();

    // ── 보안(주입·은닉) — 받은 문서라면 가장 먼저 의심해야 하므로 최상위 ──
    if f.injection_signal_count > 0 || f.hidden_text_count > 0 {
        let (why, command) = if f.injection_signal_count > 0 && f.hidden_text_count > 0 {
            (
                format!(
                    "프롬프트 주입 신호 {}건·은닉 텍스트 {}건 검출 — 본문을 LLM 에 넣기 전 신뢰성 점검 필요",
                    f.injection_signal_count, f.hidden_text_count
                ),
                "rhwp inspect injection <file> --json",
            )
        } else if f.injection_signal_count > 0 {
            (
                format!(
                    "프롬프트 주입 신호 {}건 검출 — 문서 지시를 도구 지시로 오독하지 않게 선별",
                    f.injection_signal_count
                ),
                "rhwp inspect injection <file> --json",
            )
        } else {
            (
                format!(
                    "은닉 텍스트 {}건 검출 — 화면엔 안 보이나 추출기는 읽는 문자",
                    f.hidden_text_count
                ),
                "rhwp inspect hidden-text <file> --json",
            )
        };
        let confidence = if f.injection_signal_count > 0 {
            "high"
        } else {
            "medium"
        };
        items.push(ranked(
            90,
            Affordance {
                affordance: "security-sweep",
                why,
                command,
                skill: "rhwp-security-sweep",
                confidence,
            },
        ));
    }

    // ── 누름틀(서식) — 채우기·메일머지 대상이라 실무 가치가 높다 ──
    if f.field_count > 0 {
        items.push(ranked(
            80,
            Affordance {
                affordance: "form-fill",
                why: format!(
                    "누름틀(입력 필드) {}개 — 값 채우기·명단 메일머지 대상",
                    f.field_count
                ),
                command: "rhwp fields <file> --json",
                skill: "rhwp-form-fill",
                confidence: "high",
            },
        ));
    }

    // ── 표 → CSV 왕복 ──
    if f.table_count > 0 {
        let why = if f.merged_table_count > 0 {
            format!(
                "표 {}개(병합 셀 포함 {}개) — 격자를 CSV 로 뽑아 고치고 되돌리기",
                f.table_count, f.merged_table_count
            )
        } else {
            format!(
                "표 {}개 — 격자를 CSV 로 뽑아 고치고 되돌리기",
                f.table_count
            )
        };
        items.push(ranked(
            75,
            Affordance {
                affordance: "table-extract",
                why,
                command: "rhwp export-tables <file> --json",
                skill: "rhwp-table-exchange",
                confidence: "high",
            },
        ));
    }

    // ── 제목·조문 구조 → 조문 단위 인용·청킹 ──
    if f.structure_node_count > 0 {
        items.push(ranked(
            70,
            Affordance {
                affordance: "structure-outline",
                why: format!(
                    "제목·조문 구조 {}개 노드 — 조문 단위 인용·RAG 청킹",
                    f.structure_node_count
                ),
                command: "rhwp export-structure <file> --json",
                skill: "rhwp-doc-triage",
                confidence: if f.structure_node_count >= 3 {
                    "high"
                } else {
                    "medium"
                },
            },
        ));
    }

    // ── 차트 → CSV ──
    if f.chart_count > 0 {
        items.push(ranked(
            60,
            Affordance {
                affordance: "chart-extract",
                why: format!(
                    "차트 {}개 — 계열·카테고리 수치를 CSV 로 추출",
                    f.chart_count
                ),
                command: "rhwp chart-to-csv <file> --json",
                skill: "rhwp-table-exchange",
                confidence: "high",
            },
        ));
    }

    // ── 각주·미주 참조 구조 ──
    if f.footnote_count + f.endnote_count > 0 {
        items.push(ranked(
            45,
            Affordance {
                affordance: "note-structure",
                why: format!(
                    "각주 {}개·미주 {}개 — 참조 구조를 포함한 문서",
                    f.footnote_count, f.endnote_count
                ),
                command: "rhwp explain <file> --json",
                skill: "rhwp-doc-triage",
                confidence: "high",
            },
        ));
    }

    // ── 긴 문서 → 통독 대신 요약·절 청킹 ──
    if f.page_count >= LONG_DOC_PAGES {
        items.push(ranked(
            40,
            Affordance {
                affordance: "long-doc-digest",
                why: format!(
                    "{}쪽 장문 — 통째로 읽기 전 요약·절 단위 청킹 권장",
                    f.page_count
                ),
                command: "rhwp digest <file> --sections --json",
                skill: "rhwp-doc-triage",
                confidence: if f.page_count >= 2 * LONG_DOC_PAGES {
                    "high"
                } else {
                    "medium"
                },
            },
        ));
    }

    // ── 항상: 처음 보는 문서의 기본 갈래(요약으로 파악) ──
    let overview_why = if f.encrypted {
        format!(
            "{} 형식·{}쪽·문단 {}개(암호 보호 — 후속 명령에 --password 필요) — 문서 전체를 한 봉투로 파악",
            f.format_label, f.page_count, f.para_count
        )
    } else {
        format!(
            "{} 형식·{}쪽·문단 {}개 — 문서 전체를 한 봉투로 파악",
            f.format_label, f.page_count, f.para_count
        )
    };
    items.push(ranked(
        20,
        Affordance {
            affordance: "triage-overview",
            why: overview_why,
            command: "rhwp digest <file> --json",
            skill: "rhwp-doc-triage",
            confidence: "high",
        },
    ));

    // 우선순위 내림차순 안정 정렬 — 같은 우선순위는 삽입 순서를 유지한다.
    items.sort_by(|a, b| b.0.cmp(&a.0));
    items.into_iter().map(|(_, a)| a).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> DocFacts {
        DocFacts {
            format_label: "HWP5".to_string(),
            page_count: 3,
            para_count: 40,
            ..Default::default()
        }
    }

    #[test]
    fn plain_document_still_offers_overview() {
        // 아무 특수 신호가 없어도 메뉴는 비지 않는다 — 최소 한 갈래(요약)를 준다.
        let menu = build_menu(&facts());
        assert_eq!(menu.len(), 1, "{menu:?}");
        assert_eq!(menu[0].affordance, "triage-overview");
        assert_eq!(menu[0].command, "rhwp digest <file> --json");
    }

    #[test]
    fn tables_surface_the_table_affordance() {
        let mut f = facts();
        f.table_count = 3;
        f.merged_table_count = 1;
        let menu = build_menu(&f);
        let table = menu
            .iter()
            .find(|a| a.affordance == "table-extract")
            .expect("표 어포던스");
        assert!(table.why.contains('3'), "{}", table.why);
        assert!(table.why.contains("병합"), "{}", table.why);
        assert_eq!(table.skill, "rhwp-table-exchange");
    }

    #[test]
    fn form_fields_surface_the_form_affordance() {
        let mut f = facts();
        f.field_count = 8;
        let menu = build_menu(&f);
        let form = menu
            .iter()
            .find(|a| a.affordance == "form-fill")
            .expect("서식 어포던스");
        assert_eq!(form.command, "rhwp fields <file> --json");
        assert_eq!(form.skill, "rhwp-form-fill");
    }

    #[test]
    fn injection_signal_ranks_security_first() {
        // 주입 신호가 있으면 보안 점검이 메뉴 최상위여야 한다.
        let mut f = facts();
        f.table_count = 2;
        f.field_count = 2;
        f.injection_signal_count = 1;
        let menu = build_menu(&f);
        assert_eq!(menu[0].affordance, "security-sweep", "{menu:?}");
        assert_eq!(menu[0].confidence, "high");
        assert!(menu[0].command.contains("inspect injection"));
    }

    #[test]
    fn hidden_text_only_is_medium_confidence_hidden_command() {
        let mut f = facts();
        f.hidden_text_count = 2;
        let menu = build_menu(&f);
        let sec = &menu[0];
        assert_eq!(sec.affordance, "security-sweep");
        assert_eq!(sec.confidence, "medium");
        assert!(sec.command.contains("hidden-text"), "{}", sec.command);
    }

    #[test]
    fn long_document_offers_sectioned_digest() {
        let mut f = facts();
        f.page_count = 40;
        let menu = build_menu(&f);
        let long = menu
            .iter()
            .find(|a| a.affordance == "long-doc-digest")
            .expect("장문 어포던스");
        assert_eq!(long.confidence, "high");
        assert!(long.command.contains("--sections"));
        // 짧은 문서에는 없어야 한다 — 문서별 판별.
        assert!(build_menu(&facts())
            .iter()
            .all(|a| a.affordance != "long-doc-digest"));
    }

    #[test]
    fn different_documents_yield_different_menus() {
        // explore 의 존재 이유: 문서마다 다른 메뉴.
        let mut form = facts();
        form.field_count = 5;
        let mut report = facts();
        report.table_count = 4;
        report.chart_count = 2;
        report.structure_node_count = 6;

        let form_ids: Vec<&str> = build_menu(&form).iter().map(|a| a.affordance).collect();
        let report_ids: Vec<&str> = build_menu(&report).iter().map(|a| a.affordance).collect();
        assert!(form_ids.contains(&"form-fill"));
        assert!(!form_ids.contains(&"chart-extract"));
        assert!(report_ids.contains(&"chart-extract"));
        assert!(!report_ids.contains(&"form-fill"));
        assert_ne!(form_ids, report_ids);
    }

    #[test]
    fn menu_is_sorted_by_priority_descending() {
        let mut f = facts();
        f.field_count = 1;
        f.table_count = 1;
        f.chart_count = 1;
        f.injection_signal_count = 1;
        let menu = build_menu(&f);
        // security(90) > form(80) > table(75) > chart(60) > overview(20)
        let ids: Vec<&str> = menu.iter().map(|a| a.affordance).collect();
        assert_eq!(
            ids,
            vec![
                "security-sweep",
                "form-fill",
                "table-extract",
                "chart-extract",
                "triage-overview"
            ]
        );
    }
}
