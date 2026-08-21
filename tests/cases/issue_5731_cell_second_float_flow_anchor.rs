//! [Issue #5731] 다문단 셀의 둘째 자리차지 그림이 셀 상단에 붙어 앞 그림과 겹친다.
//!
//! 156522760 3쪽: 한 셀에 TAC 그림(저장 vpos 1760) → 캡션 문단(12302) → 자리차지
//! (TopAndBottom·vertRelTo=Para) 그림(저장 vpos 14062)이 차례로 있는데, 셀-valign
//! 강제(#2071)가 둘째 그림을 셀 상단(y=492.0)에 붙여 첫 그림(515.5~654.6)과 145px
//! 겹쳤다. 한글 2022 COM PDF 오라클: 두 그림 y=514.8/678.7 — 둘째는 앵커 문단의
//! 저장 lineseg vpos 로 흐름 배치된다. (이슈 원문의 "한글 6개" 계측은 재검증에서
//! 반증 — 한글도 7개를 그린다.)
//!
//! 수정: 저장 좌표 신뢰 프로파일에서 앵커 문단 첫 lineseg vpos>0 이면 셀-valign
//! 강제 대신 `content_top + vpos + vOffset` 흐름 배치 (table_layout/table_partial 동일).
//!
//! 픽스처는 원본에서 대형 BinData 를 1×1 스텁으로 바꾼 축소본(35KB).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5731/cell_second_float_flow_anchor.hwpx";

fn image_rects(svg: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for cap in svg.split("<image ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = s + head[s..].find('"')?;
            head[s..e].parse().ok()
        };
        if let (Some(y), Some(h)) = (attr("y"), attr("height")) {
            out.push((y, h));
        }
    }
    out
}

#[test]
fn issue_5731_second_cell_float_uses_stored_flow_anchor() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(2).expect("page 3 svg");

    let rects = image_rects(&svg);
    assert_eq!(rects.len(), 2, "3쪽 셀 그림은 2개다: {rects:?}");
    let first = rects
        .iter()
        .find(|(_, h)| (h - 139.1).abs() < 0.6)
        .expect("첫 그림(139.1px)");
    let second = rects
        .iter()
        .find(|(_, h)| (h - 144.6).abs() < 0.6)
        .expect("둘째 그림(144.6px)");

    // 한글 오라클: 첫 514.8, 둘째 678.7 (PDF pt 반올림 ±1px).
    assert!(
        (first.0 - 515.5).abs() < 1.5,
        "첫 그림 y={:.1} — 한글 514.8 근방이어야 한다",
        first.0
    );
    assert!(
        (second.0 - 679.5).abs() < 1.5,
        "둘째 그림 y={:.1} — 한글 678.7 근방이어야 한다(결함 시 492.0 셀 상단)",
        second.0
    );
    // 겹침 금지: 둘째 그림 top 이 첫 그림 bottom 아래.
    assert!(
        second.0 >= first.0 + first.1 - 0.5,
        "그림이 겹친다: 첫 {:.1}+{:.1} vs 둘째 {:.1}",
        first.0,
        first.1,
        second.0
    );
}
