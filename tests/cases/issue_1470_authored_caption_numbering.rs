//! 공개 삽입/캡션 API의 번호 소유와 HWP/HWPX 저장 재열기.
//! 독립 Hancom2020 Print / Native+fresh WASM 전쪽 및 px 선정 근거: stage24 보고서.
//! 빈 문서 원형에서 공개 API로 편집하며 parser의 저장 번호 캐시를 덮지 않는다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{model::control::Control, wasm_api::HwpDocument, DocumentCore};

fn fresh() -> HwpDocument {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/hwpx/hancom-hwp/blank_hwpx.hwp");
    HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}
fn owners(doc: &HwpDocument, picture: bool) -> Vec<usize> {
    doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .filter_map(|(i, c)| match c {
            Control::Picture(_) if picture => Some(i),
            Control::Table(_) if !picture => Some(i),
            _ => None,
        })
        .collect()
}
fn numbers(doc: &HwpDocument, picture: bool) -> Vec<(u16, u16)> {
    owners(doc, picture)
        .into_iter()
        .flat_map(|i| {
            let caption = match &doc.document().sections[0].paragraphs[0].controls[i] {
                Control::Picture(p) => &p.caption,
                Control::Table(t) => &t.caption,
                _ => unreachable!(),
            };
            caption
                .iter()
                .flat_map(|c| &c.paragraphs)
                .flat_map(|p| &p.controls)
                .filter_map(|c| {
                    if let Control::AutoNumber(a) = c {
                        Some((a.assigned_number, a.number))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
fn painted(core: &DocumentCore) -> Vec<u16> {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn walk(n: &RenderNode, caption: bool, out: &mut Vec<u16>) {
        let caption = if let RenderNodeType::TextLine(l) = &n.node_type {
            l.caption_owner.is_some()
        } else {
            caption
        };
        if caption {
            if let RenderNodeType::TextRun(r) = &n.node_type {
                if let Some(value) = r.display_text.as_deref().and_then(|s| s.parse().ok()) {
                    out.push(value)
                }
            }
        }
        for child in &n.children {
            walk(child, caption, out)
        }
    }
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        walk(
            &core.build_page_render_tree(page).unwrap().root,
            false,
            &mut out,
        )
    }
    out
}
fn assert_table_caption_flow(core: &DocumentCore) {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn walk(
        n: &RenderNode,
        tables: &mut Vec<(usize, f64, f64)>,
        captions: &mut Vec<(usize, f64, f64)>,
    ) {
        match &n.node_type {
            RenderNodeType::Table(t) => {
                if let Some(ci) = t.control_index {
                    tables.push((ci, n.bbox.y, n.bbox.y + n.bbox.height))
                }
            }
            RenderNodeType::TextLine(l) => {
                if let Some(o) = &l.caption_owner {
                    captions.push((o.control_idx, n.bbox.y, n.bbox.y + n.bbox.height))
                }
            }
            _ => {}
        }
        for c in &n.children {
            walk(c, tables, captions)
        }
    }
    let mut tables = Vec::new();
    let mut captions = Vec::new();
    for page in 0..core.page_count() {
        walk(
            &core.build_page_render_tree(page).unwrap().root,
            &mut tables,
            &mut captions,
        )
    }
    assert_eq!(tables.len(), 3);
    // 전쪽/완료 페이지 qualification 후 보호하는 독립 Print의 둘째 표 윗선.
    // 96dpi y=193.228 CSSpx±2에서 실제 Native PNG의 긴 잉크를 검사한다.
    #[cfg(feature = "native-skia")]
    {
        let bytes = core
            .render_page_png_native_with_profile_and_export_options(
                0,
                rhwp::paint::RenderProfile::Print,
                &rhwp::document_core::queries::rendering::PngExportOptions {
                    dpi: Some(96.),
                    ..Default::default()
                },
            )
            .unwrap();
        let png = image::load_from_memory(&bytes).unwrap().to_rgb8();
        let y = 193.228_f64;
        let columns = 118..676;
        let ink = columns
            .clone()
            .filter(|&x| {
                ((y - 2.).ceil() as u32..=(y + 2.).floor() as u32).any(|row| {
                    let p = png.get_pixel(x, row);
                    p[0].max(p[1]).max(p[2]) <= 220
                })
            })
            .count();
        assert!(
            ink * 10 >= columns.len() * 9,
            "독립 Print 둘째 표 윗선±2CSSpx의 잉크가 부족하다: {ink}/{}",
            columns.len()
        );
    }
    for (i, &(ci, top, bottom)) in tables.iter().enumerate() {
        for &(owner, cap_top, cap_bottom) in &captions {
            if owner != ci {
                continue;
            }
            assert!(cap_top>=bottom,"아래 캡션이 자기 표 본체 아래에 속한다: table={top}..{bottom}, caption={cap_top}..{cap_bottom}");
            if let Some(next) = tables.get(i + 1) {
                assert!(cap_bottom<=next.1,
                "이전 표의 캡션 상자에 다음 표가 진입하지 않는다: caption end={cap_bottom}, next start={}",next.1)
            }
        }
    }
}
fn assert_roundtrip(doc: &HwpDocument, picture: bool, expected: &[u16]) {
    assert_eq!(
        numbers(doc, picture)
            .iter()
            .map(|n| n.0)
            .collect::<Vec<_>>(),
        expected
    );
    if !picture {
        assert_table_caption_flow(doc)
    }
    assert_eq!(
        painted(doc),
        expected,
        "실제 캡션 paint의 내용 소유·순서·누락/중복"
    );
    for bytes in [
        doc.export_hwp_native().unwrap(),
        doc.export_hwpx_native().unwrap(),
    ] {
        let reopened = HwpDocument::from_bytes(&bytes).unwrap();
        assert_eq!(
            owners(&reopened, picture).len(),
            3,
            "캡션 없는 원문 개체도 보존한다"
        );
        assert_eq!(
            numbers(&reopened, picture)
                .iter()
                .map(|n| n.0)
                .collect::<Vec<_>>(),
            expected,
            "저장·재열기 뒤에도 같은 원문 개체가 같은 번호를 소유한다"
        );
        if !picture {
            assert_table_caption_flow(&reopened)
        }
        assert_eq!(
            painted(&reopened),
            expected,
            "저장된 원시 번호 범주를 실제 paint까지 전달한다"
        );
    }
}
fn exercise(picture: bool) {
    let mut doc = fresh();
    if picture {
        // 독립 Print 생산 입력과 같은 1px 투명 원본. 번호 검사는 그림 픽셀에 의존하지 않는다.
        let image: Vec<u8> = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0x0D, 0x49, 0x48, 0x44, 0x52,
            0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 0x1F, 0x15, 0xC4, 0x89, 0, 0, 0, 0x0A, 0x49,
            0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0, 1, 0, 0, 0, 5, 0, 1, 0x0D, 0x0A, 0x2D, 0xB4, 0,
            0, 0, 0, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        for i in 0..3 {
            doc.insert_picture_native(
                0,
                0,
                0,
                &[],
                &image,
                5000,
                5000,
                1,
                1,
                "png",
                &format!("picture{i}"),
                Some(3000 + i * 7000),
                Some(3000),
            )
            .unwrap();
        }
    } else {
        for _ in 0..3 {
            doc.create_table_ex_native(0, 0, 0, 1, 1, true, None, None)
                .unwrap();
        }
    }
    let owned = owners(&doc, picture);
    assert_eq!(owned.len(), 3);
    for &i in &owned {
        if picture {
            doc.set_picture_properties_native(0, 0, i, r#"{"hasCaption":true}"#)
                .unwrap();
        } else {
            doc.set_table_properties_native(0, 0, i, r#"{"hasCaption":true}"#)
                .unwrap();
        }
    }
    assert_roundtrip(&doc, picture, &[1, 2, 3]);
    let cached = numbers(&doc, picture)
        .iter()
        .map(|n| n.1)
        .collect::<Vec<_>>();
    if picture {
        doc.set_picture_properties_native(0, 0, owned[1], r#"{"hasCaption":false}"#)
            .unwrap();
    } else {
        doc.set_table_properties_native(0, 0, owned[1], r#"{"hasCaption":false}"#)
            .unwrap();
    }
    assert_eq!(
        numbers(&doc, picture)
            .iter()
            .map(|n| n.1)
            .collect::<Vec<_>>(),
        [cached[0], cached[2]],
        "표시 번호를 재계산하더라도 원본 캐시를 인위적으로 덮지 않는다"
    );
    assert_roundtrip(&doc, picture, &[1, 3]);
}
#[test]
fn authored_table_caption_numbers_survive_caption_removal_and_hwp_hwpx_save() {
    exercise(false)
}
#[test]
fn authored_picture_caption_numbers_survive_caption_removal_and_hwp_hwpx_save() {
    exercise(true)
}

#[test]
fn side_caption_keeps_its_source_owned_row_in_live_and_saved_paint() {
    let mut doc = fresh();
    for _ in 0..3 {
        doc.create_table_ex_native(0, 0, 0, 1, 1, true, None, None)
            .unwrap();
    }
    let ids = owners(&doc, false);
    for &ci in &ids {
        doc.set_table_properties_native(0, 0, ci, r#"{"hasCaption":true}"#)
            .unwrap();
    }
    doc.set_table_properties_native(0, 0, ids[1], r#"{"hasCaption":false}"#)
        .unwrap();
    doc.set_table_properties_native(
        0,
        0,
        ids[2],
        r#"{"captionDirection":0,"captionVertAlign":1,"captionWidth":2400,"captionSpacing":600}"#,
    )
    .unwrap();
    // Independent Hancom2020 Print p1: third-table upper edge y=671.625pt,
    // MediaBox top841pt ->225.833333CSSpx at96dpi. Do not derive from engine layout.
    fn check(doc: &HwpDocument, kind: &str) -> serde_json::Value {
        assert_eq!(
            painted(doc),
            [1, 3],
            "source owners remain after side-caption editing"
        );
        #[cfg(feature = "native-skia")]
        {
            let bytes = doc
                .render_page_png_native_with_profile_and_export_options(
                    0,
                    rhwp::paint::RenderProfile::Print,
                    &rhwp::document_core::queries::rendering::PngExportOptions {
                        dpi: Some(96.),
                        ..Default::default()
                    },
                )
                .unwrap();
            let png = image::load_from_memory(&bytes).unwrap().to_rgb8();
            let y = 225.833333_f64;
            let columns = 159..714;
            let ink = columns
                .clone()
                .filter(|&x| {
                    ((y - 2.).ceil() as u32..=(y + 2.).floor() as u32).any(|row| {
                        let p = png.get_pixel(x, row);
                        p[0].max(p[1]).max(p[2]) <= 220
                    })
                })
                .count();
            assert!(
                ink * 10 >= columns.len() * 9,
                "{kind}: independent Print side-caption table top ±2CSSpx ink={ink}/{}",
                columns.len()
            );
        }
        let layout: serde_json::Value =
            serde_json::from_str(&doc.get_page_control_layout_native(0).unwrap()).unwrap();
        assert_eq!(
            layout["controls"].as_array().unwrap().len(),
            3,
            "{kind}: each original table remains owned once"
        );
        layout
    }
    let live = check(&doc, "live");
    for (kind, bytes) in [
        ("HWP", doc.export_hwp_native().unwrap()),
        ("HWPX", doc.export_hwpx_native().unwrap()),
    ] {
        assert_eq!(
            check(&HwpDocument::from_bytes(&bytes).unwrap(), kind),
            live,
            "same source-owned rows retain positions after save/reopen"
        );
    }
}
