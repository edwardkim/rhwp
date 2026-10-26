//! [#8294] HWP5 개체 번호 범주는 CTRL_HEADER 공통속성 bit 26-27 의 2비트다.
//!
//! bit 28 은 범주와 별개로 켜진다. 3비트로 읽으면 원본 6·5·7(표·그림·수식 + bit 28)이
//! 범주 없음이 되어, 캡션 번호가 #8191 의 「범주 없는 개체 캡션은 1」 규칙으로 1이 된다.
//!
//! 독립 기대값:
//! - 한글 2024(13.0.0.564) COM `NumberingType`: 이 문서의 그림 개체는 모두 그림(1), 표는
//!   모두 표(2)다. 같은 측정에서 한글문서파일형식 표 166 은 원본 6 → 표 162 · 4 → 없음 4,
//!   exam_math_no 는 수식 3·7 → 수식, 그림 1·5 → 그림, 표 2·6 → 표였다.
//! - 정본 `pdf/hwp3-sample-hwp5-hwp-2020.pdf`·`-2022.pdf` 의 그림 캡션은 2·3·4·5 다.
//!
//! 시각 근거: 전 16쪽 Native·fresh WASM raw 실루엣 최저 90% 이상, 쪽수 16 = 16.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::{AutoNumberType, Control};
use rhwp::model::shape::ObjectNumberingType;
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/hwp3-sample-hwp5.hwp";

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("read sample")).unwrap()
}

#[test]
fn pictures_and_tables_with_bit28_keep_their_category() {
    let core = load();
    let mut seen = std::collections::BTreeMap::<(&str, u32), usize>::new();
    for section in &core.document().sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                let (kind, common, expected) = match ctrl {
                    Control::Table(t) => ("tbl", &t.common, ObjectNumberingType::Table),
                    Control::Picture(p) => ("pic", &p.common, ObjectNumberingType::Picture),
                    _ => continue,
                };
                let raw = (common.attr >> 26) & 0x7;
                *seen.entry((kind, raw)).or_default() += 1;
                assert_eq!(common.numbering_type, expected, "{kind} raw={raw}");
            }
        }
    }
    // bit 28 이 켜진 그림·표와 꺼진 그림이 모두 실제로 검사됐는지 확인한다.
    for key in [("pic", 1), ("pic", 5), ("tbl", 6)] {
        assert!(
            seen.get(&key).copied().unwrap_or(0) > 0,
            "{key:?} 표본 없음: {seen:?}"
        );
    }
}

#[test]
fn figure_captions_count_on_through_bit28_pictures() {
    let core = load();
    let mut numbers = Vec::new();
    for section in &core.document().sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                let Control::Picture(p) = ctrl else { continue };
                let Some(caption) = &p.caption else { continue };
                for c in caption.paragraphs.iter().flat_map(|p| p.controls.iter()) {
                    if let Control::AutoNumber(a) = c {
                        if a.number_type == AutoNumberType::Picture {
                            numbers.push(a.assigned_number);
                        }
                    }
                }
            }
        }
    }
    // 정본 PDF: 그림 2 · 그림 3 · 그림 4(NAT) · 그림 5(IP 터널링)
    assert_eq!(numbers, vec![2, 3, 4, 5]);
}
