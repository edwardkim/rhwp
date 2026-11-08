//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8443의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:36:46+09:00 (`bee94f016ddf9971b0f18c219b1b62a394f45462`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `mydocs/pr/assets/issue_8440/integration_20261103/pr8443-emc-edit.hwpx`; 독립 Print `pdf/issue-8440/emc_edit-2024.pdf`.
//! 전체 14쪽, 전쪽 최저 Native 99.61% / fresh WASM 99.78%.
//! 주장 7쪽 raw: Native 99.92% / fresh WASM 99.98%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8443_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-02T07:20:00+09:00
//!
//! [#8440] 자리차지(비-TAC, 위아래) 표를 가진 문단 바로 앞 문단의 글자를 고치면, 뒤 문단의
//! vpos 가 그 표의 점유 높이만큼 당겨져 다음 표가 앞 표와 겹쳤다.
//! `recalculate_section_vpos` 가 편집 문단 다음 경계를 스타일 gap 으로만 다시 이으면서
//! 다음 문단 저장 first 에 들어 있던 표 점유(편집 전 저장 end 와의 차 − 스타일 gap)를 버렸다.
//!
//! 원본: samples/issue6123/3112461_railway_emc_criteria.hwpx 구역 0
//!   pi=114 "2) 신호 및 통신, …" (개체 없음) → pi=115 자리차지 표(4×6) 호스트 →
//!   pi=116 빈 문단 → pi=117 " 3) 함체 포트" → pi=118 자리차지 표 호스트.
//!
//! 시각 측정 (2026-11-02, `visual_sweep.py --silhouette-only --compat 2024 --dpi 96`, pdftoppm,
//! raw = silhouette_raw_match_percent, RHWP_FONT_PATH=ttfs/opensource):
//!   기준 PDF `pdf/issue-8440/emc_edit-2024.pdf` = 위 편집본(문단 114 첫 글자 1자 삭제)의
//!   한컴 2024 Print 출력, 14쪽. 대상 쪽 7: Native 99.25%, fresh WASM 99.08% (수정 전 편집본 60.76%).
//!   전체 쪽수 한컴 14 = rhwp 14. 전쪽 최저 Native 98.86%(11쪽), fresh WASM 98.43%(10쪽).
//!   증거: mydocs/pr/assets/issue_8440/ (tsv/native·wasm·base_native, png/emc_edit-p007-*).
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
