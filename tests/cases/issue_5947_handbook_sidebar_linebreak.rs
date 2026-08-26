//! [Issue #5947] 행정업무운영편람 137쪽 바탕쪽 세로 제목 "업무관리시스템" 줄나눔.
//!
//! 마스터페이지3 그룹 자식 글상자(groupLevel=1, sy≈0.43, lastWidth=2374 HU)는
//! 저장 lineSeg 7줄(각 textpos 1글자, horzsize=1440 HU)을 가진다. Y축만 축소된
//! 그룹 자식이라 해서 가용 폭으로 재래핑하면 2글자 줄이 되어 제목이 깨진다.
//! 저장 분할을 유지해야 한 글자씩 세로로 쌓인다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::ShapeObject;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::renderer::composer::compose_paragraph;

const SAMPLE: &str = "samples/2025 행정업무운영 편람(최종).hwpx";

fn walk_shapes<'a>(shape: &'a ShapeObject, out: &mut Vec<&'a ShapeObject>) {
    out.push(shape);
    if let ShapeObject::Group(group) = shape {
        for child in &group.children {
            walk_shapes(child, out);
        }
    }
}

fn handbook_sidebar_para(doc: &Document) -> &Paragraph {
    for section in &doc.sections {
        for master in &section.section_def.master_pages {
            for para in &master.paragraphs {
                for control in &para.controls {
                    let Control::Shape(shape) = control else {
                        continue;
                    };
                    let mut shapes = Vec::new();
                    walk_shapes(shape, &mut shapes);
                    for shape in shapes {
                        let Some(drawing) = shape.drawing() else {
                            continue;
                        };
                        let Some(text_box) = drawing.text_box.as_ref() else {
                            continue;
                        };
                        if let Some(found) = text_box
                            .paragraphs
                            .iter()
                            .find(|p| p.text == "업무관리시스템")
                        {
                            return found;
                        }
                    }
                }
            }
        }
    }
    panic!("바탕쪽 글상자 '업무관리시스템' 문단을 찾지 못했다");
}

#[test]
fn issue_5947_handbook_sidebar_keeps_one_char_stored_lines() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = parse_hwpx(&bytes).expect("parse hwpx");
    let para = handbook_sidebar_para(&doc);

    assert_eq!(
        para.line_segs.len(),
        7,
        "저장 줄은 한 글자씩 7줄이어야 한다"
    );
    for (i, seg) in para.line_segs.iter().enumerate() {
        assert_eq!(seg.text_start, i as u32, "textpos {i}");
        assert_eq!(seg.segment_width, 1440, "horzsize {i}");
    }

    let composed = compose_paragraph(para);
    assert_eq!(composed.lines.len(), 7);
    let chars: Vec<String> = composed
        .lines
        .iter()
        .map(|line| {
            line.runs
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>()
        })
        .collect();
    assert_eq!(
        chars,
        vec!["업", "무", "관", "리", "시", "스", "템"],
        "재래핑 없이 저장 1글자 줄을 유지해야 한다: {chars:?}"
    );
}
