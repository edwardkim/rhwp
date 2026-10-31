//! [#8352] HWP5 문단 모양의 줄간격은 구 필드(attr1 bit 0-1 종류 + offset 24 `line_spacing`)와
//! 5.0.2.5 필드(attr3 bit 0-4 종류 + offset 50 `line_spacing_v2`) 두 곳에 있다. 한/글은 새 필드를
//! 읽는다. 한/글 저장본은 비율·고정·여백만은 두 필드가 같고, «최소»만 구 필드가 비율 100
//! 자리표시이며 실제 값은 새 필드에 있다(10k 코퍼스 HWP 954개·문단 모양 52,409개 예외 0).
//!
//! rhwp 가 줄간격을 바꾼 문단 모양을 저장할 때 새 필드를 원본 값 그대로 쓰면 한/글이 옛
//! 줄간격으로 다시 짠다 — PDF 편집 우선 가져오기 저장본이 한/글 Print 에서 줄 피치 24.0px,
//! rhwp 에서 25.6px 로 갈려 전 쪽이 밀렸다.

use rhwp::model::style::{LineSpacingType, ParaShape};
use rhwp::serializer::doc_info::serialize_para_shape;
use rhwp::DocumentCore;

const OLD_TYPE: usize = 0;
const OLD_VALUE: usize = 24;
const NEW_TYPE: usize = 46;
const NEW_VALUE: usize = 50;

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn i32_at(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

/// (구 종류, 구 값, 새 종류, 새 값)
fn fields(bytes: &[u8]) -> (u32, i32, u32, u32) {
    (
        u32_at(bytes, OLD_TYPE) & 0x03,
        i32_at(bytes, OLD_VALUE),
        u32_at(bytes, NEW_TYPE) & 0x1f,
        u32_at(bytes, NEW_VALUE),
    )
}

/// 한/글 저장본의 규약: «최소»(새 종류 3)는 구 필드 비율 100, 그 밖은 두 필드가 같다.
fn follows_hancom_convention(bytes: &[u8]) -> bool {
    let (old_type, old_value, new_type, new_value) = fields(bytes);
    if new_type == 3 {
        old_type == 0 && old_value == 100
    } else {
        old_type == new_type && old_value.max(0) as u32 == new_value
    }
}

fn open(path: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    DocumentCore::from_bytes(&std::fs::read(&path).unwrap()).unwrap()
}

#[test]
fn changed_line_spacing_rewrites_both_hwp5_fields() {
    // 비율 160% 문단 모양을 복제해 구 필드만 바꾼 모양 — PDF 가져오기·편집 명령이 만드는 상태.
    let stale = ParaShape {
        attr3: 0,
        line_spacing_v2: 160,
        ..ParaShape::default()
    };
    for (kind, value, code) in [
        (LineSpacingType::Fixed, 3838, 1),
        (LineSpacingType::SpaceOnly, 600, 2),
        (LineSpacingType::Percent, 130, 0),
    ] {
        let shape = ParaShape {
            line_spacing_type: kind,
            line_spacing: value,
            ..stale.clone()
        };
        let bytes = serialize_para_shape(&shape);
        assert_eq!(
            fields(&bytes),
            (code, value, code, value as u32),
            "{kind:?} {value}: 구·새 필드가 같은 줄간격이어야 한다"
        );
    }
    // «최소»는 한/글처럼 구 필드에 비율 100 을 두고 값은 새 필드에만 쓴다.
    let minimum = ParaShape {
        line_spacing_type: LineSpacingType::Minimum,
        line_spacing: 4600,
        ..stale
    };
    assert_eq!(fields(&serialize_para_shape(&minimum)), (0, 100, 3, 4600));
}

#[test]
fn hancom_minimum_line_spacing_is_read_from_the_new_field() {
    // 한/글 저장본: 구 필드 비율 100 + 새 필드 «최소».
    let core = open("samples/k-water-rfp.hwp");
    let shapes = &core.document().doc_info.para_shapes;
    let minimum: Vec<_> = shapes
        .iter()
        .filter(|ps| {
            ps.raw_data
                .as_deref()
                .is_some_and(|raw| raw.len() >= 54 && u32_at(raw, NEW_TYPE) & 0x1f == 3)
        })
        .collect();
    assert!(
        !minimum.is_empty(),
        "k-water-rfp 에 «최소» 줄간격 문단 모양이 있다"
    );
    for ps in minimum {
        let raw = ps.raw_data.as_deref().unwrap();
        assert_eq!(ps.line_spacing_type, LineSpacingType::Minimum);
        assert_eq!(ps.line_spacing, i32_at(raw, NEW_VALUE));
        // 다시 직렬화해도 한/글의 네 필드와 같다.
        let rewritten = serialize_para_shape(&ParaShape {
            raw_data: None,
            ..ps.clone()
        });
        assert_eq!(fields(&rewritten), fields(raw));
    }
}

#[cfg(feature = "pdf-import")]
#[test]
fn pdf_editable_import_saves_hancom_line_spacing_fields() {
    use rhwp::parser::pdf::{PdfImportLayout, PdfImportOptions};

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/21_언어_기출_편집가능본-2010.pdf");
    let options = PdfImportOptions {
        layout: PdfImportLayout::Editable,
        ..PdfImportOptions::default()
    };
    let imported = DocumentCore::from_pdf_bytes(&std::fs::read(path).unwrap(), &options).unwrap();
    let saved = imported.export_hwp_native().unwrap();
    let reopened = DocumentCore::from_bytes(&saved).unwrap();
    let shapes = &reopened.document().doc_info.para_shapes;
    let mut fixed = 0;
    for (index, ps) in shapes.iter().enumerate() {
        let raw = ps
            .raw_data
            .as_deref()
            .expect("저장본 문단 모양 원본 바이트");
        assert!(
            follows_hancom_convention(raw),
            "문단 모양 {index}: 구·새 줄간격 필드 {:?}",
            fields(raw)
        );
        if ps.line_spacing_type == LineSpacingType::Fixed {
            fixed += 1;
        }
    }
    // 가져온 본문 문단은 고정 줄간격이다 — 검사 대상이 비지 않았음을 확인한다.
    assert!(fixed > 100, "고정 줄간격 문단 모양 {fixed}개");
}
