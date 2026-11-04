//! HWP adapter metadata wire compatibility, without renderer coordinates or goldens.
use rhwp::{
    document_core::converters::hwpx_to_hwp::convert_if_hwpx_source,
    model::{
        control::Control,
        document::{Document, Section},
        image::Picture,
        paragraph::Paragraph,
    },
    parser::FileFormat,
};

#[test]
fn picture_href_keeps_parameter_packet_escape_and_utf16_cut_contract() {
    let hrefs = [
        "https://example.test/a?x=1".to_owned(),
        r"https\://example.test/a/https://second.test".to_owned(),
        "file:///한글/그림😀.png".to_owned(),
        "://://".to_owned(),
        "relative/그림.png".to_owned(),
        "A".repeat(65534) + "😀tail",
        "A".repeat(70000),
    ];
    for href in hrefs {
        let mut picture = Picture::default();
        picture.href = Some(href.clone());
        let mut doc = Document::default();
        doc.sections.push(Section {
            paragraphs: vec![Paragraph {
                controls: vec![Control::Picture(Box::new(picture))],
                ..Default::default()
            }],
            ..Default::default()
        });
        convert_if_hwpx_source(&mut doc, FileFormat::Pdf);
        let paragraph = &doc.sections[0].paragraphs[0];
        let index = paragraph
            .controls
            .iter()
            .position(|c| matches!(c, Control::Picture(_)))
            .unwrap();
        let actual = paragraph.ctrl_data_records[index].as_deref().unwrap();
        // Independent existing ParameterSet contract: nested set IDs then a
        // length-prefixed UTF-16 string. The final high surrogate is preserved
        // when the u16-length limit cuts an astral character in half.
        let normalized = if href.contains(r"\://") {
            href.clone()
        } else {
            href.replace("://", r"\://")
        };
        let units: Vec<_> = normalized.encode_utf16().take(u16::MAX as usize).collect();
        let mut expected = Vec::new();
        for field in [
            0x021bu16,
            1,
            0,
            0x026f,
            0x8000,
            0x026f,
            1,
            0,
            0x0265,
            1,
            units.len() as u16,
        ] {
            expected.extend_from_slice(&field.to_le_bytes());
        }
        for unit in units {
            expected.extend_from_slice(&unit.to_le_bytes());
        }
        assert_eq!(actual, expected, "href packet length {}", href.len());
    }
}

#[test]
fn bin_order_retains_first_content_invalid_tail_and_lazy_resolution() {
    use rhwp::model::{
        bin_data::{BinData, BinDataBytes, BinDataContent, BinDataResolver, BinDataType},
        shape::{OleShape, ShapeObject},
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    #[derive(Debug)]
    struct Resolver(Arc<AtomicUsize>);
    impl BinDataResolver for Resolver {
        fn resolve(&self, _: &str) -> Vec<u8> {
            self.0.fetch_add(1, Ordering::SeqCst);
            vec![0x22]
        }
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let mut doc = Document::default();
    doc.doc_info.bin_data_list = (1..=3)
        .map(|id| BinData {
            storage_id: id,
            data_type: if id == 1 {
                BinDataType::Storage
            } else {
                BinDataType::Embedding
            },
            extension: Some(format!("kind{id}")),
            ..Default::default()
        })
        .collect();
    let mut controls = Vec::new();
    let mut first = Picture::default();
    first.image_attr.bin_data_id = 3;
    controls.push(Control::Picture(Box::new(first)));
    controls.push(Control::Shape(Box::new(ShapeObject::Ole(Box::new(
        OleShape {
            bin_data_id: 1,
            ..Default::default()
        },
    )))));
    let mut last = Picture::default();
    last.image_attr.bin_data_id = 2;
    controls.push(Control::Picture(Box::new(last)));
    doc.sections.push(Section {
        paragraphs: vec![Paragraph {
            controls,
            ..Default::default()
        }],
        ..Default::default()
    });
    let loaded = |id, byte| BinDataContent {
        id,
        extension: "bin".into(),
        data: BinDataBytes::Loaded(vec![byte]),
    };
    doc.bin_data_content = vec![
        loaded(3, 0x33),
        loaded(3, 0xff),
        loaded(99, 0x99),
        loaded(1, 0x11),
        loaded(0, 0),
        BinDataContent {
            id: 2,
            extension: "lazy".into(),
            data: BinDataBytes::Lazy {
                resolver: Arc::new(Resolver(calls.clone())),
                key: "asset".into(),
            },
        },
    ];
    let report = convert_if_hwpx_source(&mut doc, FileFormat::Hwpx);
    assert_eq!(report.bin_data_order_materialized, 1);
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "adapter must not resolve lazy bytes"
    );
    assert_eq!(
        doc.doc_info
            .bin_data_list
            .iter()
            .map(|b| b.extension.as_deref().unwrap())
            .collect::<Vec<_>>(),
        ["kind3", "kind1", "kind2"]
    );
    assert_eq!(
        doc.doc_info
            .bin_data_list
            .iter()
            .map(|b| b.storage_id)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert_eq!(
        doc.bin_data_content
            .iter()
            .map(|c| (c.id, c.data.load()))
            .collect::<Vec<_>>(),
        [
            (1, vec![0x33]),
            (2, vec![0x11]),
            (3, vec![0x22]),
            (99, vec![0x99]),
            (0, vec![0])
        ]
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let controls = &doc.sections[0].paragraphs[0].controls;
    let references: Vec<_> = controls
        .iter()
        .filter_map(|c| match c {
            Control::Picture(p) => Some(p.image_attr.bin_data_id as u32),
            Control::Shape(s) => match s.as_ref() {
                ShapeObject::Ole(o) => Some(o.bin_data_id),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(references, [1, 2, 3]);
}
