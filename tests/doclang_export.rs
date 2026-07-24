//! `rhwp::doclang` (export-doclang) 통합 테스트.
//!
//! rhwp 자체 샘플 코퍼스(`samples/`)의 실제 HWP5 / HWPX 문서를 라이브러리
//! API(`rhwp::doclang::convert`)로 DocLang v0.6 XML 로 변환하고, 산출물이
//! well-formed XML 이며 기대한 의미 요소를 담는지 검증한다.
//!
//! 골든 스냅샷은 쓰지 않는다 — 파서/라이터 세부가 바뀌어도 깨지지 않도록
//! "루트 버전 속성", "알려진 텍스트 런", "표 마크업 존재" 처럼 회복 탄력적인
//! 최소 불변식만 확인한다.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rhwp::doclang::{convert, ConvertOptions};

/// `samples/` 아래 상대 경로의 절대 경로.
fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(rel)
}

fn unique_temp_path(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rhwp_doclang_{label}_{}_{nonce}",
        std::process::id()
    ))
}

fn run_export_doclang(input: &Path, output: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-doclang")
        .arg(input)
        .arg("-o")
        .arg(output)
        .output()
        .expect("run rhwp export-doclang")
}

/// 샘플을 읽어 기본 옵션(Lean·인라인 자원)으로 DocLang XML 로 변환한다.
fn convert_sample(rel: &str) -> String {
    let path = sample(rel);
    let data = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let outcome = convert(&data, &ConvertOptions::default())
        .unwrap_or_else(|e| panic!("convert {}: {}", path.display(), e));
    outcome.xml
}

/// quick-xml 로 문서를 끝까지 파싱해 well-formed 임을 확인하고, 첫 요소가
/// `<doclang version="0.6">` 인지 검증한다.
fn assert_wellformed_doclang_root(xml: &str) {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut root_checked = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(e)) if !root_checked => {
                assert_eq!(
                    e.name().as_ref(),
                    b"doclang",
                    "root element must be <doclang>"
                );
                let mut version: Option<String> = None;
                for attr in e.attributes() {
                    let attr = attr.expect("parse root attribute");
                    if attr.key.as_ref() == b"version" {
                        version = Some(String::from_utf8_lossy(&attr.value).into_owned());
                    }
                }
                assert_eq!(
                    version.as_deref(),
                    Some("0.6"),
                    "root <doclang> must carry version=\"0.6\""
                );
                root_checked = true;
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {}
            Err(e) => panic!("DocLang output is not well-formed XML: {}", e),
        }
        buf.clear();
    }

    assert!(root_checked, "no root element found in DocLang output");
}

#[test]
fn hwp_paragraph_sample_converts_to_wellformed_doclang() {
    // para-001.hwp: 문단 + 인라인 런(한글/한자 혼용).
    let xml = convert_sample("para-001.hwp");
    assert_wellformed_doclang_root(&xml);

    assert!(xml.contains("<text>"), "no <text> element emitted");
    // 알려진 도입 텍스트 런이 그대로 나타나야 한다.
    assert!(
        xml.contains("오호라"),
        "known opening run '오호라' missing from output"
    );
    // 한글/한자 혼용 문서임을 확인 — 한자 '乾坤' 이 보존되어야 한다.
    assert!(
        xml.contains("乾坤"),
        "mixed-Hanja run '乾坤' missing from output"
    );
}

#[test]
fn hwp_table_sample_emits_table_markup() {
    // table-001.hwp: 표(헤더셀·병합)를 OTSL 마크업으로 내보낸다.
    let xml = convert_sample("table-001.hwp");
    assert_wellformed_doclang_root(&xml);

    assert!(xml.contains("<table>"), "no <table> element emitted");
    // OTSL 셀 어휘: 첫 셀 <fcel/> 과 행 종료 <nl/> 이 있어야 한다.
    assert!(xml.contains("<fcel/>"), "no <fcel/> table cell emitted");
    assert!(xml.contains("<nl/>"), "no <nl/> row terminator emitted");
    // 표 안의 알려진 셀 텍스트.
    assert!(
        xml.contains("품질관리협의체 운영계획 수립"),
        "known table cell text missing from output"
    );
}

#[test]
fn hwpx_paragraph_sample_converts() {
    // hwpx/para-001.hwpx: 동일 내용의 HWPX(zip+xml) 판도 변환되어야 한다.
    let xml = convert_sample("hwpx/para-001.hwpx");
    assert_wellformed_doclang_root(&xml);
    assert!(
        xml.contains("오호라"),
        "hwpx conversion missing known text run"
    );
}

#[cfg(unix)]
#[test]
fn export_doclang_never_overwrites_a_symlink_to_its_input() {
    let input = unique_temp_path("cli_symlink_input").with_extension("hwp");
    let output = unique_temp_path("cli_symlink_output").with_extension("xml");
    let original = include_bytes!("../samples/para-001.hwp");
    std::fs::write(&input, original).expect("write HWP input");
    std::os::unix::fs::symlink(&input, &output).expect("create symlink output alias");

    let command = run_export_doclang(&input, &output);

    assert_eq!(command.status.code(), Some(2));
    assert_eq!(
        std::fs::read(&input).expect("read unchanged input"),
        original
    );
}

#[cfg(unix)]
#[test]
fn export_doclang_never_overwrites_a_hard_link_to_its_input() {
    let input = unique_temp_path("cli_hard_link_input").with_extension("hwp");
    let output = unique_temp_path("cli_hard_link_output").with_extension("xml");
    let original = include_bytes!("../samples/para-001.hwp");
    std::fs::write(&input, original).expect("write HWP input");
    std::fs::hard_link(&input, &output).expect("create hard-link output alias");

    let command = run_export_doclang(&input, &output);

    assert_eq!(command.status.code(), Some(2));
    assert_eq!(
        std::fs::read(&input).expect("read unchanged input"),
        original
    );
}
