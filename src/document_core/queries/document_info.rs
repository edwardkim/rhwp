//! 한컴 파일 > 문서 정보 창이 읽고 쓰는 값 — 문서 요약, 그림 정보, 저작권, 개체 수.
//!
//! 문서 요약은 HWP 의 `\u{5}HwpSummaryInformation`(OLE 속성 집합)과 HWPX 의
//! `Contents/content.hpf` `<opf:metadata>` 두 곳에 있다. 직렬화기는 두 원문을 그대로
//! 싣으므로(HWP 스트림 복사, HWPX metadata splice) 고칠 때는 두 원문을 함께 고친다.
//! 그래야 HWP 로 연 문서를 HWPX 로(또는 거꾸로) 저장해도 요약이 따라간다.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

use crate::document_core::queries::field_query::shape_lists;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::bin_data::BinDataType;
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::shape::ShapeObject;

/// 문서 요약 값. 읽을 때는 모든 칸을 채우고(날짜는 없으면 `None`), 쓸 때는 `Some` 인
/// 칸만 바꾼다. 날짜는 `YYYY-MM-DDTHH:MM:SSZ`(UTC)이고, 쓸 때 빈 문자열은 날짜를 지운다.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DocumentSummary {
    pub title: Option<String>,
    pub subject: Option<String>,
    pub author: Option<String>,
    /// 문서 요약 탭의 '날짜' — 한컴이 사람이 읽는 글로 적는 값(`2010년 2월 6일 토요일 오후 7:09:42`).
    pub date: Option<String>,
    pub keywords: Option<String>,
    /// 문서 요약 탭의 '기타'.
    pub comments: Option<String>,
    pub last_saved_by: Option<String>,
    pub created_at: Option<String>,
    pub last_saved_at: Option<String>,
}

impl DocumentSummary {
    fn merge(&mut self, patch: DocumentSummary) {
        macro_rules! take {
            ($($f:ident),*) => {$(if patch.$f.is_some() { self.$f = patch.$f; })*};
        }
        take!(
            title,
            subject,
            author,
            date,
            keywords,
            comments,
            last_saved_by,
            created_at,
            last_saved_at
        );
    }
}

// HWP5 `HWPPIDSI_*` 속성 번호.
const PID_TITLE: u32 = 0x02;
const PID_SUBJECT: u32 = 0x03;
const PID_AUTHOR: u32 = 0x04;
const PID_KEYWORDS: u32 = 0x05;
const PID_COMMENTS: u32 = 0x06;
const PID_LAST_SAVED_BY: u32 = 0x08;
const PID_CREATED: u32 = 0x0C;
const PID_LAST_SAVED: u32 = 0x0D;
const PID_DATE: u32 = 0x14;
const VT_LPSTR: u32 = 0x1E;
const VT_LPWSTR: u32 = 0x1F;
const VT_FILETIME: u32 = 0x40;

/// 한컴 저장본의 속성 집합 머리(바이트 순서·버전·CLSID·집합 1개·FMTID·구역 위치 48).
/// 요약 스트림이 아예 없는 문서에 새로 만들 때 쓴다.
const SUMMARY_HEADER: [u8; 48] = [
    0xFE, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0xB6, 0xA2, 0x9F, 0x61, 0x10, 0xD4, 0x11,
    0xB4, 0xC6, 0x00, 0x60, 0x97, 0xC0, 0x9D, 0x8C, 0x01, 0x00, 0x00, 0x00, 0x60, 0xB6, 0xA2, 0x9F,
    0x61, 0x10, 0xD4, 0x11, 0xB4, 0xC6, 0x00, 0x60, 0x97, 0xC0, 0x9D, 0x8C, 0x30, 0x00, 0x00, 0x00,
];

fn is_summary_stream(path: &str) -> bool {
    path.trim_start_matches(['/', '\u{5}'])
        .eq_ignore_ascii_case("HwpSummaryInformation")
}

fn u32_at(d: &[u8], off: usize) -> Option<u32> {
    d.get(off..off + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// 속성 집합 하나(구역 하나)를 속성별 원문 값으로 푼 것. 모르는 속성은 바이트 그대로 둔다.
struct PropertySet {
    /// 구역 앞 바이트(머리와 FMTID 표).
    head: Vec<u8>,
    props: Vec<(u32, Vec<u8>)>,
}

impl PropertySet {
    fn parse(data: &[u8]) -> Option<Self> {
        // 집합이 하나일 때만 다룬다 — 둘째 집합의 위치까지 고쳐 쓰지 않는다.
        if data.len() < 48 || data[0..2] != [0xFE, 0xFF] || u32_at(data, 24)? != 1 {
            return None;
        }
        let at = u32_at(data, 44)? as usize;
        let size = u32_at(data, at)? as usize;
        let count = u32_at(data, at + 4)? as usize;
        let table_end = 8usize.checked_add(count.checked_mul(8)?)?;
        if count > 4096 || at.checked_add(size)? > data.len() || table_end > size {
            return None;
        }
        let entries: Vec<(u32, usize)> = (0..count)
            .map(|i| {
                Some((
                    u32_at(data, at + 8 + i * 8)?,
                    u32_at(data, at + 12 + i * 8)? as usize,
                ))
            })
            .collect::<Option<_>>()?;
        let mut starts: Vec<usize> = entries.iter().map(|(_, off)| *off).collect();
        starts.sort_unstable();
        let mut props = Vec::with_capacity(count);
        for (pid, off) in entries {
            // 값의 끝은 다음 값의 시작이다(마지막은 구역 끝).
            let end = starts.iter().copied().find(|s| *s > off).unwrap_or(size);
            if off < table_end || end > size {
                return None;
            }
            props.push((pid, data[at + off..at + end].to_vec()));
        }
        Some(Self {
            head: data[..at].to_vec(),
            props,
        })
    }

    fn get(&self, pid: u32) -> Option<&[u8]> {
        self.props
            .iter()
            .find(|(p, _)| *p == pid)
            .map(|(_, v)| v.as_slice())
    }

    fn set(&mut self, pid: u32, value: Vec<u8>) {
        match self.props.iter_mut().find(|(p, _)| *p == pid) {
            Some((_, v)) => *v = value,
            None => self.props.push((pid, value)),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let table_end = 8 + 8 * self.props.len();
        let mut table = Vec::with_capacity(table_end);
        let mut values = Vec::new();
        for (pid, value) in &self.props {
            table.extend_from_slice(&pid.to_le_bytes());
            table.extend_from_slice(&((table_end + values.len()) as u32).to_le_bytes());
            values.extend_from_slice(value);
            values.resize(values.len().next_multiple_of(4), 0);
        }
        let mut out = self.head.clone();
        out.extend_from_slice(&((table_end + values.len()) as u32).to_le_bytes());
        out.extend_from_slice(&(self.props.len() as u32).to_le_bytes());
        out.extend_from_slice(&table);
        out.extend_from_slice(&values);
        out
    }
}

fn prop_string(value: &[u8]) -> Option<String> {
    let cch = u32_at(value, 4)? as usize;
    match u32_at(value, 0)? {
        VT_LPWSTR => {
            let units: Vec<u16> = value
                .get(8..8 + cch.checked_mul(2)?)?
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .take_while(|u| *u != 0)
                .collect();
            Some(String::from_utf16_lossy(&units))
        }
        VT_LPSTR => {
            let bytes = value.get(8..8 + cch)?;
            let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
            Some(String::from_utf8_lossy(&bytes[..end]).into_owned())
        }
        _ => None,
    }
}

fn prop_filetime(value: &[u8]) -> Option<u64> {
    if u32_at(value, 0)? != VT_FILETIME {
        return None;
    }
    let raw = value.get(4..12)?;
    let ft = u64::from_le_bytes(raw.try_into().ok()?);
    (ft != 0).then_some(ft)
}

fn lpwstr(text: &str) -> Vec<u8> {
    let units: Vec<u16> = text.encode_utf16().chain([0]).collect();
    let mut out = Vec::with_capacity(8 + units.len() * 2);
    out.extend_from_slice(&VT_LPWSTR.to_le_bytes());
    out.extend_from_slice(&(units.len() as u32).to_le_bytes());
    for u in units {
        out.extend_from_slice(&u.to_le_bytes());
    }
    out
}

fn filetime(ft: u64) -> Vec<u8> {
    let mut out = VT_FILETIME.to_le_bytes().to_vec();
    out.extend_from_slice(&ft.to_le_bytes());
    out
}

const SECS_1601_TO_1970: i64 = 11_644_473_600;

/// FILETIME(1601-01-01 UTC 기준 100ns) → `YYYY-MM-DDTHH:MM:SSZ`.
fn filetime_to_iso(ft: u64) -> String {
    let secs = (ft / 10_000_000) as i64 - SECS_1601_TO_1970;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // Howard Hinnant, civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// `YYYY-MM-DDTHH:MM:SSZ` → FILETIME. 꼴이 다르면 `None`.
fn iso_to_filetime(iso: &str) -> Option<u64> {
    let b = iso.as_bytes();
    if b.len() != 20 || b[4] != b'-' || b[7] != b'-' || b[10] != b'T' || b[19] != b'Z' {
        return None;
    }
    let num = |r: std::ops::Range<usize>| iso.get(r)?.parse::<i64>().ok();
    let (y, m, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
    let (hh, mm, ss) = (num(11..13)?, num(14..16)?, num(17..19)?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || hh > 23 || mm > 59 || ss > 60 {
        return None;
    }
    // Howard Hinnant, days_from_civil.
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    let secs = days * 86_400 + hh * 3600 + mm * 60 + ss + SECS_1601_TO_1970;
    u64::try_from(secs).ok().map(|s| s * 10_000_000)
}

fn summary_from_stream(data: &[u8]) -> Option<DocumentSummary> {
    let set = PropertySet::parse(data)?;
    let text = |pid| Some(set.get(pid).and_then(prop_string).unwrap_or_default());
    let time = |pid| set.get(pid).and_then(prop_filetime).map(filetime_to_iso);
    Some(DocumentSummary {
        title: text(PID_TITLE),
        subject: text(PID_SUBJECT),
        author: text(PID_AUTHOR),
        date: text(PID_DATE),
        keywords: text(PID_KEYWORDS),
        comments: text(PID_COMMENTS),
        last_saved_by: text(PID_LAST_SAVED_BY),
        created_at: time(PID_CREATED),
        last_saved_at: time(PID_LAST_SAVED),
    })
}

fn write_summary_stream(original: Option<&[u8]>, summary: &DocumentSummary) -> Vec<u8> {
    let mut set = original
        .and_then(PropertySet::parse)
        .unwrap_or_else(|| PropertySet {
            head: SUMMARY_HEADER.to_vec(),
            props: Vec::new(),
        });
    for (pid, value) in [
        (PID_TITLE, &summary.title),
        (PID_SUBJECT, &summary.subject),
        (PID_AUTHOR, &summary.author),
        (PID_KEYWORDS, &summary.keywords),
        (PID_COMMENTS, &summary.comments),
        (PID_LAST_SAVED_BY, &summary.last_saved_by),
        (PID_DATE, &summary.date),
    ] {
        if let Some(value) = value {
            set.set(pid, lpwstr(value));
        }
    }
    for (pid, value) in [
        (PID_CREATED, &summary.created_at),
        (PID_LAST_SAVED, &summary.last_saved_at),
    ] {
        match value.as_deref() {
            Some("") => set.set(pid, filetime(0)),
            Some(iso) => {
                if let Some(ft) = iso_to_filetime(iso) {
                    set.set(pid, filetime(ft));
                }
            }
            None => {}
        }
    }
    set.to_bytes()
}

fn attr_value(e: &quick_xml::events::BytesStart<'_>, key: &str) -> Option<String> {
    let attr = e.attributes().flatten().find(|a| a.key.as_ref() == key)?;
    let raw = attr.value.as_ref().to_string();
    Some(
        quick_xml::escape::unescape(&raw)
            .map(|v| v.into_owned())
            .unwrap_or(raw),
    )
}

fn metadata_range(hpf: &str) -> Option<(usize, usize)> {
    let open = hpf.find("<opf:metadata")?;
    let rest = &hpf[open..];
    let gt = rest.find('>')?;
    if rest.as_bytes()[gt - 1] == b'/' {
        return Some((open, open + gt + 1));
    }
    let close = rest.find("</opf:metadata>")? + "</opf:metadata>".len();
    Some((open, open + close))
}

/// `<opf:metadata>` 의 값을 `(이름, 값)` 으로 푼다. 제목·언어는 `title`·`language` 다.
fn metadata_entries(block: &str) -> Vec<(String, String)> {
    let mut reader = Reader::from_str(block);
    let mut out: Vec<(String, String)> = Vec::new();
    let mut current: Option<String> = None;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = match e.name().as_ref() {
                    "opf:title" => Some("title".to_string()),
                    "opf:language" => Some("language".to_string()),
                    "opf:meta" => attr_value(&e, "name"),
                    _ => None,
                };
                if let Some(name) = name {
                    out.push((name.clone(), String::new()));
                    current = Some(name);
                }
            }
            Ok(Event::Text(t)) => {
                if let (Some(_), Some(last)) = (&current, out.last_mut()) {
                    last.1.push_str(t.as_ref());
                }
            }
            Ok(Event::GeneralRef(r)) => {
                if let (Some(_), Some(last)) = (&current, out.last_mut()) {
                    last.1
                        .push_str(&crate::parser::hwpx::section::decode_xml_general_ref(&r));
                }
            }
            Ok(Event::End(_)) => current = None,
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    out
}

fn summary_from_metadata(block: &str) -> DocumentSummary {
    let entries = metadata_entries(block);
    let get = |name: &str| {
        entries
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.clone())
    };
    let text = |name: &str| Some(get(name).unwrap_or_default());
    let time = |name: &str| get(name).filter(|v| iso_to_filetime(v).is_some());
    DocumentSummary {
        title: text("title"),
        subject: text("subject"),
        author: text("creator"),
        date: text("date"),
        keywords: text("keyword"),
        comments: text("description"),
        last_saved_by: text("lastsaveby"),
        created_at: time("CreatedDate"),
        last_saved_at: time("ModifiedDate"),
    }
}

/// 한컴 12.30 이 쓰는 차례로 `<opf:metadata>` 를 다시 짓는다. 모르는 `opf:meta` 는 끝에 둔다.
fn write_metadata(original: Option<&str>, summary: &DocumentSummary) -> String {
    let escape = |v: &str| quick_xml::escape::escape(v).into_owned();
    let entries = original.map(metadata_entries).unwrap_or_default();
    let language = entries
        .iter()
        .find(|(n, _)| n == "language")
        .map(|(_, v)| v.clone())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "ko".to_string());
    let value = |v: &Option<String>| v.clone().unwrap_or_default();
    let mut out = String::from("<opf:metadata>");
    let title = value(&summary.title);
    if title.is_empty() {
        out.push_str("<opf:title/>");
    } else {
        // 한컴 12.30 은 앞뒤 공백이 있는 제목에 공백 보존 표시를 단다.
        let space = if title.trim() != title {
            r#" xml:space="preserve""#
        } else {
            ""
        };
        out.push_str(&format!("<opf:title{space}>{}</opf:title>", escape(&title)));
    }
    out.push_str(&format!(
        "<opf:language>{}</opf:language>",
        escape(&language)
    ));
    let known = [
        ("creator", value(&summary.author)),
        ("subject", value(&summary.subject)),
        ("description", value(&summary.comments)),
        ("lastsaveby", value(&summary.last_saved_by)),
        ("CreatedDate", value(&summary.created_at)),
        ("ModifiedDate", value(&summary.last_saved_at)),
        ("date", value(&summary.date)),
        ("keyword", value(&summary.keywords)),
    ];
    let unknown = entries
        .iter()
        .filter(|(n, _)| n != "title" && n != "language" && known.iter().all(|(k, _)| k != n))
        .cloned();
    for (name, v) in known
        .iter()
        .map(|(n, v)| (n.to_string(), v.clone()))
        .chain(unknown)
    {
        if v.is_empty() {
            out.push_str(&format!(
                r#"<opf:meta name="{}" content="text"/>"#,
                escape(&name)
            ));
        } else {
            out.push_str(&format!(
                r#"<opf:meta name="{}" content="text">{}</opf:meta>"#,
                escape(&name),
                escape(&v)
            ));
        }
    }
    out.push_str("</opf:metadata>");
    out
}

const CONTENT_HPF: &str = "Contents/content.hpf";

/// 그림 정보 탭의 한 줄 — 문서에 든 그림 파일(BinData) 하나.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinDataInfo {
    /// BinData 순번(1부터). 그림 개체의 `binDataId` 와 같다.
    pub id: usize,
    /// `embed`(삽입) 또는 `link`(연결).
    pub kind: &'static str,
    /// 연결 그림의 경로. 삽입 그림은 빈 문자열이다.
    pub path: String,
    pub extension: String,
    /// 그 그림을 처음 그리는 쪽(1부터). 어느 쪽에도 그리지 않으면 `None`.
    pub first_page: Option<u32>,
}

/// 문서 통계의 개체 수 — 본문·표 셀·글상자·머리말·꼬리말·바탕쪽에 든 것.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectCount {
    pub tables: u32,
    pub pictures: u32,
    pub text_boxes: u32,
}

fn count_objects(paragraphs: &[Paragraph], n: &mut ObjectCount) {
    for para in paragraphs {
        for ctrl in &para.controls {
            match ctrl {
                Control::Table(table) => {
                    n.tables += 1;
                    for cell in &table.cells {
                        count_objects(&cell.paragraphs, n);
                    }
                }
                Control::Picture(_) => n.pictures += 1,
                Control::Shape(shape) => {
                    if matches!(shape.as_ref(), ShapeObject::Picture(_)) {
                        n.pictures += 1;
                    } else if shape.drawing().and_then(|d| d.text_box.as_ref()).is_some() {
                        n.text_boxes += 1;
                    }
                    for (_, list) in shape_lists(shape) {
                        count_objects(list, n);
                    }
                }
                Control::Header(_) | Control::Footer(_) => {
                    if let Some(list) = ctrl.header_footer_paragraphs() {
                        count_objects(list, n);
                    }
                }
                _ => {}
            }
        }
    }
}

impl DocumentCore {
    /// 문서 요약을 읽는다. HWPX 의 metadata 가 있으면 그것을, 없으면 HWP 요약 스트림을 읽는다.
    pub fn document_summary(&self) -> DocumentSummary {
        let doc = &self.document;
        if let Some(block) = doc
            .hwpx_aux_entry(CONTENT_HPF)
            .and_then(|b| std::str::from_utf8(b).ok())
            .and_then(|hpf| metadata_range(hpf).map(|(s, e)| &hpf[s..e]))
        {
            return summary_from_metadata(block);
        }
        doc.extra_streams
            .iter()
            .find(|(path, _)| is_summary_stream(path))
            .and_then(|(_, data)| summary_from_stream(data))
            .unwrap_or_else(|| summary_from_metadata(""))
    }

    /// 문서 요약을 고친다. `patch` 에서 `Some` 인 칸만 바꾸고 HWP 스트림과 HWPX metadata 를 함께 쓴다.
    pub fn update_document_summary(&mut self, patch: DocumentSummary) {
        let mut summary = self.document_summary();
        summary.merge(patch);
        let doc = &mut self.document;
        let mut wrote = false;
        for (path, data) in doc.extra_streams.iter_mut() {
            if is_summary_stream(path) {
                *data = write_summary_stream(Some(data), &summary);
                wrote = true;
            }
        }
        if !wrote {
            doc.extra_streams.push((
                "/\u{5}HwpSummaryInformation".to_string(),
                write_summary_stream(None, &summary),
            ));
        }
        let entry = doc
            .hwpx_aux_entries
            .iter_mut()
            .find(|(p, _)| p == CONTENT_HPF);
        match entry {
            Some((_, bytes)) => {
                let hpf = String::from_utf8_lossy(bytes).into_owned();
                let rebuilt = match metadata_range(&hpf) {
                    Some((s, e)) => {
                        let meta = write_metadata(Some(&hpf[s..e]), &summary);
                        format!("{}{}{}", &hpf[..s], meta, &hpf[e..])
                    }
                    None => match hpf
                        .find("<opf:package")
                        .and_then(|s| hpf[s..].find('>').map(|g| s + g + 1))
                    {
                        Some(at) => format!(
                            "{}{}{}",
                            &hpf[..at],
                            write_metadata(None, &summary),
                            &hpf[at..]
                        ),
                        None => write_metadata(None, &summary),
                    },
                };
                *bytes = rebuilt.into_bytes();
            }
            // HWP 로 연 문서: HWPX 로 저장할 때 직렬화기가 이 metadata 를 옮겨 싣는다.
            None => doc.hwpx_aux_entries.push((
                CONTENT_HPF.to_string(),
                write_metadata(None, &summary).into_bytes(),
            )),
        }
    }

    /// 그림 정보 탭의 그림 파일 목록. OLE 개체(Storage)는 그림이 아니어서 뺀다.
    pub fn bin_data_infos(&self) -> Vec<BinDataInfo> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        let list = &self.document.doc_info.bin_data_list;
        let mut first: Vec<Option<u32>> = vec![None; list.len()];
        let mut left = list.len();
        fn visit(node: &RenderNode, page: u32, first: &mut [Option<u32>], left: &mut usize) {
            if let RenderNodeType::Image(image) = &node.node_type {
                let slot = usize::from(image.bin_data_id)
                    .checked_sub(1)
                    .and_then(|i| first.get_mut(i));
                if let Some(slot @ None) = slot {
                    *slot = Some(page);
                    *left -= 1;
                }
            }
            for child in &node.children {
                visit(child, page, first, left);
            }
        }
        for page in 0..self.page_count() {
            if left == 0 {
                break;
            }
            let _ = self.with_page_tree_cached(page, |tree| {
                visit(&tree.root, page + 1, &mut first, &mut left);
                Ok(())
            });
        }
        list.iter()
            .enumerate()
            .filter(|(_, bd)| bd.data_type != BinDataType::Storage)
            .map(|(i, bd)| {
                let link = bd.data_type == BinDataType::Link;
                BinDataInfo {
                    id: i + 1,
                    kind: if link { "link" } else { "embed" },
                    path: if link {
                        bd.abs_path
                            .clone()
                            .filter(|p| !p.is_empty())
                            .or_else(|| bd.rel_path.clone())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    },
                    extension: bd.extension.clone().unwrap_or_default(),
                    first_page: first[i],
                }
            })
            .collect()
    }

    /// 저작권 탭의 값 — HWPX `hh:docOption/hh:licensemark`, HWP FileHeader(속성 bit 11 CCL·
    /// bit 15 공공누리, 오프셋 40 조건 비트, 48 언어). 조건 비트는 한컴 12.30 Mac 저장본에서
    /// 쟀다: bit 0 영리 불허, bit 1 변경 불허, bit 2 동일 조건 변경 허락. 없으면 `None`.
    pub fn license_mark(&self) -> Option<(String, u32, u32)> {
        let doc = &self.document;
        if let Some(tail) = &doc.doc_info.hwpx_head_tail {
            let mut reader = Reader::from_str(tail);
            loop {
                match reader.read_event() {
                    Ok(Event::Start(e)) | Ok(Event::Empty(e))
                        if e.name().as_ref() == "hh:licensemark" =>
                    {
                        let num = |key: &str| {
                            attr_value(&e, key)
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(0)
                        };
                        return Some((attr_value(&e, "type")?, num("flag"), num("lang")));
                    }
                    Ok(Event::Eof) | Err(_) => return None,
                    _ => {}
                }
            }
        }
        let raw = doc.header.raw_data.as_deref()?;
        let props = u32_at(raw, 36)?;
        let kind = if props & (1 << 11) != 0 {
            "CCL"
        } else if props & (1 << 15) != 0 {
            "KOGL"
        } else {
            return None;
        };
        Some((kind.to_string(), u32_at(raw, 40)?, u32_at(raw, 48)?))
    }

    /// 문서 통계의 표·그림·글상자 수 — 본문과 그 속(표 셀·글상자·캡션), 머리말·꼬리말, 바탕쪽.
    pub fn object_count(&self) -> ObjectCount {
        let mut n = ObjectCount::default();
        for section in &self.document.sections {
            count_objects(&section.paragraphs, &mut n);
            for master in &section.section_def.master_pages {
                count_objects(&master.paragraphs, &mut n);
            }
        }
        n
    }

    pub fn document_summary_json(&self) -> String {
        serde_json::to_string(&self.document_summary()).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn set_document_summary_json(&mut self, json: &str) -> Result<String, HwpError> {
        let patch: DocumentSummary =
            serde_json::from_str(json).map_err(|e| HwpError::InvalidField(e.to_string()))?;
        for date in [&patch.created_at, &patch.last_saved_at]
            .into_iter()
            .flatten()
        {
            if !date.is_empty() && iso_to_filetime(date).is_none() {
                return Err(HwpError::InvalidField(format!("날짜 꼴이 아님: {date}")));
            }
        }
        self.update_document_summary(patch);
        Ok("{\"ok\":true}".to_string())
    }

    pub fn bin_data_infos_json(&self) -> String {
        serde_json::to_string(&self.bin_data_infos()).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn license_mark_json(&self) -> String {
        match self.license_mark() {
            Some((kind, flag, lang)) => {
                serde_json::json!({ "type": kind, "flag": flag, "lang": lang }).to_string()
            }
            None => "null".to_string(),
        }
    }

    pub fn object_count_json(&self) -> String {
        serde_json::to_string(&self.object_count()).unwrap_or_else(|_| "{}".to_string())
    }
}
