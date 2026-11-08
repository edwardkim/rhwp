//! [#7888] 문단 테두리 묶음의 아래 변은 뒤따르는 묶음이 있든 없든 같은 자리에 놓인다.
//!
//! 수정 전 rhwp 는 연결(attr1 bit 28) 테두리 묶음이 아래 인접 묶음과 닿지 않으면 위·아래
//! 변을 최소 2px 바깥으로 넓혔다(Task #321 v6 의 시각 여유 추정). 그래서 쪽·단 마지막
//! 묶음만 아래 변이 2px 더 내려갔다.
//!
//! # 독립 근거
//!
//! `exam_eng` 8쪽은 같은 문단 모양(ps_id=60, 테두리 아래 간격 0, 연결)의 테두리 상자 다섯을
//! 담는다. 한/글 2020 PDF(`pdf/exam_eng-hwp-2020.pdf`, 쪽 척도 1.00044 보정)의 아래 변은
//! 단 마지막 상자(pi=298) 1422.40px, 단 중간 상자(pi=300) 432.32px 이다. 두 상자 모두
//! rhwp 의 마지막 글줄 아래 + 줄간격 끝과 0.15px 안으로 맞는다. 수정 전 rhwp 는 pi=298 만
//! 1424.39px 로 2px 아래였다. 같은 차이를 `exam_kor` 4·20쪽, 온새미로 NOTE 쪽 마지막
//! 묶음에서도 쟀다.
//!
//! 검사는 "같은 문단 모양 상자의 아래 변 − 마지막 글줄 아래" 가 상자마다 같은지 본다.
//! 절대 좌표는 PDF Visual Sweep 으로 확인한다.

use rhwp::document_core::DocumentCore;

#[derive(Debug, Clone, Copy)]
struct BBox {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl BBox {
    fn bottom(&self) -> f64 {
        self.y + self.h
    }
    fn right(&self) -> f64 {
        self.x + self.w
    }
    fn contains(&self, other: &BBox) -> bool {
        other.x >= self.x - 0.5
            && other.right() <= self.right() + 0.5
            && other.y >= self.y - 0.5
            && other.bottom() <= self.bottom() + 0.5
    }
}

fn load(rel: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{rel} 읽기 실패: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{rel} 로드 실패: {e:?}"))
}

fn bbox(node: &serde_json::Value) -> Option<BBox> {
    let b = node.get("bbox")?;
    Some(BBox {
        x: b.get("x")?.as_f64()?,
        y: b.get("y")?.as_f64()?,
        w: b.get("w")?.as_f64()?,
        h: b.get("h")?.as_f64()?,
    })
}

/// 본문 단의 테두리 사각형과 (문단 번호, 글줄 상자) 를 모은다. 바탕쪽·머리말·꼬리말은 뺀다.
fn collect(
    node: &serde_json::Value,
    rects: &mut Vec<BBox>,
    lines: &mut Vec<(u64, BBox)>,
    in_body: bool,
) {
    let ty = node.get("type").and_then(|v| v.as_str()).unwrap_or("");
    if matches!(ty, "Header" | "Footer" | "PageBg" | "MasterPage") {
        return;
    }
    let in_body = in_body || ty == "Column";
    if in_body {
        match ty {
            "Rectangle" | "Rect" => {
                if let Some(b) = bbox(node) {
                    rects.push(b);
                }
            }
            "TextLine" => {
                if let (Some(b), Some(pi)) = (bbox(node), node.get("pi").and_then(|v| v.as_u64())) {
                    lines.push((pi, b));
                }
                return;
            }
            _ => {}
        }
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        collect(child, rects, lines, in_body);
    }
}

/// (상자, 상자 안 마지막 글줄의 문단 번호, 상자 아래 변 − 그 글줄 아래)
fn border_boxes(core: &DocumentCore, page: u32) -> Vec<(BBox, u64, f64)> {
    let tree = core
        .build_page_render_tree(page)
        .expect("render tree 생성 실패");
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("render tree JSON 파싱");
    let mut rects = Vec::new();
    let mut lines = Vec::new();
    collect(&json, &mut rects, &mut lines, false);
    rects
        .into_iter()
        .filter(|r| r.w > 300.0)
        .filter_map(|r| {
            lines
                .iter()
                .filter(|(_, l)| r.contains(l))
                .max_by(|a, b| a.1.bottom().total_cmp(&b.1.bottom()))
                .map(|(pi, l)| (r, *pi, r.bottom() - l.bottom()))
        })
        .collect()
}

#[test]
fn column_last_border_box_bottom_matches_mid_column_boxes() {
    let core = load("samples/exam_eng.hwp");
    // 8쪽(0-기준 7). 같은 문단 모양(ps_id=60)의 테두리 상자: 왼쪽 단 끝 pi=298,
    // 오른쪽 단 pi=300·302·304.
    let boxes = border_boxes(&core, 7);
    let by_pi = |pi: u64| {
        boxes
            .iter()
            .find(|(_, last_pi, _)| *last_pi == pi)
            .unwrap_or_else(|| panic!("pi={pi} 테두리 상자가 없다 — 표본 전제 불성립: {boxes:?}"))
    };
    let column_last = by_pi(298);
    let mids = [by_pi(300), by_pi(302), by_pi(304)];
    for mid in mids {
        assert!(
            (column_last.2 - mid.2).abs() <= 0.25,
            "같은 문단 모양 상자의 아래 변 − 마지막 글줄 아래가 달라졌다: \
             단 끝 pi=298 {:.2}px, 단 중간 pi={} {:.2}px (한/글 PDF 는 같다)",
            column_last.2,
            mid.1,
            mid.2
        );
    }
}
