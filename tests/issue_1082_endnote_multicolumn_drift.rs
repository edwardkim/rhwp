//! Issue #1082: 다단(2단) 미주(endnote) 영역에서 typeset 누적이 미주 para 내부 vpos span 만
//! 더해 미주 간 vpos 간격(빈 줄/문단 간격) 을 누락 → 단(특히 col 1) 과충전 → 렌더 vpos 정규화
//! 시 본문 하단을 수백 px 초과하는 결함 회귀 가드.
//!
//! 재현 문서 (tracked 공개 샘플 — 시험지 정답/해설 미주 다수):
//! - `samples/3-09월_교육_통합_2023.hwp` / `.hwpx` (최악 627 → 24px)
//! - `samples/3-09월_교육_통합_2022.hwp` (277 → 26)
//! - `samples/3-10월_교육_통합_2022.hwp` (159 → 17)
//! - `samples/3-11월_실전_통합_2022.hwp` (561 → 9)
//!
//! 정정 (typeset.rs): 다단 미주 누적을 렌더 vpos 정규화와 정합 — 직전 배치 아이템 bottom 기준
//! vpos delta(px) 로 누적. 시드 = 본문 last bottom(body→endnote 전환 정합); 단 advance 시 None.
//! #1062 안전 floor(fmt.height_for_fit) 유지.

use std::fs;
use std::path::Path;

fn load_doc(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

/// 전 페이지 max overflow px 합산. typeset 다단 미주 드리프트 회귀 시 수백 px.
fn doc_total_overflow_px(rel: &str) -> f64 {
    let doc = load_doc(rel);
    let pages = doc.page_count();
    let mut total = 0.0_f64;
    for p in 0..pages {
        let svg = match doc.render_page_svg_native(p) {
            Ok(s) => s,
            Err(_) => continue,
        };
        // svg height = 페이지 물리 높이
        let h = {
            let i = svg.find("height=\"").unwrap() + 8;
            let rest = &svg[i..];
            let e = rest.find('"').unwrap();
            rest[..e].parse::<f64>().unwrap_or(0.0)
        };
        // 모든 <text y="..."> 의 y 최대 — 페이지 초과분 합산
        let mut max_y = 0.0_f64;
        let mut rest = svg.as_str();
        while let Some(o) = rest.find("<text") {
            let end = rest[o..].find('>').map(|g| o + g).unwrap_or(rest.len());
            let t = &rest[o..end];
            if let Some(yi) = t.find(" y=\"") {
                let yr = &t[yi + 4..];
                if let Some(ye) = yr.find('"') {
                    if let Ok(y) = yr[..ye].parse::<f64>() {
                        if y > max_y {
                            max_y = y;
                        }
                    }
                }
            }
            rest = &rest[end..];
        }
        if max_y > h {
            total += max_y - h;
        }
    }
    total
}

/// 회귀 한계: C군 잔여(~30px, 본문 fmt 드리프트의 작은 영향). 종전 회귀 시 수백 px(627 등).
const REG_LIMIT_PX: f64 = 60.0;

#[test]
fn exam_3_09_2023_hwp_endnote_drift_capped() {
    let t = doc_total_overflow_px("samples/3-09월_교육_통합_2023.hwp");
    assert!(
        t <= REG_LIMIT_PX,
        "3-09'23 hwp endnote drift {t:.1}px > {REG_LIMIT_PX}"
    );
}

#[test]
fn exam_3_09_2023_hwpx_endnote_drift_capped() {
    let t = doc_total_overflow_px("samples/3-09월_교육_통합_2023.hwpx");
    assert!(
        t <= REG_LIMIT_PX,
        "3-09'23 hwpx endnote drift {t:.1}px > {REG_LIMIT_PX}"
    );
}

#[test]
fn exam_3_09_2022_hwp_endnote_drift_capped() {
    let t = doc_total_overflow_px("samples/3-09월_교육_통합_2022.hwp");
    assert!(
        t <= REG_LIMIT_PX,
        "3-09'22 hwp endnote drift {t:.1}px > {REG_LIMIT_PX}"
    );
}

#[test]
fn exam_3_11_2022_hwp_endnote_drift_capped() {
    let t = doc_total_overflow_px("samples/3-11월_실전_통합_2022.hwp");
    assert!(
        t <= REG_LIMIT_PX,
        "3-11'22 hwp endnote drift {t:.1}px > {REG_LIMIT_PX}"
    );
}
