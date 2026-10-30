//! Issue #1061: HWPX 수식 저장 회귀 가드.
//!
//! 본 task 정정 3 영역:
//!
//! 1. **HWPX 어댑터 Equation arm** (`src/document_core/converters/hwpx_to_hwp.rs`) —
//!    `adapt_paragraph` match 에 `Control::Equation` 추가 + `adapt_equation` 함수 신규.
//!    GenShape attr bit 27 (0x08000000) 보강 + raw_ctrl_data clear.
//!
//! 2. **EQEDIT unknown UINT2 field** (`src/model/control.rs::Equation`,
//!    `src/parser/control.rs`, `src/serializer/control.rs`) — HWP5 spec 표 105 에
//!    누락된 baseline 과 version_info 사이 UINT16 zero. hwplib `ForEQEdit.readUInt2()` 정합.
//!    누락 시 정답지 parsing 결과 (version_info, font_name) 자리값 swap → 한컴이 본문 미표시.
//!
//! 3. **HWPX `font` 속성 매핑 보존** (`src/parser/hwpx/section.rs::parse_equation`) —
//!    `font="HYhwpEQ"` → `equation.font_name` (정답지 정합 — Stage 2 EQEDIT 정정 후).
//!
//! 작업지시자 한컴 한글 2020 시각 판정 통과: "output/poc/issue_1061/repro_stage1.hwp 도
//! 본문이 잘 보입니다".

use rhwp::model::control::Control;
use std::fs;
use std::path::Path;

fn load(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

/// HWPX 출처 math-001.hwpx 의 수식이 어댑터 통한 저장 후 attr bit 27 보강 + 정답지 정합.
#[test]
fn issue_1061_equation_attr_bit27_materialized() {
    let mut doc = load("samples/hwpx/math-001.hwpx");
    let hwp_bytes = doc.export_hwp_with_adapter().expect("export");
    let re_doc = rhwp::parser::parse_document(&hwp_bytes).expect("re-parse");

    let mut total = 0;
    let mut bit27_count = 0;
    for section in &re_doc.sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Equation(eq) = ctrl {
                    total += 1;
                    if eq.common.attr & 0x0800_0000 != 0 {
                        bit27_count += 1;
                    }
                }
            }
        }
    }
    assert!(total > 0, "HWPX math-001.hwpx 의 수식 컨트롤 존재해야 함");
    assert_eq!(
        bit27_count, total,
        "모든 Equation 의 attr bit 27 (0x08000000) set 되어야 함 (HWPX_EQUATION_NUMBERING_BIT)"
    );
}

/// HWPX → HWP 라운드트립 시 첫 수식의 attr 가 정답지 (samples/math-001.hwp) 의 0x0C2A2211 정합.
///
/// [#8339] bit 28 만 다르다. 정답지 HWP 는 이 HWPX 를 저장한 것이 아니고, 한글 2024 가 이
/// HWPX(trackchageConfig flags=56)를 HWP 로 저장하면 수식 전부 0x1C2A2211 이 된다.
#[test]
fn issue_1061_first_equation_attr_matches_oracle() {
    let mut doc = load("samples/hwpx/math-001.hwpx");
    let hwp_bytes = doc.export_hwp_with_adapter().expect("export");
    let re_doc = rhwp::parser::parse_document(&hwp_bytes).expect("re-parse");

    let oracle_bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/math-001.hwp"))
        .expect("read oracle");
    let oracle_doc = rhwp::parser::parse_document(&oracle_bytes).expect("parse oracle");

    let first_attr = |doc: &rhwp::model::document::Document| -> Option<u32> {
        for section in &doc.sections {
            for para in &section.paragraphs {
                for ctrl in &para.controls {
                    if let Control::Equation(eq) = ctrl {
                        return Some(eq.common.attr);
                    }
                }
            }
        }
        None
    };

    let saved = first_attr(&re_doc).expect("저장본 첫 수식");
    let oracle = first_attr(&oracle_doc).expect("정답지 첫 수식") | (1 << 28);
    assert_eq!(
        saved, oracle,
        "첫 수식 attr 정답지 정합 필요. saved=0x{:08X} oracle=0x{:08X}",
        saved, oracle
    );
}

/// EQEDIT unknown UINT2 field 정합 — 정답지 parse 결과 font_name="HYhwpEQ", version_info="Equation Version 60"
/// (hwplib 정합 — 누락 시 swap).
#[test]
fn issue_1061_equation_unknown_field_parsed_correctly() {
    let oracle_bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/math-001.hwp"))
        .expect("read oracle");
    let oracle_doc = rhwp::parser::parse_document(&oracle_bytes).expect("parse oracle");

    let first_eq = oracle_doc
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .find_map(|c| {
            if let Control::Equation(eq) = c {
                Some(eq)
            } else {
                None
            }
        })
        .expect("정답지 첫 수식");

    assert_eq!(
        first_eq.font_name, "HYhwpEQ",
        "정답지 첫 수식 font_name 정합 (unknown UINT2 누락 시 swap)"
    );
    assert_eq!(
        first_eq.version_info, "Equation Version 60",
        "정답지 첫 수식 version_info 정합 (unknown UINT2 누락 시 swap)"
    );
}

/// HWPX → HWP 라운드트립 시 첫 수식 font_name + version_info 정답지 정합.
#[test]
fn issue_1061_saved_equation_font_version_matches_oracle() {
    let mut doc = load("samples/hwpx/math-001.hwpx");
    let hwp_bytes = doc.export_hwp_with_adapter().expect("export");
    let re_doc = rhwp::parser::parse_document(&hwp_bytes).expect("re-parse");

    let first_eq = re_doc
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .find_map(|c| {
            if let Control::Equation(eq) = c {
                Some(eq)
            } else {
                None
            }
        })
        .expect("저장본 첫 수식");

    assert_eq!(first_eq.font_name, "HYhwpEQ", "저장본 font_name 정합");
    assert_eq!(
        first_eq.version_info, "Equation Version 60",
        "저장본 version_info 정합"
    );
}

/// 수식 개수 보존 (HWPX 2 + 본문 inline 수식 다수).
#[test]
fn issue_1061_equation_count_preserved() {
    let mut doc = load("samples/hwpx/math-001.hwpx");
    let hwp_bytes = doc.export_hwp_with_adapter().expect("export");
    let re_doc = rhwp::parser::parse_document(&hwp_bytes).expect("re-parse");

    let count_eqs = |doc: &rhwp::model::document::Document| -> usize {
        doc.sections
            .iter()
            .flat_map(|s| &s.paragraphs)
            .flat_map(|p| &p.controls)
            .filter(|c| matches!(c, Control::Equation(_)))
            .count()
    };

    let oracle_bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/math-001.hwp"))
        .expect("read oracle");
    let oracle_doc = rhwp::parser::parse_document(&oracle_bytes).expect("parse oracle");

    let saved = count_eqs(&re_doc);
    let oracle = count_eqs(&oracle_doc);
    assert!(
        saved >= 2,
        "저장본 수식 개수 ≥ 2 (HWPX 출처 최소). saved={}",
        saved
    );
    assert_eq!(
        saved, oracle,
        "저장본 수식 개수 정답지 정합. saved={} oracle={}",
        saved, oracle
    );
}

/// 수식 script 보존 — 첫 수식 의 script 가 정답지 정합.
#[test]
fn issue_1061_equation_script_preserved() {
    let mut doc = load("samples/hwpx/math-001.hwpx");
    let hwp_bytes = doc.export_hwp_with_adapter().expect("export");
    let re_doc = rhwp::parser::parse_document(&hwp_bytes).expect("re-parse");

    let oracle_bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/math-001.hwp"))
        .expect("read oracle");
    let oracle_doc = rhwp::parser::parse_document(&oracle_bytes).expect("parse oracle");

    let first_script = |doc: &rhwp::model::document::Document| -> Option<String> {
        for section in &doc.sections {
            for para in &section.paragraphs {
                for ctrl in &para.controls {
                    if let Control::Equation(eq) = ctrl {
                        return Some(eq.script.clone());
                    }
                }
            }
        }
        None
    };

    let saved = first_script(&re_doc).expect("저장본 첫 수식");
    let oracle = first_script(&oracle_doc).expect("정답지 첫 수식");
    assert_eq!(saved, oracle, "수식 script 정합");
}

/// 본문 paragraph (수식 inline) 의 char_offsets 보존 — 정답지 정합.
#[test]
fn issue_1061_paragraph_char_offsets_with_inline_equation_preserved() {
    let mut doc = load("samples/hwpx/math-001.hwpx");
    let hwp_bytes = doc.export_hwp_with_adapter().expect("export");
    let re_doc = rhwp::parser::parse_document(&hwp_bytes).expect("re-parse");

    let oracle_bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/math-001.hwp"))
        .expect("read oracle");
    let oracle_doc = rhwp::parser::parse_document(&oracle_bytes).expect("parse oracle");

    // 수식 inline 본문 paragraph 찾기 (controls.len() >= 2 + 본문 text 가 있는)
    let find_inline = |doc: &rhwp::model::document::Document| -> Option<Vec<u32>> {
        for section in &doc.sections {
            for para in &section.paragraphs {
                let has_eq = para
                    .controls
                    .iter()
                    .any(|c| matches!(c, Control::Equation(_)));
                if has_eq && !para.text.is_empty() {
                    return Some(para.char_offsets.clone());
                }
            }
        }
        None
    };

    let saved = find_inline(&re_doc).expect("저장본 본문 inline 수식 paragraph");
    let oracle = find_inline(&oracle_doc).expect("정답지 본문 inline 수식 paragraph");
    assert_eq!(
        saved, oracle,
        "본문 inline 수식 paragraph char_offsets 정합"
    );
}
