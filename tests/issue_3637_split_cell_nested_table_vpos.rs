//! Issue #3637: 분할된 표 셀에서 중첩 표 뒤 문단이 셀 밖으로 밀려 사라진다.
//!
//! ## 근인
//!
//! `table_partial.rs` 는 중첩 표를 가진 문단을 배치한 뒤 **다음 문단의
//! `LINE_SEG.vertical_pos`** 로 `para_y` 를 밀어 원본의 세로 간격을 재현한다.
//! 그런데 `vertical_pos` 는 **셀 전체** 좌표라, 연속 조각(`start_cut` 이 큰
//! 조각)에서는 이미 앞 쪽들이 소비한 만큼의 원점이 빠져 있지 않다.
//!
//! 재현 문서 10쪽 실측 — 조각은 유닛 `[201..239]`, 셀 높이 957.9px 다.
//!
//! ```text
//! cp=164 (중첩 표 보유) 다음 문단의 vpos = 72846 HU = 971.3px
//! para_y = text_y_start(47.2) + 971.3 = 1018.5   ← 셀 바닥 1005.1 을 넘는다
//! 그 아래로 문단이 계속 쌓여 최대 287.6px 밖, 351글자가 쪽을 벗어난다
//! ```
//!
//! 벗어난 글자는 `export-text` 에도 SVG `<text>` 에도 남아 있어 **텍스트 diff 나
//! IR diff 로는 잡히지 않는다**. 좌표가 쪽 밖이라 안 보일 뿐이다
//! ([`issue_3637_para_topbottom_vpos_base`] 와 같은 계열의 다른 기전).
//!
//! ## 계약
//!
//! 조각 셀 안 문단은 셀 상자 안에 놓인다. 두 가지가 함께 필요하다.
//!
//! 1. 스냅 기준을 **이 조각에서 실제로 그려지는 첫 문단**의 vpos 로 옮긴다.
//! 2. 그래도 남는 셀 내부 vpos 도약에 대비해 셀 바닥으로 상한을 둔다.
//!
//! 1번만으로는 셀 안에 큰 vpos 도약이 있는 문서에서 여전히 밀려나고(선행 시도
//! 실측: 표 코호트 96→535자 악화), 2번만으로는 상한에서 멈춘 뒤 뒤 문단이 그
//! 아래로 쌓인다(같은 코호트 297.7→284.3px 로 부분 개선에 그침).
//!
//! ## 임계값 근거
//!
//! 표본을 수정 전후로 재서 그 사이에 둔다.
//!
//! ```text
//! 수정 전  넘친 쪽 3/13   최대 초과 287.6px   글자 351
//! 수정 후  넘친 쪽 1/13   최대 초과   3.5px   글자  10
//! ```
//!
//! 잔여 3.5px 는 이 기전과 별개다(경계 걸침). 0 을 요구하면 다른 축의 결함까지
//! 이 테스트가 지게 되므로 두 상태 사이의 값으로 계약한다.
#![cfg(not(target_arch = "wasm32"))]

/// 금융위 보도자료. pi=5 의 1×1 RowBreak 표가 13쪽에 걸치고, 그 셀 안에 중첩 표를
/// 가진 문단이 여럿 있다.
///
/// 대한민국 정책브리핑 공개 자료를 HWPX 로 변환한 것이다(원본 `.hwp` 는
/// `ir_field_sweep` 표본 래칫에 걸린다).
const SAMPLE: &str = "samples/issue3637/press_release_split_cell_nested_table.hwpx";

/// 수정 전 최대 초과 287.6px, 수정 후 3.5px — 그 사이.
const MAX_OVERFLOW_PX: f64 = 30.0;

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// SVG 한 쪽에서 (뷰박스 높이, 쪽 높이를 넘는 글자와 그 y) 를 뽑는다.
fn out_of_page_glyphs(svg: &str) -> (f64, Vec<(f64, String)>) {
    let height = svg
        .split("height=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let mut over = Vec::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(rest) = chunk.split(" y=\"").nth(1) else {
            continue;
        };
        let Some(y) = rest.split('"').next().and_then(|s| s.parse::<f64>().ok()) else {
            continue;
        };
        if y > height + MAX_OVERFLOW_PX {
            let text = chunk
                .split('>')
                .nth(1)
                .and_then(|s| s.split('<').next())
                .unwrap_or("")
                .to_string();
            over.push((y, text));
        }
    }
    (height, over)
}

/// 분할 셀의 중첩 표 뒤 문단이 셀 밖으로 밀려나지 않는다.
#[test]
fn nested_table_snap_stays_inside_the_split_cell() {
    let bytes = std::fs::read(sample_path()).expect("표본 읽기");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("파싱");
    let page_count = doc.page_count();
    assert!(
        page_count >= 10,
        "쪽이 너무 적다({page_count}) — 표가 여러 쪽에 걸쳐야 조각 경로를 탄다. \
         표본이 바뀌었는지 확인하라"
    );

    let mut offenders = Vec::new();
    for p in 0..page_count {
        let svg = doc.render_page_svg(p).expect("SVG 렌더");
        let (h, over) = out_of_page_glyphs(&svg);
        assert!(
            h > 0.0,
            "{}쪽 SVG 뷰박스 높이를 읽지 못했다 — 방출 형식 확인",
            p + 1
        );
        for (y, t) in over {
            offenders.push(format!(
                "{}쪽 y={y:.1} (쪽 높이 {h:.1}, 초과 {:.1}px) {t:?}",
                p + 1,
                y - h
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "쪽 높이를 {MAX_OVERFLOW_PX}px 넘게 벗어난 글자가 {}개 있다.\n  {}\n\
         분할 표 셀에서 중첩 표 뒤 문단을 다음 문단의 절대 vpos 로 밀면, 그 vpos 에는 \
         조각 원점이 빠져 있지 않아 문단이 셀 상자 밖으로 나간다. 나간 글자는 텍스트 \
         추출에는 남아 있어 텍스트/IR diff 로는 잡히지 않는다.",
        offenders.len(),
        offenders
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
