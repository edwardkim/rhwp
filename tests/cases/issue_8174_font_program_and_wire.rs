//! Public font measurements and binary format contracts.
#[cfg(feature = "pdf-import")]
#[test]
fn nimbus_metric_aliases_match_every_independently_embedded_cff_advance() {
    let pdf = pdf_oxide::PdfDocument::from_bytes_with_limits(
        include_bytes!("../../pdf/issue8174/fonts/cff-cid.pdf").to_vec(),
        Default::default(),
    )
    .unwrap();
    let fonts = pdf.extract_embedded_fonts().unwrap();
    assert_eq!(fonts.len(), 1);
    let face = ttf_parser::Face::parse(&fonts[0].1, 0).unwrap();
    for name in [
        "Nimbus Roman",
        "Nimbus Roman Regular",
        "NimbusRoman-Regular",
    ] {
        let selected = rhwp::renderer::font_metrics_data::find_metric(name, false, false).unwrap();
        let metric = selected.metric;
        assert_eq!(metric.em_size, face.units_per_em());
        let mut checked = 0;
        for table in face.tables().cmap.unwrap().subtables {
            if !table.is_unicode() {
                continue;
            }
            table.codepoints(|code| {
                if let Some(ch) = char::from_u32(code) {
                    if let Some(glyph) = face.glyph_index(ch).filter(|id| id.0 != 0) {
                        assert_eq!(
                            metric.get_width(ch),
                            face.glyph_hor_advance(glyph),
                            "U+{code:04X}"
                        );
                        checked += 1;
                    }
                }
            });
        }
        assert!(checked >= 854);
        assert_eq!(metric.get_width(' '), Some(250));
        assert_eq!(metric.get_width('–'), Some(500));
        assert_eq!(metric.get_width('—'), Some(1000));
        assert_eq!(metric.get_width('\u{10ffff}'), None);
    }
}

#[test]
fn public_record_stream_has_independent_small_and_extended_headers() {
    use rhwp::parser::record::Record;
    let input = vec![
        Record {
            tag_id: 17,
            level: 3,
            size: 3,
            data: vec![1, 2, 3],
        },
        Record {
            tag_id: 100,
            level: 7,
            size: 4095,
            data: vec![0x42; 4095],
        },
    ];
    let bytes = rhwp::serializer::record_writer::write_records(&input);
    assert_eq!(&bytes[..4], &(17u32 | (3 << 10) | (3 << 20)).to_le_bytes());
    assert_eq!(
        &bytes[7..11],
        &(100u32 | (7 << 10) | (0xfff << 20)).to_le_bytes()
    );
    assert_eq!(&bytes[11..15], &4095u32.to_le_bytes());
    assert_eq!(bytes.len(), 4 + 3 + 8 + 4095);
    let parsed = Record::read_all(&bytes).unwrap();
    assert_eq!(parsed.len(), input.len());
    for (actual, expected) in parsed.iter().zip(input) {
        assert_eq!(
            (actual.tag_id, actual.level, actual.size),
            (expected.tag_id, expected.level, expected.size)
        );
        assert_eq!(actual.data, expected.data);
    }
}

#[test]
fn public_cfb_keeps_standard_paths_and_mini_and_regular_streams() {
    use std::io::{Cursor, Read};
    let data = [vec![0x31; 96], vec![0x42; 8192]];
    let names = ["/A/small", "/large"];
    let streams: Vec<_> = names
        .iter()
        .zip(&data)
        .map(|(n, v)| (*n, v.as_slice()))
        .collect();
    let bytes = rhwp::serializer::mini_cfb::build_cfb_with_root_clsid(&streams, [7; 16]).unwrap();
    let mut cfb = cfb::CompoundFile::open(Cursor::new(bytes)).unwrap();
    assert_eq!(cfb.root_entry().clsid().as_bytes(), &[7; 16]);
    for (name, expected) in names.iter().zip(&data) {
        let mut actual = Vec::new();
        cfb.open_stream(name)
            .unwrap()
            .read_to_end(&mut actual)
            .unwrap();
        assert_eq!(&actual, expected, "{name}");
    }
}

#[cfg(feature = "pdf-import")]
#[test]
fn public_first_save_refuses_final_cfb_backing_before_allocation_and_recovers() {
    use rhwp::{parser::pdf::PdfImportPolicy, PdfImportSession};
    const PDF: &[u8] = include_bytes!("../../pdf/issue8174/fonts/cff-cid.pdf");
    let mut baseline = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = baseline.open(PDF, &Default::default()).unwrap();
    let expected = baseline.export_hwp(&mut core, 1024 * 1024).unwrap();
    let mut limited = PdfImportPolicy::default();
    limited.backend.decoded_bytes = baseline.resource_usage().decoded_bytes - 1;
    let mut request = PdfImportSession::new(limited).unwrap();
    let mut core = request.open(PDF, &Default::default()).unwrap();
    assert!(request.export_hwp(&mut core, 1024 * 1024).is_err());
    let usage = request.resource_usage();
    let failure = usage.failure.unwrap();
    assert_eq!(failure.requested, Some(expected.len() as u64));
    assert!(usage.decoded_bytes <= limited.backend.decoded_bytes);
    assert!(request.export_hwp(&mut core, 1024 * 1024).is_err());
    assert_eq!(core.export_hwp_native().unwrap(), expected);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = fresh.open(PDF, &Default::default()).unwrap();
    assert_eq!(fresh.export_hwp(&mut core, 1024 * 1024).unwrap(), expected);
}
