//! RowBreak 표에서 앞 행의 병합 칸이 덮는 행이 쪽 끝에 걸릴 때, 그 행의 글줄과 안 여백이
//! 남은 높이에 다 들어가면 한/글은 그 행을 이 쪽에 남은 높이로 끝내고 다음 쪽은 다음 행에서
//! 시작한다. 남는 빈 띠의 크기는 조건이 아니다.
//!
//! 입력 `samples/rowspan-band-fit/rowspan-band-fit.hwp` 는 합성 HWPX(열0 이 행1~4 를 병합한
//! 8×3 RowBreak 표, 쪽 끝 남은 높이 1280/1480 HU)를 한/글 2020 으로 열어 HWP 로 저장한 원본이다.
//! 정답은 같은 저장본의 한/글 2020 Print PDF(`pdf/rowspan-band-fit/rowspan-band-fit-2020.pdf`)다.
//! 남은 높이 1480 HU 에서 내용+안 여백(17.1px)은 남은 18.4px 에 들어가지만 빈 띠는 1.3px 뿐이다.

#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::path::Path;
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned())
}

fn pages() -> Value {
    let input =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/rowspan-band-fit/rowspan-band-fit.hwp");
    assert!(input.exists(), "fixture missing: {}", input.display());
    let out = Command::new(rhwp_bin())
        .arg("dump-pages")
        .arg(&input)
        .arg("--json")
        .output()
        .expect("run rhwp dump-pages");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("dump-pages json")
}

/// (para, startRow, endRow, startCut 비었나) — 쪽 순서대로.
fn table_fragments(doc: &Value) -> Vec<(u64, u64, u64, bool)> {
    let mut out = Vec::new();
    for page in doc["pages"].as_array().unwrap() {
        for column in page["columns"].as_array().unwrap() {
            for item in column["items"].as_array().unwrap() {
                if item["kind"] == "partialTable" || item["kind"] == "table" {
                    out.push((
                        item["paraIndex"].as_u64().unwrap(),
                        item["startRow"].as_u64().unwrap_or(0),
                        item["endRow"].as_u64().unwrap_or(0),
                        item["startCut"].as_array().is_none_or(|cut| cut.is_empty()),
                    ));
                }
            }
        }
    }
    out
}

#[test]
fn covered_row_that_fits_its_content_ends_on_the_current_page() {
    let doc = pages();
    // 한/글 2020 정본도 9쪽(제목 1쪽 + 표 네 개가 각각 두 쪽).
    assert_eq!(doc["pageCount"], 9);
    let fragments = table_fragments(&doc);
    // 정본의 첫 조각 마지막 행: 병합 앞 2행 쪽 끝(1280 → 행1까지, 1480 → 행2까지),
    // 병합 셋째 행 쪽 끝(1280 → 행2까지, 1480 → 행3까지).
    let expected_first_end = [(1u64, 2u64), (2, 3), (3, 3), (4, 4)];
    for (para, end) in expected_first_end {
        let mine: Vec<_> = fragments.iter().filter(|f| f.0 == para).collect();
        assert_eq!(mine.len(), 2, "para {para}: {mine:?}");
        assert_eq!(
            (mine[0].1, mine[0].2),
            (0, end),
            "para {para} first: {mine:?}"
        );
        // 다음 쪽은 다음 행에서, 이월 컷 없이 시작한다 — 끝낸 행을 다시 그리지 않는다.
        assert_eq!(
            (mine[1].1, mine[1].2),
            (end, 8),
            "para {para} next: {mine:?}"
        );
        assert!(
            mine[1].3,
            "para {para} continuation must not resume a cut: {mine:?}"
        );
    }
}
