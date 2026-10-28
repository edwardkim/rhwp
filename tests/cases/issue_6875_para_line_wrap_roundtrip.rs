//! Issue #6875: 문단 "한 줄로 입력"(OWPML `hh:breakSetting@lineWrap`) 이 HWP5↔HWPX 저장에서
//! 사라지지 않는다.
//!
//! HWP5 ParaShape `attr2` bits 0-1 과 OWPML `lineWrap` 은 같은 속성이다. 종전 rhwp 는
//! HWPX 파서가 `lineWrap` 을 읽지 않고 직렬화기가 상수 `BREAK` 를 써서, h2x·x2x·x2h 세 경로
//! 모두에서 `SQUEEZE` 문단이 `BREAK` 로 굳었다. 한/글은 그 문단을 여러 줄로 다시 나눠 쪽수가
//! 갈렸다(s39 코호트 02869 h2x 18→19, 01821 x2h·x2x 36→37).
//!
//! 기대값은 rhwp 구현과 독립인 한컴 쌍둥이에서 온다: `samples/hwpx/mel-001.hwpx` 를 한컴이
//! HWP5 로 저장한 `samples/hwpx/hancom-hwp/mel-001.hwp` 에서 paraPr 358개 중 SQUEEZE 4개
//! (id 117·120·121·156)가 정확히 `attr2 & 3 == 1` 인 ParaShape 4개와 같은 id 다.

use rhwp::document_core::DocumentCore;

const HWPX_SRC: &str = "samples/hwpx/mel-001.hwpx";
const HANCOM_HWP: &str = "samples/hwpx/hancom-hwp/mel-001.hwp";
const SQUEEZE_IDS: [usize; 4] = [117, 120, 121, 156];
const PARA_SHAPE_COUNT: usize = 358;

fn load(path: &str) -> DocumentCore {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path} 읽기 실패: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{path} 열기 실패: {e}"))
}

fn header_xml(hwpx: &[u8]) -> String {
    use std::io::Read;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("zip 열기 실패");
    let mut f = zip
        .by_name("Contents/header.xml")
        .expect("header.xml 이 없다");
    let mut out = String::new();
    f.read_to_string(&mut out).expect("header.xml 읽기");
    out
}

/// header.xml 의 paraPr 를 등장 순서대로 (id, lineWrap) 로 뽑는다.
fn para_pr_line_wraps(header: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut rest = header;
    while let Some(start) = rest.find("<hh:paraPr ") {
        let block_end = rest[start..]
            .find("</hh:paraPr>")
            .expect("paraPr 닫힘 없음");
        let block = &rest[start..start + block_end];
        let id = attr_value(block, "id").expect("paraPr id 없음");
        let bs = block
            .find("<hh:breakSetting")
            .map(|i| &block[i..])
            .expect("breakSetting 없음");
        let wrap = attr_value(bs, "lineWrap").expect("lineWrap 없음");
        out.push((id.parse().expect("paraPr id 숫자"), wrap.to_string()));
        rest = &rest[start + block_end..];
    }
    out
}

fn attr_value<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let i = tag.find(&key)? + key.len();
    let j = tag[i..].find('"')?;
    Some(&tag[i..i + j])
}

fn squeeze_ids_from_hwpx(hwpx: &[u8]) -> Vec<usize> {
    let wraps = para_pr_line_wraps(&header_xml(hwpx));
    assert_eq!(wraps.len(), PARA_SHAPE_COUNT, "paraPr 개수");
    for (_, w) in &wraps {
        assert!(
            matches!(w.as_str(), "BREAK" | "SQUEEZE"),
            "이 표본의 lineWrap 은 BREAK/SQUEEZE 뿐이다: {w}"
        );
    }
    wraps
        .into_iter()
        .filter(|(_, w)| w == "SQUEEZE")
        .map(|(id, _)| id)
        .collect()
}

fn single_line_ids(core: &DocumentCore) -> Vec<usize> {
    let shapes = &core.document().doc_info.para_shapes;
    assert_eq!(shapes.len(), PARA_SHAPE_COUNT, "ParaShape 개수");
    shapes
        .iter()
        .enumerate()
        .filter(|(_, ps)| ps.attr2 & 0x03 != 0)
        .map(|(i, ps)| {
            assert_eq!(
                ps.attr2 & 0x03,
                1,
                "ParaShape {i}: 한 줄로 입력 값은 1(SQUEEZE)"
            );
            i
        })
        .collect()
}

/// 전제: 독립 기준 두 벌이 같은 문단 모양을 가리킨다.
#[test]
fn issue_6875_hancom_twin_agrees_on_single_line_shapes() {
    let src = std::fs::read(HWPX_SRC).expect("HWPX 원본");
    assert_eq!(
        squeeze_ids_from_hwpx(&src),
        SQUEEZE_IDS,
        "한컴 HWPX 의 SQUEEZE"
    );
    assert_eq!(
        single_line_ids(&load(HANCOM_HWP)),
        SQUEEZE_IDS,
        "한컴 HWP5 의 attr2 bits 0-1"
    );
}

/// x2h 축: HWPX 파서가 lineWrap 을 attr2 bits 0-1 로 받고, HWP5 저장 후에도 남는다.
#[test]
fn issue_6875_x2h_keeps_single_line_as_hancom_does() {
    let core = load(HWPX_SRC);
    assert_eq!(single_line_ids(&core), SQUEEZE_IDS, "HWPX 파싱 직후");

    let hwp = core.export_hwp_with_adapter().expect("HWP5 저장");
    let reparsed = DocumentCore::from_bytes(&hwp).expect("HWP5 재파싱");
    assert_eq!(
        single_line_ids(&reparsed),
        single_line_ids(&load(HANCOM_HWP)),
        "x2h 산출이 한컴 HWP5 와 같은 한 줄로 입력 문단을 가져야 한다"
    );
}

/// x2x 축: HWPX 원본의 SQUEEZE 가 되쓴 HWPX 에 그대로 나온다(나머지는 BREAK 유지).
#[test]
fn issue_6875_x2x_keeps_squeeze() {
    let out = load(HWPX_SRC).export_hwpx_native().expect("HWPX 저장");
    assert_eq!(squeeze_ids_from_hwpx(&out), SQUEEZE_IDS);
}

/// h2x 축: 한컴 HWP5 의 attr2 bits 0-1 이 한컴 HWPX 와 같은 SQUEEZE 로 나간다.
#[test]
fn issue_6875_h2x_emits_squeeze_like_hancom() {
    let out = load(HANCOM_HWP).export_hwpx_native().expect("HWPX 저장");
    let src = std::fs::read(HWPX_SRC).expect("HWPX 원본");
    assert_eq!(
        squeeze_ids_from_hwpx(&out),
        squeeze_ids_from_hwpx(&src),
        "h2x 산출이 한컴 HWPX 와 같은 SQUEEZE 문단을 가져야 한다"
    );
}
