//! [#7080] `ㆍ`(U+318D) 의 전진폭이 반각이다 — 한/글은 전각을 준다.
//!
//! ## 증상
//!
//! 공문서에서 `내과ㆍ결핵과ㆍ흉부외과` 처럼 항목 구분자로 매우 흔한 글자인데, rhwp 는
//! 0.5 em 으로 전진시킨다. 한 줄에 여러 번 나오면 뒤 글자가 통째로 왼쪽으로 밀린다.
//!
//! ## 근인 — 메트릭보다 앞서는 폴백
//!
//! `text_measurement::area_dot_fallback_width` 가 한양신명조를 뺀 **모든 글꼴에
//! `font_size * 0.5`** 를 박았다. 이 함수는 `char_width_decision` 이 메트릭을 조회하기
//! **전에** 부르므로, 메트릭 DB(`BatangChe`·`Dotum`·`Batang` 전부 1024/1024)도
//! `is_cjk_char` 휴리스틱(`0x3130..=0x318F` 포함)도 전각을 주는데 결과는 반각이 된다.
//!
//! ## 정본 실측 — 반각이 한 건도 없다
//!
//! ```text
//!   저장소 한컴 정본 PDF 608개(앞 6쪽)   U+318D 158회 / 문서 38개
//!     0.80 em 이상  158회 = 전건       반각(0.35~0.65)  0회
//!
//!   samples/hwpx/form-002.hwpx 의 정본 두 판본(pdf/hwpx/form-002-2022.pdf ·
//!   pdf/hwpx/hancom-hwp/form-002-hwp-2020.pdf)은 글자 단위로 같다
//!     `글로벌 제약ㆍ바이오 생산기지 구축`   face MalgunGothic  /W 0.9767 em
//!       첫~끝 글자 원점간  정본 211.68 px
//!                          수정 전 204.84 px (-6.84)   수정 후 211.84 px (+0.16)
//! ```
//!
//! `#2070` 이 반각 근거로 든 80168 도 쪽 제한 없이 읽으면 `시ㆍ도조례` 가 세 글꼴 모두
//! 전각이다(216회 전건 0.8 em 이상). 다만 `#2070` 이 본 것은 같은 문서번호의 **개정안
//! 첨부**이고 그것은 저장소에 없다 — 그 관측 자체를 반증한 것은 아니라고 기록해 둔다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

/// 정본 `pdf/80168_regulatory_analysis-2022.pdf` 외 2판. `ㆍ` 를 사용자 슬롯 글꼴
/// `명조`(메트릭 없음)와 `맑은 고딕`에 태운다.
const SAMPLE_80168: &str = "samples/80168_regulatory_analysis.hwp";
/// 정상 원본의 `pdf/86712_regulatory_analysis-hwp-2024.pdf`: MalgunGothic 45개와
/// BatangChe 1개에서 다음 글리프 원점까지 1.0 em을 확인했다(2026-09-17).
const SAMPLE_86712: &str = "samples/86712_regulatory_analysis.hwp";
/// 정본 `pdf/3249937_asset_management_rules-2020.pdf`. 정본 face 는 `휴먼명조`(1.001 em, n=40).
const SAMPLE_3249937: &str = "samples/issue6031/3249937_asset_management_rules.hwpx";

/// 한 문서에서 `ㆍ` 의 (글꼴, 글자크기 대비 전진폭) 을 모은다.
fn area_dot_advances(sample: &str, max_pages: u32) -> Vec<(String, f64)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let mut out = Vec::new();
    for page in 0..core.page_count().min(max_pages) {
        let Ok(raw) = core.get_page_text_layout_native(page) else {
            continue;
        };
        let layout: serde_json::Value = serde_json::from_str(&raw).expect("text-layout JSON");
        for run in layout["runs"].as_array().into_iter().flatten() {
            let text: Vec<char> = run["text"].as_str().unwrap_or_default().chars().collect();
            let font_size = run["fontSize"].as_f64().unwrap_or(0.0);
            let family = run["fontFamily"].as_str().unwrap_or_default().to_string();
            let char_x: Vec<f64> = run["charX"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_f64)
                .collect();
            if font_size <= 0.0 || char_x.len() < text.len() + 1 {
                continue;
            }
            for (i, &c) in text.iter().enumerate() {
                if c == '\u{318D}' {
                    out.push((family.clone(), (char_x[i + 1] - char_x[i]) / font_size));
                }
            }
        }
    }
    out
}

/// `ㆍ` 는 전각으로 전진한다 — 글꼴 셋 계열이 함께 잠긴다.
///
/// 수정 전에는 전건 0.497~0.503 em 이었다.
#[test]
fn area_dot_is_full_width_across_font_families() {
    let mut dots: Vec<(String, f64)> = Vec::new();
    for sample in [SAMPLE_80168, SAMPLE_86712, SAMPLE_3249937] {
        dots.extend(area_dot_advances(sample, 40));
    }
    assert!(
        dots.len() >= 20,
        "`ㆍ` 를 20개 이상 봐야 한다 — 검사 대상이 0건이면 통과 증거가 아니다. got {}",
        dots.len()
    );
    let narrow: Vec<_> = dots.iter().filter(|(_, adv)| *adv < 0.85).collect();
    assert!(
        narrow.is_empty(),
        "`ㆍ` 는 정본에서 전건 전각이다(0.80 em 이상 158/158). 반각으로 잰 것: {:?} (전체 {}개)",
        &narrow[..narrow.len().min(8)],
        dots.len()
    );
}

/// 메트릭이 **없는** 글꼴도 전각으로 전진한다.
///
/// 이 문서들은 `ㆍ` 를 사용자(USER) 슬롯 글꼴 `명조` 에 태우는데, `명조` 는 별칭도 메트릭도
/// 없다. 폴백을 남겨 둔 이유가 그것이고, 그때의 답도 전각이다 — 폴백을 통째로 지우면
/// 이 경로가 휴리스틱으로 새어 계약이 흐려진다.
#[test]
fn area_dot_without_a_metric_is_also_full_width() {
    let dots = area_dot_advances(SAMPLE_80168, 40);
    let no_metric: Vec<_> = dots.iter().filter(|(f, _)| f == "명조").collect();
    assert!(
        !no_metric.is_empty(),
        "메트릭 없는 글꼴(`명조`)의 `ㆍ` 가 하나도 없다 — 표본 전제가 깨졌다"
    );
    let narrow: Vec<_> = no_metric.iter().filter(|(_, adv)| *adv < 0.85).collect();
    assert!(
        narrow.is_empty(),
        "메트릭 없는 글꼴의 `ㆍ` 도 전각이어야 한다: {narrow:?}"
    );
}

/// 전각 ㆍ와 NO_LS 공백 보정을 함께 검증한다. 독립 기준인 한컴 2022 PDF
/// p108은 오른쪽 셀의 9호와 마지막 3호를 각각 한 줄로 같은 쪽에 둔다.
#[test]
fn issue_7080_fullwidth_area_dot_keeps_no_ls_statute_rows_on_hancom_page_108() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn lines(node: &RenderNode, out: &mut Vec<String>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            out.push(
                node.children
                    .iter()
                    .filter_map(|n| match &n.node_type {
                        RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                        _ => None,
                    })
                    .collect(),
            );
        }
        for child in &node.children {
            lines(child, out);
        }
    }
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE_80168)).unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let tree = core.build_page_render_tree(107).unwrap();
    let mut text = Vec::new();
    lines(&tree.root, &mut text);
    for item in ["9.", "3."] {
        let expected = format!("{item} 그 밖에 시ㆍ도조례로 정하는 사항");
        assert!(
            text.iter().any(|line| line.trim() == expected),
            "Hancom p108 keeps {expected:?} on a single physical line"
        );
    }
}

/// 한컴 p49의 다줄 대조군: 한 줄 문단의 고유 공백을 전역 전파하면
/// 첫 줄에 `용`까지 당겨지고, 마지막 `인` 행이 없어져 다음 항이 올라간다.
#[test]
fn issue_7080_multiline_statute_keeps_hancom_page_49_three_rows() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn visit(node: &RenderNode, out: &mut Vec<String>) {
        if let RenderNodeType::TextLine(line) = &node.node_type {
            if line.para_index == Some(2)
                && node.bbox.x > 390.0
                && node.bbox.y > 500.0
                && node.bbox.y < 610.0
            {
                out.push(
                    node.children
                        .iter()
                        .filter_map(|c| match &c.node_type {
                            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                            _ => None,
                        })
                        .collect::<String>()
                        .split_whitespace()
                        .collect(),
                );
            }
        }
        for child in &node.children {
            visit(child, out);
        }
    }
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE_80168)).unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let mut lines = Vec::new();
    visit(&core.build_page_render_tree(48).unwrap().root, &mut lines);
    assert_eq!(
        lines,
        [
            "1.자산관리회사와자산의투자ㆍ운",
            "용에관한위탁계약을체결한법",
            "인"
        ]
    );
}

/// p75의 다줄 문단 마지막 `다)`를 별도 줄로 밀어 p76~77을 늘리지 않는다.
#[test]
fn issue_7080_multiline_terminal_row_does_not_push_the_next_page() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn collect(node: &RenderNode, lines: &mut Vec<String>) {
        if let RenderNodeType::TextLine(line) = &node.node_type {
            if line.para_index == Some(2) && node.bbox.x > 390.0 && node.bbox.y > 300.0 {
                lines.push(
                    node.children
                        .iter()
                        .filter_map(|c| match &c.node_type {
                            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                            _ => None,
                        })
                        .collect::<String>(),
                );
            }
        }
        for child in &node.children {
            collect(child, lines);
        }
    }
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE_80168)).unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let mut lines = Vec::new();
    collect(&core.build_page_render_tree(74).unwrap().root, &mut lines);
    let compact: Vec<String> = lines
        .iter()
        .map(|line| line.split_whitespace().collect())
        .collect();
    assert_eq!(
        compact,
        [
            "2.건축물이아닌부대시설ㆍ복리",
            "시설의설치규모를확대하는때",
            "(위치가변경되는경우는제외한다)"
        ],
        "한컴 p75의 부대시설 조항과 같은 3행을 보존한다"
    );
}
