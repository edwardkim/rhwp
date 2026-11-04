//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8429의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:36:33+09:00 (`756ca7812f269c0c383622eb0d688d98910da8b7`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `mydocs/pr/assets/issue_8424/inputs/after/business_plan.hwpx`; 독립 Print `pdf/issue-8424/business_plan-2024.pdf`.
//! 전체 3쪽, 전쪽 최저 Native 99.47% / fresh WASM 99.51%.
//! 주장 2쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `mydocs/pr/assets/issue_8424/inputs/after/cellwrap.hwpx`; 독립 Print `pdf/issue-8424/cellwrap-2024.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8429_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-01T15:47:00+09:00 (새 파일).
//! 2026-11-01 측정 raw(`visual_sweep.py --silhouette-only`, 96dpi, pdftoppm, `--compat 2024`):
//! cellwrap 1쪽 Native 99.99% / fresh WASM 100.00% (전1쪽, 한컴 1쪽);
//! business_plan 1쪽 97.74% / 97.71%, 2쪽 98.59% / 98.63%, 3쪽 100.00% / 100.00%
//! (전3쪽 최저 Native 97.74% / fresh WASM 97.71%, 한컴 3쪽). 수정 전 cellwrap 56.73%,
//! business_plan 은 rhwp 2쪽 / 한컴 3쪽으로 측정 불가.
//! 글꼴: rhwp `RHWP_FONT_PATH=ttfs/opensource`(함초롬바탕 미설치), 한컴 font_scope verified.
//! 근거: mydocs/pr/assets/issue_8424/README.md, 정답지 pdf/issue-8424/.
//!
//! [#8424] scaffold 가 글 문단(제목·본문·표 셀)의 저장 줄 정보(LineSeg)를 꾸며 넣지 않는다.
//!
//! ## 무엇이 문제였나
//!
//! scaffold 는 글꼴 측정·줄 나눔을 하지 않으면서 셀 문단마다 1줄짜리 LineSeg 를 만들었다.
//! rhwp 셀 배치는 그 저장 줄을 믿고 긴 셀 텍스트를 한 줄에 놓아 셀 경계에서 잘라 그렸고,
//! 한컴 2024 는 다시 조판해 셀 안에서 줄을 바꾸고 행을 늘렸다. 표가 있는 긴 문서는
//! 전체 쪽수까지 달라졌다(사업계획서형 명세: rhwp 2쪽, 한컴 3쪽).
//!
//! ## 기대값의 출처 — 수정과 독립적인 근거
//!
//! - 한컴오피스 2024 `13.0.0.3901` Print PDF(`frame_print_to_pdf_ex_one_up`):
//!   `pdf/issue-8424/` 의 정답지. 셀 "세부 과업 1: 요구사항 정의, 설계, 구현 및 시험" 은
//!   3줄로 줄바꿈되고 같은 행의 다른 칸보다 행이 높다. 사업계획서형 명세는 3쪽이다.
//! - 한컴은 저장 줄과 무관하게 다시 조판한다: 같은 명세 산출물에서 `<hp:linesegarray>` 를
//!   모두 지운 파일의 한컴 Print 출력이 원본과 픽셀 단위로 같았다.
//! - rhwp 는 HWPX 의 빈 저장 줄을 로드 시 재조판 대상으로 본다(#1380).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::scaffold::{build_scaffold, parse_scaffold_str};
use rhwp::serializer::serialize_hwpx;

const LONG_CELL: &str = "세부 과업 1: 요구사항 정의, 설계, 구현 및 시험";

/// `pdf/issue-8424/cellwrap-2024.pdf` 의 입력 명세.
const CELLWRAP_SPEC: &str = r#"{"version":"1","title":"셀 줄바꿈 재현","blocks":[
  {"type":"table","rows":[
    ["단계","주요 내용","기간","산출물"],
    ["1단계","세부 과업 1: 요구사항 정의, 설계, 구현 및 시험","1~2월","결과 보고서"],
    ["2단계","짧은 내용","3~4월","결과 보고서"]
  ]},
  {"type":"paragraph","text":"표 뒤 문단"}
]}"#;

/// `pdf/issue-8424/business_plan-2024.pdf` 의 입력 명세(본문 문단 15개 + 4×7 표).
fn business_plan_spec() -> String {
    let para = "공공기관과 기업의 보고서·계획서는 대부분 한글 문서로 작성되며, 동일한 양식을 반복해서 채우는 작업에 많은 시간이 소요된다. 특히 개요 번호, 표 서식, 쪽 번호와 같은 형식 요소를 손으로 맞추는 과정에서 오류가 자주 발생하고, 제출 직전의 검토 부담이 크다.";
    let mut blocks = vec![
        r#"{"type":"heading","level":1,"text":"Ⅰ. 사업 개요"}"#.to_string(),
        r#"{"type":"table","cell_align":["center","left"],"rows":[["사업명","HWPX 기반 문서 자동 작성 플랫폼 구축"],["사업 기간","2027. 3. 1. ~ 2028. 2. 29.(12개월)"],["총사업비","350,000천원(정부 250,000 / 민간 100,000)"],["주관 기관","(주)예시소프트"]]}"#.to_string(),
        r#"{"type":"heading","level":1,"text":"Ⅱ. 추진 필요성"}"#.to_string(),
    ];
    let p = format!(r#"{{"type":"paragraph","text":"{para}"}}"#);
    for i in 1..=3 {
        blocks.push(format!(
            r#"{{"type":"heading","level":2,"text":"{i}. 현황 분석 {i}"}}"#
        ));
        for _ in 0..4 {
            blocks.push(p.clone());
        }
    }
    blocks.push(r#"{"type":"heading","level":1,"text":"Ⅲ. 추진 체계 및 일정"}"#.to_string());
    let mut rows = vec![r#"["단계","주요 내용","기간","산출물"]"#.to_string()];
    for k in 1..=6 {
        rows.push(format!(
            r#"["{k}단계","세부 과업 {k}: 요구사항 정의, 설계, 구현 및 시험","{}~{}월","결과 보고서"]"#,
            k * 2 - 1,
            k * 2
        ));
    }
    blocks.push(format!(
        r#"{{"type":"table","cell_align":["center","left","center","center"],"rows":[{}]}}"#,
        rows.join(",")
    ));
    blocks.push(r#"{"type":"heading","level":1,"text":"Ⅳ. 기대 효과"}"#.to_string());
    for _ in 0..6 {
        blocks.push(p.clone());
    }
    format!(
        r#"{{"version":"1","title":"2027년 디지털 전환 지원사업 사업계획서","blocks":[{}]}}"#,
        blocks.join(",")
    )
}

fn hwpx_of(spec: &str) -> Vec<u8> {
    let spec = parse_scaffold_str(spec).expect("scaffold 명세");
    serialize_hwpx(&build_scaffold(&spec)).expect("HWPX 직렬화")
}

fn core_of(spec: &str) -> DocumentCore {
    DocumentCore::from_bytes(&hwpx_of(spec)).expect("scaffold 산출물 로드")
}

fn text_paragraphs(paras: &[Paragraph], out: &mut Vec<Paragraph>) {
    for p in paras {
        if !p.text.is_empty() {
            out.push(p.clone());
        }
        for c in &p.controls {
            if let Control::Table(t) = c {
                for cell in &t.cells {
                    text_paragraphs(&cell.paragraphs, out);
                }
            }
        }
    }
}

/// 생성 계약: 글이 있는 문단은 측정하지 않은 저장 줄을 갖지 않고, HWPX 에도 쓰지 않는다.
#[test]
fn scaffold_does_not_fabricate_line_segs_for_text_paragraphs() {
    let spec = parse_scaffold_str(CELLWRAP_SPEC).expect("scaffold 명세");
    let doc = build_scaffold(&spec);
    let mut paras = Vec::new();
    text_paragraphs(&doc.sections[0].paragraphs, &mut paras);
    assert!(
        paras.iter().any(|p| p.text == LONG_CELL),
        "셀 문단이 수집되어야 한다"
    );
    for p in &paras {
        assert!(
            p.line_segs.is_empty(),
            "{:?}: 저장 줄을 꾸며 넣으면 안 된다 ({}개)",
            p.text,
            p.line_segs.len()
        );
    }

    let hwpx = hwpx_of(CELLWRAP_SPEC);
    let section = section0_xml(&hwpx);
    let long = section.find(LONG_CELL).expect("긴 셀 텍스트");
    let para_end = long + section[long..].find("</hp:p>").expect("셀 문단 끝");
    assert!(
        !section[long..para_end].contains("linesegarray"),
        "긴 셀 문단에 linesegarray 를 쓰면 안 된다"
    );
}

fn section0_xml(hwpx: &[u8]) -> String {
    use std::io::Read;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("HWPX zip");
    let mut xml = String::new();
    zip.by_name("Contents/section0.xml")
        .expect("section0.xml")
        .read_to_string(&mut xml)
        .expect("UTF-8");
    xml
}

fn cells_of(node: &RenderNode, out: &mut Vec<(u16, u16, BoundingBox, String, Vec<BoundingBox>)>) {
    if let RenderNodeType::TableCell(cell) = &node.node_type {
        let mut text = String::new();
        let mut runs = Vec::new();
        collect_runs(node, &mut text, &mut runs);
        out.push((cell.row, cell.col, node.bbox, text, runs));
    }
    for child in &node.children {
        cells_of(child, out);
    }
}

fn collect_runs(node: &RenderNode, text: &mut String, runs: &mut Vec<BoundingBox>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.push_str(run.display_or_text());
        if !run.display_or_text().trim().is_empty() {
            runs.push(node.bbox);
        }
    }
    for child in &node.children {
        collect_runs(child, text, runs);
    }
}

fn strip_ws(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 긴 셀 텍스트는 글자 하나 빠짐없이 셀 안에 그려지고, 여러 줄을 차지해 행이 높아진다.
#[test]
fn long_cell_text_is_kept_inside_its_cell_on_several_lines() {
    let core = core_of(CELLWRAP_SPEC);
    let tree = core.build_page_render_tree(0).expect("render tree");
    let mut cells = Vec::new();
    cells_of(&tree.root, &mut cells);

    let (_, _, cell_box, text, runs) = cells
        .iter()
        .find(|(r, c, ..)| (*r, *c) == (1, 1))
        .expect("1행 1열 칸");
    assert_eq!(
        strip_ws(text),
        strip_ws(LONG_CELL),
        "셀 글자가 누락·중복 없이 그려져야 한다"
    );
    const EPS: f64 = 0.5;
    for run in runs {
        assert!(
            run.x >= cell_box.x - EPS
                && run.x + run.width <= cell_box.x + cell_box.width + EPS
                && run.y >= cell_box.y - EPS
                && run.y + run.height <= cell_box.y + cell_box.height + EPS,
            "글 조각 {run:?} 이 칸 {cell_box:?} 밖으로 나가면 안 된다"
        );
    }
    let mut line_tops: Vec<i64> = runs.iter().map(|b| b.y.round() as i64).collect();
    line_tops.sort_unstable();
    line_tops.dedup();
    assert!(
        line_tops.len() >= 2,
        "열 폭보다 긴 셀 텍스트는 여러 줄이어야 한다 (줄 {})",
        line_tops.len()
    );

    let short_row = cells
        .iter()
        .find(|(r, c, ..)| (*r, *c) == (2, 1))
        .expect("2행 1열 칸");
    assert!(
        cell_box.height > short_row.2.height + 1.0,
        "여러 줄 칸의 행({:.1}px)이 한 줄 칸의 행({:.1}px)보다 높아야 한다",
        cell_box.height,
        short_row.2.height
    );
}

/// 독립 한컴 2024 Print PDF 와 전체 쪽수가 같다.
#[test]
fn page_counts_match_hancom_print() {
    assert_eq!(
        core_of(CELLWRAP_SPEC).page_count(),
        1,
        "cellwrap-2024.pdf 는 1쪽"
    );
    assert_eq!(
        core_of(&business_plan_spec()).page_count(),
        3,
        "business_plan-2024.pdf 는 3쪽"
    );
}
