//! 메모 댓글 관계(HWP `MemoExtended` 스트림, HWPX `Contents/memoExtended.xml`).
//!
//! 한컴 12.30 Mac에서 메모의 댓글은 같은 기준 글에 단 메모이고, 댓글과 부모 메모의 필드 번호
//! 쌍을 따로 적는다(댓글을 단 차례).
//! - HWP: 레코드 하나(태그 0x65, 수준 0) = u32 개수 + (댓글 번호 u32, 부모 번호 u32) 쌍. 스트림은
//!   파일의 압축 여부를 따른다.
//! - HWPX: `<he:memosEx xmlns:he="http://www.hancom.co.kr/hwpml/2021/extended">` 안의
//!   `<he:memo id="댓글" parentId="부모"/>`. content.hpf에 `memoextended` 항목으로 올린다.

use super::control::{Control, FieldType};
use super::document::Document;
use super::identity::walk::{walk_ref, RefNode};
use std::collections::HashSet;

/// HWP 스트림 경로.
pub const HWP_STREAM: &str = "/MemoExtended";
/// HWPX 파트 경로.
pub const HWPX_ENTRY: &str = "Contents/memoExtended.xml";
/// HWPX 파트를 읽을 때의 상한.
pub const MAX_BYTES: usize = 4 * 1024 * 1024;
const TAG: u32 = 0x65;

/// HWP 스트림(풀어 쓴 레코드)을 읽는다. 형식이 다르면 None이다.
pub fn decode_hwp(data: &[u8]) -> Option<Vec<(u32, u32)>> {
    let header = u32::from_le_bytes(data.get(..4)?.try_into().ok()?);
    let (tag, size) = (header & 0x3FF, header >> 20);
    let (size, body): (usize, usize) = if size == 0xFFF {
        (
            u32::from_le_bytes(data.get(4..8)?.try_into().ok()?) as usize,
            8,
        )
    } else {
        (size as usize, 4)
    };
    let record = data.get(body..body.checked_add(size)?)?;
    if tag != TAG || record.len() < 4 {
        return None;
    }
    let at = |i: usize| -> Option<u32> {
        Some(u32::from_le_bytes(record.get(i..i + 4)?.try_into().ok()?))
    };
    let count = at(0)? as usize;
    if record.len() != 4 + count.checked_mul(8)? {
        return None;
    }
    (0..count)
        .map(|i| Some((at(4 + i * 8)?, at(8 + i * 8)?)))
        .collect()
}

/// HWP 스트림(압축 전 레코드).
pub fn encode_hwp(links: &[(u32, u32)]) -> Vec<u8> {
    let size = 4 + links.len() * 8;
    let mut data = Vec::with_capacity(size + 8);
    if size < 0xFFF {
        data.extend_from_slice(&(TAG | ((size as u32) << 20)).to_le_bytes());
    } else {
        data.extend_from_slice(&(TAG | (0xFFF << 20)).to_le_bytes());
        data.extend_from_slice(&(size as u32).to_le_bytes());
    }
    data.extend_from_slice(&(links.len() as u32).to_le_bytes());
    for (id, parent) in links {
        data.extend_from_slice(&id.to_le_bytes());
        data.extend_from_slice(&parent.to_le_bytes());
    }
    data
}

/// HWPX 파트의 `<he:memo id parentId>` 쌍. 숫자가 아닌 항목은 건너뛴다.
pub fn decode_hwpx(xml: &str) -> Vec<(u32, u32)> {
    let attr = |tag: &str, name: &str| -> Option<u32> {
        let start = tag.find(&format!(" {name}=\""))? + name.len() + 3;
        tag[start..].split('"').next()?.parse().ok()
    };
    xml.split("<he:memo ")
        .skip(1)
        .filter_map(|rest| {
            let tag = format!(" {}", rest.split('>').next()?);
            Some((attr(&tag, "id")?, attr(&tag, "parentId")?))
        })
        .collect()
}

/// HWPX 파트. 한컴과 같은 머리말을 쓴다.
pub fn encode_hwpx(links: &[(u32, u32)]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes" ?><he:memosEx xmlns:he="http://www.hancom.co.kr/hwpml/2021/extended">"#,
    );
    for (id, parent) in links {
        xml.push_str(&format!(r#"<he:memo id="{id}" parentId="{parent}"/>"#));
    }
    xml.push_str("</he:memosEx>");
    xml
}

/// 문서에 있는 메모 필드 번호.
pub fn memo_field_ids(doc: &Document) -> HashSet<u32> {
    let mut ids = HashSet::new();
    for section in &doc.sections {
        let Ok(()) = walk_ref(&section.paragraphs, &crate::storage::Unbounded, |node| {
            if let RefNode::Control(Control::Field(field)) = node {
                if field.field_type == FieldType::Memo || field.command.starts_with("MEMO/") {
                    ids.insert(field.field_id);
                }
            }
            Ok(())
        });
    }
    ids
}

/// 저장할 관계. 댓글이나 부모 메모가 지워진 관계는 버린다.
pub fn live_links(doc: &Document) -> Vec<(u32, u32)> {
    if doc.memo_parents.is_empty() {
        return Vec::new();
    }
    let ids = memo_field_ids(doc);
    doc.memo_parents
        .iter()
        .copied()
        .filter(|(id, parent)| ids.contains(id) && ids.contains(parent))
        .collect()
}
