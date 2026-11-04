//! 실제 고용동향 OLE 차트의 29개 막대와 비트맵 브러시를 보존한다.

use rhwp::emf::parser::records::Record;
use std::io::{Cursor, Read};

#[test]
fn official_employment_chart_preserves_all_twenty_nine_bars() {
    let input = include_bytes!(
        "../../mydocs/pr/assets/pr8319_evidence_alignment_20261027/official-inputs/pr8027-official.hwpx"
    );
    let mut zip = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut ole = Vec::new();
    zip.by_name("BinData/ole7.ole")
        .unwrap()
        .read_to_end(&mut ole)
        .unwrap();
    let container = rhwp::parser::ole_container::parse_ole_container(&ole).unwrap();
    let emf = container.preview_emf.unwrap();
    let records = rhwp::emf::parse_emf(&emf).unwrap();
    assert!(
        !records.iter().any(|r| matches!(
            r,
            Record::Unknown {
                record_type: 0x5b | 0x5e,
                ..
            }
        )),
        "막대와 브러시 레코드를 버리지 않음"
    );
    let output = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\">{}</svg>",
        rhwp::emf::convert_to_svg(&emf, (0.0, 0.0, 287.2, 143.6)).unwrap()
    );
    let xml = roxmltree::Document::parse(&output).unwrap();
    let bars = xml
        .descendants()
        .find(|node| {
            node.has_tag_name("path")
                && node
                    .attribute("fill")
                    .is_some_and(|fill| fill.starts_with("url(#"))
                && node
                    .attribute("d")
                    .is_some_and(|d| d.matches('M').count() == 29 && d.matches('Z').count() == 29)
        })
        .expect("독립 원본의 29개 닫힌 막대를 같은 채움으로 보존");
    let id = bars
        .attribute("fill")
        .unwrap()
        .trim_start_matches("url(#")
        .trim_end_matches(')');
    let pattern = xml
        .descendants()
        .find(|node| node.has_tag_name("pattern") && node.attribute("id") == Some(id))
        .unwrap();
    assert!(pattern.children().any(|node| node.has_tag_name("image")
        && node
            .attribute("href")
            .is_some_and(|href| href.starts_with("data:image/png;base64,"))));
    assert!(
        xml.descendants().any(|node| {
            node.has_tag_name("path")
                && node.attribute("fill") == Some("rgb(255,255,255)")
                && node
                    .attribute("d")
                    .is_some_and(|d| d.matches('C').count() >= 4)
        }),
        "선 아래에 가려진 원형 표식의 흰 내부를 보존"
    );
}
