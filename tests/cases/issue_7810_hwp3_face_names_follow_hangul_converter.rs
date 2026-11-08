//! [#7810] HWP3 글꼴 이름은 한/글 변환기(HWP3 → HWP5/HWPX)가 쓰는 이름으로 읽는다.
//!
//! 한/글은 HWP3 의 `신명조`·`중고딕`·`견고딕`·`그래픽`·`궁서` 를 `한양*` HFT 글꼴로
//! 옮긴다. 수정 전 rhwp 는 이를 `HY신명조` 등으로 읽었다. `HY*` 는 이름이 비슷한 별개의
//! TrueType 글꼴이어서, HWPX 로 저장하면 한/글도 그 TrueType 글리프로 그린다. 한/글 2020 으로
//! 출력해 보면 rhwp 가 저장한 hwp3-sample HWPX 의 `“` 는 1.0em 이고, 원본 HWP3 와 한/글 자신의
//! 변환본은 0.446em 이다.
//!
//! 기대값은 저장소에 있는 한/글 변환본의 글꼴 이름 목록이다. 한/글은 변환할 때 기본 글꼴
//! `HCR Batang` 을 덧붙이고 쓰지 않는 글꼴을 빼므로, 비교는 «변환본의 이름이 모두 원본
//! 해석에 있다» 는 방향으로 한다.

use std::collections::BTreeSet;

use rhwp::parser::parse_document;

/// (HWP3 원본, 한/글 변환본)
const PAIRS: [(&str, &str); 8] = [
    ("samples/hwp3-sample.hwp", "samples/hwp3-sample-hwp5.hwp"),
    ("samples/hwp3-sample4.hwp", "samples/hwp3-sample4-hwp5.hwp"),
    ("samples/hwp3-sample5.hwp", "samples/hwp3-sample5-hwpx.hwpx"),
    (
        "samples/hwp3-sample10.hwp",
        "mydocs/pr/assets/issue7774/sql-sample10-regressions-20261015/hwp3-sample10-hwpx.hwpx",
    ),
    (
        "samples/hwp3-sample11.hwp",
        "samples/hwp3-sample11-hwp5.hwp",
    ),
    (
        "samples/hwp3-sample13.hwp",
        "samples/hwp3-sample13-hwp5.hwp",
    ),
    (
        "samples/hwp3-sample14.hwp",
        "samples/hwp3-sample14-hwp5.hwpx",
    ),
    (
        "samples/hwp3-sample19.hwp",
        "samples/hwp3-sample19-hwpx.hwpx",
    ),
];

/// 한/글 변환기가 덧붙이는 기본 글꼴.
const CONVERTER_DEFAULT_FACE: &str = "HCR Batang";

fn face_names(path: &str) -> BTreeSet<String> {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let doc = parse_document(&bytes).unwrap_or_else(|e| panic!("parse {path}: {e:?}"));
    doc.doc_info
        .font_faces
        .iter()
        .flatten()
        .map(|font| font.name.clone())
        .collect()
}

#[test]
fn hwp3_face_names_are_the_names_hangul_converts_to() {
    for (original, converted) in PAIRS {
        let ours = face_names(original);
        let hangul = face_names(converted);
        assert!(
            hangul.iter().any(|name| name.starts_with("한양")),
            "{converted}: 한양 글꼴이 있는 변환본이어야 이 비교가 뜻을 갖는다 — {hangul:?}"
        );
        let missing: Vec<&String> = hangul
            .iter()
            .filter(|name| name.as_str() != CONVERTER_DEFAULT_FACE && !ours.contains(*name))
            .collect();
        assert!(
            missing.is_empty(),
            "{original}: 한/글 변환본 {converted} 의 글꼴 {missing:?} 이 원본 해석에 없다 — rhwp {ours:?}"
        );
        let foreign: Vec<&String> = ours.iter().filter(|name| name.starts_with("HY")).collect();
        assert!(
            foreign.is_empty(),
            "{original}: 한/글 변환본에 없는 HY 글꼴 {foreign:?} 로 읽었다"
        );
    }
}
