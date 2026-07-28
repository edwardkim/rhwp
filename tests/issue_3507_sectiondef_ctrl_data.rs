//! [#3507] HWP5 SectionDef CTRL_DATA 중복 직렬화 회귀 테스트.
//!
//! `samples/복학원서.hwp`의 표 셀을 실제 CLI 경로로 편집하면 Section raw passthrough가
//! 무효화된다. 이때 원본 SectionDef의 직접 자식 CTRL_DATA가 동일 payload로 두 번
//! materialize되면 한컴이 저장본을 파일 손상으로 거부한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rhwp::parser::cfb_reader::decompress_stream;
use rhwp::parser::{record::Record, tags};

const SAMPLE: &str = "samples/복학원서.hwp";

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_out() -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-issue-3507-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn section0_records(bytes: &[u8]) -> Vec<Record> {
    let cursor = Cursor::new(bytes);
    let mut compound = cfb::CompoundFile::open(cursor).expect("CFB 열기");
    let mut stream = compound
        .open_stream("/BodyText/Section0")
        .expect("BodyText/Section0 열기");
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).expect("Section0 읽기");
    let section = decompress_stream(&raw).unwrap_or(raw);
    Record::read_all(&section).expect("Section0 record 파싱")
}

fn section_def_direct_ctrl_data(records: &[Record]) -> Vec<Vec<u8>> {
    let section_def_pos = records
        .iter()
        .position(|record| {
            record.tag_id == tags::HWPTAG_CTRL_HEADER
                && record
                    .data
                    .starts_with(&tags::CTRL_SECTION_DEF.to_le_bytes())
        })
        .expect("SectionDef CTRL_HEADER");
    let section_def_level = records[section_def_pos].level;

    records[section_def_pos + 1..]
        .iter()
        .take_while(|record| record.level > section_def_level)
        .filter(|record| {
            record.tag_id == tags::HWPTAG_CTRL_DATA
                && record.level == section_def_level.saturating_add(1)
        })
        .map(|record| record.data.clone())
        .collect()
}

#[test]
fn edited_bokhak_form_keeps_one_section_def_ctrl_data_with_original_payload() {
    let sample = sample_path();
    let original = std::fs::read(&sample).expect("복학원서 원본 읽기");
    let original_ctrl_data = section_def_direct_ctrl_data(&section0_records(&original));
    assert_eq!(
        original_ctrl_data.len(),
        1,
        "원본 SectionDef CTRL_DATA 개수"
    );
    assert_eq!(
        original_ctrl_data[0].len(),
        280,
        "원본 SectionDef CTRL_DATA payload 크기"
    );

    let out = temp_out();
    let args = [
        "edit",
        "set-cell",
        sample.to_str().unwrap(),
        "--table",
        "0",
        "--row",
        "1",
        "--col",
        "1",
        "--text",
        "강남대학교",
        "--keep-style",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let saved = std::fs::read(&out).expect("편집 저장본 읽기");
    let saved_ctrl_data = section_def_direct_ctrl_data(&section0_records(&saved));
    assert_eq!(
        saved_ctrl_data.len(),
        1,
        "편집 저장본은 SectionDef 직접 자식 CTRL_DATA를 한 번만 출력해야 한다"
    );
    assert_eq!(
        saved_ctrl_data, original_ctrl_data,
        "편집과 무관한 SectionDef CTRL_DATA payload는 원본과 같아야 한다"
    );

    let verify_args = ["export-tables", out.to_str().unwrap(), "--json"];
    let verify = run(&verify_args);
    assert_eq!(
        verify.status.code(),
        Some(0),
        "{}",
        describe(&verify_args, &verify)
    );
    let tables: serde_json::Value =
        serde_json::from_slice(&verify.stdout).expect("export-tables JSON");
    let edited_cell = tables["tables"]
        .as_array()
        .expect("tables")
        .iter()
        .find(|table| table["index"].as_u64() == Some(0))
        .and_then(|table| table["cells"].as_array())
        .and_then(|cells| {
            cells
                .iter()
                .find(|cell| cell["row"].as_u64() == Some(1) && cell["col"].as_u64() == Some(1))
        })
        .expect("편집 대상 셀");
    assert_eq!(edited_cell["text"], "강남대학교");

    let _ = std::fs::remove_file(out);
}
