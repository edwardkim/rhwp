//! 관계형 (편집 문단 높이 변화 ↔ 뒤 문단 이동량, 자리차지 표 점유 ↔ 보존, 표끼리 겹침 없음)
//! 주석 갱신: (측정 후 기입)
//!
//! [#8440] 자리차지(비-TAC, 위아래) 표를 가진 문단 바로 앞 문단의 글자를 고치면, 뒤 문단의
//! vpos 가 그 표의 점유 높이만큼 당겨져 다음 표가 앞 표와 겹쳤다.
//! `recalculate_section_vpos` 가 편집 문단 다음 경계를 스타일 gap 으로만 다시 이으면서
//! 다음 문단 저장 first 에 들어 있던 표 점유(편집 전 저장 end 와의 차 − 스타일 gap)를 버렸다.
//!
//! 원본: samples/issue6123/3112461_railway_emc_criteria.hwpx 구역 0
//!   pi=114 "2) 신호 및 통신, …" (개체 없음) → pi=115 자리차지 표(4×6) 호스트 →
//!   pi=116 빈 문단 → pi=117 " 3) 함체 포트" → pi=118 자리차지 표 호스트.
//! 시각 측정: (측정 후 기입)
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::model::paragraph::LineSeg;
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6123/3112461_railway_emc_criteria.hwpx";
const EDITED: usize = 114;
const TABLE_HOST: usize = 115;
const LAST_CHECKED: usize = 118;

fn load() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    HwpDocument::from_bytes(&std::fs::read(&path).expect("sample")).expect("load")
}

fn first_vpos(doc: &HwpDocument, pi: usize) -> i32 {
    doc.document().sections[0].paragraphs[pi].line_segs[0].vertical_pos
}

/// 로드 경로와 같은 줄 전진(TAC 호스트 줄은 th) 기준의 문단 end.
fn flow_end(doc: &HwpDocument, pi: usize) -> i32 {
    let ls: &LineSeg = doc.document().sections[0].paragraphs[pi]
        .line_segs
        .last()
        .expect("line segs");
    let h = if ls.line_height > ls.text_height && ls.text_height > 0 {
        ls.text_height
    } else {
        ls.line_height
    };
    ls.vertical_pos + h + ls.line_spacing
}

fn host_has_non_tac_table(doc: &HwpDocument, pi: usize) -> bool {
    use rhwp::model::control::Control;
    doc.document().sections[0].paragraphs[pi]
        .controls
        .iter()
        .any(|c| matches!(c, Control::Table(t) if !t.common.treat_as_char))
}

#[test]
fn fixture_has_table_host_after_plain_paragraph() {
    let doc = load();
    assert!(doc.document().sections[0].paragraphs[EDITED]
        .controls
        .is_empty());
    assert!(host_has_non_tac_table(&doc, TABLE_HOST));
    assert!(host_has_non_tac_table(&doc, LAST_CHECKED));
    let occupied = first_vpos(&doc, TABLE_HOST) - flow_end(&doc, EDITED);
    assert!(
        occupied > 10_000,
        "원본 저장 경계에 표 점유가 있어야 한다: {occupied}"
    );
}

#[test]
fn editing_without_line_change_keeps_following_vpos() {
    let before = load();
    let mut doc = load();
    doc.delete_text_native(0, EDITED, 0, 1)
        .expect("delete one char");
    assert_eq!(
        doc.document().sections[0].paragraphs[EDITED]
            .line_segs
            .len(),
        before.document().sections[0].paragraphs[EDITED]
            .line_segs
            .len(),
        "글자 1자 삭제는 줄 수를 바꾸지 않는다"
    );
    for pi in TABLE_HOST..=LAST_CHECKED {
        assert_eq!(
            first_vpos(&doc, pi),
            first_vpos(&before, pi),
            "pi={pi}: 편집 문단 높이가 그대로면 뒤 문단도 그대로여야 한다"
        );
    }
}

#[test]
fn growing_edit_moves_following_paragraphs_by_growth_and_keeps_table_gap() {
    let before = load();
    let mut doc = load();
    let filler = " 가나다라마바사아자차카타파하".repeat(4);
    doc.insert_text_native(0, EDITED, 3, &filler)
        .expect("insert text");
    let growth = flow_end(&doc, EDITED) - flow_end(&before, EDITED);
    assert!(growth > 0, "긴 글을 넣으면 편집 문단이 늘어나야 한다");
    let gap_before = first_vpos(&before, TABLE_HOST) - flow_end(&before, EDITED);
    let gap_after = first_vpos(&doc, TABLE_HOST) - flow_end(&doc, EDITED);
    assert_eq!(
        gap_after, gap_before,
        "표 점유를 포함한 경계 간격은 보존된다"
    );
    for pi in TABLE_HOST..=LAST_CHECKED {
        assert_eq!(
            first_vpos(&doc, pi) - first_vpos(&before, pi),
            growth,
            "pi={pi}: 뒤 문단은 편집 문단이 늘어난 만큼만 움직인다"
        );
    }
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp_out() -> PathBuf {
    std::env::temp_dir().join(format!("rhwp-8440-{}.hwpx", std::process::id()))
}

#[test]
fn edited_document_has_no_new_table_overlap() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let out = temp_out();
    let edit = Command::new(rhwp_bin())
        .args(["edit", "delete-text"])
        .arg(&src)
        .args([
            "--section",
            "0",
            "--para",
            "114",
            "--offset",
            "0",
            "--count",
            "1",
            "-o",
        ])
        .arg(&out)
        .output()
        .unwrap();
    assert_eq!(edit.status.code(), Some(0), "{edit:?}");
    for path in [src.as_path(), out.as_path()] {
        let run = Command::new(rhwp_bin())
            .args(["layout-anomaly"])
            .arg(path)
            .arg("--json")
            .output()
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&run.stdout).expect("anomaly json");
        assert_eq!(v["overlapCount"], 0, "{}: {v}", path.display());
        assert_eq!(v["textOverlapCount"], 0, "{}: {v}", path.display());
        assert_eq!(v["pageCount"], 14, "{}", path.display());
    }
    let _ = std::fs::remove_file(&out);
}
