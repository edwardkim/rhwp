//! [Issue #5877] 쪽 넘어온 표 조각의 세로 괘선을 균등 격자에 찍어 셀과 어긋난다
//! (2961515 점검 총괄표, 150행×32열, 9쪽).
//!
//! 근인: `h_edges`/`v_edges`/`grid_row_y` 는 **조각-지역 행 인덱스**(`render_rows`
//! 순서)인데 `row_col_x` 는 **원본 행 전체**다. 테두리 렌더러는 그 인덱스로
//! `row_col_x` 를 직접 참조하므로, 조각이 원본 행 18 부터 시작해도 조각-지역
//! 0·1 이 원본 행 0·1 의 격자를 집는다. 그 두 행은 전폭 단일 셀(제목 행)이라
//! `declared_row_col_x` 가 내부 열 경계를 **균등 보간**한다 — 표폭/열수 = 21.13px
//! 간격의 유령 세로 괘선이 조각 상단에만 그려졌다(셀 좌표는 정상).
//!
//! 수정: `render_rows` 순서에 맞춘 조각용 격자를 만들어 테두리 렌더러에 넘긴다.
//! 15쪽 조각 상단 x 실측: 37.8/58.9/105.4/380.1/425/439.3/464.1/481.6/511/523.9/
//! 566.1/714.1(두 벌) → 37.8/105.4/380.1/425/464.1/511/714.1(한 벌).
//! 한글 2022 14쪽 같은 내용: 39.7/107.8/382.0/424.1/463.6/510.8/715.6.
//! 10k 쪽수 A/B 변동 0 (테두리 좌표만 바뀐다).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5877/fragment_ghost_vrules.hwp";
/// [#7063] 표 전체가 자기 `outMargin.left`(141HU=1.88px)만큼 오른쪽으로 갔다. 아래 두
/// 상수는 그만큼 옮긴 값이다. 한/글 2022 대비로 보면 **표의 양끝은 좋아지고 가운데는
/// 나빠진다** — 두 오류가 서로 상쇄하고 있었다는 뜻이다.
///
/// ```text
///          왼끝    2번째   3번째    4번째    5번째    6번째   오른끝
///   한/글  39.7   107.8   382.0   424.1   463.6   510.8   715.6
///   수정전 37.8   105.4   380.1   425.0   464.1   511.0   714.1   (−1.9 −2.4 −1.9 +0.9 +0.5 +0.2 −1.5)
///   수정후 39.67  107.30  381.97  426.89  465.97  512.87  715.95  (−0.03 −0.50 −0.03 +2.79 +2.37 +2.07 +0.35)
/// ```
///
/// 곧 표 원점은 정본에 붙었고(±0.5px 안), 남은 것은 **가운데 열 폭**이 한/글보다 누적
/// 2px 넓은 별개의 축이다. 이 시험은 그 축을 다루지 않는다(유령 격자 부재가 표적이다).
///
/// 유령 격자(표폭/32 = 21.13px 간격)에만 나타나는 x — 실제 칸 경계가 아니다.
const GHOST_X: [f64; 5] = [60.8, 441.2, 483.5, 525.8, 568.0];
/// 실제 칸 경계(조각 상단·아래 공통).
const REAL_X: [f64; 4] = [107.3, 382.0, 466.0, 512.9];

#[test]
fn issue_5877_continuation_fragment_uses_its_own_column_grid() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(14).expect("page 15 svg");

    // 조각 상단 밴드(y < 289)의 세로 괘선 x 를 모은다.
    let mut xs: Vec<f64> = Vec::new();
    for cap in svg.split("<line ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let (Some(x1), Some(x2), Some(y1)) = (
            attr(head, "x1=\""),
            attr(head, "x2=\""),
            attr(head, "y1=\""),
        ) else {
            continue;
        };
        if (x1 - x2).abs() < 0.1 && y1 < 289.0 {
            xs.push(x1);
        }
    }
    assert!(
        !xs.is_empty(),
        "조각 상단 세로 괘선을 찾아야 검증이 유효하다"
    );

    for ghost in GHOST_X {
        assert!(
            !xs.iter().any(|x| (x - ghost).abs() < 1.0),
            "균등 격자 유령 괘선이 남아 있다 (x≈{ghost}): {xs:?}"
        );
    }
    for real in REAL_X {
        assert!(
            xs.iter().any(|x| (x - real).abs() < 1.0),
            "실제 칸 경계 괘선(x≈{real})이 있어야 한다: {xs:?}"
        );
    }
}

fn attr(head: &str, key: &str) -> Option<f64> {
    let rest = head.split_once(key)?.1;
    rest[..rest.find('"')?].parse().ok()
}
