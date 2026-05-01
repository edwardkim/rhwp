//! Issue #501: mel-001.hwp 2쪽 s0:pi=22 표 셀 높이 처리 회귀
//!
//! TAC 표 (treat_as_char=true) 의 raw_table_height 가 IR common.height 보다
//! 크게 계산되어 비례 축소 (height_measurer.rs:743-752) 가 적용되면서
//! 모든 행 높이가 정상보다 작게 축소됨.
//!
//! 본 회귀 케이스: mel-001.hwp 2쪽 8x12 인원현황 표
//! - IR common.height = 10960 HU = 146.13px
//! - 행 0 (헤더) IR cell.height = 1980 HU = 26.4px
//! - 회귀 SVG: 행 0 영역 ~12.36px (50% 축소)
//!
//! 정정 후: 행 0 영역이 IR cell.height 정합 (~26.4px)

use std::fs;
use std::path::Path;

#[test]
fn mel001_pi22_row0_height_matches_ir() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/mel-001.hwp");
    let bytes = fs::read(&hwp_path)
        .unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse mel-001.hwp");

    // 페이지 2 = index 1
    let svg = doc
        .render_page_svg_native(1)
        .expect("render mel-001.hwp page 2");

    // pi=22 디버그 라벨 위치 추출 (--debug-overlay 미사용 시 SVG 의 첫 cell-clip 추적)
    // pi=22 표는 페이지 2 의 마지막 표 영역 — 표 시작 y ≈ 637px (debug overlay 측정값)
    // 정정 후: 행 0 (헤더) cell-clip 의 y 영역 ≈ 26.4px (IR cell.height 정합)
    // 회귀 시: ≈ 12.36px (50% 축소)
    //
    // SVG 의 cell-clip rect 중 y=637.4 (pi=22 시작) 의 행 0 영역 높이 측정
    // 행 0 영역은 cell-clip-N 의 y=637.4 + height ≈ 26.4 가 되어야 함

    // 행 0 의 첫 cell-clip 추출: y="637.4" 또는 y="637.40000..." 패턴
    // 정상: height ≈ 26 ± 2 px (IR 26.4)
    // 회귀: height ≈ 12 ± 2 px

    // pi=22 표 시작 y 좌표는 페이지 레이아웃 변동 시 미세 변화 가능 — 동적 추출.
    // dump-pages 정합으로 mel-001 p2 의 pi=22 는 다른 표 (pi=17/19/20) 다음의
    // 마지막 8x12 표. 헤더 row (12개 균등 cs=1 셀들) 의 첫 번째 cell-clip 의 y 추출.
    //
    // 가용 정합 — pi=22 행 0 의 cs=1 셀들 (계, 정무직, 고공단, 3․4급, 4급, ...) 의
    // x 좌표는 211.05, 266.05, 321.05, ... (55px 간격). 그 중 첫 셀 (계 → x=211.05)
    // 의 y 가 pi=22 표 시작 y 와 정합.
    let table_start_y = {
        // x="211.0533333333333" 와 y=... 인 cell-clip rect 추출
        let needle = "x=\"211.0533333333333\" y=\"";
        let idx = svg.find(needle)
            .expect("pi=22 row 0 첫 cs=1 셀 (x=211.05) 을 찾지 못함");
        let after = &svg[idx + needle.len()..];
        let end = after.find('"').expect("y 종료 인용부호 없음");
        let y: f64 = after[..end].parse().expect("y 값 파싱 실패");
        // dump-pages 정합으로 pi=22 는 page 2 후반부 (y > 600)
        assert!(y > 600.0 && y < 800.0,
            "pi=22 행 0 시작 y 가 예상 범위 (600~800) 밖: {}", y);
        y
    };

    // y="<table_start_y>..." 인 cell-clip 의 height 추출
    // SVG 는 부동소수 그대로 출력 (예: y="636.3866666666667") — 추출한 정확값 사용
    let pattern_y = format!("y=\"{}", table_start_y);

    // 본 회귀 검증의 핵심:
    // - 회귀 (현재): 행 0 영역 높이 < 20px (50% 이하 축소)
    // - 정정 후: 행 0 영역 높이 ≥ 22px (IR cell.height 26.4 정합 ± 4px tolerance)

    let mut found_row0_heights: Vec<f64> = Vec::new();
    for chunk in svg.split(&pattern_y) {
        // chunk 의 시작은 행 0 영역의 cell-clip rect 의 y 직후 (예: " width=... height=...")
        // height 속성 추출
        if let Some(h_idx) = chunk.find("height=\"") {
            let after = &chunk[h_idx + 8..];
            if let Some(end_idx) = after.find('"') {
                let h_str = &after[..end_idx];
                if let Ok(h) = h_str.parse::<f64>() {
                    if h > 5.0 && h < 100.0 {
                        // 노이즈 제거: 5px 미만 또는 100px 초과는 패딩/라벨 등
                        found_row0_heights.push(h);
                    }
                }
            }
        }
    }

    assert!(!found_row0_heights.is_empty(),
        "pi=22 표의 행 0 영역 cell-clip 을 찾지 못함 (table_start_y={})", table_start_y);

    // 행 0 영역의 cell-clip 들 중 가장 큰 height 가 IR cell.height (≈26.4px) 정합
    let max_h = found_row0_heights.iter().cloned().fold(0.0f64, f64::max);
    println!("pi=22 행 0 영역 cell-clip heights: {:?}, max={:.2}",
        found_row0_heights, max_h);

    assert!(max_h >= 22.0,
        "pi=22 행 0 영역 최대 높이 {:.2}px < 22px (IR cell.height=26.4 ± 4 tolerance) — \
         TAC 표 비례 축소 회귀 (raw_table_height 가 common_h 보다 잘못 큰 값)",
        max_h);
}
