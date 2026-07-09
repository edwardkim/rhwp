//! Issue #2069: 한셀 OLE 미리보기 RawSvg/placeholder도 선택 가능한 개체여야 한다.
//!
//! `samples/한셀OLE.hwp`/`.hwpx`는 빈 문단에 비-TAC OLE 하나가 놓인 형태다.
//! 렌더 트리는 OLE preview를 RawSvg로 만들지만, 원본 control 좌표를 잃으면 Studio가
//! 클릭 선택/개체 속성 진입을 할 수 없고 빈 문단 커서 rect도 찾지 못한다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::{ShapeObject, TextWrap};
use rhwp::renderer::hwpunit_to_px;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use serde_json::Value;

fn load_core(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e))
}

fn assert_ole_layout_and_caret(rel: &str) {
    let core = load_core(rel);
    let layout_json = core
        .get_page_control_layout_native(0)
        .unwrap_or_else(|e| panic!("layout {}: {:?}", rel, e));
    let layout: Value = serde_json::from_str(&layout_json)
        .unwrap_or_else(|e| panic!("parse layout {} `{}`: {}", rel, layout_json, e));
    let controls = layout["controls"]
        .as_array()
        .unwrap_or_else(|| panic!("layout controls missing for {}", rel));
    let ole = controls
        .iter()
        .find(|control| control["type"] == "ole")
        .unwrap_or_else(|| panic!("OLE control missing for {}: {}", rel, layout_json));

    assert_eq!(ole["secIdx"], 0, "OLE section index");
    assert_eq!(ole["paraIdx"], 0, "OLE paragraph index");
    assert_eq!(ole["controlIdx"], 2, "OLE control index");
    assert!(
        ole["w"].as_f64().unwrap_or_default() > 300.0
            && ole["h"].as_f64().unwrap_or_default() > 30.0,
        "OLE bbox should expose the preview area: {}",
        ole
    );

    let cursor_json = core
        .get_cursor_rect_native(0, 0, 0)
        .unwrap_or_else(|e| panic!("cursor {}: {:?}", rel, e));
    let cursor: Value = serde_json::from_str(&cursor_json)
        .unwrap_or_else(|e| panic!("parse cursor {} `{}`: {}", rel, cursor_json, e));
    let ole_left = ole["x"].as_f64().unwrap();
    let expected_x = ole_left + ole["w"].as_f64().unwrap();
    let actual_x = cursor["x"].as_f64().unwrap();
    let expected_y = ole["y"].as_f64().unwrap();
    let actual_y = cursor["y"].as_f64().unwrap();
    let cursor_h = cursor["height"].as_f64().unwrap();
    let ole_h = ole["h"].as_f64().unwrap();

    assert_eq!(cursor["pageIndex"], 0, "cursor page index");
    assert!(
        (actual_x - expected_x).abs() <= 0.6,
        "cursor x should be at OLE right edge for {}: cursor={}, ole={}",
        rel,
        cursor,
        ole
    );
    assert!(
        (actual_y - expected_y).abs() <= 0.6,
        "cursor y should follow OLE top for {}: cursor={}, ole={}",
        rel,
        cursor,
        ole
    );
    assert!(
        (10.0..ole_h / 2.0).contains(&cursor_h),
        "cursor height should use text line metrics, not full OLE height for {}: cursor={}, ole={}",
        rel,
        cursor,
        ole
    );

    fn collect_para_end_anchors<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text.is_empty()
                && run.section_index == Some(0)
                && run.para_index == Some(0)
                && run.char_start == Some(0)
                && run.is_para_end
            {
                out.push(node);
            }
        }
        for child in &node.children {
            collect_para_end_anchors(child, out);
        }
    }

    let tree = core
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render tree {}: {:?}", rel, e));
    let mut anchors = Vec::new();
    collect_para_end_anchors(&tree.root, &mut anchors);
    assert_eq!(
        anchors.len(),
        1,
        "OLE paragraph mark count should follow the stored paragraph structure, not preview rows for {}",
        rel
    );
    let anchor = anchors[0];
    assert!(
        (anchor.bbox.x - expected_x).abs() <= 0.6,
        "paragraph mark anchor should follow OLE right edge for {}: anchor={:?}, ole={}",
        rel,
        anchor.bbox,
        ole
    );
    assert!(
        (anchor.bbox.y - expected_y).abs() <= 0.6,
        "paragraph mark anchor should follow the single stored paragraph line for {}: anchor={:?}, ole={}",
        rel,
        anchor.bbox,
        ole
    );
}

fn collect_para_end_anchors<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.is_empty() && run.section_index == Some(0) && run.is_para_end {
            out.push(node);
        }
    }
    for child in &node.children {
        collect_para_end_anchors(child, out);
    }
}

fn collect_text_run_nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextRun(_)) {
        out.push(node);
    }
    for child in &node.children {
        collect_text_run_nodes(child, out);
    }
}

fn assert_enter_after_square_ole_keeps_wrap_zone(rel: &str) {
    let mut core = load_core(rel);
    let original_line_seg = core.document().sections[0].paragraphs[0].line_segs[0].clone();
    assert!(
        original_line_seg.column_start > 0 && original_line_seg.segment_width > 0,
        "{} should encode the OLE-side wrap zone in LINE_SEG: {:?}",
        rel,
        original_line_seg
    );
    let expected_line_pitch_px = hwpunit_to_px(
        original_line_seg.line_height + original_line_seg.line_spacing,
        96.0,
    );

    core.split_paragraph_native(0, 0, 0)
        .unwrap_or_else(|e| panic!("split after OLE {}: {:?}", rel, e));

    let section = &core.document().sections[0];
    assert_eq!(
        section.paragraphs.len(),
        2,
        "Enter after OLE should create one following paragraph for {}",
        rel
    );
    assert!(
        matches!(
            section.paragraphs[0].controls.get(2),
            Some(Control::Shape(shape))
                if matches!(shape.as_ref(), ShapeObject::Ole(_))
                    && matches!(shape.common().text_wrap, TextWrap::Square)
                    && !shape.common().treat_as_char
        ),
        "Square OLE should stay anchored to the original paragraph after Enter for {}",
        rel
    );
    assert!(
        section.paragraphs[1].controls.is_empty(),
        "the paragraph inserted by Enter should be an empty following paragraph for {}",
        rel
    );
    assert_eq!(
        section.paragraphs[1].line_segs[0].column_start, original_line_seg.column_start,
        "the following empty paragraph should preserve the stored wrap-zone x for {}",
        rel
    );
    assert_eq!(
        section.paragraphs[1].line_segs[0].segment_width, original_line_seg.segment_width,
        "the following empty paragraph should preserve the stored wrap-zone width for {}",
        rel
    );

    let layout_json = core
        .get_page_control_layout_native(0)
        .unwrap_or_else(|e| panic!("layout after Enter {}: {:?}", rel, e));
    let layout: Value = serde_json::from_str(&layout_json)
        .unwrap_or_else(|e| panic!("parse layout {} `{}`: {}", rel, layout_json, e));
    let controls = layout["controls"]
        .as_array()
        .unwrap_or_else(|| panic!("layout controls missing after Enter for {}", rel));
    let ole = controls
        .iter()
        .find(|control| control["type"] == "ole")
        .unwrap_or_else(|| {
            panic!(
                "OLE control missing after Enter for {}: {}",
                rel, layout_json
            )
        });
    assert_eq!(ole["paraIdx"], 0, "OLE should remain in paragraph 0");

    let ole_left = ole["x"].as_f64().unwrap();
    let expected_x = ole_left + ole["w"].as_f64().unwrap();
    let tree = core
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render tree after Enter {}: {:?}", rel, e));
    let mut anchors = Vec::new();
    collect_para_end_anchors(&tree.root, &mut anchors);
    let para0_anchor = anchors
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextRun(run)
                    if run.para_index == Some(0) && run.char_start == Some(0)
            )
        })
        .unwrap_or_else(|| panic!("original paragraph mark missing after Enter for {}", rel));
    let para1_anchor = anchors
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextRun(run)
                    if run.para_index == Some(1) && run.char_start == Some(0)
            )
        })
        .unwrap_or_else(|| panic!("following paragraph mark missing after Enter for {}", rel));

    assert!(
        (para0_anchor.bbox.x - expected_x).abs() <= 0.6,
        "original OLE paragraph mark should remain at OLE right edge for {}: anchor={:?}, ole={}",
        rel,
        para0_anchor.bbox,
        ole
    );
    assert!(
        (para1_anchor.bbox.x - expected_x).abs() <= 0.6,
        "Enter-created paragraph mark should stay in the OLE-side wrap zone for {}: anchor={:?}, ole={}",
        rel,
        para1_anchor.bbox,
        ole
    );
    assert!(
        para1_anchor.bbox.y > para0_anchor.bbox.y,
        "Enter-created paragraph mark should be on the following line for {}: para0={:?}, para1={:?}",
        rel,
        para0_anchor.bbox,
        para1_anchor.bbox
    );
    assert!(
        (para1_anchor.bbox.y - para0_anchor.bbox.y - expected_line_pitch_px).abs() <= 1.0,
        "Enter-created paragraph mark should follow the stored OLE line pitch for {}: para0={:?}, para1={:?}, pitch={:.2}",
        rel,
        para0_anchor.bbox,
        para1_anchor.bbox,
        expected_line_pitch_px
    );

    let cursor_json = core
        .get_cursor_rect_native(0, 1, 0)
        .unwrap_or_else(|e| panic!("cursor after Enter {}: {:?}", rel, e));
    let cursor: Value = serde_json::from_str(&cursor_json)
        .unwrap_or_else(|e| panic!("parse cursor after Enter {} `{}`: {}", rel, cursor_json, e));
    let actual_cursor_x = cursor["x"].as_f64().unwrap();
    let actual_cursor_y = cursor["y"].as_f64().unwrap();
    assert!(
        (actual_cursor_x - expected_x).abs() <= 0.6,
        "active caret after Enter should stay in the OLE-side wrap zone for {}: cursor={}, ole={}",
        rel,
        cursor,
        ole
    );
    assert!(
        actual_cursor_y > para0_anchor.bbox.y,
        "active caret after Enter should move to the following line for {}: cursor={}, para0={:?}",
        rel,
        cursor,
        para0_anchor.bbox
    );

    core.split_paragraph_native(0, 1, 0)
        .unwrap_or_else(|e| panic!("second split after OLE {}: {:?}", rel, e));

    let section = &core.document().sections[0];
    assert_eq!(
        section.paragraphs.len(),
        3,
        "two consecutive Enters after OLE should create two following paragraphs for {}",
        rel
    );
    assert!(
        section.paragraphs[2].controls.is_empty(),
        "the second Enter-created paragraph should be empty for {}",
        rel
    );
    assert_eq!(
        section.paragraphs[2].line_segs[0].column_start, original_line_seg.column_start,
        "the second following paragraph should preserve the stored wrap-zone x for {}",
        rel
    );
    assert_eq!(
        section.paragraphs[2].line_segs[0].segment_width, original_line_seg.segment_width,
        "the second following paragraph should preserve the stored wrap-zone width for {}",
        rel
    );

    let tree = core
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render tree after second Enter {}: {:?}", rel, e));
    let mut anchors = Vec::new();
    collect_para_end_anchors(&tree.root, &mut anchors);
    let para1_anchor_after_second = anchors
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextRun(run)
                    if run.para_index == Some(1) && run.char_start == Some(0)
            )
        })
        .unwrap_or_else(|| {
            panic!(
                "first following paragraph mark missing after second Enter for {}",
                rel
            )
        });
    let para2_anchor = anchors
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextRun(run)
                    if run.para_index == Some(2) && run.char_start == Some(0)
            )
        })
        .unwrap_or_else(|| {
            panic!(
                "second following paragraph mark missing after second Enter for {}",
                rel
            )
        });

    assert!(
        (para1_anchor_after_second.bbox.x - expected_x).abs() <= 0.6,
        "first following paragraph mark should remain in the OLE-side wrap zone after second Enter for {}: anchor={:?}, ole={}",
        rel,
        para1_anchor_after_second.bbox,
        ole
    );
    assert!(
        (para2_anchor.bbox.x - expected_x).abs() <= 0.6,
        "second following paragraph mark should stay in the OLE-side wrap zone for {}: anchor={:?}, ole={}",
        rel,
        para2_anchor.bbox,
        ole
    );
    assert!(
        para2_anchor.bbox.y > para1_anchor_after_second.bbox.y,
        "second following paragraph mark should be below the first for {}: para1={:?}, para2={:?}",
        rel,
        para1_anchor_after_second.bbox,
        para2_anchor.bbox
    );
    assert!(
        (para2_anchor.bbox.y
            - para1_anchor_after_second.bbox.y
            - expected_line_pitch_px)
            .abs()
            <= 1.0,
        "second following paragraph mark should follow the stored OLE line pitch for {}: para1={:?}, para2={:?}, pitch={:.2}",
        rel,
        para1_anchor_after_second.bbox,
        para2_anchor.bbox,
        expected_line_pitch_px
    );

    let cursor_json = core
        .get_cursor_rect_native(0, 2, 0)
        .unwrap_or_else(|e| panic!("cursor after second Enter {}: {:?}", rel, e));
    let cursor: Value = serde_json::from_str(&cursor_json).unwrap_or_else(|e| {
        panic!(
            "parse cursor after second Enter {} `{}`: {}",
            rel, cursor_json, e
        )
    });
    let actual_cursor_x = cursor["x"].as_f64().unwrap();
    let actual_cursor_y = cursor["y"].as_f64().unwrap();
    assert!(
        (actual_cursor_x - expected_x).abs() <= 0.6,
        "active caret after second Enter should stay in the OLE-side wrap zone for {}: cursor={}, ole={}",
        rel,
        cursor,
        ole
    );
    assert!(
        actual_cursor_y > para1_anchor_after_second.bbox.y,
        "active caret after second Enter should move to the second following line for {}: cursor={}, para1={:?}",
        rel,
        cursor,
        para1_anchor_after_second.bbox
    );

    core.split_paragraph_native(0, 2, 0)
        .unwrap_or_else(|e| panic!("third split after OLE {}: {:?}", rel, e));

    let section = &core.document().sections[0];
    assert_eq!(
        section.paragraphs.len(),
        4,
        "three consecutive Enters after OLE should create three following paragraphs for {}",
        rel
    );
    assert!(
        section.paragraphs[3].controls.is_empty(),
        "the paragraph created after leaving the OLE height should be empty for {}",
        rel
    );
    assert_eq!(
        section.paragraphs[2].line_segs[0].column_start, original_line_seg.column_start,
        "the last line that overlaps the OLE height should still preserve the wrap-zone x for {}",
        rel
    );
    assert_eq!(
        section.paragraphs[3].line_segs[0].column_start, 0,
        "the first line below the OLE height should return to normal body flow for {}",
        rel
    );
    assert!(
        section.paragraphs[3].line_segs[0].segment_width > original_line_seg.segment_width,
        "the first line below the OLE height should recover body-width line metrics for {}: {:?}",
        rel,
        section.paragraphs[3].line_segs[0]
    );

    let tree = core
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render tree after third Enter {}: {:?}", rel, e));
    let mut anchors = Vec::new();
    collect_para_end_anchors(&tree.root, &mut anchors);
    let para2_anchor_after_third = anchors
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextRun(run)
                    if run.para_index == Some(2) && run.char_start == Some(0)
            )
        })
        .unwrap_or_else(|| {
            panic!(
                "second following paragraph mark missing after third Enter for {}",
                rel
            )
        });
    let para3_anchor = anchors
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextRun(run)
                    if run.para_index == Some(3) && run.char_start == Some(0)
            )
        })
        .unwrap_or_else(|| {
            panic!(
                "body-flow paragraph mark missing after third Enter for {}",
                rel
            )
        });

    assert!(
        (para2_anchor_after_third.bbox.x - expected_x).abs() <= 0.6,
        "the final OLE-overlapping paragraph mark should stay at OLE right edge for {}: anchor={:?}, ole={}",
        rel,
        para2_anchor_after_third.bbox,
        ole
    );
    assert!(
        para3_anchor.bbox.x <= ole_left + 0.6 && expected_x - para3_anchor.bbox.x > 100.0,
        "paragraph mark below the OLE height should return to body-left flow for {}: anchor={:?}, ole={}",
        rel,
        para3_anchor.bbox,
        ole
    );
    assert!(
        para3_anchor.bbox.y > para2_anchor_after_third.bbox.y,
        "body-flow paragraph mark should be below the last OLE-overlapping line for {}: para2={:?}, para3={:?}",
        rel,
        para2_anchor_after_third.bbox,
        para3_anchor.bbox
    );
    assert!(
        (para3_anchor.bbox.y - para2_anchor_after_third.bbox.y - expected_line_pitch_px).abs()
            <= 1.0,
        "first body-flow paragraph mark below OLE should continue the stored line pitch for {}: para2={:?}, para3={:?}, pitch={:.2}",
        rel,
        para2_anchor_after_third.bbox,
        para3_anchor.bbox,
        expected_line_pitch_px
    );

    let cursor_json = core
        .get_cursor_rect_native(0, 3, 0)
        .unwrap_or_else(|e| panic!("cursor after third Enter {}: {:?}", rel, e));
    let cursor: Value = serde_json::from_str(&cursor_json).unwrap_or_else(|e| {
        panic!(
            "parse cursor after third Enter {} `{}`: {}",
            rel, cursor_json, e
        )
    });
    let actual_cursor_x = cursor["x"].as_f64().unwrap();
    let actual_cursor_y = cursor["y"].as_f64().unwrap();
    assert!(
        actual_cursor_x <= ole_left + 0.6 && expected_x - actual_cursor_x > 100.0,
        "active caret below the OLE height should return to body-left flow for {}: cursor={}, ole={}",
        rel,
        cursor,
        ole
    );
    assert!(
        actual_cursor_y > para2_anchor_after_third.bbox.y,
        "active caret after third Enter should move below the last OLE-overlapping line for {}: cursor={}, para2={:?}",
        rel,
        cursor,
        para2_anchor_after_third.bbox
    );
}

fn assert_enter_backspace_reenter_after_square_ole(rel: &str) {
    let mut core = load_core(rel);
    let original_line_seg = core.document().sections[0].paragraphs[0].line_segs[0].clone();
    assert!(
        original_line_seg.column_start > 0 && original_line_seg.segment_width > 0,
        "{} should start from an OLE-side wrap-zone line: {:?}",
        rel,
        original_line_seg
    );
    let expected_line_pitch_px = hwpunit_to_px(
        original_line_seg.line_height + original_line_seg.line_spacing,
        96.0,
    );

    core.split_paragraph_native(0, 0, 0)
        .unwrap_or_else(|e| panic!("initial split after OLE {}: {:?}", rel, e));
    core.merge_paragraph_native(0, 1)
        .unwrap_or_else(|e| panic!("Backspace merge after OLE {}: {:?}", rel, e));

    let section = &core.document().sections[0];
    assert_eq!(
        section.paragraphs.len(),
        1,
        "Backspace should remove the Enter-created paragraph for {}",
        rel
    );
    assert!(
        matches!(
            section.paragraphs[0].controls.get(2),
            Some(Control::Shape(shape))
                if matches!(shape.as_ref(), ShapeObject::Ole(_))
                    && matches!(shape.common().text_wrap, TextWrap::Square)
                    && !shape.common().treat_as_char
        ),
        "Square OLE should remain anchored to paragraph 0 after Backspace for {}",
        rel
    );
    assert_eq!(
        section.paragraphs[0].line_segs[0].column_start, original_line_seg.column_start,
        "Backspace should preserve the original OLE wrap-zone x for {}",
        rel
    );
    assert_eq!(
        section.paragraphs[0].line_segs[0].segment_width, original_line_seg.segment_width,
        "Backspace should preserve the original OLE wrap-zone width for {}",
        rel
    );

    core.split_paragraph_native(0, 0, 0)
        .unwrap_or_else(|e| panic!("re-enter split after Backspace {}: {:?}", rel, e));

    let section = &core.document().sections[0];
    assert_eq!(
        section.paragraphs.len(),
        2,
        "re-enter Enter after Backspace should recreate one following paragraph for {}",
        rel
    );
    assert!(
        matches!(
            section.paragraphs[0].controls.get(2),
            Some(Control::Shape(shape))
                if matches!(shape.as_ref(), ShapeObject::Ole(_))
                    && matches!(shape.common().text_wrap, TextWrap::Square)
                    && !shape.common().treat_as_char
        ),
        "re-enter Enter should keep OLE in paragraph 0 for {}",
        rel
    );
    assert!(
        section.paragraphs[1].controls.is_empty(),
        "re-enter Enter should create an empty following paragraph for {}",
        rel
    );
    assert_eq!(
        section.paragraphs[1].line_segs[0].column_start, original_line_seg.column_start,
        "re-enter Enter should preserve OLE-side wrap-zone x for {}",
        rel
    );
    assert_eq!(
        section.paragraphs[1].line_segs[0].segment_width, original_line_seg.segment_width,
        "re-enter Enter should preserve OLE-side wrap-zone width for {}",
        rel
    );

    let layout_json = core
        .get_page_control_layout_native(0)
        .unwrap_or_else(|e| panic!("layout after re-enter Enter {}: {:?}", rel, e));
    let layout: Value = serde_json::from_str(&layout_json)
        .unwrap_or_else(|e| panic!("parse layout {} `{}`: {}", rel, layout_json, e));
    let controls = layout["controls"]
        .as_array()
        .unwrap_or_else(|| panic!("layout controls missing after re-enter Enter for {}", rel));
    let ole = controls
        .iter()
        .find(|control| control["type"] == "ole")
        .unwrap_or_else(|| panic!("OLE control missing after re-enter Enter for {}", rel));
    assert_eq!(
        ole["paraIdx"], 0,
        "OLE should not move to the Enter-created paragraph after Backspace for {}",
        rel
    );

    let ole_left = ole["x"].as_f64().unwrap();
    let expected_x = ole_left + ole["w"].as_f64().unwrap();
    let tree = core
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render tree after re-enter Enter {}: {:?}", rel, e));
    let mut anchors = Vec::new();
    collect_para_end_anchors(&tree.root, &mut anchors);
    let para0_anchor = anchors
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextRun(run)
                    if run.para_index == Some(0) && run.char_start == Some(0)
            )
        })
        .unwrap_or_else(|| panic!("original paragraph mark missing after re-enter {}", rel));
    let para1_anchor = anchors
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextRun(run)
                    if run.para_index == Some(1) && run.char_start == Some(0)
            )
        })
        .unwrap_or_else(|| panic!("following paragraph mark missing after re-enter {}", rel));

    assert!(
        (para0_anchor.bbox.x - expected_x).abs() <= 0.6,
        "original mark after re-enter should stay at OLE right edge for {}: anchor={:?}, ole={}",
        rel,
        para0_anchor.bbox,
        ole
    );
    assert!(
        (para1_anchor.bbox.x - expected_x).abs() <= 0.6,
        "following mark after re-enter should stay in OLE-side wrap zone for {}: anchor={:?}, ole={}",
        rel,
        para1_anchor.bbox,
        ole
    );
    assert!(
        (para1_anchor.bbox.y - para0_anchor.bbox.y - expected_line_pitch_px).abs() <= 1.0,
        "following mark after re-enter should match initial Enter line pitch for {}: para0={:?}, para1={:?}, pitch={:.2}",
        rel,
        para0_anchor.bbox,
        para1_anchor.bbox,
        expected_line_pitch_px
    );

    let cursor_json = core
        .get_cursor_rect_native(0, 1, 0)
        .unwrap_or_else(|e| panic!("cursor after re-enter Enter {}: {:?}", rel, e));
    let cursor: Value = serde_json::from_str(&cursor_json).unwrap_or_else(|e| {
        panic!(
            "parse cursor after re-enter Enter {} `{}`: {}",
            rel, cursor_json, e
        )
    });
    assert!(
        (cursor["x"].as_f64().unwrap() - expected_x).abs() <= 0.6,
        "active caret after re-enter should stay in OLE-side wrap zone for {}: cursor={}, ole={}",
        rel,
        cursor,
        ole
    );
}

fn assert_ole_caption_properties_roundtrip(rel: &str) {
    let mut core = load_core(rel);
    let before_json = core
        .get_shape_properties_native(0, 0, 2)
        .unwrap_or_else(|e| panic!("get OLE props before caption {}: {:?}", rel, e));
    let before: Value = serde_json::from_str(&before_json).unwrap_or_else(|e| {
        panic!(
            "parse OLE props before caption {} `{}`: {}",
            rel, before_json, e
        )
    });
    assert_eq!(
        before["hasCaption"], false,
        "{} should start without an OLE caption",
        rel
    );

    core.set_shape_properties_native(
        0,
        0,
        2,
        r#"{"hasCaption":true,"captionDirection":"Right","captionVertAlign":"Bottom","captionWidth":8504,"captionSpacing":850,"captionIncludeMargin":true}"#,
    )
    .unwrap_or_else(|e| panic!("set OLE caption props {}: {:?}", rel, e));

    let after_json = core
        .get_shape_properties_native(0, 0, 2)
        .unwrap_or_else(|e| panic!("get OLE props after caption {}: {:?}", rel, e));
    let after: Value = serde_json::from_str(&after_json).unwrap_or_else(|e| {
        panic!(
            "parse OLE props after caption {} `{}`: {}",
            rel, after_json, e
        )
    });
    assert_eq!(after["hasCaption"], true, "OLE caption should be created");
    assert_eq!(
        after["captionDirection"], "Right",
        "OLE caption direction should persist"
    );
    assert_eq!(
        after["captionVertAlign"], "Bottom",
        "OLE caption vertical alignment should persist"
    );
    assert_eq!(
        after["captionWidth"], 8504,
        "OLE caption width should persist"
    );
    assert_eq!(
        after["captionSpacing"], 850,
        "OLE caption spacing should persist"
    );
    assert_eq!(
        after["captionIncludeMargin"], true,
        "OLE caption include-margin should persist"
    );

    let section = &core.document().sections[0];
    let Some(Control::Shape(shape)) = section.paragraphs[0].controls.get(2) else {
        panic!("OLE shape missing after caption set for {}", rel);
    };
    let ShapeObject::Ole(ole) = shape.as_ref() else {
        panic!("control 2 should remain OLE after caption set for {}", rel);
    };
    let caption = ole
        .caption
        .as_ref()
        .unwrap_or_else(|| panic!("OLE caption model missing for {}", rel));
    let cap_para = caption
        .paragraphs
        .first()
        .unwrap_or_else(|| panic!("OLE caption paragraph missing for {}", rel));
    assert_eq!(cap_para.text, "그림  ", "OLE caption text prefix");
    assert_eq!(
        cap_para.char_offsets,
        vec![0, 1, 2, 11],
        "OLE caption AutoNumber placeholder offsets"
    );
    let auto_number = cap_para
        .controls
        .iter()
        .find_map(|ctrl| match ctrl {
            Control::AutoNumber(auto)
                if matches!(
                    auto.number_type,
                    rhwp::model::control::AutoNumberType::Picture
                ) =>
            {
                Some(auto)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("OLE caption AutoNumber missing for {}", rel));
    assert_eq!(
        auto_number.assigned_number, 1,
        "new OLE caption should be assigned as figure 1 for {}",
        rel
    );
    assert_eq!(
        auto_number.suffix_char, '.',
        "new OLE caption should keep the HWP figure-number suffix for {}",
        rel
    );

    let layout_json = core
        .get_page_control_layout_native(0)
        .unwrap_or_else(|e| panic!("layout after OLE caption {}: {:?}", rel, e));
    let layout: Value = serde_json::from_str(&layout_json).unwrap_or_else(|e| {
        panic!(
            "parse layout after OLE caption {} `{}`: {}",
            rel, layout_json, e
        )
    });
    let ole_layout = layout["controls"]
        .as_array()
        .and_then(|controls| controls.iter().find(|control| control["type"] == "ole"))
        .unwrap_or_else(|| panic!("OLE layout missing after caption set for {}", rel));
    let ole_right = ole_layout["x"].as_f64().unwrap() + ole_layout["w"].as_f64().unwrap();

    let tree = core
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render tree after OLE caption {}: {:?}", rel, e));
    let mut text_runs = Vec::new();
    collect_text_run_nodes(&tree.root, &mut text_runs);
    let rendered_text = text_runs
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect::<String>();
    let caption_run = text_runs
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextRun(run)
                    if run.text.contains("그림") && run.text.contains("1.")
            )
        })
        .unwrap_or_else(|| {
            panic!(
                "OLE caption should render as visible figure text for {}: rendered={:?}",
                rel, rendered_text
            )
        });
    assert!(
        caption_run.bbox.x > ole_right,
        "OLE right caption should render to the right of the OLE preview for {}: caption={:?}, ole={}",
        rel,
        caption_run.bbox,
        ole_layout
    );
    assert!(
        caption_run.bbox.width > 0.0 && caption_run.bbox.height > 0.0,
        "new OLE caption should contain a picture AutoNumber control for {}",
        rel
    );

    core.set_shape_properties_native(0, 0, 2, r#"{"hasCaption":false}"#)
        .unwrap_or_else(|e| panic!("remove OLE caption props {}: {:?}", rel, e));
    let removed_json = core
        .get_shape_properties_native(0, 0, 2)
        .unwrap_or_else(|e| panic!("get OLE props after caption removal {}: {:?}", rel, e));
    let removed: Value = serde_json::from_str(&removed_json).unwrap_or_else(|e| {
        panic!(
            "parse OLE props after caption removal {} `{}`: {}",
            rel, removed_json, e
        )
    });
    assert_eq!(
        removed["hasCaption"], false,
        "center caption grid should remove the OLE caption for {}",
        rel
    );
    let section = &core.document().sections[0];
    let Some(Control::Shape(shape)) = section.paragraphs[0].controls.get(2) else {
        panic!("OLE shape missing after caption removal for {}", rel);
    };
    let ShapeObject::Ole(ole) = shape.as_ref() else {
        panic!(
            "control 2 should remain OLE after caption removal for {}",
            rel
        );
    };
    assert!(
        ole.caption.is_none() && ole.drawing.caption.is_none(),
        "OLE caption slots should be cleared for {}",
        rel
    );
    let tree = core
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render tree after OLE caption removal {}: {:?}", rel, e));
    let mut text_runs = Vec::new();
    collect_text_run_nodes(&tree.root, &mut text_runs);
    let rendered_text = text_runs
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect::<String>();
    assert!(
        !rendered_text.contains("그림"),
        "removed OLE caption should not remain visible for {}: rendered={:?}",
        rel,
        rendered_text
    );
}

#[test]
fn hwp_ole_preview_is_selectable_and_drives_empty_para_caret() {
    assert_ole_layout_and_caret("samples/한셀OLE.hwp");
}

#[test]
fn hwpx_ole_preview_is_selectable_and_drives_empty_para_caret() {
    assert_ole_layout_and_caret("samples/한셀OLE.hwpx");
}

#[test]
fn hwp_enter_after_square_ole_respects_height_boundary() {
    assert_enter_after_square_ole_keeps_wrap_zone("samples/한셀OLE.hwp");
}

#[test]
fn hwpx_enter_after_square_ole_respects_height_boundary() {
    assert_enter_after_square_ole_keeps_wrap_zone("samples/한셀OLE.hwpx");
}

#[test]
fn hwp_enter_backspace_reenter_keeps_ole_anchor_flow() {
    assert_enter_backspace_reenter_after_square_ole("samples/한셀OLE.hwp");
}

#[test]
fn hwpx_enter_backspace_reenter_keeps_ole_anchor_flow() {
    assert_enter_backspace_reenter_after_square_ole("samples/한셀OLE.hwpx");
}

#[test]
fn hwp_ole_caption_properties_roundtrip() {
    assert_ole_caption_properties_roundtrip("samples/한셀OLE.hwp");
}

#[test]
fn hwpx_ole_caption_properties_roundtrip() {
    assert_ole_caption_properties_roundtrip("samples/한셀OLE.hwpx");
}
