//! Issue #5708: 저장 LINE_SEG 가 없는 각주 문단의 줄 전진이 글자 크기보다 작아 각주 줄이
//! 서로 겹쳐 그려진다.
//!
//! `compose_lines` 는 LINE_SEG 가 없는 문단에 합성 폴백 줄높이 400 HWPUNIT(=5.33px)을 넣는다.
//! 본문·표 경로는 #674 로 문단 줄간격 설정에 맞춰 이 값을 보정하는데, 각주 경로만 그 보정이
//! 빠져 있어 9pt(12px) 글자가 5.3px 간격으로 쌓였다(코퍼스 00464 1쪽 하단 주석 8줄 겹침).
//!
//! 계약: 각주 줄의 전진(다음 줄까지 거리)은 그 줄의 글자 크기 이상이다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::footnote::Footnote;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::{LineSpacingType, ParaShape};
use rhwp::serializer::hwpx::serialize_hwpx;

/// 각주 본문 — 한 문단이 여러 줄로 접히도록 충분히 길게.
const NOTE_TEXT: &str =
    "사건의 개요에는 사건번호, 사건명, 사건본인, 신청인(신청대리인을 포함한다)과 \
신청일자에 대하여 기재한다. 발견경위에는 위조된 재판서의 발견자, 발견시기와 발견과정 등에 대하여 \
기재한다. 조치사항에는 수사의뢰기관의 명칭과 수사기관에 수사의뢰를 하였다는 취지를 적는다.";

fn document_with_footnote() -> Document {
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape {
        line_spacing: 160,
        line_spacing_type: LineSpacingType::Percent,
        ..Default::default()
    }];

    // 각주 문단은 line_segs 를 비워 둔다 — 합성 폴백(400 HWPUNIT) 경로.
    let note_para = Paragraph {
        text: NOTE_TEXT.to_string(),
        ..Default::default()
    };
    let footnote = Footnote {
        number: 1,
        paragraphs: vec![note_para],
        ..Default::default()
    };

    let mut body = Paragraph {
        text: "본문 한 줄".to_string(),
        ..Default::default()
    };
    body.controls.push(Control::Footnote(Box::new(footnote)));

    let mut section = Section::default();
    section.paragraphs.push(body);
    doc.sections.push(section);
    doc
}

/// 각주 영역 줄들의 (y, 줄 상자 높이, 그 줄 안 글자 상자 최대 높이) — 위에서 아래 순.
fn footnote_lines(doc: &Document) -> Vec<(f64, f64, f64)> {
    let bytes = serialize_hwpx(doc).expect("HWPX 직렬화 실패");
    let core = DocumentCore::from_bytes(&bytes).expect("재로드 실패");
    let tree = core
        .build_page_render_tree(0)
        .expect("render tree 생성 실패");
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("render tree JSON 파싱");

    let mut lines: Vec<(f64, f64, f64)> = Vec::new();

    /// 이 줄 안 TextRun 들의 최대 상자 높이 — 글자가 실제로 차지하는 세로 크기.
    fn max_run_height(node: &serde_json::Value) -> f64 {
        let mut best = 0.0f64;
        if let Some(obj) = node.as_object() {
            if obj.get("type").and_then(|v| v.as_str()) == Some("TextRun") {
                let text = obj.get("text").and_then(|v| v.as_str()).unwrap_or("");
                if !text.trim().is_empty() {
                    if let Some(h) = obj.get("bbox").and_then(|b| b.get("h")?.as_f64()) {
                        best = best.max(h);
                    }
                }
            }
            for (_, v) in obj {
                best = best.max(max_run_height(v));
            }
        } else if let Some(arr) = node.as_array() {
            for v in arr {
                best = best.max(max_run_height(v));
            }
        }
        best
    }

    fn walk(node: &serde_json::Value, in_footnote: bool, out: &mut Vec<(f64, f64, f64)>) {
        if let Some(obj) = node.as_object() {
            let ty = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let in_footnote = in_footnote || ty.contains("Footnote");
            if in_footnote && ty == "TextLine" {
                if let Some(b) = obj.get("bbox") {
                    if let (Some(y), Some(h)) = (
                        b.get("y").and_then(|v| v.as_f64()),
                        b.get("h").and_then(|v| v.as_f64()),
                    ) {
                        out.push((y, h, max_run_height(node)));
                    }
                }
            }
            for (_, v) in obj {
                walk(v, in_footnote, out);
            }
        } else if let Some(arr) = node.as_array() {
            for v in arr {
                walk(v, in_footnote, out);
            }
        }
    }
    walk(&json, false, &mut lines);
    lines.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    lines
}

#[test]
fn issue_5708_footnote_lines_do_not_overlap() {
    let lines = footnote_lines(&document_with_footnote());
    assert!(
        lines.len() >= 2,
        "각주가 두 줄 이상 접혀야 이 계약을 잴 수 있다 (줄 {}개)",
        lines.len()
    );

    // 폴백 줄높이 400 HWPUNIT = 5.33px. 보정 전에는 모든 줄 전진이 이 값이었다.
    const FALLBACK_PX: f64 = 400.0 / 7200.0 * 96.0;
    for pair in lines.windows(2) {
        let advance = pair[1].0 - pair[0].0;
        assert!(
            advance > FALLBACK_PX + 1.0,
            "각주 줄 전진이 합성 폴백값({FALLBACK_PX:.1}px)에 머물러 줄이 겹친다: {advance:.1}px (#5708)"
        );
    }
}

#[test]
fn issue_5708_footnote_line_box_contains_its_glyphs() {
    let lines = footnote_lines(&document_with_footnote());
    assert!(!lines.is_empty(), "각주 줄을 찾지 못했다");

    // 줄 상자는 그 줄이 담은 글자 상자를 품어야 한다. 합성 폴백(5.33px)을 그대로 쓰면
    // 9pt(12.0px) 글자가 5.33px 줄 상자에 들어가 아래 줄과 겹친다.
    for (y, line_h, run_h) in &lines {
        assert!(
            *run_h > 0.0,
            "각주 줄에 글자가 없다 (y={y:.1}) — 전제 불성립"
        );
        assert!(
            *line_h >= *run_h - 0.5,
            "각주 줄 상자 {line_h:.1}px 가 글자 상자 {run_h:.1}px 를 담지 못한다 (y={y:.1}) (#5708)"
        );
    }
}
