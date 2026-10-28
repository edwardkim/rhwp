//! HWP5/HWPX에서 읽는 저장 프로그램 메타데이터.
//!
//! HWP5는 `HwpSummaryInformation.revisionNumber`, HWPX는
//! `version.xml`의 `appVersion`을 사용한다. 둘 다 사용자가 지우거나 바꿀 수 있으므로
//! 문서의 원 작성 도구가 아니라 **마지막 저장 도구의 메타데이터**로만 취급한다.

use quick_xml::events::Event;
use quick_xml::Reader;

/// `HwpSummaryInformation`의 `PIDSI_REVNUMBER` 속성 ID.
const REVISION_NUMBER_PID: u32 = 0x0000_0009;
const VT_LPSTR: u32 = 0x001E;
const VT_LPWSTR: u32 = 0x001F;

/// HWP5 요약 정보가 가리키는 마지막 저장 한컴오피스 제품.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HancomOfficeSaveVersion {
    /// `HwpSummaryInformation.revisionNumber`에서 읽은 네 자리 빌드 버전.
    pub version: String,
    /// 알려진 한컴오피스 연도. 매핑 근거가 없는 주 버전은 `None`이다.
    pub product: Option<&'static str>,
}

/// HWP5 extra stream에서 마지막 저장 제품 메타데이터를 읽는다.
///
/// 요약 정보가 없거나 손상됐거나 version 형식을 해석하지 못하면 `None`이다. 이 경우
/// 호출자는 저장 제품을 추정하지 않아야 한다.
pub fn last_saved_with(extra_streams: &[(String, Vec<u8>)]) -> Option<HancomOfficeSaveVersion> {
    let summary = extra_streams
        .iter()
        .find(|(path, _)| {
            path.trim_start_matches('/') == "\u{5}HwpSummaryInformation"
                || path.trim_start_matches(['/', '\u{5}']) == "HwpSummaryInformation"
        })
        .map(|(_, data)| data.as_slice())?;
    let revision_number = summary_property_string(summary, REVISION_NUMBER_PID)?;
    save_version(&revision_number)
}

/// HWPX 보조 엔트리의 `version.xml/appVersion`에서 마지막 저장 제품을 읽는다.
///
/// HWPX 파서는 `version.xml` 원문을 라운드트립용 보조 엔트리에 보존한다. 속성이
/// 없거나 XML·버전 문자열을 해석하지 못하면 `None`이며 제품 연도를 추정하지 않는다.
pub fn hwpx_last_saved_with(
    hwpx_aux_entries: &[(String, Vec<u8>)],
) -> Option<HancomOfficeSaveVersion> {
    let version_xml = hwpx_aux_entries
        .iter()
        .find(|(path, _)| path.trim_start_matches('/') == "version.xml")
        .map(|(_, data)| data.as_slice())?;
    let app_version = hwpx_app_version(version_xml)?;
    save_version(&app_version)
}

fn hwpx_app_version(data: &[u8]) -> Option<String> {
    let xml = std::str::from_utf8(data).ok()?;
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(element)) | Ok(Event::Empty(element)) => {
                let qualified_name = element.name();
                let name = local_name(qualified_name.as_ref());
                if name != b"HCFVersion" && name != b"version" {
                    buf.clear();
                    continue;
                }
                for attr in element.attributes() {
                    let attr = attr.ok()?;
                    if local_name(attr.key.as_ref()) == b"appVersion" {
                        let raw = attr.value.as_ref();
                        return Some(
                            quick_xml::escape::unescape(&raw)
                                .map(|value| value.into_owned())
                                .unwrap_or_else(|_| raw.to_owned()),
                        );
                    }
                }
                return None;
            }
            Ok(Event::Eof) | Err(_) => return None,
            _ => {}
        }
        buf.clear();
    }
}

fn local_name(name: &str) -> &[u8] {
    name.rsplit(':').next().unwrap_or(name).as_bytes()
}

fn save_version(value: &str) -> Option<HancomOfficeSaveVersion> {
    let parts = parse_revision_number(value)?;
    let version = format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], parts[3]);
    let product = match parts[0] {
        8 => Some("hancom-office-2010"),
        10 => Some("hancom-office-2018"),
        11 => Some("hancom-office-2020"),
        12 => Some("hancom-office-2022"),
        13 => Some("hancom-office-2024"),
        _ => None,
    };

    Some(HancomOfficeSaveVersion { version, product })
}

fn summary_property_string(data: &[u8], target_pid: u32) -> Option<String> {
    if data.get(..2) != Some(&[0xFE, 0xFF]) {
        return None;
    }
    let section_offset = u32_at(data, 44)? as usize;
    let property_count = u32_at(data, section_offset + 4)? as usize;
    if property_count > 4096
        || section_offset.checked_add(8 + property_count.checked_mul(8)?)? > data.len()
    {
        return None;
    }

    for index in 0..property_count {
        let entry_offset = section_offset + 8 + index * 8;
        let pid = u32_at(data, entry_offset)?;
        if pid != target_pid {
            continue;
        }
        let relative_offset = u32_at(data, entry_offset + 4)? as usize;
        let value_offset = section_offset.checked_add(relative_offset)?;
        let value_type = u32_at(data, value_offset)?;
        let length = u32_at(data, value_offset + 4)? as usize;
        let payload_offset = value_offset.checked_add(8)?;

        return match value_type {
            VT_LPWSTR => decode_utf16le(
                data.get(payload_offset..payload_offset.checked_add(length.checked_mul(2)?)?)?,
            ),
            VT_LPSTR => {
                decode_lpstr(data.get(payload_offset..payload_offset.checked_add(length)?)?)
            }
            _ => None,
        };
    }
    None
}

fn u32_at(data: &[u8], offset: usize) -> Option<u32> {
    let bytes: [u8; 4] = data.get(offset..offset.checked_add(4)?)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

fn decode_utf16le(data: &[u8]) -> Option<String> {
    let units = data
        .chunks_exact(2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .take_while(|unit| *unit != 0)
        .collect::<Vec<_>>();
    String::from_utf16(&units).ok()
}

fn decode_lpstr(data: &[u8]) -> Option<String> {
    let bytes = data.split(|byte| *byte == 0).next()?;
    std::str::from_utf8(bytes).ok().map(str::to_owned)
}

fn parse_revision_number(value: &str) -> Option<[u32; 4]> {
    let mut chars = value.chars().peekable();
    let mut parts = [0u32; 4];

    for (index, part) in parts.iter_mut().enumerate() {
        while chars.peek().is_some_and(|ch| ch.is_whitespace()) {
            chars.next();
        }

        let mut digits = String::new();
        while chars.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            digits.push(chars.next()?);
        }
        *part = digits.parse().ok()?;

        while chars.peek().is_some_and(|ch| ch.is_whitespace()) {
            chars.next();
        }
        if index < 3 && chars.next()? != ',' {
            return None;
        }
    }

    Some(parts)
}
