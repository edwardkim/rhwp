//! HWP5 취소선 판정 회귀 가드 — HWPX 경로와의 자기정합.
//!
//! 한컴은 취소선이 없는 문자에도 CharShape `attr` 의 취소선 비트(bit 18-20)에 1 을
//! 기본값으로 넣는다. 그래서 비트만으로는 취소선 유무를 판정할 수 없고, 실제
//! 판별자는 취소선 모양(bit 26-29, 표 27 선 종류)이다. 취소선이 없으면 모양에
//! 선 종류가 아닌 placeholder(15 등)가 들어온다 — HWPX 의 `shape="3D"` 와 같다.
//!
//! HWPX 경로는 PR #154 에서 이미 모양 화이트리스트(`is_real_strike_shape`)로
//! 전환했으나 HWP5 경로는 `strikethrough_bits > 1` 로 남아 있었다. 그 결과
//! 같은 문서를 HWP5 로 열면 취소선이 사라지고 HWPX 로 열면 그려졌다.
//!
//! fixture: `samples/issue1949_giant_cell_nested_tables_perf.{hwp,hwpx}` — 같은
//! 문서의 두 포맷 쌍. 취소선 CharShape 4개(모양 SOLID 2, DOUBLE_SLIM 2)를 가진다.
//! 수정 전: HWP5 파스 결과 취소선 0개 / HWPX 파스 결과 4개.

use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

const HWP: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwp";
const HWPX: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwpx";

/// (char_shape 인덱스, 취소선 모양) 목록.
fn strikeout_shapes(path: &str) -> Vec<(usize, u8)> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let bytes = fs::read(&p).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let doc = HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {path}: {e:?}"));
    doc.document()
        .doc_info
        .char_shapes
        .iter()
        .enumerate()
        .filter(|(_, cs)| cs.strikethrough)
        .map(|(index, cs)| (index, cs.strike_shape))
        .collect()
}

/// 같은 문서의 HWP5 와 HWPX 파스가 같은 취소선 집합을 낸다.
#[test]
fn hwp5_strikeout_matches_hwpx_counterpart() {
    let hwpx = strikeout_shapes(HWPX);
    assert!(
        !hwpx.is_empty(),
        "전제 불성립: {HWPX} 에 취소선 CharShape 가 없다"
    );

    let hwp = strikeout_shapes(HWP);
    assert_eq!(
        hwp, hwpx,
        "HWP5 취소선 판정이 HWPX 와 갈렸다. 취소선 비트(bit 18-20)만으로 판정하면 \
         한컴이 기본값으로 넣는 1 을 걸러내려다 실제 취소선(비트=1 + 유효한 모양)까지 \
         버린다. 모양(bit 26-29)이 표 27 선 종류인지 함께 확인해야 한다."
    );
}

/// 취소선으로 인정한 모양은 모두 표 27 선 종류(0..=12)다 — placeholder 는 제외된다.
#[test]
fn strikeout_shapes_are_real_line_types() {
    for (index, shape) in strikeout_shapes(HWP) {
        assert!(
            shape <= 12,
            "char_shape[{index}] 의 취소선 모양 {shape} 은 표 27 선 종류가 아니다 \
             (placeholder 를 취소선으로 오인식 — 본문 전체에 취소선이 그어진다)"
        );
    }
}
