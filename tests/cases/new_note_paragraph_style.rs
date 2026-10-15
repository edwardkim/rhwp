//! 새 각주·미주 문단은 문서 스타일 목록에서 이름으로 찾은 '각주'·'미주' 스타일과 그 문단
//! 모양을 쓴다. 스타일 번호는 서식마다 다르다. 빈 문서(blank2010)는 11번이 '개요 10'이고
//! '각주'는 14번이다. `143E433F503322BD33` 은 11번이 '쪽 번호'이고 '각주'는 13번이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

fn blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core
}

fn open(path: &str) -> DocumentCore {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    DocumentCore::from_bytes(&std::fs::read(&full).unwrap()).unwrap()
}

/// 0구역 0문단 맨 앞에 각주(`endnote` 이면 미주)를 넣는다. 넣은 직후 문서와 HWP·HWPX 로
/// 저장해 다시 연 문서를, 넣은 주석의 컨트롤 번호와 함께 돌려준다.
fn insert_and_reopen(
    mut core: DocumentCore,
    endnote: bool,
) -> (usize, Vec<(&'static str, DocumentCore)>) {
    let result = if endnote {
        core.insert_endnote_native(0, 0, 0)
    } else {
        core.insert_footnote_native(0, 0, 0)
    }
    .unwrap();
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    let ctrl = json["controlIdx"].as_u64().unwrap() as usize;
    let hwp = DocumentCore::from_bytes(&core.export_hwp_with_adapter().unwrap()).unwrap();
    let hwpx = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    (
        ctrl,
        vec![
            ("넣은 직후", core),
            ("HWP 다시 열기", hwp),
            ("HWPX 다시 열기", hwpx),
        ],
    )
}

fn note_paragraph(core: &DocumentCore, ctrl: usize) -> &Paragraph {
    match &core.document().sections[0].paragraphs[0].controls[ctrl] {
        Control::Footnote(note) => &note.paragraphs[0],
        Control::Endnote(note) => &note.paragraphs[0],
        other => panic!("주석이 아닙니다: {other:?}"),
    }
}

fn assert_named_note_styles(label: &str, open: impl Fn() -> DocumentCore) {
    for (endnote, name) in [(false, "각주"), (true, "미주")] {
        let (ctrl, docs) = insert_and_reopen(open(), endnote);
        for (stage, core) in &docs {
            let para = note_paragraph(core, ctrl);
            let style = core.document().doc_info.styles.get(para.style_id as usize);
            assert_eq!(
                style.map(|style| style.local_name.as_str()),
                Some(name),
                "{label} {stage}: 새 {name} 문단의 스타일 {}번",
                para.style_id
            );
            assert_eq!(
                para.para_shape_id,
                style.unwrap().para_shape_id,
                "{label} {stage}: 새 {name} 문단은 {name} 스타일의 문단 모양을 써야 합니다"
            );
        }
    }
}

#[test]
fn blank_document_note_takes_named_style() {
    assert_named_note_styles("빈 문서", blank);
}

#[test]
fn hancom_note_takes_named_style_from_its_own_style_table() {
    assert_named_note_styles("143E433F503322BD33.hwp", || {
        open("samples/143E433F503322BD33.hwp")
    });
    assert_named_note_styles("143E433F503322BD33.hwpx", || {
        open("samples/hwpx/143E433F503322BD33.hwpx")
    });
}

/// '각주' 스타일도 기존 각주도 없는 문서(스타일 6개)는 커서 문단의 스타일과 문단 모양을 쓴다.
#[test]
fn footnote_without_footnote_style_follows_cursor_paragraph() {
    let (ctrl, docs) = insert_and_reopen(open("samples/hwp3-sample19-hwp5.hwp"), false);
    for (stage, core) in &docs {
        let note = note_paragraph(core, ctrl);
        let host = &core.document().sections[0].paragraphs[0];
        let styles = core.document().doc_info.styles.len();
        assert!(
            (note.style_id as usize) < styles,
            "{stage}: 새 각주 문단의 스타일 {}번이 스타일 목록({styles}개) 밖입니다",
            note.style_id
        );
        assert_eq!(
            (note.style_id, note.para_shape_id),
            (host.style_id, host.para_shape_id),
            "{stage}: 새 각주 문단은 커서 문단의 스타일과 문단 모양을 써야 합니다"
        );
    }
}
