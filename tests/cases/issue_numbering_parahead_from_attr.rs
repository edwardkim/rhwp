//! HWP5 → HWPX 저장에서 문단 번호(numbering) paraHead 의 정렬·번호너비·자동내어쓰기가
//! 하드코딩되어 소실되던 문제(#5309).
//!
//! HWP5 `HWPTAG_NUMBERING` 의 각 수준 '문단 머리 정보'(표 41) `attr` 은 bits0–1=정렬,
//! bit2=번호너비 사용(useInstWidth), bit3=자동 내어쓰기(autoIndent)를 담는다. HWPX
//! 직렬화기는 원본 HWPX paraHead splice 가 없는 HWP5 경유 경로에서 종전에 이 셋을
//! align="LEFT"/useInstWidth="1"/autoIndent="1" 상수로 방출해, 저장할 때마다 문단 번호
//! 정렬·들여쓰기가 기본값으로 리셋됐다(numFormat 은 #2947 에서 이미 de-hardcode).
//!
//! 비트↔토큰은 한컴 저작 HWPX 쌍(samples/143E433F503322BD33.{hwp,hwpx})으로 직접 1:1
//! 대응 확증: attr bit2→useInstWidth, bit3→autoIndent(같은 방향), bits0–1→align.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;

use rhwp::model::document::Document;
use rhwp::model::style::{Numbering, NumberingHead};
use rhwp::serializer::hwpx::serialize_hwpx;

fn head(attr: u32) -> NumberingHead {
    NumberingHead {
        attr,
        ..Default::default()
    }
}

/// 방출된 header.xml 에서 앞 6개 <hh:paraHead> 의 (align, useInstWidth, autoIndent) 추출.
fn para_head_flags(hwpx: &[u8]) -> Vec<(String, String, String)> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("zip");
    let mut xml = String::new();
    zip.by_name("Contents/header.xml")
        .expect("header.xml")
        .read_to_string(&mut xml)
        .expect("read");

    let pick = |tag: &str, key: &str| -> String {
        let needle = format!("{key}=\"");
        tag.find(&needle)
            .map(|s| {
                let v = &tag[s + needle.len()..];
                v[..v.find('"').expect("닫는 따옴표")].to_string()
            })
            .unwrap_or_default()
    };

    let mut out = Vec::new();
    let mut rest = xml.as_str();
    while let Some(p) = rest.find("<hh:paraHead") {
        rest = &rest[p..];
        let end = rest.find('>').expect("태그 끝");
        let tag = &rest[..end + 1];
        out.push((
            pick(tag, "align"),
            pick(tag, "useInstWidth"),
            pick(tag, "autoIndent"),
        ));
        rest = &rest[end + 1..];
        if out.len() == 6 {
            break;
        }
    }
    out
}

#[test]
fn hwp5_numbering_parahead_derived_from_attr_bits() {
    let mut doc = Document::default();
    let mut numbering = Numbering {
        start_number: 1,
        ..Default::default()
    };
    // 수준별 attr 저위 비트를 다양하게 — 각 필드가 상수가 아니라 attr 에서 유도됨을 검증.
    numbering.heads[0] = head(0x00); // align=LEFT,   uiw=0, ai=0
    numbering.heads[1] = head(0x04); // bit2:         uiw=1, ai=0
    numbering.heads[2] = head(0x08); // bit3:         uiw=0, ai=1
    numbering.heads[3] = head(0x01); // bits0-1=1:    align=CENTER
    numbering.heads[4] = head(0x02); // bits0-1=2:    align=RIGHT
    numbering.heads[5] = head(0x0c); // bit2|bit3:    uiw=1, ai=1
    doc.doc_info.numberings.push(numbering);

    let hwpx = serialize_hwpx(&doc).expect("HWPX 직렬화");
    let flags = para_head_flags(&hwpx);

    assert_eq!(
        flags,
        vec![
            ("LEFT".into(), "0".into(), "0".into()),
            ("LEFT".into(), "1".into(), "0".into()),
            ("LEFT".into(), "0".into(), "1".into()),
            ("CENTER".into(), "0".into(), "0".into()),
            ("RIGHT".into(), "0".into(), "0".into()),
            ("LEFT".into(), "1".into(), "1".into()),
        ],
        "paraHead 의 align/useInstWidth/autoIndent 는 head.attr 저위 비트에서 유도돼야 한다"
    );
}
