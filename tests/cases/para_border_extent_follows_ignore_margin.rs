//! 문단 테두리·배경의 범위는 문단 모양 attr1 bit 29(HWPX `ignoreMargin`)가 정한다.
//!
//! 수정 전 rhwp 는 채움 배경을 늘 마지막 줄 상자까지만 칠했다. 이어진 같은 배경 문단 사이에
//! 줄간격만큼 흰 틈이 남았다. HWP3 `hwp3-sample4` 코드 블록이 그 예다. 한/글은 7줄 블록을
//! 140px 띠 하나로 칠하고, rhwp 는 13.3px 띠 일곱 개를 그렸다.
//!
//! # 독립 근거
//!
//! 합성 HWPX 를 한/글 2020 으로 HWP 로 저장했다. 그 저장본을 다시 한/글 2020 으로 PDF 로
//! 출력해 실측했다(`pdf/para_border_extent/*-hwp-2020.pdf`). 변인은 bit 29, 연결(bit 28),
//! 위·아래 테두리 여백, 앞·뒤 간격, 좌우 여백, 테두리 선 유무, 쪽 끝·단 끝이다.
//!
//! - bit 29 꺼짐: 단 너비 전체를 덮는다. 세로로는 앞 간격부터 마지막 줄의 줄간격과 뒤
//!   간격까지 문단 흐름 전체를 덮는다. 이어진 문단은 연결과 무관하게 틈 없는 한 띠가 된다.
//! - bit 29 켜짐: 문단 좌우 여백 안쪽만 덮는다. 세로로는 첫 줄 위부터 마지막 줄 상자
//!   아래까지 덮는다. 문단 안의 줄간격은 칠하고 마지막 줄간격은 칠하지 않는다.
//!   `issue1835` 제목 배경이 이 경우다.
//! - 테두리 선이 없는 채움만의 bit 29 문단은 연결(bit 28)이 켜져 있어도 문단 사이 틈을
//!   칠하지 않는다. 테두리 선이 있으면 연결된 문단이 한 상자로 이어진다.
//!
//! 검사는 배경 사각형과 글줄의 포함·틈 관계만 본다. 절대 좌표는 PDF Visual Sweep 으로
//! 확인한다.

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
}

struct PageShapes {
    column: BBox,
    rects: Vec<BBox>,
    lines: Vec<(String, BBox)>,
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

fn collect(node: &serde_json::Value, column: &mut Option<BBox>, shapes: &mut PageShapes) {
    let ty = node.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match ty {
        "Header" | "Footer" | "PageBg" => return,
        "Column" if column.is_none() => *column = bbox(node),
        "Rect" => {
            if let Some(b) = bbox(node) {
                shapes.rects.push(b);
            }
        }
        "TextLine" => {
            let mut text = String::new();
            for run in node
                .get("children")
                .and_then(|c| c.as_array())
                .into_iter()
                .flatten()
            {
                if let Some(t) = run.get("text").and_then(|t| t.as_str()) {
                    text.push_str(t);
                }
            }
            if let Some(b) = bbox(node) {
                shapes.lines.push((text, b));
            }
            return;
        }
        _ => {}
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        collect(child, column, shapes);
    }
}

fn page(core: &DocumentCore, page: u32) -> PageShapes {
    let tree = core
        .build_page_render_tree(page)
        .expect("render tree 생성 실패");
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("render tree JSON 파싱");
    let mut column = None;
    let mut shapes = PageShapes {
        column: BBox {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        },
        rects: Vec::new(),
        lines: Vec::new(),
    };
    collect(&json, &mut column, &mut shapes);
    shapes.column = column.expect("본문 단을 찾지 못했다");
    shapes
}

impl PageShapes {
    /// 글자 머리 표지(`A1`, `V2` …)로 시작하는 글줄.
    fn line(&self, tag: &str) -> BBox {
        self.lines
            .iter()
            .find(|(text, _)| text.starts_with(tag))
            .map(|(_, b)| *b)
            .unwrap_or_else(|| panic!("`{tag}` 글줄을 찾지 못했다 — 표본 전제 불성립"))
    }

    /// 세로 위치 `y` 를 칠하는 배경 사각형(가로로 단 가운데를 지나는 것).
    fn rect_at(&self, y: f64) -> Option<BBox> {
        let mid_x = self.column.x + self.column.w / 2.0;
        self.rects
            .iter()
            .copied()
            .find(|r| r.y <= y && y < r.bottom() && r.x <= mid_x && mid_x <= r.right())
    }

    fn painted(&self, y: f64) -> bool {
        self.rect_at(y).is_some()
    }

    /// 배경이 `[top, bottom)` 을 틈 없이 덮는지 0.5px 간격으로 본다.
    fn covers(&self, top: f64, bottom: f64) -> Result<(), f64> {
        let mut y = top;
        while y < bottom {
            if !self.painted(y) {
                return Err(y);
            }
            y += 0.5;
        }
        Ok(())
    }
}

const PAGE: &str = "samples/para_border_extent/border-extent-page.hwp";
const IGN: &str = "samples/para_border_extent/border-extent-ign.hwp";
const COL: &str = "samples/para_border_extent/border-extent-col.hwp";

/// bit 29 꺼진 이어진 채움 문단은 다음 일반 문단 첫 줄 위까지 틈 없이 한 띠다.
#[test]
fn consecutive_fill_paragraphs_paint_trailing_line_spacing() {
    let p = page(&load(PAGE), 0);
    // (첫 채움 문단, 다음 문단, 다음 문단이 일반 문단인가)
    for (first, next, next_is_plain) in [
        ("A1", "B0", true),
        ("B1", "C0", true),
        ("D1", "E0", true),
        ("Z00", "Z05", false),
    ] {
        let top = p.line(first).y;
        let next_top = p.line(next).y;
        if let Err(y) = p.covers(top + 0.5, next_top - 0.5) {
            panic!(
                "`{first}` 부터 `{next}` 직전까지 배경이 이어져야 한다 — y={y:.1} 에 틈 \
                 (줄 {top:.1}..{next_top:.1})"
            );
        }
        assert!(
            !next_is_plain || !p.painted(next_top + 1.0),
            "배경이 다음 일반 문단 `{next}` 의 줄 상자를 침범했다"
        );
    }
}

/// bit 29 켜진 채움 문단은 마지막 줄 상자 아래까지만 칠한다(#1835 제목 배경과 같은 규칙).
#[test]
fn ignore_margin_fill_stops_at_last_line_box() {
    let page_probe = page(&load(PAGE), 0);
    let e1 = page_probe.line("E1");
    let rect = page_probe
        .rect_at(e1.y + e1.h / 2.0)
        .expect("E1 배경이 없다");
    assert!(
        (rect.bottom() - e1.bottom()).abs() <= 0.5,
        "bit 29 채움은 줄 상자 아래({:.1})에서 끝나야 한다 — 배경 아래 {:.1}",
        e1.bottom(),
        rect.bottom()
    );

    let ign = page(&load(IGN), 0);
    // 두 줄 문단: 문단 안의 줄간격은 칠하고 마지막 줄간격은 칠하지 않는다.
    let x1 = ign.line("X1");
    let x2 = ign.line("X2");
    assert!(
        ign.covers(x1.y + 0.5, x1.bottom() + 5.0).is_ok(),
        "두 줄 bit 29 문단의 첫 줄 아래 줄간격이 비었다"
    );
    assert!(
        !ign.painted(x2.y - 2.0),
        "bit 29 문단의 마지막 줄간격을 칠했다"
    );
    // 연결(bit 28)이어도 테두리 없는 채움은 문단 사이 틈을 칠하지 않는다.
    for (a, b) in [("Y1", "Y2"), ("Q1", "Q2")] {
        let gap = (ign.line(a).bottom() + ign.line(b).y) / 2.0;
        assert!(
            !ign.painted(gap),
            "bit 29 채움 문단 `{a}`·`{b}` 사이 틈(y={gap:.1})을 칠했다"
        );
    }
}

/// bit 29 켜진 문단은 좌우 여백 안쪽만, 꺼진 문단은 단 너비 전체를 덮는다.
#[test]
fn ignore_margin_selects_horizontal_extent() {
    let ign = page(&load(IGN), 0);
    let col = ign.column;
    let u1 = ign.line("U1");
    let inset = ign
        .rects
        .iter()
        .find(|r| r.y <= u1.y + 1.0 && u1.y + 1.0 < r.bottom())
        .copied()
        .expect("U1 배경이 없다");
    assert!(
        inset.x > col.x + 10.0 && inset.right() < col.right() - 10.0,
        "bit 29 문단 배경은 좌우 여백 안쪽이어야 한다 (배경 {:.1}..{:.1}, 단 {:.1}..{:.1})",
        inset.x,
        inset.right(),
        col.x,
        col.right()
    );
    // 들여 쓴 글자는 배경 안에 있다.
    assert!(inset.x <= u1.x + 0.5, "U1 글자가 배경 왼쪽 밖에 있다");

    let v1 = ign.line("V1");
    let full = ign.rect_at(v1.y + 1.0).expect("V1 배경이 없다");
    assert!(
        (full.x - col.x).abs() <= 0.5 && (full.right() - col.right()).abs() <= 0.5,
        "bit 29 꺼진 문단 배경은 단 너비 전체여야 한다"
    );
    // 앞·뒤 간격까지 덮어 V1·V2 와 그 사이가 한 띠다.
    let v2 = ign.line("V2");
    let v3 = ign.line("V3");
    if let Err(y) = ign.covers(v1.y - 5.0, v3.y - 0.5) {
        panic!("앞·뒤 간격이 있는 bit 29 꺼진 문단 띠에 틈이 있다 — y={y:.1}");
    }
    assert!(v2.y > v1.bottom());
}

/// 쪽 끝·단 끝에 걸린 채움 문단도 마지막 줄의 줄간격을 칠한다(단 아래 경계까지).
#[test]
fn fill_band_reaches_column_end() {
    let col_probe = page(&load(COL), 0);
    let column_bottom = col_probe.column.bottom();
    let mid_x_left = col_probe.column.x + 10.0;
    let last_left = col_probe
        .lines
        .iter()
        .filter(|(_, b)| b.x < mid_x_left)
        .map(|(_, b)| *b)
        .max_by(|a, b| a.y.total_cmp(&b.y))
        .expect("왼쪽 단 글줄이 없다");
    let rect = col_probe
        .rects
        .iter()
        .filter(|r| r.x < mid_x_left)
        .map(|r| r.bottom())
        .fold(f64::MIN, f64::max);
    let expected = column_bottom.min(last_left.bottom() + 5.0);
    assert!(
        rect >= expected - 0.5,
        "왼쪽 단 끝 채움이 마지막 줄 아래 줄간격을 칠하지 않았다 (배경 아래 {rect:.1}, 기대 ≥ {expected:.1})"
    );
}
