//! Issue #4599: 어울림(Square) 표 옆 띠에 **접두 줄만** 두고 나머지를 전폭 꼬리로
//! 내보낸 문단에서, 꼬리 줄이 표·접두 줄 위로 되감겨 겹쳐 그려지던 결함의 가드.
//!
//! 재현 문서 `samples/issue4599/156714641_wrap_tail_min.hwpx` 는 코퍼스
//! `156714641_26-2_국내찐고구마품종별맛특성지도로한눈에(식량원).hwpx` 의 1쪽 문단
//! (pi0..pi14)만 남기고 그림을 같은 형식의 작은 그림으로 바꾼 것이다(1쪽 본문 좌표는
//! 원본과 동일함을 `dump-extents` 로 확인).
//!
//! ```text
//! 문단 0.12  빈 host, 표 2x1 wrap=어울림(Square) treat_as_char=false 340.7x205.6px
//! 문단 0.13  9줄 — ls[0..7] cs=25835 sw=22353 (표 오른쪽 띠), ls[7..9] cs=0 sw=48188 (전폭)
//! 조판 항목  Table pi=12 → PartialParagraph pi=13 lines=7..9 (+ WrapAroundPara pi=13 0..7)
//! ```
//!
//! 한/글(한컴 2020 변환 PDF)과 저장 사다리는 꼬리 두 줄을 **표 아래**(접두 마지막 줄
//! 다음 저장 vpos)에 둔다. 수정 전 rhwp 는 `#6778` 옆 레인 판정이 다음 항목의
//! **문단 첫 줄**(띠 안 `cs=25835`)을 보고 전폭 꼬리를 레인으로 오판해, 흐름을 host 줄
//! 높이(띠 시작)로 되감았다 — 꼬리 두 줄이 표와 접두 줄 1·2 위에 겹쳐 그려졌다.
//!
//! 검사는 절대 좌표가 아니라 관계로 한다: 꼬리 줄은 표·접두 줄보다 아래, 다음 문단은
//! 꼬리보다 아래, 줄 수 보존(접두 7 · 꼬리 2).

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue4599/156714641_wrap_tail_min.hwpx";
const HOST_PI: u64 = 12;
const WRAP_PI: u64 = 13;
const NEXT_PI: u64 = 14;

#[derive(Debug, Clone, Copy)]
struct Bbox {
    y: f64,
    w: f64,
    h: f64,
}

impl Bbox {
    fn bottom(self) -> f64 {
        self.y + self.h
    }
}

fn bbox(node: &serde_json::Value) -> Option<Bbox> {
    let b = node.get("bbox")?;
    Some(Bbox {
        y: b.get("y")?.as_f64()?,
        w: b.get("w")?.as_f64()?,
        h: b.get("h")?.as_f64()?,
    })
}

fn collect(node: &serde_json::Value, kind: &str, pi: u64, out: &mut Vec<Bbox>) {
    if node.get("type").and_then(|t| t.as_str()) == Some(kind)
        && node.get("pi").and_then(|p| p.as_u64()) == Some(pi)
    {
        if let Some(b) = bbox(node) {
            out.push(b);
        }
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        collect(child, kind, pi, out);
    }
}

#[test]
fn square_wrap_full_width_tail_is_drawn_below_table_and_prefix_lines() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));
    let json = document
        .get_page_render_tree(0)
        .expect("render tree page 0");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

    let mut tables = Vec::new();
    collect(&tree, "Table", HOST_PI, &mut tables);
    assert_eq!(
        tables.len(),
        1,
        "1쪽에 pi={HOST_PI} 어울림 표가 하나 있어야 한다"
    );
    let table = tables[0];

    let mut wrap_lines = Vec::new();
    collect(&tree, "TextLine", WRAP_PI, &mut wrap_lines);
    // 띠 안 접두 줄은 표 오른쪽 좁은 폭, 꼬리는 단 전폭이다.
    let (prefix, tail): (Vec<Bbox>, Vec<Bbox>) =
        wrap_lines.iter().partition(|b| b.w < table.w * 1.2);
    assert_eq!(
        (prefix.len(), tail.len()),
        (7, 2),
        "pi={WRAP_PI} 는 접두 7줄(띠)과 전폭 꼬리 2줄을 모두 한 번씩 그려야 한다: {wrap_lines:?}"
    );

    let prefix_bottom = prefix.iter().map(|b| b.bottom()).fold(f64::MIN, f64::max);
    for (i, line) in tail.iter().enumerate() {
        assert!(
            line.y >= table.bottom() - 0.5,
            "전폭 꼬리 {i} (y={:.1}) 가 어울림 표(바닥 {:.1}) 위에 겹친다 — #4599 회귀: \
             옆 레인 판정이 꼬리의 첫 줄이 아니라 문단 첫 줄(띠 안)을 보고 흐름을 되감았다",
            line.y,
            table.bottom()
        );
        assert!(
            line.y >= prefix_bottom - 0.5,
            "전폭 꼬리 {i} (y={:.1}) 가 띠 안 접두 줄(바닥 {prefix_bottom:.1})보다 위에 있다",
            line.y
        );
    }
    let tail_bottom = tail.iter().map(|b| b.bottom()).fold(f64::MIN, f64::max);

    let mut next_lines = Vec::new();
    collect(&tree, "TextLine", NEXT_PI, &mut next_lines);
    assert_eq!(
        next_lines.len(),
        1,
        "다음 문단 pi={NEXT_PI} 줄이 1쪽에 있어야 한다"
    );
    assert!(
        next_lines[0].y >= tail_bottom - 0.5,
        "다음 문단(y={:.1})이 꼬리 끝({tail_bottom:.1})보다 위에 있다",
        next_lines[0].y
    );
}

/// 같은 표본 1쪽 머리: pi1 은 저장 줄 하나(`lh 3448 = 표 2882 + 바깥여백 566`,
/// `ls -600`)에 글자처럼취급 표 하나를 싣는다. 저장 사다리는 다음 문단을
/// `vpos + lh + ls` 에 두고(한/글 2020 PDF 도 같은 자리), 조판도 그 값을 쓴다.
/// 수정 전 렌더는 표 바닥까지 전진해 뒤 본문 전체를 4.2px 내렸다.
/// 관계 검사: pi1 표 윗변 → pi3 표 윗변 간격 = 저장 vpos 차(두 표 바깥여백 같음).
#[test]
fn single_stored_line_with_negative_spacing_advances_by_stored_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));
    let json = document
        .get_page_render_tree(0)
        .expect("render tree page 0");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

    let table_top = |pi: u64| {
        let mut found = Vec::new();
        collect(&tree, "Table", pi, &mut found);
        assert_eq!(found.len(), 1, "1쪽에 pi={pi} 표가 하나 있어야 한다");
        found[0].y
    };
    // 저장 LineSeg: pi1 vpos 5699, pi3 vpos 9027 (HWPUNIT, 75 = 1px @96dpi).
    let stored_gap_px = (9027.0 - 5699.0) / 75.0;
    let rendered_gap_px = table_top(3) - table_top(1);
    assert!(
        (rendered_gap_px - stored_gap_px).abs() <= 1.0,
        "pi1→pi3 표 윗변 간격 {rendered_gap_px:.1}px 가 저장 사다리 {stored_gap_px:.1}px 와 \
         다르다 — #4599 회귀: 음수 줄간격 저장 줄을 표 바닥까지 전진시켰다"
    );
}
