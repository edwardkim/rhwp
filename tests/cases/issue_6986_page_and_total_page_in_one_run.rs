//! [Issue #6986] 한 문단(같은 런)에 `PAGE` 와 `TOTAL_PAGE` 가 나란히 있으면 한쪽이
//! 빈칸으로 렌더된다 — v0.8.3 회귀.
//!
//! 제보(국가법령정보센터 법령 HWPX)의 꼬리말 표 셀이 이 형상이다.
//!
//! ```xml
//! <hp:t>- </hp:t><hp:ctrl><hp:autoNum numType="PAGE"/></hp:ctrl>
//! <hp:t> / </hp:t><hp:ctrl><hp:autoNum numType="TOTAL_PAGE"/></hp:ctrl>
//! <hp:t> -</hp:t>
//! ```
//!
//! 제보자의 버전 대조에서 **두 필드가 동시에 맞은 버전이 없었다** — v0.7.x 는
//! `TOTAL_PAGE` 가 `PAGE` 값을 따라가고, v0.8.0~v0.8.2 는 `TOTAL_PAGE` 가 빈칸,
//! v0.8.3 부터는 반대로 `PAGE` 가 빈칸(`- / 187 -`)이다.
//!
//! 근인은 둘이다.
//!
//! 1. `replace_composed_char_with_display` 가 호출마다 `display_text` 를 `run.text`
//!    에서 **새로 만들었다.** 같은 런에 치환 자리가 둘이면 뒤 치환이 앞 치환을 통째로
//!    버린다 — 이것이 `- / N -` 의 직접 원인이다. 치환을 모아 한 번에 재구성한다.
//! 2. `auto_number_placeholder_positions` 가 종류가 다른 `AutoNumber` 를 건너뛰면서
//!    `search_from` 을 전진시키지 않아, 두 종류가 **같은 자리**를 가리켰다
//!    (단위 실측: `page=[2] total=[2]` → 수정 후 `page=[2] total=[6]`).
//!
//! v0.8.3 의 `e69a2d286`(꼬리말 총쪽수 치환 도입)이 ②의 조건을 만들었고, ①은 그때
//! 처음 두 번 호출되면서 드러났다.
//!
//! 픽스처는 `samples/issue5590_per_row_column_widths.hwpx` 의 첫 칸 문단을 제보 구조로
//! 바꾼 것이다(1쪽 문서라 기대값은 `- 1 / 1 -`).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6986/cell-page-and-total-page-in-one-run.hwpx";

#[test]
fn issue_6986_page_and_total_page_both_render() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(core.page_count(), 1, "1쪽 픽스처다");

    let svg = core.render_page_svg_native(0).expect("page 1 svg");
    let first_line = first_text_line(&svg);

    // 결함 시 `- / 1 -` (PAGE 빈칸). 종전 v0.8.2 이하는 `- 1 / -`.
    assert!(
        first_line.starts_with("-1/1-"),
        "첫 칸이 '- 1 / 1 -' 로 렌더돼야 한다 (PAGE·TOTAL_PAGE 동시): {first_line:?}"
    );
}

/// SVG 의 첫 글줄을 x 순으로 이어붙인다.
///
/// ⚠ 이 렌더러는 **글자마다 `<text>` 를 따로** 낸다 — 문자열 grep 은 항상 0 건이라
/// 존재 판정이 뒤집힌다.
fn first_text_line(svg: &str) -> String {
    let mut rows: Vec<(f64, f64, String)> = Vec::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(tag_end) = chunk.find('>') else {
            continue;
        };
        let (Some(x), Some(y)) = (attr(&chunk[..tag_end], "x"), attr(&chunk[..tag_end], "y"))
        else {
            continue;
        };
        let Some(close) = chunk[tag_end + 1..].find("</text>") else {
            continue;
        };
        rows.push((y, x, chunk[tag_end + 1..tag_end + 1 + close].to_string()));
    }
    let Some(top) = rows
        .iter()
        .map(|(y, _, _)| *y)
        .fold(None, |acc: Option<f64>, y| {
            Some(acc.map_or(y, |a| a.min(y)))
        })
    else {
        return String::new();
    };
    let mut line: Vec<(f64, String)> = rows
        .into_iter()
        .filter(|(y, _, _)| (*y - top).abs() < 0.5)
        .map(|(_, x, t)| (x, t))
        .collect();
    line.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("finite x"));
    line.into_iter().map(|(_, t)| t).collect()
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = head.find(&needle)? + needle.len();
    let rest = &head[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}
