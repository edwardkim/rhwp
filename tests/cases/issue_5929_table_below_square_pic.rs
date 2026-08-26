//! [#5929] 어울림(Square) 그림 아래 자리차지(T&B) 표가 그림과 겹치지 않는다.
//!
//! 이슈 첨부 `새 한글(1).hwpx`: 그림은 `textWrap=SQUARE`·`treatAsChar=0`·
//! `allowOverlap=0`·`vertRelTo=PARA` 이고, 몇 빈 문단 뒤 표는
//! `textWrap=TOP_AND_BOTTOM`. 한컴은 표를 그림 하단에 두고, 결함 rhwp 는
//! 표 상단(≈594px)이 그림 밴드(377..628) 안에 들어가 겹친다.
//!
//! 수정: Square 그림 페인트 bbox 를 `blocks_text=false` exclusion 으로 남기고,
//! 후속 T&B 표만 그 하단으로 민다. 본문 텍스트는 옆 흐름을 유지한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5929/table_below_square_pic.hwpx";

fn load_doc() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open")
}

fn image_box(svg: &str) -> Option<(f64, f64, f64, f64)> {
    let rest = svg.split("<image ").nth(1)?;
    let head = rest.split('>').next()?;
    let attr = |name: &str| -> Option<f64> {
        let key = format!("{name}=\"");
        let s = head.find(&key)? + key.len();
        let e = s + head[s..].find('"')?;
        head[s..e].parse().ok()
    };
    Some((attr("x")?, attr("y")?, attr("width")?, attr("height")?))
}

fn first_table_top(svg: &str) -> Option<f64> {
    // 셀 clipPath 가 표 칸의 페인트 상단을 준다.
    let mut top: Option<f64> = None;
    for cap in svg.split("id=\"cell-clip-").skip(1) {
        let rect = cap.split("<rect ").nth(1)?;
        let head = rect.split('>').next()?;
        let key = "y=\"";
        let s = head.find(key)? + key.len();
        let e = s + head[s..].find('"')?;
        let y: f64 = head[s..e].parse().ok()?;
        top = Some(match top {
            Some(t) => t.min(y),
            None => y,
        });
    }
    top
}

#[test]
fn issue_5929_topbottom_table_clears_square_picture() {
    let doc = load_doc();
    let svg = doc.render_page_svg_native(0).expect("page 1 svg");
    let (_, img_y, _, img_h) = image_box(&svg).expect("그림 <image> 가 있어야 한다");
    let table_y = first_table_top(&svg).expect("표 셀 clip 이 있어야 한다");
    let img_bottom = img_y + img_h;
    assert!(
        table_y + 0.5 >= img_bottom,
        "#5929: 자리차지 표 상단({table_y:.1})이 어울림 그림 하단({img_bottom:.1}) 아래여야 한다 \
         (결함 시 표 y≈594, 그림 377..628 겹침)"
    );
    assert!(
        (table_y - 594.4).abs() > 5.0,
        "#5929: 결함 위치(표 y≈594.4, 그림 밴드 안)에 표가 남아 있으면 안 된다: {table_y:.1}"
    );
}
