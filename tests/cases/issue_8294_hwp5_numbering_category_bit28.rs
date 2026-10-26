//! [#8294] HWP5 개체 번호 범주는 CTRL_HEADER 공통속성 bit 26-27 의 2비트다.
//!
//! bit 28 은 범주와 별개로 켜진다. 한글 2024(13.0.0.564) COM `NumberingType` 실측:
//! - `samples/한글문서파일형식_5.0_revision1.3.hwp` 최상위 표 166개 — 원본 bit 26-28 이
//!   6 인 162개는 표(2), 4 인 4개는 없음(0).
//! - `samples/exam_math_no.hwp` — 수식(원본 3·7)은 전부 수식(3), 그림 개체(1·5)는 전부
//!   그림(1), 표(2·6)는 전부 표(2). 범주 없음(0)은 하나도 없다.
//!
//! 3비트로 읽으면 6·5·7 이 범주 없음이 되어, 캡션 번호가 #8191 의 「범주 없는 개체는 1」
//! 규칙으로 모두 1이 된다. 정본 `pdf/한글문서파일형식_5.0_revision1.3-hancom-official.pdf`
//! 19쪽 캡션은 「표 10」·「표 11」·「표 12」다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::{AutoNumberType, Control};
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::ObjectNumberingType;
use rhwp::DocumentCore;

fn load(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{name}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap()
}

fn caption_numbers(caption: &[Paragraph]) -> Vec<u16> {
    caption
        .iter()
        .flat_map(|p| p.controls.iter())
        .filter_map(|c| match c {
            Control::AutoNumber(a) if a.number_type == AutoNumberType::Table => {
                Some(a.assigned_number)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn spec_tables_with_bit28_keep_the_table_category() {
    let core = load("samples/한글문서파일형식_5.0_revision1.3.hwp");
    let (mut table, mut none, mut other) = (0, 0, 0);
    for section in &core.document().sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Table(t) = ctrl {
                    match ((t.common.attr >> 26) & 0x7, t.common.numbering_type) {
                        (6, ObjectNumberingType::Table) => table += 1,
                        (4, ObjectNumberingType::None) => none += 1,
                        (raw, kind) => {
                            other += 1;
                            eprintln!("raw={raw} numbering={kind:?}");
                        }
                    }
                }
            }
        }
    }
    // 한글 2024 COM NumberingType 분포: 표 162 · 없음 4.
    assert_eq!((table, none, other), (162, 4, 0));
}

#[test]
fn spec_page_19_captions_count_on_from_earlier_tables() {
    let core = load("samples/한글문서파일형식_5.0_revision1.3.hwp");
    let section = &core.document().sections[3];
    let numbers: Vec<Vec<u16>> = [121usize, 123, 125]
        .iter()
        .map(|&pi| {
            let table = section.paragraphs[pi]
                .controls
                .iter()
                .find_map(|c| match c {
                    Control::Table(t) => Some(t),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("pi={pi} 에 표가 없다"));
            let caption = table.caption.as_ref().expect("캡션");
            caption_numbers(&caption.paragraphs)
        })
        .collect();
    // 정본 19쪽: 표 10 Schema 이름 정보 · 표 11 Schema 길이 정보 · 표 12 Instance 정보
    assert_eq!(numbers, vec![vec![10], vec![11], vec![12]]);
}

#[test]
fn equations_pictures_and_tables_with_bit28_keep_their_category() {
    let core = load("samples/exam_math_no.hwp");
    let mut seen = std::collections::BTreeMap::<(&str, u32), usize>::new();
    for section in &core.document().sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                let (kind, common, expected) = match ctrl {
                    Control::Equation(e) => ("eqed", &e.common, ObjectNumberingType::Equation),
                    Control::Table(t) => ("tbl", &t.common, ObjectNumberingType::Table),
                    Control::Picture(p) => ("pic", &p.common, ObjectNumberingType::Picture),
                    _ => continue,
                };
                let raw = (common.attr >> 26) & 0x7;
                *seen.entry((kind, raw)).or_default() += 1;
                // 한글 2024 COM: 이 문서의 수식·그림·표에 범주 없음(0)은 없다.
                assert_eq!(common.numbering_type, expected, "{kind} raw={raw}");
            }
        }
    }
    // bit 28 이 켜진 쪽과 꺼진 쪽이 모두 실제로 검사됐는지 확인한다.
    for key in [("eqed", 3), ("eqed", 7), ("tbl", 2), ("tbl", 6)] {
        assert!(
            seen.get(&key).copied().unwrap_or(0) > 0,
            "{key:?} 표본 없음: {seen:?}"
        );
    }
}
