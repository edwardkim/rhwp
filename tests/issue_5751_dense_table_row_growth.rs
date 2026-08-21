//! Issue #5751: 조밀한 표의 행을 내용에 맞춰 안 늘려 셀 글자가 아래 괘선을 넘는다.
//!
//! `#501` 은 "안 여백이 셀 선언 높이를 넘는 비정상 저장값" 을 막는 한컴 방어 로직
//! 모방인데, 그 발동 기준이 측정과 렌더에서 갈려 있었다.
//!
//! | | 파일 | 임계 | 동작 |
//! |---|---|---|---|
//! | 측정 | `height_measurer` | `pad > cell_h * 0.5` | 행 높이를 선언 높이로 clamp |
//! | 렌더 | `layout/table_layout` | `pad >= cell_h` | padding 을 `cell_h*0.5` 로 축소 |
//!
//! 여백이 셀 높이의 **절반~1배** 인 정상 조밀 표는 측정 기준만 넘어서, 측정은 행을
//! 안 늘리는데 렌더는 저장 여백을 그대로 쓴다. 그래서 글자가 칸 밖으로 나간다.
//!
//! Regression shape (`samples/2026_oss_rst.hwp` s0:pi=7 12×2 표, 행 5 = 셀[9]):
//! - 선언 `h=1810 HU`(24.13px), `aim=false` 라 표 기본 여백 상하 `510+510=1020 HU`
//!   (13.60px)가 적용되고 내용은 한 줄 `1000 HU` 다.
//! - 필요 높이 = `510 + 1000 + 510 = 2020 HU` = 26.93px > 선언 24.13px.
//! - 수정 전: `13.60 > 24.13*0.5 = 12.07` 이라 가드가 발동해 측정이 24.13px 로 묶였다.
//!   렌더 컷 회계는 26.9px 를 쓰고 있어서 `RHWP_TABLE_DRIFT` 의 `TABLE_CUT_DRIFT` 가
//!   `diff=-2.8` 로 그 불일치를 그대로 보고했다.
//! - 수정 후: `13.60 < 24.13` 이라 미발동 → 26.93px 로 측정·렌더가 일치(`diff=+0.0`).
//!
//! 표적 문서(`156505020_…대입전형시행계획 주요사항.hwp` 마지막 쪽)는 같은 형상이
//! 크게 나타난 사례다 — 한글 2022 는 행을 32.46px 로 그리는데 rhwp 는 21.09px 로
//! 눌러 글자 baseline 이 아래 괘선보다 1.19px 아래에 놓였다. 코퍼스가 필요해 여기서는
//! 저장소에 있는 같은 형상의 최소 사례로 고정한다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/2026_oss_rst.hwp";

/// 표 `pi` 의 행 `row` 에 속한 셀 높이(px).
fn find_row_cell_height(node: &serde_json::Value, pi: u64, row: u64) -> Option<f64> {
    if node.get("type").and_then(|t| t.as_str()) == Some("Table")
        && node.get("pi").and_then(|p| p.as_u64()) == Some(pi)
    {
        for c in node.get("children")?.as_array()? {
            if c.get("type").and_then(|t| t.as_str()) == Some("Cell")
                && c.get("row").and_then(|r| r.as_u64()) == Some(row)
            {
                return c.get("bbox")?.get("h")?.as_f64();
            }
        }
    }
    for c in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        if let Some(h) = find_row_cell_height(c, pi, row) {
            return Some(h);
        }
    }
    None
}

#[test]
fn issue_5751_dense_row_grows_to_content_plus_padding() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    // s0:pi=7 표는 2쪽(index 1)에서 행 0~8 을 그린다.
    let json = doc.get_page_render_tree(1).expect("render tree page 2");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");
    let h = find_row_cell_height(&tree, 7, 5).expect("s0:pi=7 표의 행 5 셀");

    // 510 + 1000 + 510 = 2020 HU = 26.93px. 선언 1810 HU(24.13px)로 눌리면 회귀.
    assert!(
        (h - 26.93).abs() < 0.5,
        "행 5 는 내용+여백(2,020 HU = 26.93px)만큼 늘어야 한다 — 선언 1,810 HU\
         (24.13px)로 눌렸다면 #501 가드가 정상 조밀 표에서 오발동한 것이다, got {h}"
    );
}

#[test]
fn issue_5751_guard_threshold_is_declared_height_itself() {
    use rhwp::model::table::Cell;

    let px = |hu: i32| hu as f64 * 96.0 / 7200.0;

    // #501 원 사례 (mel-001 p2 셀[21]) — 여백이 셀 높이 자체를 넘는다: 발동.
    assert!(
        Cell::vertical_padding_is_abnormal(px(1280), px(3400)),
        "여백 3,400 HU > 셀 1,280 HU 는 비정상으로 판정해야 한다 — #501 가드 회귀"
    );

    // 2026_oss_rst s0:pi=7 행 5 — 여백이 셀 높이의 절반~1배: 미발동.
    assert!(
        !Cell::vertical_padding_is_abnormal(px(1810), px(1020)),
        "여백 1,020 HU 는 셀 1,810 HU 안이라 정상이다 — #5751 오발동"
    );

    // 156505020 데이터 셀 — 같은 구간(절반 초과, 1배 미만): 미발동.
    assert!(
        !Cell::vertical_padding_is_abnormal(px(1582), px(1132)),
        "여백 1,132 HU 는 셀 1,582 HU 안이라 정상이다 — #5751 오발동"
    );

    // 선언 높이 결측(0)은 판정 대상이 아니다.
    assert!(!Cell::vertical_padding_is_abnormal(0.0, 100.0));
}
