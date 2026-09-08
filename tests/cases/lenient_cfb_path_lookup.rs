//! [parser] `LenientCfbReader` 가 CFB 스트림을 **경로가 아니라 이름으로** 찾던 회귀.
//!
//! `BodyText/Section0` 과 `ViewText/Section0` 은 **이름이 같다**. 종전 `find_entry_idx` 는
//! 경로의 마지막 세그먼트만 떼어 전체 엔트리에서 첫 일치를 돌려주었으므로, 명시적으로
//! `/BodyText/Section0` 을 물어도 디렉터리에 먼저 나오는 `ViewText/Section0` 이 왔다.
//! (주석의 근거 "HWP에서는 이름이 유일하므로 단순 매칭" 이라는 전제가 성립하지 않는다.)
//!
//! 표본 `samples/issue5169_viewtext_changetracking.hwp` 의 두 스트림:
//!   `ViewText/Section0` = 30,738 B (디렉터리에서 **먼저** 나온다)
//!   `BodyText/Section0` =  6,974 B
//!
//! 계약: 두 경로는 서로 **다른** 스트림으로 풀려야 하고, 각각 제 스토리지 아래의 것이어야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::cfb_reader::LenientCfbReader;

const SAMPLE: &str = "samples/issue5169_viewtext_changetracking.hwp";
const CAP: usize = 64 * 1024 * 1024;
const BODY_LEN: usize = 6_974;
const VIEW_LEN: usize = 30_738;

fn sample() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

#[test]
fn lenient_cfb_resolves_same_named_streams_by_path() {
    let data = sample();
    let lenient = LenientCfbReader::open(&data).expect("lenient open");

    // 표본이 이 회귀를 실제로 태우는지 먼저 확인한다 — 이름이 같은 스트림이 둘이어야 한다.
    let same_named = lenient
        .list_entries()
        .iter()
        .filter(|(name, _, _, _)| name == "Section0")
        .count();
    assert_eq!(
        same_named, 2,
        "표본에 이름이 Section0 인 스트림이 둘이어야 이 테스트가 의미가 있다"
    );

    let body = lenient
        .read_stream_raw_limited("/BodyText/Section0", CAP)
        .expect("/BodyText/Section0 를 읽을 수 있어야 한다");
    let view = lenient
        .read_stream_raw_limited("/ViewText/Section0", CAP)
        .expect("/ViewText/Section0 를 읽을 수 있어야 한다");

    assert_ne!(
        body.len(),
        view.len(),
        "두 경로가 같은 스트림으로 풀렸다 — 이름만 비교하고 있다"
    );
    assert_eq!(
        body.len(),
        BODY_LEN,
        "/BodyText/Section0 가 ViewText 쪽으로 풀렸다"
    );
    assert_eq!(view.len(), VIEW_LEN, "/ViewText/Section0 크기가 다르다");
}

#[test]
fn lenient_body_text_section_reads_bodytext_storage() {
    let data = sample();
    let lenient = LenientCfbReader::open(&data).expect("lenient open");

    let raw = lenient
        .read_body_text_section_raw_limited(0, CAP)
        .expect("BodyText Section0 raw");
    assert_eq!(
        raw.len(),
        BODY_LEN,
        "본문 섹션 읽기가 ViewText 스트림을 집었다"
    );
}
