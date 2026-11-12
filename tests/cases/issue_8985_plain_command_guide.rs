//! 관계형·고정형 — [#8985] 평문 command 누름틀(`안내문;…`)의 미기입 안내문은 값이 아니며 Print 에 그리지 않는다.
//! 주석 갱신: 2026-11-12 11:20 KST (새 파일)
//!
//! 입력(둘 다 한컴 저장 HWPX, 10k 공개 코퍼스):
//! - `samples/issue8985/36385713-plain-command-guide.hwpx` — 공개 결재문서. 2쪽 누름틀 「본문2」가
//!   `Command="본문내용을 입력하세요;"`, `dirty="0"`, 내용도 같은 글자다.
//! - `samples/issue8985/3066825-plain-command-guides.hwpx` — [별지 193] 관리미제사건 등록. 평문 command
//!   `0000;`·`0000.00.00.;0000.00.00.\;charshape=b;`·`계급;` 누름틀이 모두 `dirty="0"`, 내용=첫 항목.
//!   대조군: 같은 쪽 `dirty="1"` 누름틀 「성명」은 값으로 인쇄된다.
//!
//! 기준:한컴 engine 2020 Print PDF(PrintMethod 0) `pdf/issue8985/36385713-2020.pdf`(2쪽),
//! `pdf/issue8985/3066825-2020.pdf`(1쪽). 두 PDF 모두 위 안내문을 그리지 않고 「성명」은 그린다.
//! - 고정형: 쪽수 2·1(독립 PDF).
//! - 관계형: 안내문은 필드 값이 아니라 안내문으로 남고, Print 글자 조각에 없다. 본문 「끝.」·「발신명의」와
//!   dirty 누름틀 「성명」은 남는다.
//!
//! 측정(2026-11-12, 코드 base eb84afc036 → head 작업 트리, `scripts/visual_sweep.py --silhouette-only --dpi 96`,
//! raw = `silhouette_raw_match_percent`, `RHWP_FONT_PATH` 글꼴 공급, `--compat 2022`):
//! - 36385713 Native 2쪽 / 2쪽: p1 99.91 (base 99.91), p2 94.95 (base 53.27).
//! - 3066825 Native 1쪽 / 1쪽: p1 99.96 (base 98.18).
//! - 36385713 fresh WASM 2쪽 / 2쪽: p1 99.90, p2 94.95. 3066825 fresh WASM 1쪽 / 1쪽: p1 100.00.
//! - 두 경로 전쪽 최저: 36385713 Native 94.95·WASM 94.95, 3066825 Native 99.96·WASM 100.00.
//!   36385713 2쪽의 남은 차이는 「발신명의」 표 위 테두리 clip(#8986)이다.
//! - 증거: `mydocs/pr/assets/issue_8985/`
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::paint::RenderProfile;
use serde_json::Value;

const BODY: &str = "samples/issue8985/36385713-plain-command-guide.hwpx";
const FORM: &str = "samples/issue8985/3066825-plain-command-guides.hwpx";

fn core(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("open")
}

/// Print 레이어 트리의 글자 조각.
fn printed_texts(core: &DocumentCore, page: u32) -> Vec<String> {
    let json: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_profile_native(page, RenderProfile::Print)
            .expect("layer tree"),
    )
    .expect("json");
    let mut out = Vec::new();
    collect_texts(&json, &mut out);
    out
}

fn collect_texts(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if key == "text" {
                    if let Value::String(text) = child {
                        out.push(text.clone());
                    }
                }
                collect_texts(child, out);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect_texts(item, out)),
        _ => {}
    }
}

/// 평문 command(`Clickhere:` 꼴이 아닌) 누름틀의 (command, guide, value).
fn plain_command_fields(core: &DocumentCore) -> Vec<(String, String, String)> {
    let fields: Value = serde_json::from_str(&core.get_field_list_json()).expect("field json");
    fields
        .as_array()
        .expect("field list")
        .iter()
        .filter(|f| f["fieldType"] == "clickhere")
        .filter_map(|f| {
            let command = f["command"].as_str()?;
            (!command.is_empty() && !command.starts_with("Clickhere:")).then(|| {
                (
                    command.to_string(),
                    f["guide"].as_str().unwrap_or_default().to_string(),
                    f["value"].as_str().unwrap_or_default().to_string(),
                )
            })
        })
        .collect()
}

#[test]
fn unfilled_plain_command_body_guide_is_not_printed() {
    let core = core(BODY);
    assert_eq!(core.page_count(), 2, "한컴 Print 2쪽");

    let body2 = plain_command_fields(&core)
        .into_iter()
        .find(|(command, _, _)| command == "본문내용을 입력하세요;")
        .expect("본문2 누름틀");
    assert_eq!(body2.1, "본문내용을 입력하세요", "첫 `;` 항목이 안내문");
    assert_eq!(body2.2, "", "미기입 안내문은 값이 아니다");

    let texts = printed_texts(&core, 1);
    assert!(
        texts.iter().all(|t| !t.contains("본문내용")),
        "2쪽 Print 에 안내문이 없어야 한다: {texts:?}"
    );
    for kept in ["끝.", "발신명의"] {
        assert!(
            texts.iter().any(|t| t.contains(kept)),
            "2쪽 본문 「{kept}」 보존: {texts:?}"
        );
    }
}

#[test]
fn unfilled_plain_command_form_guides_are_not_printed_but_dirty_value_is() {
    let core = core(FORM);
    assert_eq!(core.page_count(), 1, "한컴 Print 1쪽");

    let fields = plain_command_fields(&core);
    for (command, guide) in [
        ("0000;", "0000"),
        ("0000.00.00.;0000.00.00.\\;charshape=b;", "0000.00.00."),
        ("계급;", "계급"),
    ] {
        let (_, actual_guide, value) = fields
            .iter()
            .find(|(c, _, _)| c == command)
            .unwrap_or_else(|| panic!("command {command:?} 누름틀: {fields:?}"));
        assert_eq!(actual_guide, guide, "{command:?} 의 첫 항목이 안내문");
        assert_eq!(value, "", "{command:?} 미기입 안내문은 값이 아니다");
    }

    let texts = printed_texts(&core, 0);
    for guide in ["0000", "0000.00.00.", "계급"] {
        assert!(
            texts.iter().all(|t| t.trim() != guide),
            "Print 에 안내문 「{guide}」 조각이 없어야 한다: {texts:?}"
        );
    }
    assert!(
        texts.iter().any(|t| t.trim() == "성명"),
        "dirty 누름틀 「성명」 값은 인쇄된다: {texts:?}"
    );
}

/// 평문 command 의 경계: 첫 항목만 안내문, 빈 항목·이스케이프 항목·`;` 없는 command 는 안내문 없음,
/// `Clickhere:` 꼴은 종전대로 `Direction:` 만 읽는다. 누름틀이 아닌 필드는 안내문이 없다.
#[test]
fn plain_command_guide_boundaries() {
    use rhwp::model::control::{Field, FieldType};
    let field = |field_type, command: &str| Field {
        field_type,
        command: command.to_string(),
        ..Default::default()
    };
    let guide = |command: &str| {
        field(FieldType::ClickHere, command)
            .guide_text()
            .map(str::to_string)
    };
    assert_eq!(guide("계급;").as_deref(), Some("계급"));
    assert_eq!(
        guide(r"0000.00.00.;0000.00.00.\;charshape=b;").as_deref(),
        Some("0000.00.00.")
    );
    assert_eq!(guide(";뒤;"), None, "빈 첫 항목");
    assert_eq!(guide(r"앞\;뒤;"), None, "이스케이프가 걸친 첫 항목");
    assert_eq!(guide("Target:이름"), None, "`;` 없는 command");
    assert_eq!(
        guide(&Field::build_clickhere_command("여기에 입력", "")).as_deref(),
        Some("여기에 입력")
    );
    assert_eq!(
        guide("Clickhere:set:0:HelpState:wstring:0:  ;"),
        None,
        "`Clickhere:` 꼴은 평문으로 읽지 않는다"
    );
    assert_eq!(
        field(FieldType::Hyperlink, r"javascript\:\;;1;0;0;").guide_text(),
        None
    );
}
