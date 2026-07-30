//! Issue #3504: HWP3 → HWP5 변환에서 미주 문단마다 말미 공백 1칸이 붙는다.
//!
//! ## 근인
//!
//! 자동번호(`0x0012`)는 공통 IR 규약상 **확장 컨트롤 8 코드유닛**을 차지한다. HWP5 파서는
//! 그렇게 세는데(`parser/body_text.rs`), HWP3 파서는 placeholder 공백을 넣으면서 **1 유닛만**
//! 셌다.
//!
//! 직렬화는 placeholder 를 "다음 문자 오프셋이 8 이상 뛰는가" 로 알아본다
//! (`serializer/body_text.rs` 의 `next_offset >= offset + 8`). HWP3 쪽 오프셋이 연속이라 그
//! 판정이 실패해 공백은 리터럴로 쓰이고, 남은 컨트롤은 문단 **끝**에 다시 방출됐다.
//! 재파싱하면 그 끝 컨트롤의 placeholder 가 본문 끝에 생겨 **말미 공백 1칸**이 된다.
//!
//! `samples/SO-SUEOP.hwp`(HWP 3.0, 미주 223개) 기준 213개 문단이 영향받았고,
//! `export-text` 바이트가 136,066 → 136,279 로 늘었다.
//!
//! ## 계약
//!
//! 1. HWP3 파싱에서 자동번호 컨트롤은 8 코드유닛을 차지한다 — 뒤따르는 문자의
//!    `char_offsets` 가 8 이상 뛴다.
//! 2. HWP3 → HWP5 변환이 **텍스트-안정**이다 — 미주 문단 본문이 그대로다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

/// HWP 3.0 문서. 미주 223개, 그중 213개가 회귀 대상이었다.
const SAMPLE: &str = "samples/SO-SUEOP.hwp";

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn load(path: &std::path::Path) -> rhwp::model::document::Document {
    let bytes = std::fs::read(path).expect("파일 읽기");
    rhwp::parser::parse_document(&bytes).expect("파싱")
}

/// 미주 본문 문단만 모은다 (`Control::Endnote` 안에 있어 재귀가 필요하다).
fn endnote_paragraphs(doc: &rhwp::model::document::Document) -> Vec<Paragraph> {
    fn walk(paras: &[Paragraph], out: &mut Vec<Paragraph>) {
        for p in paras {
            for c in &p.controls {
                match c {
                    Control::Endnote(e) => {
                        out.extend(e.paragraphs.iter().cloned());
                        walk(&e.paragraphs, out);
                    }
                    Control::Table(t) => {
                        for cell in &t.cells {
                            walk(&cell.paragraphs, out);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    let mut out = Vec::new();
    for s in &doc.sections {
        walk(&s.paragraphs, &mut out);
    }
    out
}

/// 자동번호 컨트롤을 가진 문단에서 placeholder 뒤 오프셋이 8 이상 뛴다.
#[test]
fn hwp3_auto_number_occupies_eight_code_units() {
    let doc = load(&sample_path());
    let paras = endnote_paragraphs(&doc);
    assert!(
        paras.len() > 100,
        "미주 문단이 너무 적다({}) — 표본이 바뀌었는지 확인하라",
        paras.len()
    );

    let mut checked = 0usize;
    let mut violations = Vec::new();
    for (pi, p) in paras.iter().enumerate() {
        if !p
            .controls
            .iter()
            .any(|c| matches!(c, Control::AutoNumber(_)))
        {
            continue;
        }
        // placeholder 공백의 위치를 찾는다: 뒤 문자가 있는 자리만 검사한다.
        for i in 0..p.char_offsets.len().saturating_sub(1) {
            let Some(' ') = p.text.chars().nth(i) else {
                continue;
            };
            let gap = p.char_offsets[i + 1] as i64 - p.char_offsets[i] as i64;
            if gap == 1 {
                continue; // 진짜 공백
            }
            checked += 1;
            if gap < 8 && violations.len() < 3 {
                violations.push(format!(
                    "미주 문단 {pi}: placeholder 뒤 오프셋이 {gap} 만 뛴다 (8 이상이어야 함)"
                ));
            }
        }
    }
    assert!(
        checked > 0,
        "자동번호 placeholder 를 하나도 찾지 못했다 — 표본이 바뀌었는지 확인하라"
    );
    assert!(
        violations.is_empty(),
        "자동번호가 8 코드유닛을 차지하지 않는다:\n  {}",
        violations.join("\n  ")
    );
}

/// HWP3 → HWP5 변환이 미주 본문을 그대로 보존한다 (말미 공백이 붙지 않는다).
#[test]
fn hwp3_to_hwp5_keeps_endnote_text() {
    let src = load(&sample_path());
    let bytes = rhwp::serializer::serialize_document(&src).expect("직렬화");
    let round = rhwp::parser::parse_document(&bytes).expect("재파싱");

    let a = endnote_paragraphs(&src);
    let b = endnote_paragraphs(&round);
    assert_eq!(a.len(), b.len(), "미주 문단 수가 달라졌다");
    assert!(a.len() > 100, "미주 문단이 너무 적다 — 표본 확인");

    let mut drift = Vec::new();
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        if x.text != y.text {
            if drift.len() < 3 {
                drift.push(format!("문단 {i}: {:?} -> {:?}", x.text, y.text));
            }
        }
    }
    assert!(
        drift.is_empty(),
        "저장 후 미주 본문이 달라졌다 ({}건):\n  {}\n\
         자동번호 placeholder 가 컨트롤로 인식되지 않으면 공백이 리터럴로 남고 \
         컨트롤이 문단 끝에 다시 방출된다.",
        a.iter()
            .zip(b.iter())
            .filter(|(x, y)| x.text != y.text)
            .count(),
        drift.join("\n  ")
    );
}
