//! [Issue #5734] 셀 안에 세로로 쌓여야 할 자리차지 그림들이 셀 상단 한 자리에 겹친다.
//!
//! 156684746 9쪽 왼쪽 칸: 문단마다 자리차지(TopAndBottom·Para) 그림이 하나씩 있고
//! 저장 lineseg vpos 는 0 → 6336 → 9136 HU 계단인데, 셀-valign 강제(#2071)가 세 그림을
//! 전부 `content_top + vOffset`(616.6/640.3/635.6px)에 붙여 서로 포갰다. 한글은
//! 655.6 → 749.3 → 781.9 로 차곡차곡 쌓는다.
//!
//! 수정: table_partial 의 첫 조각(su==0)에서 앵커 문단 첫 lineseg vpos>0 이면 저장
//! 흐름 배치(content_top + vpos + vOffset) — 겹침이 풀리고 계단이 복원된다(724.8/
//! 757.5; 한글과의 잔여 +24.5px 균일 시프트는 행 기하 축 #5714, 이 테스트 범위 밖).
//!
//! 픽스처 `cell_float_stack_stored_vpos.hwpx` 는 원본 9쪽을 extract-pages 로 떼고
//! 대형 BinData 를 1×1 스텁으로 바꾼 marker-HWPX(43KB) — 원본 .hwp 와 같은 기하를
//! 재현한다(hwp5_stored_pagination_layout 프로파일 유지).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5734/cell_float_stack_stored_vpos.hwpx";

fn image_rects(svg: &str) -> Vec<(f64, f64, f64)> {
    let mut out = Vec::new();
    for cap in svg.split("<image ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = s + head[s..].find('"')?;
            head[s..e].parse().ok()
        };
        if let (Some(x), Some(y), Some(h)) = (attr("x"), attr("y"), attr("height")) {
            out.push((x, y, h));
        }
    }
    out
}

#[test]
fn issue_5734_left_cell_floats_stack_by_stored_vpos() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 왼쪽 칸(x≈80)의 세 그림: 높이 84.5 / 28.0 / 59.3.
    let rects = image_rects(&svg);
    let pick = |h_want: f64| {
        rects
            .iter()
            .find(|(x, _, h)| *x < 200.0 && (h - h_want).abs() < 0.6)
            .unwrap_or_else(|| panic!("왼쪽 칸 그림(h={h_want})이 없다: {rects:?}"))
    };
    let a = pick(84.5);
    let b = pick(28.0);
    let c = pick(59.3);

    // 저장 vpos 계단(0 → 6336 → 9136 HU)이 흐름 배치로 복원되어야 한다.
    assert!(
        (b.1 - a.1 - 84.5 - 23.7).abs() < 1.5,
        "둘째 그림이 첫 그림 아래 저장 vpos 간격이어야 한다: a={:.1} b={:.1} (결함 시 640.3-616.6=23.7 겹침)",
        a.1,
        b.1
    );
    // 겹침 금지: b 는 a 아래, c 는 b 아래.
    assert!(
        b.1 >= a.1 + a.2 - 0.5 && c.1 >= b.1 + b.2 - 0.5,
        "왼쪽 칸 그림이 겹친다: a={:.1}+{:.1} b={:.1}+{:.1} c={:.1}",
        a.1,
        a.2,
        b.1,
        b.2,
        c.1
    );
}
