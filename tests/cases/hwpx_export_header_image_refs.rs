//! HWP를 HWPX로 내보내고 다시 읽어도 header.xml의 그림 참조가 같은 그림을 가리키는지 잠근다.
//!
//! 쪽·칸 배경의 그림 채우기와 그림 글머리표는 header.xml에서 `binaryItemIDRef`로 그림을 가리킨다.
//! 수정 전에는 두 가지로 그림을 잃었다.
//!
//! - 내보낸 매니페스트 이름은 `image{저장 ID}`라 저장 ID에 구멍이 있으면 위치와 어긋난다
//!   (148776468: `image2`…`image12`). 본문 참조는 읽을 때 위치 번호로 고치지만 header.xml은
//!   이름의 숫자를 그대로 써서, 칸 배경 그림 둘 중 하나는 사라지고 하나는 본문 그림으로 바뀌었다.
//! - HWP 그림 글머리표는 여부(1)와 그림 ID(이미지 글머리 데이터의 넷째 바이트)를 따로 둔다.
//!   여부를 ID로 읽어 본문 그림을 글머리표로 그렸고, HWPX에는 `<hh:img>` 없이 `useImage="1"`만
//!   써서 다시 읽으면 1번 그림이 됐다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use rhwp::model::bin_data::BinDataContent;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;

fn parse(bytes: &[u8]) -> Document {
    rhwp::parser::parse_document(bytes).expect("문서 파싱")
}

/// 렌더러와 같은 규칙으로 그림 바이트를 찾는다: 레코드 순번이 먼저, 범위 밖이면 저장 ID.
fn image_bytes(contents: &[BinDataContent], id: u16) -> Option<Vec<u8>> {
    let index = usize::from(id.checked_sub(1)?);
    let content = contents
        .get(index)
        .or_else(|| contents.iter().find(|content| content.id == id))?;
    Some(content.data.load().to_vec())
}

fn image_hash(contents: &[BinDataContent], id: u16) -> Option<u64> {
    let bytes = image_bytes(contents, id)?;
    if bytes.is_empty() {
        return None;
    }
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    Some(hasher.finish())
}

fn collect_pictures(doc: &Document, paragraphs: &[Paragraph], out: &mut Vec<String>) {
    for control in paragraphs.iter().flat_map(|p| &p.controls) {
        match control {
            Control::Picture(picture) => {
                let id = picture.image_attr.bin_data_id;
                out.push(format!("그림 {:?}", image_hash(&doc.bin_data_content, id)));
            }
            Control::Table(table) => {
                for cell in &table.cells {
                    collect_pictures(doc, &cell.paragraphs, out);
                }
            }
            Control::Header(header) => collect_pictures(doc, &header.paragraphs, out),
            Control::Footer(footer) => collect_pictures(doc, &footer.paragraphs, out),
            _ => {}
        }
    }
}

/// 그림 참조마다 `(종류와 번호, 가리키는 바이트)`를 모은다. 그릴 바이트가 없으면 `None`이다.
fn image_refs(doc: &Document) -> Vec<String> {
    let mut out = Vec::new();
    for (index, border_fill) in doc.doc_info.border_fills.iter().enumerate() {
        if let Some(image) = &border_fill.fill.image {
            out.push(format!(
                "테두리·배경 {} {:?}",
                index + 1,
                image_hash(&doc.bin_data_content, image.bin_data_id)
            ));
        }
    }
    for (index, bullet) in doc.doc_info.bullets.iter().enumerate() {
        if let Ok(id @ 1..) = u16::try_from(bullet.image_bullet) {
            out.push(format!(
                "글머리표 {} {:?}",
                index + 1,
                image_hash(&doc.bin_data_content, id)
            ));
        }
    }
    for section in &doc.sections {
        collect_pictures(doc, &section.paragraphs, &mut out);
    }
    out
}

fn assert_hwpx_round_trip_keeps_images(rel: &str, expected_refs: usize) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let source = parse(&std::fs::read(&path).expect("표본 읽기"));
    let before = image_refs(&source);
    assert_eq!(
        before.len(),
        expected_refs,
        "{rel} 원본 그림 참조: {before:#?}"
    );
    assert!(
        before.iter().all(|line| !line.ends_with("None")),
        "{rel} 원본은 모든 그림을 그릴 수 있다: {before:#?}"
    );

    let hwpx = rhwp::serializer::hwpx::serialize_hwpx(&source).expect("HWPX 내보내기");
    let after = image_refs(&parse(&hwpx));
    assert_eq!(after, before, "{rel} HWPX로 내보내고 다시 읽은 그림 참조");
}

/// 저장 ID 2~12(1·6 없음). 칸 배경 그림 둘(순번 1 → 저장 ID 11, 순번 10 → 저장 ID 10)과 본문 그림 여덟.
#[test]
fn hwpx_export_keeps_cell_background_images_with_sparse_storage_ids() {
    assert_hwpx_round_trip_keeps_images(
        "samples/issue6929/148776468_search_ad_terms_press_release.hwp",
        10,
    );
}

/// 그림 글머리표 하나(그림 ID 2 → 저장 ID 3, 60×60 png)와 본문 그림 하나(gif).
#[test]
fn hwpx_export_keeps_hwp_image_bullet() {
    assert_hwpx_round_trip_keeps_images("samples/basic/NewYear_s_Day.hwp", 2);
}

/// 첫 그림 글머리표가 가리키는 그림의 바이트 길이.
fn bullet_image_len(doc: &Document) -> Option<usize> {
    let bullet = doc.doc_info.bullets.iter().find(|b| b.image_bullet != 0)?;
    let id = u16::try_from(bullet.image_bullet).ok()?;
    Some(image_bytes(&doc.bin_data_content, id)?.len())
}

/// HWP 그림 글머리표는 여부가 1이고 그림 ID는 이미지 글머리 데이터의 넷째 바이트다.
/// 두 표본 모두 글머리표 그림은 다른 곳에서 쓰지 않는 60×60 png이고, 여부 1이 가리키는
/// 1번 그림은 본문 그림(NewYear 16726바이트 gif, hwpspec 1688598바이트 bmp)이다.
/// HWP → HWPX → HWP를 거쳐도 같은 그림을 가리켜야 한다.
#[test]
fn hwp_image_bullet_uses_image_id_across_hwpx_and_hwp() {
    for (rel, bullet_len) in [
        ("samples/basic/NewYear_s_Day.hwp", 4557),
        ("samples/hwpspec.hwp", 1124),
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
        let source = parse(&std::fs::read(&path).expect("표본 읽기"));
        assert_eq!(bullet_image_len(&source), Some(bullet_len), "{rel} 원본");

        let hwpx = rhwp::serializer::hwpx::serialize_hwpx(&source).expect("HWPX 내보내기");
        let from_hwpx = parse(&hwpx);
        assert_eq!(bullet_image_len(&from_hwpx), Some(bullet_len), "{rel} HWPX");

        let hwp = rhwp::document_core::DocumentCore::from_bytes(&hwpx)
            .expect("HWPX 로드")
            .export_hwp_with_adapter()
            .expect("HWP 저장");
        assert_eq!(
            bullet_image_len(&parse(&hwp)),
            Some(bullet_len),
            "{rel} HWPX → HWP"
        );
    }
}
