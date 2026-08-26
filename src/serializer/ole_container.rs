//! [#4100] 중첩 OLE CFB 의 스트림 하나를 갈아끼운다.
//!
//! 차트 편집은 `OOXMLChartContents` 만 바꾸고 레거시 `Contents` 와 `\x02OlePres000`
//! EMF 프리뷰는 원본 바이트 그대로 되실어야 한다. 그래서 재포장은
//! [`parse_ole_container`](crate::parser::ole_container::parse_ole_container) 가 아니라
//! [`all_ole_streams`](crate::parser::ole_container::all_ole_streams) 전수 열거 위에 선다 —
//! 아는 이름만 뽑아 다시 쓰면 나머지가 소실된다.
//!
//! ## 루트 CLSID 는 반드시 보존한다
//!
//! OLE 개체는 루트 스토리지 엔트리의 CLSID 로 서버를 식별한다. 재포장이 이것을 0 으로
//! 떨구면 한컴이 개체를 알아보지 못해 **틀만 그리고 내용을 비운다.** rhwp 자신의 왕복
//! 검증은 전부 통과하므로 한컴에서만 드러났다 (#4097, `mini_cfb::build_cfb` 가 아니라
//! `build_cfb_with_root_clsid` 를 써야 하는 이유).
//!
//! ## 바뀐 게 없으면 다시 쓰지 않는다
//!
//! 재포장은 **바이트 동일을 보장하지 않는다** — 섹터 배치가 원본 작성기와 다르다.
//! 그래서 새 바이트가 기존과 같으면 원본을 그대로 돌려준다. 이 짧은 회로가 없으면
//! "무편집 왕복이 바이트 동일"(#4100 수용 기준 2)이 재포장만으로 깨진다.

use crate::parser::ole_container::{all_ole_streams, ole_root_clsid};
use crate::serializer::mini_cfb::build_cfb_with_root_clsid;

/// 재포장 실패 사유.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OleRepackError {
    /// 입력이 열리는 CFB 가 아니다.
    NotACompoundFile,
    /// 원본 루트 CLSID 를 읽지 못했다 — CLSID 없이 다시 쓰면 한컴이 개체를 잃는다.
    RootClsidUnreadable,
    /// 지목한 스트림이 없다. **새로 만들지 않는다** — 이름 오타를 조용히 새 스트림으로
    /// 바꾸면 한컴이 못 읽는 파일이 나간다.
    StreamNotFound(String),
    /// CFB 조립 실패.
    Rebuild(String),
}

impl std::fmt::Display for OleRepackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotACompoundFile => write!(f, "중첩 OLE 가 CFB 로 열리지 않는다"),
            Self::RootClsidUnreadable => write!(f, "중첩 CFB 의 루트 CLSID 를 읽지 못했다"),
            Self::StreamNotFound(name) => write!(f, "중첩 CFB 에 스트림 `{name}` 이 없다"),
            Self::Rebuild(msg) => write!(f, "중첩 CFB 재포장 실패: {msg}"),
        }
    }
}

/// 이름 비교용 정규화 — 구분자를 `/` 로 맞추고 앞의 `/` 를 뗀다.
fn normalized(path: &str) -> &str {
    path.trim_start_matches('/')
}

/// 중첩 CFB 의 스트림 하나를 새 바이트로 갈아끼운 CFB 를 만든다.
///
/// 나머지 스트림과 루트 CLSID 는 원본 그대로다. `new_bytes` 가 기존 내용과 같으면
/// **원본 바이트를 그대로** 돌려준다(재포장하지 않는다).
///
/// `stream_name` 은 앞의 `/` 유무를 가리지 않는다 — `OOXMLChartContents` 와
/// `/OOXMLChartContents` 가 같다.
pub fn replace_ole_stream(
    original: &[u8],
    stream_name: &str,
    new_bytes: &[u8],
) -> Result<Vec<u8>, OleRepackError> {
    let mut streams = all_ole_streams(original).ok_or(OleRepackError::NotACompoundFile)?;

    let target = normalized(stream_name);
    let slot = streams
        .iter_mut()
        .find(|(path, _)| normalized(path) == target)
        .ok_or_else(|| OleRepackError::StreamNotFound(stream_name.to_string()))?;

    if slot.1 == new_bytes {
        return Ok(original.to_vec());
    }
    slot.1 = new_bytes.to_vec();

    let clsid = ole_root_clsid(original).ok_or(OleRepackError::RootClsidUnreadable)?;
    let refs: Vec<(&str, &[u8])> = streams
        .iter()
        .map(|(name, data)| (name.as_str(), data.as_slice()))
        .collect();
    build_cfb_with_root_clsid(&refs, clsid).map_err(OleRepackError::Rebuild)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serializer::mini_cfb::build_cfb;

    const CLSID: [u8; 16] = [
        0x37, 0xa1, 0x3d, 0x4c, 0x90, 0xdc, 0xb9, 0x47, 0x9b, 0xed, 0x59, 0xda, 0xe3, 0x52, 0xa2,
        0x80,
    ];

    /// 차트 중첩 CFB 를 흉내 낸 3스트림 컨테이너.
    fn sample() -> Vec<u8> {
        build_cfb_with_root_clsid(
            &[
                ("OOXMLChartContents", b"<c:chartSpace/>".as_slice()),
                ("Contents", b"legacy-grid".as_slice()),
                ("\u{2}OlePres000", b"emf-preview".as_slice()),
            ],
            CLSID,
        )
        .expect("샘플 CFB")
    }

    fn stream(cfb: &[u8], name: &str) -> Option<Vec<u8>> {
        all_ole_streams(cfb)?
            .into_iter()
            .find(|(p, _)| normalized(p) == name)
            .map(|(_, d)| d)
    }

    #[test]
    fn replaces_only_the_named_stream() {
        let original = sample();
        let out = replace_ole_stream(&original, "OOXMLChartContents", b"<c:new/>").expect("재포장");

        assert_eq!(
            stream(&out, "OOXMLChartContents").as_deref(),
            Some(&b"<c:new/>"[..])
        );
        assert_eq!(
            stream(&out, "Contents").as_deref(),
            Some(&b"legacy-grid"[..])
        );
        assert_eq!(
            stream(&out, "\u{2}OlePres000").as_deref(),
            Some(&b"emf-preview"[..])
        );
    }

    #[test]
    fn preserves_the_root_clsid() {
        let original = sample();
        let out = replace_ole_stream(&original, "OOXMLChartContents", b"<c:new/>").expect("재포장");
        assert_eq!(ole_root_clsid(&out), Some(CLSID));
    }

    #[test]
    fn identical_content_is_not_repacked() {
        let original = sample();
        let out = replace_ole_stream(&original, "OOXMLChartContents", b"<c:chartSpace/>")
            .expect("재포장");
        assert_eq!(out, original, "바뀐 게 없으면 원본 바이트 그대로여야 한다");
    }

    /// 재포장은 바이트 동일을 보장하지 않는다 — 짧은 회로가 필요한 이유의 근거.
    #[test]
    fn repacking_is_not_byte_preserving() {
        let original = sample();
        let streams = all_ole_streams(&original).expect("열거");
        let refs: Vec<(&str, &[u8])> = streams
            .iter()
            .map(|(n, d)| (n.as_str(), d.as_slice()))
            .collect();
        let repacked = build_cfb_with_root_clsid(&refs, CLSID).expect("재포장");
        // 같아도 이 테스트는 무해하지만, 다르면 짧은 회로가 필수라는 근거가 된다.
        // 어느 쪽이든 스트림 내용은 살아 있어야 한다.
        assert_eq!(
            stream(&repacked, "Contents").as_deref(),
            Some(&b"legacy-grid"[..])
        );
    }

    #[test]
    fn leading_slash_is_ignored_in_the_name() {
        let original = sample();
        let a =
            replace_ole_stream(&original, "OOXMLChartContents", b"<c:x/>").expect("슬래시 없이");
        let b =
            replace_ole_stream(&original, "/OOXMLChartContents", b"<c:x/>").expect("슬래시 붙여");
        assert_eq!(a, b);
    }

    #[test]
    fn missing_stream_is_refused_not_created() {
        let original = sample();
        assert_eq!(
            replace_ole_stream(&original, "OOXMLChartContent", b"x"),
            Err(OleRepackError::StreamNotFound(
                "OOXMLChartContent".to_string()
            ))
        );
    }

    #[test]
    fn non_cfb_input_is_refused() {
        assert_eq!(
            replace_ole_stream(b"not a cfb at all", "Contents", b"x"),
            Err(OleRepackError::NotACompoundFile)
        );
    }

    /// 루트 CLSID 가 0 인 CFB 도 그대로 0 을 유지한다 — 없는 값을 지어내지 않는다.
    #[test]
    fn zero_root_clsid_stays_zero() {
        let original = build_cfb(&[("Contents", b"x".as_slice())]).expect("CFB");
        let out = replace_ole_stream(&original, "Contents", b"y").expect("재포장");
        assert_eq!(ole_root_clsid(&out), Some([0u8; 16]));
    }
}
