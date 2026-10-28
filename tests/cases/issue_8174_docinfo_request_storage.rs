//! Binary storage contracts; no rendering baseline or private test support API.
#[test]
fn utf16_prefix_and_surrogate_cut_match_independent_wire_units() {
    let text = "A".repeat(65534) + "😀suffix";
    let mut writer = rhwp::serializer::byte_writer::ByteWriter::new();
    writer.write_hwp_string(&text).unwrap();
    let bytes = writer.into_bytes();
    assert_eq!(&bytes[..2], &[0xfe, 0xff]);
    assert_eq!(bytes.len(), 2 + 65534 * 2);
    assert!(bytes[2..].chunks_exact(2).all(|unit| unit == [0x41, 0]));
    let text = "A".repeat(65533) + "😀suffix";
    let mut writer = rhwp::serializer::byte_writer::ByteWriter::new();
    writer.write_hwp_string(&text).unwrap();
    let bytes = writer.into_bytes();
    assert_eq!(&bytes[..2], &[0xff, 0xff]);
    assert_eq!(&bytes[bytes.len() - 4..], &[0x3d, 0xd8, 0, 0xde]);
}

#[test]
fn streaming_record_removal_preserves_extended_records_and_malformed_tails() {
    let keep = rhwp::serializer::record_writer::write_record(
        rhwp::parser::tags::HWPTAG_STYLE,
        1,
        &vec![0x42; 4095],
    );
    let target = rhwp::serializer::record_writer::write_record(
        rhwp::parser::tags::HWPTAG_DISTRIBUTE_DOC_DATA,
        0,
        b"remove",
    );
    for tail in [vec![0xff, 0xff, 0xff], vec![0xff; 8]] {
        let mut stream = [
            keep.as_slice(),
            target.as_slice(),
            keep.as_slice(),
            target.as_slice(),
            tail.as_slice(),
        ]
        .concat();
        let expected = [keep.as_slice(), keep.as_slice(), tail.as_slice()].concat();
        assert_eq!(
            rhwp::serializer::doc_info::surgical_remove_records(
                &mut stream,
                rhwp::parser::tags::HWPTAG_DISTRIBUTE_DOC_DATA
            ),
            2
        );
        assert_eq!(stream, expected);
        assert_eq!(
            rhwp::serializer::doc_info::surgical_remove_records(
                &mut stream,
                rhwp::parser::tags::HWPTAG_DISTRIBUTE_DOC_DATA
            ),
            0
        );
        assert_eq!(stream, expected);
    }
}

#[cfg(feature = "pdf-import")]
#[test]
fn bounded_first_save_refuses_doc_info_payload_preserves_live_ir_and_recovers() {
    use rhwp::{
        parser::pdf::{PdfImportOptions, PdfImportPolicy},
        DocumentCore, PdfImportSession,
    };
    const PDF: &[u8] = include_bytes!("../../pdf/issue8174/fonts/cff-cid.pdf");
    fn edit(core: &mut DocumentCore) {
        core.document_mut().doc_info.font_faces[0][0].name = "가나다😀".repeat(256);
    }
    let mut healthy = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = healthy.open(PDF, &PdfImportOptions::default()).unwrap();
    edit(&mut core);
    let initial = healthy.resource_usage().decoded_bytes;
    let expected = healthy.export_hwp(&mut core, 1024 * 1024).unwrap();
    let full = healthy.resource_usage().decoded_bytes;
    let mut exercised_payload_boundary = false;
    // Discover the first-save boundary through observable request usage, not a
    // hardcoded implementation size. The public resource error must identify
    // payload preflight; an unbounded encoder cannot satisfy this contract.
    for cap in (initial..full).step_by(256) {
        let mut policy = PdfImportPolicy::default();
        policy.backend.decoded_bytes = cap;
        let mut request = PdfImportSession::new(policy).unwrap();
        let Ok(mut core) = request.open(PDF, &PdfImportOptions::default()) else {
            continue;
        };
        edit(&mut core);
        let Err(error) = request.export_hwp(&mut core, 1024 * 1024) else {
            continue;
        };
        if !error.to_string().contains("frontend binary payload bytes") {
            continue;
        }
        let usage = request.resource_usage();
        assert!(usage.failure.is_some());
        assert!(usage.decoded_bytes <= cap);
        assert!(request.export_hwp(&mut core, 1024 * 1024).is_err());
        assert_eq!(request.resource_usage().decoded_bytes, usage.decoded_bytes);
        assert_eq!(core.export_hwp_native().unwrap(), expected);
        exercised_payload_boundary = true;
        break;
    }
    assert!(
        exercised_payload_boundary,
        "first-save payload reservation was not exercised"
    );
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = fresh.open(PDF, &PdfImportOptions::default()).unwrap();
    edit(&mut core);
    assert_eq!(fresh.export_hwp(&mut core, 1024 * 1024).unwrap(), expected);
}
