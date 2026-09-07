//! [#6776] 칸 안 **줄이 0개인 문단의 글자처럼 취급(TAC) 그림**이 `cell_units`
//! 회계에서 통째로 빠져, 조각이 자기 프레임을 넘어 용지 밖까지 그린다.
//!
//! `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx` 19쪽은 바깥
//! 7×2 표의 마지막 행 칸에 1×1 중첩 표를 담고, 그 자식 칸(33문단)의 `pi=12`·`pi=23`
//! 이 **글자 없는 문단에 TAC 그림 하나**씩만 담는다(312.4px · 725.4px).
//!
//! `cell_units` 는 그 문단들을 `line_count == 0` 가지에서 유닛화하면서 `Control::Table`
//! 만 세었다. 그림 1,037.8px 가 회계에서 빠지니 컷만 짧아지고, 페인트는 그림 높이만큼
//! 자리를 잡으므로 조각이 프레임을 넘었다.
//!
//! | 항목 | 수정 전 | 수정 후 | engine 2020 정본 |
//! | --- | --- | --- | --- |
//! | 19쪽 725.4px 그림 상단 `y` | 1,327.7 (용지 1,122.5 밖 930.5) | 21쪽 79.0 | — |
//! | 문서 전체 용지 밖 | 1 | **0** | — |
//! | 총 글자 | 27,216 | **27,312** | 27,308 |
//!
//! ⚠⚠ **줄이 있는 문단에서는 세지 않는다** — 그때 TAC 그림은 이미 그 줄 높이에 들어
//! 있어 이중 계상이 된다. 관문 없이 켜면 `table_giant_cell_overfill` 계열이 넘침
//! 3 → 15건·용지 밖 2 → 3건으로 무너진다.
//!
//! ⚠ 총 쪽수는 74 → 75 로 늘지만 이 축의 회귀가 아니다. 정본과 쪽 단위로 맞춰 보면
//! **수정 전에도 9쪽부터 이미 한 쪽 밀려 있었고**(rhwp 9쪽은 꼬리말만 있는 빈 쪽),
//! 그 +1 이 이 결함으로 잃던 −1 과 상계돼 총합만 74 로 맞았다. 빈 9쪽은 `devel` 의
//! 별개 결함이라 여기서 잠그지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

/// 양성 — 자식 칸이 TAC 그림만 담은 문단을 갖는다.
const SAMPLE: &str = "samples/issue6776/78494-virtual-convergence-industry-decree.hwpx";
/// 음성 대조 — 칸 안 그림이 **어울림(비-TAC)** 이라 회계에 들어가면 안 된다.
const NEGATIVE: &str = "samples/issue6776/36367506-water-facility-approval.hwpx";

/// 이 문서가 조판하는 A4 세로 종이 높이(px, 96dpi).
const PAPER_HEIGHT_PX: f64 = 1122.5;
/// 결함이 드러나던 쪽(0 기준) — 자식 1×1 표의 첫 조각이 앉는 자리.
const HOST_PAGE: u32 = 18;
/// 회계에서 빠져 있던 큰 TAC 그림의 높이(px). 726,000HWPUNIT 은 아니고 저장 원본 값이다.
const BIG_PICTURE_H_PX: f64 = 725.4;

fn open(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}")))
        .unwrap_or_else(|e| panic!("open {sample}: {e}"))
}

/// SVG 의 `<image …>` 를 `(y, height)` 로 걷는다.
fn images(svg: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for chunk in svg.split("<image ").skip(1) {
        let head = &chunk[..chunk.find('>').unwrap_or(chunk.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = head[s..].find('"')?;
            head[s..s + e].parse::<f64>().ok()
        };
        if let (Some(y), Some(h)) = (attr("y"), attr("height")) {
            out.push((y, h));
        }
    }
    out
}

fn page_svg(core: &DocumentCore, page: u32) -> String {
    core.render_page_svg_native(page)
        .unwrap_or_else(|e| panic!("{}쪽 svg: {e}", page + 1))
}

#[test]
fn issue_6776_host_page_paints_no_picture_past_the_paper() {
    let core = open(SAMPLE);
    let svg = page_svg(&core, HOST_PAGE);
    let imgs = images(&svg);
    assert!(
        !imgs.is_empty(),
        "{}쪽에는 자식 칸의 TAC 그림이 있어야 한다",
        HOST_PAGE + 1
    );
    for (y, h) in &imgs {
        assert!(
            y + h <= PAPER_HEIGHT_PX + 0.5,
            "{}쪽 TAC 그림이 종이({PAPER_HEIGHT_PX}) 안에 있어야 한다 — 회계에서 빠지면 \
             725.4px 그림이 y=1327.7..2053.0 으로 용지 아래 930.5px 에 그려진다 \
             (실측 y={y:.1} h={h:.1})",
            HOST_PAGE + 1
        );
    }
}

#[test]
fn issue_6776_big_picture_is_placed_whole_on_exactly_one_page() {
    let core = open(SAMPLE);
    let page_count = u32::try_from(core.page_count()).expect("page count fits u32");
    let mut hits = Vec::new();
    for page in 0..page_count {
        for (y, h) in images(&page_svg(&core, page)) {
            if (h - BIG_PICTURE_H_PX).abs() <= 1.0 {
                hits.push((page, y, h));
            }
        }
    }
    assert_eq!(
        hits.len(),
        1,
        "{BIG_PICTURE_H_PX}px TAC 그림은 딱 한 쪽에 한 번만 그려져야 한다 \
         — 잃어도(0) 중복돼도(2+) 안 된다. 실측 {hits:?}"
    );
    let (page, y, h) = hits[0];
    assert!(
        y >= -0.5 && y + h <= PAPER_HEIGHT_PX + 0.5,
        "{}쪽 그림이 종이 안에 온전히 들어가야 한다 (y={y:.1} h={h:.1})",
        page + 1
    );
}

#[test]
fn issue_6776_negative_cell_wrapped_picture_is_not_charged() {
    // 이 문서의 칸 그림은 어울림(비-TAC)이라 `para_non_inline_h` 소관이다. 회계에
    // 넣으면 이중 계상돼 3쪽 배분이 무너지고 용지 밖 1건·넘침 1건이 생긴다.
    // engine 2020 정본도 3쪽이고 쪽별 글자 수 [112, 359, 326] 로 같다.
    let core = open(NEGATIVE);
    assert_eq!(
        core.page_count(),
        3,
        "음성 대조: 어울림 그림은 칸 회계에 들어가면 안 된다 — 들어가면 배분이 무너진다"
    );
    let page_count = u32::try_from(core.page_count()).expect("page count fits u32");
    for page in 0..page_count {
        for (y, h) in images(&page_svg(&core, page)) {
            assert!(
                y + h <= PAPER_HEIGHT_PX + 0.5,
                "음성 대조 {}쪽 그림이 종이 밖으로 나가면 안 된다 (y={y:.1} h={h:.1})",
                page + 1
            );
        }
    }
}
