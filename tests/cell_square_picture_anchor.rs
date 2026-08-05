//! 표 셀 안 `Square` 어울림 그림은 셀 클립 영역 안에 놓인다 (#4059).
//!
//! `vert_rel_to=Para` 인 셀 내부 비인라인 그림의 앵커 기준점은 문단 top 이다. 그런데
//! `table_layout` 의 그림 경로는 `layout_composed_paragraph` 가 advance 시킨 뒤의
//! `para_y` 를 쓰고 있었고, 그래서 그림이 **줄 높이만큼 아래로** 내려가 셀 경계에 잘렸다.
//!
//! 같은 결함을 wrap 종류별로 하나씩 고쳐 온 자리다 — TopAndBottom(#577), 글뒤로·글앞으로
//! (#2207), 밀려난 빈 줄(#2226). `Square` 만 남아 있었다.
//!
//! 이 fixture(관세청 보도자료) 1쪽 첫 표 2행 맨오른쪽 셀의 "한국판뉴딜" 로고가 그 증상을
//! 재현한다. 한글 PDF 오라클에서는 그림 top 이 191.9px 인데 정정 전 rhwp 는 208.3px 로
//! 16.4px 낮았고, 셀 클립 하단(241.7px)을 17.3px 넘겨 아래쪽 글자가 잘렸다.
//!
//! 검사는 "셀 안에 있는가"로 고정한다. 좌표 상수를 그대로 박으면 무관한 레이아웃 변화에도
//! 깨지지만, 셀 경계 포함 관계는 이 결함이 재발하면 반드시 깨진다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::DocumentCore;

const SAMPLE: &str = "samples/156457624_210622 7월부터 해외직구 구매대행업체 등록제 시행.hwp";

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rect {
    fn bottom(&self) -> f64 {
        self.y + self.h
    }
    /// `self` 가 `outer` 안에 (허용오차만큼) 들어가는가.
    fn within(&self, outer: &Rect, tol: f64) -> bool {
        self.y >= outer.y - tol
            && self.bottom() <= outer.bottom() + tol
            && self.x >= outer.x - tol
            && self.x + self.w <= outer.x + outer.w + tol
    }
}

/// render tree JSON 에서 (셀, 그 셀이 직접 담은 Square 그림) 쌍을 모은다.
fn cell_square_pictures(node: &serde_json::Value, out: &mut Vec<(Rect, Rect)>) {
    let rect = |v: &serde_json::Value| -> Option<Rect> {
        let b = v.get("bbox")?;
        Some(Rect {
            x: b.get("x")?.as_f64()?,
            y: b.get("y")?.as_f64()?,
            w: b.get("w")?.as_f64()?,
            h: b.get("h")?.as_f64()?,
        })
    };

    if node.get("type").and_then(|t| t.as_str()) == Some("Cell") {
        if let Some(cell) = rect(node) {
            for child in node
                .get("children")
                .and_then(|c| c.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[])
            {
                let is_square = child.get("type").and_then(|t| t.as_str()) == Some("Image")
                    && child.get("textWrap").and_then(|t| t.as_str()) == Some("Square");
                if is_square {
                    if let Some(img) = rect(child) {
                        out.push((cell, img));
                    }
                }
            }
        }
    }

    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[])
    {
        cell_square_pictures(child, out);
    }
}

#[test]
fn issue_4059_cell_square_picture_stays_inside_its_cell() {
    let bytes = std::fs::read(SAMPLE).expect("fixture 를 읽을 수 있어야 한다");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture 파싱");

    let page = core
        .build_page_render_tree(0)
        .expect("1쪽 render tree 를 얻을 수 있어야 한다");
    let tree: serde_json::Value =
        serde_json::from_str(&page.root.to_json()).expect("render tree JSON");
    let root = tree.get("root").unwrap_or(&tree);

    let mut pairs = Vec::new();
    cell_square_pictures(root, &mut pairs);

    // 전제를 먼저 못박는다 — fixture 가 바뀌어 Square 그림이 사라지면 아래 단언이 공허해진다.
    assert!(
        !pairs.is_empty(),
        "1쪽 표 셀 안에 Square 그림이 있어야 이 테스트가 의미를 갖는다"
    );

    for (cell, img) in &pairs {
        assert!(
            img.within(cell, 1.0),
            "셀 안 Square 그림이 셀 밖으로 나갔다 — \
             그림 y={:.1}..{:.1} x={:.1}..{:.1}, 셀 y={:.1}..{:.1} x={:.1}..{:.1}",
            img.y,
            img.bottom(),
            img.x,
            img.x + img.w,
            cell.y,
            cell.bottom(),
            cell.x,
            cell.x + cell.w,
        );
    }
}
