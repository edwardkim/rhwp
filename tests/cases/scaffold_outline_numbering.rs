//! 관계형 및 고정형 회귀: 스캐폴드 개요 번호의 입력·검증 오류·표시 문자열 계약. 화면 절대 위치 핀은 없다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! scaffold_outline_a.hwpx (control-scaffold-a): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! scaffold_outline_b.hwpx (control-scaffold-b): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8540의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T22:11:28+09:00 (`64034040c7cb3afe4521b1f5811258491a22c0ce`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `mydocs/pr/assets/scaffold_outline_numbering/scaffold_outline_a.hwpx`; 독립 Print `pdf/issue8540/scaffold_outline_a-2024.pdf`.
//! 전체 1쪽, Native 100.00% / fresh WASM 100.00%.
//! 입력 `mydocs/pr/assets/scaffold_outline_numbering/scaffold_outline_b.hwpx`; 독립 Print `pdf/issue8540/scaffold_outline_b-2024.pdf`.
//! 전체 1쪽, Native 100.00% / fresh WASM 100.00%.
//! 정식 근거: `mydocs/pr/pr_8540_review.md`, `mydocs/pr/assets/issue_8540/`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-03 22:10 KST. 직전 파일 변경: 2026-11-03 22:00:54 KST,
//! `dfac7066081763b5a7a9a15660aa019786d13a48`.
//! 시각 측정: 2026-11-03 22:03~22:07 KST, 96 DPI, Native/fresh WASM,
//! 판정값은 `silhouette_raw_match_percent`(색상 경계 보정 전 2px 실루엣)이다.
//! A와 B의 전체 1쪽은 두 출력 경로에서 각각 100.00%이며, 미달 쪽은 없다.
//! 근거: `mydocs/pr/assets/issue_8540/{a,b}-{native,wasm}.tsv`와 같은 경로의
//! `*-review.png`·`*-overlay.png`; 원본은
//! `mydocs/pr/assets/scaffold_outline_numbering/scaffold_outline_{a,b}.hwpx`,
//! 독립 Print는 `pdf/issue8540/scaffold_outline_{a,b}-2024.pdf`이다.
//! 측정한 통합 renderer 코드 SHA는 `0884eafc7b2c34fae20a765109ff314514175e4b`;
//! 본 파일의 scaffold 모델 변경은 실제 HWPX 출력과 아래 번호 문자열 검사로 보호한다.
//!
//! scaffold 명세의 `outline_numbering` 으로 개요 번호 모양을 지정한다.
//!
//! ## 무엇이 부족했나
//!
//! scaffold 는 개요 문단(`head_type=Outline`)만 만들고 문단 번호 정의를 넣지 않았다. 그래서
//! 개요 번호 모양을 고를 수 없었다 — 조판은 번호 정의가 없을 때의 대체값(`^N`, `1.1.`)으로
//! 그린다. 공공 문서는 `Ⅰ.` `1.` `가.` 처럼 기관마다 제목 번호 체계가 다르다.
//!
//! ## 기대값의 출처
//!
//! 1. **OWPML 문단 번호 정의**: `hh:numbering/hh:paraHead` 는 수준마다 형식 문자열(`^1.`)과
//!    `numFormat`(DIGIT·ROMAN_CAPITAL·HANGUL_SYLLABLE…)을 갖고, 구역 `secPr@outlineShapeIDRef`
//!    가 개요 번호로 쓸 정의를 가리킨다(1-based). 파서(`parser/hwpx/header.rs`
//!    `parse_numbering`)와 직렬화(`serializer/hwpx/header.rs` `write_numbering`)가 이미 이
//!    계약을 왕복한다.
//! 2. **한/글 새 문서의 개요 기본값**: `1.` `가.` `1)` `가)` `(1)` `(가)` `①` — 명세가 앞쪽 수준만
//!    주면 나머지는 이 값으로 채운다.
//! 3. **한/글 출력 대조**: 이 형식으로 만든 HWPX 를 한/글 2024 Print 경로로 인쇄하면 `I.` `1.`
//!    `가.` `가)` `(1)` `(가)` `①` 과 `i)` `A.` `(a)` 가 그대로 나온다. 일이삼·一二三(코드 12·13)은
//!    HWPX 직렬화 낱말을 한/글이 숫자로 그리므로 열지 않는다.
//!
//! 기본값은 바꾸지 않는다. `outline_numbering` 을 생략하면 번호 정의 없이 종전과 같은 문서를 만든다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::document::Document;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::scaffold::{build_scaffold, parse_scaffold_str, OutlineNumFormat, OUTLINE_DEFAULTS};
use rhwp::serializer::serialize_hwpx;

const BLOCKS: &str = r#""blocks":[
    {"type":"heading","level":1,"text":"추진 배경"},
    {"type":"heading","level":2,"text":"현황"},
    {"type":"heading","level":3,"text":"세부 내용"},
    {"type":"paragraph","text":"본문"},
    {"type":"heading","level":3,"text":"둘째 세부"},
    {"type":"heading","level":1,"text":"추진 계획"}
]"#;

const ROMAN: &str = r#""outline_numbering":[
    {"text":"^1.","num_format":"roman_capital"},
    {"text":"^2."},
    {"text":"^3.","num_format":"hangul_syllable"}
]"#;

fn spec(extra: &str) -> String {
    format!(r#"{{"version":"1","title":"번호 시험",{extra}{BLOCKS}}}"#)
}

fn doc_of(extra: &str) -> Document {
    build_scaffold(&parse_scaffold_str(&spec(extra)).expect("scaffold 명세"))
}

fn codes(doc: &Document) -> Vec<u8> {
    doc.doc_info.numberings[0]
        .heads
        .iter()
        .map(|h| h.number_format)
        .collect()
}

/// 생략하면 종전과 같다 — 번호 정의 없음, 구역 개요 번호 참조 0.
#[test]
fn omitted_keeps_previous_output() {
    let doc = doc_of("");
    assert!(doc.doc_info.numberings.is_empty());
    assert_eq!(doc.sections[0].section_def.outline_numbering_id, 0);
}

/// 지정한 수준은 그대로, 나머지는 한/글 개요 기본값으로 채운다.
#[test]
fn levels_and_defaults() {
    let doc = doc_of(&format!("{ROMAN},"));
    assert_eq!(doc.doc_info.numberings.len(), 1);
    assert_eq!(doc.sections[0].section_def.outline_numbering_id, 1);
    let n = &doc.doc_info.numberings[0];
    assert_eq!(n.level_formats.len(), rhwp::model::style::NUMBERING_LEVELS);
    let formats: Vec<&str> = n.level_formats.iter().map(String::as_str).collect();
    assert_eq!(
        formats,
        vec!["^1.", "^2.", "^3.", "^4)", "(^5)", "(^6)", "^7", "", "", ""],
        "4~7 수준은 한/글 기본값이며 Scaffold가 지정하지 않는 8~10 수준은 빈 형식"
    );
    assert_eq!(
        codes(&doc),
        vec![
            OutlineNumFormat::RomanCapital.code(),
            OutlineNumFormat::Digit.code(),
            OutlineNumFormat::HangulSyllable.code(),
            OUTLINE_DEFAULTS[3].1.code(),
            OUTLINE_DEFAULTS[4].1.code(),
            OUTLINE_DEFAULTS[5].1.code(),
            OUTLINE_DEFAULTS[6].1.code(),
            OutlineNumFormat::Digit.code(),
            OutlineNumFormat::Digit.code(),
            OutlineNumFormat::Digit.code(),
        ]
    );
    for h in &n.heads {
        assert_eq!(
            (h.attr >> 5) & 0x1f,
            u32::from(h.number_format),
            "속성 bit 5~9"
        );
    }
}

/// HWPX 로 쓰고 다시 읽어도 번호 정의와 구역 참조가 그대로다.
#[test]
fn hwpx_round_trip() {
    let doc = doc_of(&format!("{ROMAN},"));
    let hwpx = serialize_hwpx(&doc).expect("HWPX 직렬화");
    let back = rhwp::parser::parse_document(&hwpx).expect("HWPX 재파싱");
    assert_eq!(back.doc_info.numberings.len(), 1);
    assert_eq!(back.sections[0].section_def.outline_numbering_id, 1);
    let n = &back.doc_info.numberings[0];
    assert_eq!(&n.level_formats[..3], &["^1.", "^2.", "^3."]);
    assert_eq!(&codes(&back)[..3], &codes(&doc)[..3]);
}

/// 조판이 개요 번호를 지정한 모양으로 그린다 — 로마 대문자 `I.`·`II.`(조판의 `format_roman` 은
/// ASCII 로마자), `1.`, `가.`·`나.` 이고 번호 정의 대체값(`1.1.`)은 남지 않는다.
#[test]
fn render_uses_the_numbering() {
    let doc = doc_of(&format!("{ROMAN},"));
    let hwpx = serialize_hwpx(&doc).expect("HWPX 직렬화");
    let core = rhwp::document_core::DocumentCore::from_bytes(&hwpx).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("render tree");
    fn walk(node: &RenderNode, out: &mut Vec<String>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push(run.display_or_text().to_string());
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut runs = Vec::new();
    walk(&tree.root, &mut runs);
    let text = runs.join("|");
    for want in ["I.", "1.", "가.", "나.", "II."] {
        assert!(text.contains(want), "{want} 번호가 그려져야 한다: {text}");
    }
    assert!(
        !text.contains("1.1."),
        "번호 정의 대체값(1.1.)이 남으면 안 된다: {text}"
    );
}

/// 잘못된 입력은 이유를 알려 주며 즉시 실패한다.
#[test]
fn invalid_input_is_rejected_with_a_hint() {
    let err = |extra: &str| {
        parse_scaffold_str(&spec(&format!("{extra},")))
            .expect_err("거부해야 한다")
            .to_string()
    };
    assert!(err(r#""outline_numbering":[]"#).contains("1~7개"));
    let eight = format!(
        r#""outline_numbering":[{}]"#,
        [r#"{"text":"^1"}"#; 8].join(",")
    );
    assert!(err(&eight).contains("1~7개"));
    assert!(err(r#""outline_numbering":[{"text":" "}]"#).contains("비었"));
    let deeper = err(r#""outline_numbering":[{"text":"^2."}]"#);
    assert!(
        deeper.contains("수준 1") && deeper.contains("1~1"),
        "{deeper}"
    );
    assert!(err(r#""outline_numbering":[{"text":"^x"}]"#).contains("'^'"));
    let fmt = err(r#""outline_numbering":[{"text":"^1.","num_format":"roman"}]"#);
    assert!(
        fmt.contains("roman_capital"),
        "지원 형식 목록을 알려 줘야 한다: {fmt}"
    );
    // 한/글이 HWPX 직렬화 낱말(HANGUL_NUMBER·HANJA_NUMBER)을 숫자로 그리는 형식은 열지 않는다.
    for closed in ["hangul_number", "hanja_number"] {
        let msg = err(&format!(
            r#""outline_numbering":[{{"text":"^1.","num_format":"{closed}"}}]"#
        ));
        assert!(msg.contains(closed), "{msg}");
    }
    let unknown = err(r#""outline_numbering":[{"text":"^1.","align":"left"}]"#);
    assert!(unknown.contains("align"), "{unknown}");
}
