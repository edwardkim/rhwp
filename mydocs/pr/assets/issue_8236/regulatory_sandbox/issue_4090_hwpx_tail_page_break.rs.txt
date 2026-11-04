//! Issue #4090: 저장 HWPX의 `vpos=0` tail 줄은 뒤 명시 쪽 나눔 앞에서 다음 쪽으로 간다.
//!
//! `156492236_규제샌드박스_min.hwpx`의 세 문단은 마지막 저장 줄 `vpos=0` 뒤에
//! 명시적 쪽 나눔이 있다. HWP 2020 MCP `PrintToPDFEx` 기준은 17쪽이며, rhwp는
//! 앞쪽 줄만 현재 쪽에 두고 tail 한 줄을 다음 쪽으로 넘겨야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

const SAMPLE: &str = "samples/issue4090/156492236_규제샌드박스_min.hwpx";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn dump_pages_json() -> Value {
    let sample = sample();
    let args = ["dump-pages", sample.to_str().unwrap(), "--json"];
    let output = Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패");
    parse_successful_json(&args, output)
}

fn parse_successful_json(args: &[&str], output: Output) -> Value {
    assert_eq!(
        output.status.code(),
        Some(0),
        "dump-pages 실패\n명령: rhwp {}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "dump-pages --json stdout 파싱 실패: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn line_ranges_on_page(dump: &Value, page_index: u64, para_index: u64) -> Vec<(u64, u64)> {
    dump["pages"]
        .as_array()
        .expect("pages는 배열")
        .iter()
        .find(|page| page["pageIndex"].as_u64() == Some(page_index))
        .unwrap_or_else(|| panic!("{}쪽을 찾지 못했다", page_index + 1))["columns"]
        .as_array()
        .expect("columns는 배열")
        .iter()
        .flat_map(|column| {
            column["items"]
                .as_array()
                .expect("column.items는 배열")
                .iter()
        })
        .filter(|item| {
            item["kind"].as_str() == Some("partialParagraph")
                && item["paraIndex"].as_u64() == Some(para_index)
        })
        .map(|item| {
            (
                item["startLine"].as_u64().expect("startLine은 정수"),
                item["endLine"].as_u64().expect("endLine은 정수"),
            )
        })
        .collect()
}

#[test]
fn issue_4090_hwpx_tail_lines_follow_the_explicit_page_break() {
    let dump = dump_pages_json();

    assert_eq!(
        dump["pageCount"].as_u64(),
        Some(17),
        "HWP 2020 MCP PrintToPDFEx 17쪽 기준과 달라졌다: {dump}"
    );

    for (para_index, before_page, before_range, after_page, after_range) in [
        (59, 4, (0, 1), 5, (1, 2)),
        (74, 6, (0, 2), 7, (2, 3)),
        (183, 14, (0, 2), 15, (2, 3)),
    ] {
        assert_eq!(
            line_ranges_on_page(&dump, before_page, para_index),
            vec![before_range],
            "pi={para_index}의 앞쪽 줄은 {}쪽에 {:?}로 남아야 한다",
            before_page + 1,
            before_range
        );
        assert_eq!(
            line_ranges_on_page(&dump, after_page, para_index),
            vec![after_range],
            "pi={para_index}의 vpos=0 tail은 명시 쪽 나눔 뒤 {}쪽에 {:?}로 와야 한다",
            after_page + 1,
            after_range
        );
    }
}
