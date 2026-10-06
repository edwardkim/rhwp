//! [#7218] 문단 시작(offset 0)의 쪽 나눔은 문단을 가르지 않는다.
//!
//! 종전 `edit insert-page-break --offset 0` 은 offset 과 관계없이 문단을 갈라, 앞쪽에 원 문단
//! 모양(개요 수준 포함)을 물려받은 **빈 문단**을 남겼다. 개요 제목 앞이면 한글이 그 빈 문단에도
//! 번호를 붙여 번호가 비고 뒤 번호가 밀린다. 수정 후에는 대상 문단의 `column_type` 만 `Page` 로
//! 바꾸고 문단 수·텍스트·문단 모양을 그대로 둔다. 봉투는 `paragraphDelta`·`pageBreakParagraph` 로
//! 후속 좌표 편집이 어긋나지 않게 알린다.
//!
//! 범위: CLI `edit insert-page-break` 와 이를 호출하는 MCP `hwp_insert_page_break`.
//! CLI 계약은 #7230에서 왔고, 아래 core_contract는 #7238의 저장 계약을 명시적 속성 setter에서 검증한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rhwp::model::paragraph::ColumnBreakType;
use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin()).args(args).output().expect("rhwp")
}

fn temp(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-7218-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

/// 이슈 재현 입력: 개요 제목 둘과 본문 둘.
fn scaffold_outline_doc() -> PathBuf {
    let json = temp("spec", "json");
    std::fs::write(
        &json,
        r#"{"version":"1","title":"repro","blocks":[
 {"type":"heading","level":1,"text":"First"},
 {"type":"paragraph","text":"body 1"},
 {"type":"heading","level":1,"text":"Second"},
 {"type":"paragraph","text":"body 2"}
]}"#,
    )
    .unwrap();
    let out = temp("doc", "hwpx");
    let output = run(&[
        "scaffold",
        json.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "scaffold: {output:?}");
    let _ = std::fs::remove_file(&json);
    out
}

struct Para {
    text: String,
    para_shape_id: u16,
    page_break: bool,
}

fn paragraphs(path: &Path) -> Vec<Para> {
    let doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    doc.document().sections[0]
        .paragraphs
        .iter()
        .map(|p| Para {
            text: p.text.clone(),
            para_shape_id: p.para_shape_id,
            page_break: p.column_type == ColumnBreakType::Page,
        })
        .collect()
}

fn index_of(paras: &[Para], text: &str) -> usize {
    paras
        .iter()
        .position(|p| p.text == text)
        .unwrap_or_else(|| panic!("`{text}` 문단이 없다"))
}

fn page_break(src: &Path, para: usize, offset: usize, tag: &str) -> (PathBuf, serde_json::Value) {
    let out = temp(tag, "hwpx");
    let para_arg = para.to_string();
    let offset_arg = offset.to_string();
    let output = run(&[
        "edit",
        "insert-page-break",
        src.to_str().unwrap(),
        "--section",
        "0",
        "--para",
        para_arg.as_str(),
        "--offset",
        offset_arg.as_str(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "insert-page-break: {output:?}"
    );
    let envelope = serde_json::from_slice(&output.stdout).expect("json envelope");
    (out, envelope)
}

#[test]
fn issue_7218_offset_zero_before_outline_heading_does_not_leave_empty_paragraph() {
    let src = scaffold_outline_doc();
    let before = paragraphs(&src);
    let target = index_of(&before, "Second");

    let (out, envelope) = page_break(&src, target, 0, "zero");
    let after = paragraphs(&out);

    assert_eq!(
        after.len(),
        before.len(),
        "문단 수가 바뀌었다 — 빈 문단이 생겼다"
    );
    assert!(
        after.iter().all(|p| !p.text.is_empty()),
        "빈 문단이 생겼다: {:?}",
        after.iter().map(|p| p.text.as_str()).collect::<Vec<_>>()
    );
    let second = &after[target];
    assert_eq!(second.text, "Second");
    assert!(second.page_break, "`Second` 문단에 쪽 나눔이 걸려야 한다");
    assert_eq!(
        second.para_shape_id, before[target].para_shape_id,
        "문단 모양(개요 수준)이 그대로여야 한다"
    );
    assert_eq!(
        after.iter().filter(|p| p.page_break).count(),
        1,
        "쪽 나눔은 대상 문단 하나에만 걸린다"
    );
    assert_eq!(envelope["paragraphDelta"], 0);
    assert_eq!(envelope["pageBreakParagraph"], target);

    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn issue_7218_repeated_offset_zero_does_not_accumulate() {
    let src = scaffold_outline_doc();
    let target = index_of(&paragraphs(&src), "Second");
    let (once, _) = page_break(&src, target, 0, "once");
    let (twice, envelope) = page_break(&once, target, 0, "twice");

    let first = paragraphs(&once);
    let second = paragraphs(&twice);
    assert_eq!(second.len(), first.len(), "반복 호출로 문단이 늘었다");
    assert!(second[target].page_break);
    assert_eq!(second.iter().filter(|p| p.page_break).count(), 1);
    assert_eq!(envelope["paragraphDelta"], 0);

    for path in [&src, &once, &twice] {
        let _ = std::fs::remove_file(path);
    }
}

#[test]
fn issue_7218_mid_paragraph_offset_still_splits() {
    let src = scaffold_outline_doc();
    let before = paragraphs(&src);
    let target = index_of(&before, "Second");

    let (out, envelope) = page_break(&src, target, 3, "mid");
    let after = paragraphs(&out);

    assert_eq!(after.len(), before.len() + 1, "중간 오프셋은 문단을 가른다");
    assert_eq!(after[target].text, "Sec");
    assert!(!after[target].page_break);
    assert_eq!(after[target + 1].text, "ond");
    assert!(after[target + 1].page_break);
    assert_eq!(envelope["paragraphDelta"], 1);
    assert_eq!(envelope["pageBreakParagraph"], target + 1);

    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn issue_7218_cli_section_start_sets_explicit_break_and_keeps_section_properties() {
    let src = Path::new("samples/issue7218/outline_headings.hwpx");
    let before = paragraphs(src);
    let (out, envelope) = page_break(src, 0, 0, "section-start");
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(std::fs::read(&out).unwrap())).unwrap();
    let mut xml = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("Contents/section0.xml").unwrap(),
        &mut xml,
    )
    .unwrap();
    let first = xml
        .split("<hp:p ")
        .nth(1)
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(first.contains("pageBreak=\"1\""));
    assert!(xml.contains("<hp:secPr"));
    assert_eq!(paragraphs(&out).len(), before.len());
    assert_eq!(envelope["paragraphDelta"], 0);
    assert_eq!(envelope["pageBreakParagraph"], 0);
    std::fs::remove_file(out).unwrap();
}

// #7238의 독립 코어 계약을 CLI 검사와 함께 보존한다.
mod core_contract {
    //! 속성 setter는 문단을 보존하며 break-before 저장 비트와 synthesized 표시를 갱신한다.
    //! 사용자 BreakPage 명령은 별도 분할 계약이다. COM 11.0.0.9136에서 첫 문단 시작의
    //! BreakPage는 빈 선행 문단과 새 쪽을 만들었다. 저장 속성의 의미만으로 명령을 바꾸지 않는다.

    #![cfg(not(target_arch = "wasm32"))]

    use rhwp::document_core::DocumentCore;
    use rhwp::model::paragraph::ColumnBreakType;
    use rhwp::scaffold::{build_scaffold, ScaffoldSpec};

    const SPEC: &str = r#"{"version":"1","title":"repro","blocks":[
     {"type":"heading","level":1,"text":"First"},
     {"type":"paragraph","text":"body 1"},
     {"type":"heading","level":1,"text":"Second"},
     {"type":"paragraph","text":"body 2"}
    ]}"#;

    /// 제목 문단 + 4블록 = 5문단. 문단 3 이 두 번째 개요 제목 `Second` 다.
    const HEADING_PARA: usize = 3;

    fn core() -> DocumentCore {
        let spec: ScaffoldSpec = serde_json::from_str(SPEC).expect("scaffold spec");
        let bytes = rhwp::serializer::serialize_hwpx(&build_scaffold(&spec)).expect("HWPX 직렬화");
        DocumentCore::from_bytes(&bytes).expect("문서 로드")
    }

    fn paragraph_texts(core: &DocumentCore) -> Vec<String> {
        core.document().sections[0]
            .paragraphs
            .iter()
            .map(|p| p.text.clone())
            .collect()
    }

    /// offset 0 은 문단을 가르지 않고 그 문단에 break-before 속성만 준다.
    #[test]
    fn a_break_at_paragraph_start_does_not_split_the_paragraph() {
        let mut core = core();
        let before = paragraph_texts(&core);
        let before_shape = core.document().sections[0].paragraphs[HEADING_PARA].para_shape_id;

        core.mark_page_break_at_paragraph_start_native(0, HEADING_PARA)
            .expect("쪽 나눔 삽입");

        let after = paragraph_texts(&core);
        assert_eq!(
            after, before,
            "문단 수·텍스트가 변하면 안 된다 — 수정 전에는 개요 서식을 물려받은 빈 문단이 \
             {HEADING_PARA}번에 생겨 문단이 하나 늘었다",
        );

        let para = &core.document().sections[0].paragraphs[HEADING_PARA];
        assert_eq!(
            para.column_type,
            ColumnBreakType::Page,
            "대상 문단이 쪽 나눔을 가져야 한다",
        );
        assert_eq!(
            para.raw_break_type & 0x04,
            0x04,
            "HWP5 문단 헤더의 쪽 나눔 비트가 켜져야 한다",
        );
        assert_eq!(
            para.para_shape_id, before_shape,
            "대상 문단의 문단모양(개요 수준)은 그대로여야 한다",
        );
        assert!(
            !para.page_break_synthesized,
            "사용자가 명시한 쪽 나눔은 합성 표시를 남기면 안 된다 — 남으면 HWP5 저장기가 \
             이 바이트를 버린다",
        );
    }

    /// 다른 축의 break 비트를 지우지 않는다 — 구역 시작 문단에 적용해도 0x01 이 남는다.
    #[test]
    fn a_break_at_paragraph_start_keeps_the_other_break_axes() {
        let mut core = core();
        // 구역 시작 문단의 저장 계약을 재현한다(구역 나누기 비트 0x01).
        core.document_mut().sections[0].paragraphs[0].raw_break_type = 0x01;

        core.mark_page_break_at_paragraph_start_native(0, 0)
            .expect("쪽 나눔 삽입");

        let raw = core.document().sections[0].paragraphs[0].raw_break_type;
        assert_eq!(
            raw & 0x01,
            0x01,
            "구역 나누기 비트가 사라졌다(raw=0x{raw:02X}) — 덮어쓰기 대신 bitwise 합성이어야 한다",
        );
        assert_eq!(raw & 0x04, 0x04, "쪽 나눔 비트도 함께 켜져야 한다");
    }

    /// 반복 호출해도 문단이 누적되지 않는다.
    #[test]
    fn repeating_the_break_at_paragraph_start_is_idempotent() {
        let mut core = core();
        let before = paragraph_texts(&core);

        for _ in 0..3 {
            core.mark_page_break_at_paragraph_start_native(0, HEADING_PARA)
                .expect("쪽 나눔 삽입");
        }

        assert_eq!(
            paragraph_texts(&core),
            before,
            "반복 호출이 빈 문단을 누적하면 안 된다",
        );
        assert_eq!(
            core.document().sections[0].paragraphs[HEADING_PARA].column_type,
            ColumnBreakType::Page,
        );
    }

    /// 문단 중간 오프셋은 종전처럼 분할한다 — 좁힌 자리는 offset 0 뿐이다.
    #[test]
    fn a_break_inside_a_paragraph_still_splits_it() {
        let mut core = core();
        let before = paragraph_texts(&core);

        core.insert_page_break_native(0, HEADING_PARA, 3)
            .expect("쪽 나눔 삽입");

        let after = paragraph_texts(&core);
        assert_eq!(
            after.len(),
            before.len() + 1,
            "중간 오프셋은 문단을 하나 늘려야 한다",
        );
        assert_eq!(
            after[HEADING_PARA], "Sec",
            "앞 조각은 오프셋 앞 글자를 갖는다"
        );
        assert_eq!(after[HEADING_PARA + 1], "ond", "뒤 조각이 나머지를 갖는다");
        assert_eq!(
            core.document().sections[0].paragraphs[HEADING_PARA + 1].column_type,
            ColumnBreakType::Page,
            "쪽 나눔은 뒤 조각에 붙는다",
        );
        assert_eq!(
            core.document().sections[0].paragraphs[HEADING_PARA].column_type,
            ColumnBreakType::None,
            "앞 조각은 쪽 나눔을 갖지 않는다",
        );
    }

    #[test]
    fn explicit_break_clears_synthesized_flag_and_survives_both_formats() {
        let mut core = core();
        let p = &mut core.document_mut().sections[0].paragraphs[HEADING_PARA];
        p.column_type = ColumnBreakType::Page;
        p.raw_break_type = 0;
        p.page_break_synthesized = true;
        assert!(core
            .mark_page_break_at_paragraph_start_native(0, HEADING_PARA)
            .unwrap());
        assert!(!core
            .mark_page_break_at_paragraph_start_native(0, HEADING_PARA)
            .unwrap());
        for bytes in [
            rhwp::serializer::serialize_hwpx(core.document()).unwrap(),
            rhwp::serializer::serialize_document(core.document()).unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            let p = &reopened.document().sections[0].paragraphs[HEADING_PARA];
            assert_eq!(p.raw_break_type & 4, 4);
            assert!(!p.page_break_synthesized);
            assert_eq!(p.text, "Second");
        }
    }

    #[test]
    fn setting_break_preserves_all_other_raw_axes_through_hwp5_save() {
        for axis in [0x01, 0x02, 0x08, 0x03] {
            let mut core = core();
            core.document_mut().sections[0].paragraphs[0].raw_break_type = axis;
            core.mark_page_break_at_paragraph_start_native(0, 0)
                .unwrap();
            let bytes = rhwp::serializer::serialize_document(core.document()).unwrap();
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(
                reopened.document().sections[0].paragraphs[0].raw_break_type & (axis | 4),
                axis | 4
            );
        }
    }

    #[test]
    fn user_break_command_still_splits_at_start_and_repeats() {
        let mut core = core();
        let before = paragraph_texts(&core);
        let result = core.insert_page_break_native(0, HEADING_PARA, 0).unwrap();
        let cursor: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(cursor["paraIdx"], HEADING_PARA + 1);
        assert_eq!(cursor["charOffset"], 0);
        assert_eq!(paragraph_texts(&core).len(), before.len() + 1);
        assert_eq!(paragraph_texts(&core)[HEADING_PARA], "");
        assert_eq!(paragraph_texts(&core)[HEADING_PARA + 1], "Second");
        core.insert_page_break_native(0, HEADING_PARA + 1, 0)
            .unwrap();
        assert_eq!(paragraph_texts(&core).len(), before.len() + 2);
        assert_eq!(paragraph_texts(&core)[HEADING_PARA + 2], "Second");
    }
}
