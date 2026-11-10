//! 함수별 유형을 아래에 구분한다. 원본 전진·소속 관계와 완료 페이지의 독립 원점을 보호한다.
//! 주석 갱신: 2026-11-10T07:31:01+09:00.
//! NewYear_s_Day.hwp 전3쪽, 독립 한컴2020 Print PDF, 96dpi/PDFium/raw2px.
//! 수정 전 최소 글꼴 Native 최저98.42814% / fresh WASM98.42806%.
//! 원본 useFontSpace=1의 한양해서 공백은341/1024em이며, 반각 공간이 아니다.
//! 2쪽 캡션의 마름모는 문자 치환이 아닌 원본 BinData 그림 글머리표다.
//! 아래 관계형은 절대 좌표를 쓰지 않는다. 고정형은 양 backend 98% 이상·직접 판독한 1·2쪽만 ±2CSSpx로 보호한다.
//! stage11 최소 글꼴 전3쪽 Native 최저98.42814% / fresh WASM98.42806%, 2쪽98.91391%/98.91391%.
//! 원본 글꼴 공급의 별도 조건과 섞지 않는다. 모든 검사는 설치 글꼴 없이 실행한다.

use rhwp::{
    model::{control::Control, style::HeadType},
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
use serde_json::Value;

fn source_core() -> DocumentCore {
    DocumentCore::from_bytes(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("samples/basic/NewYear_s_Day.hwp"),
        )
        .unwrap(),
    )
    .unwrap()
}

/// 검사 유형: 관계형. 원본의 공백 사용 속성과 프로그램 전진의 비율을 검사한다.
#[test]
fn declared_original_font_space_is_used_only_when_the_source_requests_it() {
    for use_font_space in [true, false] {
        let mut core = source_core();
        let mut document = core.document().clone();
        assert!(document.sections[0].paragraphs[0]
            .text
            .contains("설 차례 상"));
        let style_id = document.sections[0].paragraphs[0].char_shapes[0].char_shape_id as usize;
        let shape = &mut document.doc_info.char_shapes[style_id];
        assert_eq!(shape.base_size, 3500);
        shape.use_font_space = use_font_space;
        core.set_document(document);
        let trace: Value = serde_json::from_str(
            &core
                .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
                .unwrap(),
        )
        .unwrap();
        let spaces: Vec<_> = trace["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["source"]["paragraphIndex"] == 0 && r["source"]["character"] == " ")
            .collect();
        assert_eq!(
            spaces.len(),
            2,
            "원본 제목의 공백 두 개를 빠짐없이 검사한다"
        );
        for r in spaces {
            let expected = 3500.0 * if use_font_space { 341.0 / 1024.0 } else { 0.5 };
            let observed = r["layoutMetric"]["baseAdvanceHwpunit"].as_f64().unwrap();
            assert!(
                (observed - expected).abs() <= 1.0,
                "useFontSpace={use_font_space}, 원본 프로그램 공백과 일반 반각을 구분한다: {r}"
            );
        }
    }
}

fn collect_caption_lines<'a>(node: &'a RenderNode, owner: usize, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextLine(line)
        if line.caption_owner.is_some_and(|c| c.para_idx == owner))
    {
        out.push(node);
    }
    for child in &node.children {
        collect_caption_lines(child, owner, out);
    }
}

/// 검사 유형: 관계형. 캡션의 그림 머리·본문 순서와 원본 자원 소속을 검사한다.
#[test]
fn caption_picture_bullet_and_text_share_the_source_marker_geometry() {
    for with_head in [true, false] {
        let mut core = source_core();
        let mut document = core.document().clone();
        let (owner, caption_para) = document.sections[0]
            .paragraphs
            .iter()
            .enumerate()
            .find_map(|(pi, p)| {
                p.controls.iter().find_map(|control| match control {
                    Control::Picture(picture) => picture.caption.as_ref().and_then(|caption| {
                        caption
                            .paragraphs
                            .iter()
                            .find(|p| p.text.contains("추석에는 떡국"))
                            .map(|p| (pi, p.clone()))
                    }),
                    _ => None,
                })
            })
            .expect("그림 원본의 캡션 소속");
        let style = &mut document.doc_info.para_shapes[caption_para.para_shape_id as usize];
        assert_eq!(style.head_type, HeadType::Bullet);
        let bullet = &document.doc_info.bullets[usize::from(style.numbering_id) - 1];
        let image_id = bullet.image_reference().expect("원본 그림 글머리표 참조");
        let image_bytes = document.bin_data_content[usize::from(image_id) - 1]
            .data
            .load()
            .to_vec();
        if !with_head {
            style.head_type = HeadType::None;
        }
        core.set_document(document);
        let tree = core.build_page_render_tree(1).unwrap();
        let mut lines = Vec::new();
        collect_caption_lines(&tree.root, owner, &mut lines);
        assert_eq!(lines.len(), 1, "그림 캡션은 소속과 줄 개수를 보존한다");
        let line = lines[0];
        let images: Vec<_> = line
            .children
            .iter()
            .filter(|n| {
                matches!(&n.node_type,
            RenderNodeType::Image(image) if image.bin_data_id == image_id)
            })
            .collect();
        assert_eq!(
            images.len(),
            usize::from(with_head),
            "캡션 머리 속성이 그림 출력 여부를 소유한다"
        );
        let body: Vec<_> = line
            .children
            .iter()
            .filter_map(|n| match &n.node_type {
                RenderNodeType::TextRun(run) if run.char_start.is_some() => Some((n, run)),
                _ => None,
            })
            .collect();
        assert_eq!(
            body.iter()
                .map(|(_, r)| r.text.as_str())
                .collect::<String>(),
            caption_para.text,
            "머리 추가는 본문에 문자 마커를 삽입하거나 본문을 중복하지 않는다"
        );
        if let Some(marker) = images.first() {
            let RenderNodeType::Image(image) = &marker.node_type else {
                unreachable!()
            };
            assert_eq!(
                image.data.as_deref(),
                Some(image_bytes.as_slice()),
                "같은 BinData 그림을 그린다"
            );
            assert!(
                marker.bbox.x + marker.bbox.width <= body[0].0.bbox.x,
                "그림 머리의 점유 칸 뒤에 같은 줄 본문을 배치한다"
            );
            assert!(
                (marker.bbox.width - body[0].1.style.font_size).abs() < 0.001,
                "마커는 캡션의 원본 글자 크기로 배치한다"
            );
        }
    }
}

/// 검사 유형: 고정형. 독립 Print의 제목 글자 원점과 캡션 마커·본문 원점을 ±2CSSpx로 보호한다.
#[test]
fn completed_title_and_caption_keep_the_independent_print_origins() {
    fn check(observed: f64, expected: f64) {
        assert!(
            (observed - expected).abs() <= 2.0,
            "완료 페이지의 독립 Print 원점: 실제={observed}, 기대={expected}, 허용=±2CSSpx"
        );
    }
    let core = source_core();
    let svg = core.render_page_svg_native(0).unwrap();
    for (character, x) in [('차', 169.44), ('례', 211.36), ('상', 267.36)] {
        let elements: Vec<_> = svg
            .split("<text ")
            .skip(1)
            .filter(|fragment| {
                let element = fragment.split_once("</text>").unwrap().0;
                element.ends_with(&format!(">{character}"))
                    && element
                        .split_once("font-size=\"")
                        .and_then(|(_, v)| v.split('"').next())
                        .and_then(|v| v.parse::<f64>().ok())
                        .is_some_and(|size| size > 40.0)
            })
            .collect();
        assert_eq!(elements.len(), 1, "제목 글자 원점의 중복·누락: {character}");
        let translate = elements[0]
            .split_once("translate(")
            .unwrap()
            .1
            .split_once(')')
            .unwrap()
            .0;
        let mut coordinates = translate.split(',').map(|s| s.parse::<f64>().unwrap());
        check(coordinates.next().unwrap(), x);
        check(coordinates.next().unwrap(), 153.12);
    }
    let tree = core.build_page_render_tree(1).unwrap();
    let mut lines = Vec::new();
    collect_caption_lines(&tree.root, 24, &mut lines);
    assert_eq!(lines.len(), 1);
    let marker = lines[0]
        .children
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Image(_)))
        .unwrap();
    let text = lines[0]
        .children
        .iter()
        .find(|n| {
            matches!(&n.node_type,
        RenderNodeType::TextRun(run) if run.text == "추석에는 떡국 대신 송편을 놓습니다.")
        })
        .unwrap();
    check(marker.bbox.x, 153.35733);
    check(marker.bbox.y, 819.424);
    check(text.bbox.x, 171.52);
}
