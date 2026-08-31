//! [#6451] 밑줄 조각이 run 경계에서 정확히 맞닿는다 — 실틈이 남지 않는다.
//!
//! `samples/issue6451/underline_run_fragments.hwpx` 는 과학기술정보통신부 보도자료
//! (156514427, 1.5MB)의 구조 보존 슬라이스다 — BinData 를 8×8 그림으로 바꿔
//! 0.11MB 로 줄였다(이 결함은 그림과 무관하다).
//!
//! **⚠ 이슈 본문의 "PDF 는 공백 자리 8.5pt 가 비어 끊긴다"는 측정 아티팩트다.**
//! PDF 에도 조각 셋이 다 있다 — 폭 20pt 미만 조각을 거른 스캔이 가운데 8.33pt 를
//! 스스로 뺀 것이다. 실제 결손은 훨씬 작다.
//!
//! **실제 결함 — run 경계의 실틈.** 600dpi 잉크로 재면 `279.38..279.62` **0.25pt** 가
//! 비어 있고, 다음 경계는 반대로 **0.08pt 겹친다**(겹침이 studio 에서 "조금 두껍고
//! 길게" 보이는 증상이다).
//!
//! ```text
//! 종전 : 76.39..279.39 │ 279.64..287.97 │ 287.89..364.89   ← 0.25pt 틈 + 0.08pt 겹침
//! 수정 : 76.39..279.64 │ 279.64..287.89 │ 287.89..365.14   ← 정확히 맞닿음
//! 한글 : 76.28..364.60 (연속 한 줄)
//! ```
//!
//! **근인 — 장식선 폭만 작성기가 자체 측정한다.**
//!
//! 밑줄은 run 마다 `x .. x + decoration_width` 로 그려지는데, `decoration_width` 는
//! SVG 작성기가 글자를 다시 재서 얻는다. 그 값이 레이아웃이 다음 run 을 놓은
//! 위치와 어긋난다.
//!
//! | | run1 끝 | run2 시작 |
//! |---|---|---|
//! | 작성기 측정 | **372.520** | — |
//! | 레이아웃(render tree bbox) | `101.9 + 271.0` = **372.900** | **372.900** |
//!
//! render tree bbox 는 `101.9+271.0=372.9`, `372.9+11.0=383.9` 로 **정확히 연속**이다.
//! 장식선이 그 폭을 쓰면 조각이 저절로 맞닿는다.
//!
//! 말미 공백 트림(#6028)이 걸린 run 은 글자 단위로 걷어내야 하므로 종전 계산을 남겼다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6451/underline_run_fragments.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-6451-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 1쪽 SVG.
fn page1_svg() -> String {
    let dir = temp_dir();
    let out = Command::new(rhwp_bin())
        .args([
            "export-svg",
            &sample(),
            "-p",
            "0",
            "-o",
            &dir.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let path = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "svg"))
        .expect("SVG 가 없다");
    std::fs::read_to_string(path).unwrap()
}

/// 주어진 y(px) 위의 수평선 조각들을 x 순으로 (x1, x2) 로 돌려준다.
fn horizontal_segments_at(svg: &str, y_target: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for chunk in svg.split("<line ").skip(1) {
        let end = chunk.find("/>").unwrap_or(chunk.len());
        let seg = &chunk[..end];
        let attr = |k: &str| -> Option<f64> {
            seg.split(&format!("{k}=\""))
                .nth(1)
                .and_then(|r| r.split('"').next())
                .and_then(|v| v.parse::<f64>().ok())
        };
        if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
            (attr("x1"), attr("y1"), attr("x2"), attr("y2"))
        {
            if (y1 - y2).abs() < 0.3 && (y1 - y_target).abs() < 0.5 {
                out.push((x1.min(x2), x1.max(x2)));
            }
        }
    }
    out.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    out
}

/// 밑줄 조각이 서로 맞닿아 실틈도 겹침도 없다.
#[test]
fn underline_fragments_meet_exactly() {
    // 1쪽 「…포럼 1주년 기념식」 둘째 줄 밑줄 (px, 96dpi).
    let segs = horizontal_segments_at(&page1_svg(), 583.75);
    assert_eq!(
        segs.len(),
        3,
        "이 줄의 밑줄은 run 경계로 3조각이다: {segs:?}"
    );
    for pair in segs.windows(2) {
        let gap = pair[1].0 - pair[0].1;
        assert!(
            gap.abs() < 0.01,
            "밑줄 조각이 맞닿지 않는다 (종전 0.333px 틈 / −0.107px 겹침): \
             {:.3} → {:.3} = {gap:+.3}px, 전체 {segs:?}",
            pair[0].1,
            pair[1].0,
        );
    }
}
