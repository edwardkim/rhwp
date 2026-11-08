//! Synthetic font/resource contracts, not page geometry or Hancom goldens.
#![cfg(feature = "pdf-import")]

use pdf_oxide::{
    fonts::FontInfo,
    import_limits::{ImportBudget, PdfImportLimits},
    PdfDocument,
};

fn pdf(font: &[u8], additional: &[Vec<u8>]) -> Vec<u8> {
    let mut objects = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 240 200] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>".to_vec(),
        stream(b"BT /F1 12 Tf 10 150 Td (ABa) Tj ET"), font.to_vec(),
    ];
    objects.extend_from_slice(additional);
    let mut data = b"%PDF-1.7\n".to_vec();
    let mut offsets = vec![0];
    for (i, object) in objects.iter().enumerate() {
        offsets.push(data.len());
        data.extend(format!("{} 0 obj\n", i + 1).as_bytes());
        data.extend(object);
        data.extend(b"\nendobj\n");
    }
    let start = data.len();
    data.extend(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes());
    for offset in &offsets[1..] {
        data.extend(format!("{offset:010} 00000 n \n").as_bytes());
    }
    data.extend(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{start}\n%%EOF\n",
            offsets.len()
        )
        .as_bytes(),
    );
    data
}

fn stream(bytes: &[u8]) -> Vec<u8> {
    let mut output = format!("<< /Length {} >>\nstream\n", bytes.len()).into_bytes();
    output.extend(bytes);
    output.extend(b"\nendstream");
    output
}

fn load(bytes: Vec<u8>, entries: usize) -> (PdfDocument, Option<FontInfo>) {
    let doc = PdfDocument::from_bytes_with_limits(
        bytes,
        PdfImportLimits {
            font_entries: entries,
            ..Default::default()
        },
    )
    .unwrap();
    let (_, reference) = pdf_oxide::parser::parse_object(b"5 0 R").unwrap();
    let dict = doc.load_object(reference.as_reference().unwrap()).unwrap();
    let font = FontInfo::from_dict(&dict, &doc).ok();
    (doc, font)
}

fn plain() -> Vec<u8> {
    pdf(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>",
        &[],
    )
}

#[test]
fn memo_entries_hits_and_sticky_failure_share_the_request() {
    let (doc, font) = load(plain(), 2);
    let font = font.unwrap();
    assert_eq!(font.char_to_unicode(65).as_deref(), Some("A"));
    let charged = doc.import_budget().unwrap().decoded_bytes();
    assert_eq!(font.char_to_unicode(65).as_deref(), Some("A"));
    assert_eq!(doc.import_budget().unwrap().decoded_bytes(), charged + 1);
    assert_eq!(font.char_to_unicode(66).as_deref(), Some("B"));
    assert!(font.char_to_unicode(67).is_none());
    assert_eq!(font.type0_unicode_memo.lock().unwrap().len(), 2);
    assert!(doc.check_import_budget().is_err());
    assert!(font.char_to_unicode(65).is_none());
    let (fresh, font) = load(plain(), 2);
    assert_eq!(font.unwrap().char_to_unicode(67).as_deref(), Some("C"));
    assert!(fresh.check_import_budget().is_ok());
}

#[test]
fn memo_hit_copy_has_an_exact_decoded_boundary() {
    let (doc, font) = load(plain(), 10);
    let font = font.unwrap();
    assert_eq!(font.char_to_unicode(65).as_deref(), Some("A"));
    let budget = doc.import_budget().unwrap();
    budget
        .restrict_work_limits(budget.decoded_bytes() + 1, usize::MAX)
        .unwrap();
    assert_eq!(font.char_to_unicode(65).as_deref(), Some("A"));
    assert!(font.char_to_unicode(65).is_none());
    assert_eq!(
        budget.snapshot().failure.unwrap().reason,
        "document decoded bytes"
    );
}

#[test]
fn memo_cmap_range_owned_value_is_charged_before_the_cache_copy() {
    let bytes = pdf(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /ToUnicode 6 0 R >>",
        &[stream(b"1 beginbfrange <0041> <0140> <0041> endbfrange")],
    );
    let (doc, font) = load(bytes.clone(), 1000);
    let font = font.unwrap();
    let cmap = font.to_unicode.as_ref().unwrap().get().unwrap();
    assert_eq!(cmap.get(&65).as_deref(), Some("A"));
    assert!(matches!(cmap.get(&65), Some(std::borrow::Cow::Owned(_))));
    let budget = doc.import_budget().unwrap();
    // One memo work unit (128 bytes) plus the one-byte owned CMap result.
    // The second one-byte string, retained in the memo, must not fit.
    budget
        .restrict_work_limits(budget.decoded_bytes() + 129, usize::MAX)
        .unwrap();
    assert!(font.char_to_unicode(65).is_none());
    assert!(font.type0_unicode_memo.lock().unwrap().is_empty());
    assert!(doc.check_import_budget().is_err());
    let (fresh, font) = load(bytes, 1000);
    let font = font.unwrap();
    assert_eq!(font.char_to_unicode(65).as_deref(), Some("A"));
    assert_eq!(font.char_to_unicode(66).as_deref(), Some("B"));
    assert!(fresh.check_import_budget().is_ok());
}

#[test]
fn negative_memo_entries_are_bounded_too() {
    let (doc, font) = load(plain(), 1);
    let font = font.unwrap();
    assert!(font.char_to_unicode(0).is_none());
    assert_eq!(font.type0_unicode_memo.lock().unwrap().len(), 1);
    assert!(doc.check_import_budget().is_ok());
    assert!(font.char_to_unicode(1).is_none());
    assert!(doc.check_import_budget().is_err());
    assert_eq!(font.type0_unicode_memo.lock().unwrap().len(), 1);
}

#[test]
fn type1_program_insertions_are_charged_even_for_replacements() {
    use pdf_oxide::fonts::type1_encoding::{
        parse_type1_encoding, parse_type1_encoding_with_budget,
    };
    for program in [
        &b"/Encoding 256 array dup 65 /A put dup 66 /B put dup 97 /a put readonly def"[..],
        &b"/Encoding 256 array dup 65 /A put dup 65 /B put dup 65 /a put readonly def"[..],
    ] {
        let budget = ImportBudget::new(PdfImportLimits {
            font_entries: 2,
            ..Default::default()
        })
        .unwrap();
        assert!(parse_type1_encoding_with_budget(program, Some(&budget)).is_none());
        assert!(budget.check().is_err());
        let budget = ImportBudget::new(PdfImportLimits {
            font_entries: 3,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            parse_type1_encoding_with_budget(program, Some(&budget)),
            parse_type1_encoding(program)
        );
        assert!(budget.check().is_ok());
    }
    let bytes = pdf(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Custom /FontDescriptor 6 0 R >>",
        &[
            b"<< /Type /FontDescriptor /FontFile 7 0 R >>".to_vec(),
            stream(b"/Encoding 256 array dup 65 /A put dup 66 /B put dup 97 /a put readonly def"),
        ],
    );
    let (doc, _) = load(bytes.clone(), 2);
    assert!(doc.check_import_budget().is_err());
    let (doc, font) = load(bytes, 1000);
    assert_eq!(font.unwrap().char_to_unicode(65).as_deref(), Some("A"));
    assert!(doc.check_import_budget().is_ok());
}

fn index(entries: &[Vec<u8>]) -> Vec<u8> {
    if entries.is_empty() {
        return vec![0, 0];
    }
    let mut output = (entries.len() as u16).to_be_bytes().to_vec();
    output.push(4);
    let mut offset = 1u32;
    output.extend(offset.to_be_bytes());
    for entry in entries {
        offset += entry.len() as u32;
        output.extend(offset.to_be_bytes());
    }
    for entry in entries {
        output.extend(entry);
    }
    output
}

fn number(value: i32, output: &mut Vec<u8>) {
    output.push(29);
    output.extend(value.to_be_bytes());
}

fn cff(cid: bool, glyphs: u16, operands: usize) -> Vec<u8> {
    let mut prefix = vec![1, 0, 4, 1];
    prefix.extend(index(&[b"BudgetFont".to_vec()]));
    let dictionary = |charset, chars| {
        let mut dict = vec![139; operands];
        if operands != 0 {
            dict.push(0);
        }
        if cid {
            number(391, &mut dict);
            number(392, &mut dict);
            number(0, &mut dict);
            dict.extend([12, 30]);
        }
        number(charset, &mut dict);
        dict.push(15);
        number(chars, &mut dict);
        dict.push(17);
        dict
    };
    let strings = index(&[b"Adobe".to_vec(), b"Identity".to_vec()]);
    let start = prefix.len() + index(&[dictionary(0, 0)]).len() + strings.len() + 2;
    let mut charset = vec![0];
    for gid in 1..glyphs {
        charset.extend(
            (if cid {
                gid
            } else {
                [34, 35, 66][(gid - 1) as usize]
            })
            .to_be_bytes(),
        );
    }
    prefix.extend(index(&[dictionary(
        start as i32,
        (start + charset.len()) as i32,
    )]));
    prefix.extend(strings);
    prefix.extend([0, 0]);
    prefix.extend(charset);
    prefix.extend(index(&vec![vec![14]; glyphs as usize]));
    prefix
}

#[test]
fn cff_declared_expansions_reject_low_caps_and_keep_normal_gid_mapping() {
    use pdf_oxide::fonts::{cff_encoding::*, font_dict::Encoding};
    let cid = cff(true, 1001, 0);
    let budget = ImportBudget::new(PdfImportLimits {
        font_entries: 1000,
        ..Default::default()
    })
    .unwrap();
    assert!(parse_cff_cid_to_gid_with_budget(&cid, Some(&budget)).is_none());
    assert!(budget.check().is_err());
    // Two INDEX references, 1,001 charset slots, then 1,001 output slots.
    // Reject the output reservation too, after the charset itself fits.
    for cap in [2003, 2004] {
        let budget = ImportBudget::new(PdfImportLimits {
            font_entries: cap,
            ..Default::default()
        })
        .unwrap();
        let mapped = parse_cff_cid_to_gid_with_budget(&cid, Some(&budget));
        assert_eq!(mapped.is_some(), cap == 2004);
        assert_eq!(budget.check().is_ok(), cap == 2004);
    }
    let budget = ImportBudget::new(PdfImportLimits::default()).unwrap();
    let map = parse_cff_cid_to_gid_with_budget(&cid, Some(&budget)).unwrap();
    assert_eq!(map.len(), 1001);
    assert_eq!(map.get(&1000), Some(&1000));
    let simple = cff(false, 4, 0);
    let encoding = Encoding::Standard("WinAnsiEncoding".into());
    let differences = Default::default();
    let budget = ImportBudget::new(PdfImportLimits {
        font_entries: 1,
        ..Default::default()
    })
    .unwrap();
    assert!(parse_cff_gid_mapping_with_pdf_encoding_with_budget(
        &simple,
        &encoding,
        &differences,
        Some(&budget)
    )
    .is_none());
    assert!(budget.check().is_err());
    let map = parse_cff_gid_mapping_with_pdf_encoding(&simple, &encoding, &differences).unwrap();
    assert_eq!(
        (map.get(&65), map.get(&66), map.get(&97)),
        (Some(&1), Some(&2), Some(&3))
    );
    assert_eq!(
        map,
        parse_cff_gid_mapping_with_pdf_encoding_with_budget(
            &simple,
            &encoding,
            &differences,
            Some(&ImportBudget::new(Default::default()).unwrap())
        )
        .unwrap()
    );
}

#[test]
fn cff_index_and_dict_boundaries_do_not_wrap_or_expand_unchecked() {
    use pdf_oxide::fonts::cff_encoding::parse_cff_cid_to_gid_with_budget;
    for offset in [0u32, u32::MAX] {
        let mut bad = cff(true, 4, 0);
        // Name INDEX starts at 4; count + offSize occupies the next 3 bytes.
        bad[7..11].copy_from_slice(&offset.to_be_bytes());
        assert!(parse_cff_cid_to_gid_with_budget(
            &bad,
            Some(&ImportBudget::new(Default::default()).unwrap())
        )
        .is_none());
    }
    let accepted = ImportBudget::new(Default::default()).unwrap();
    assert!(parse_cff_cid_to_gid_with_budget(&cff(true, 4, 48), Some(&accepted)).is_some());
    let rejected = ImportBudget::new(Default::default()).unwrap();
    assert!(parse_cff_cid_to_gid_with_budget(&cff(true, 4, 49), Some(&rejected)).is_none());
    assert!(rejected.check().is_err());
}

#[test]
fn cff_font_matrix_real_text_obeys_the_string_cap() {
    use pdf_oxide::fonts::cff_encoding::parse_cff_font_matrix_scale_with_budget;
    let mut dict = vec![30, 0x0a, 0x00, 0x1f, 139, 139]; // 0.001 0 0
    dict.extend([30, 0x0a, 0x00, 0x1f, 139, 139, 12, 7]); // 0.001 0 0 FontMatrix
    let mut source = vec![1, 0, 4, 1];
    source.extend(index(&[b"Matrix".to_vec()]));
    source.extend(index(&[dict]));
    let normal = ImportBudget::new(Default::default()).unwrap();
    assert_eq!(
        parse_cff_font_matrix_scale_with_budget(&source, Some(&normal)),
        Some(0.001)
    );
    let limited = ImportBudget::new(PdfImportLimits {
        string_bytes: 4,
        ..Default::default()
    })
    .unwrap();
    assert!(parse_cff_font_matrix_scale_with_budget(&source, Some(&limited)).is_none());
    assert_eq!(
        limited.snapshot().failure.unwrap().reason,
        "font string bytes"
    );
}

#[test]
fn tounicode_normalization_preserves_fullwidth_and_memo_copies() {
    let bytes = pdf(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /ToUnicode 6 0 R >>",
        &[stream(b"1 beginbfchar <41> <2F4BFF10> endbfchar")],
    );
    let (doc, font) = load(bytes, 1000);
    let font = font.unwrap();
    assert_eq!(font.char_to_unicode(65).as_deref(), Some("欠０"));
    let charged = doc.import_budget().unwrap().decoded_bytes();
    assert_eq!(font.char_to_unicode(65).as_deref(), Some("欠０"));
    assert_eq!(
        doc.import_budget().unwrap().decoded_bytes() - charged,
        "欠０".len() as u64
    );
}

#[test]
fn legacy_cjk_fallbacks_keep_character_values_and_charge_cached_copies() {
    // Independent character-code pairs from the legacy encoding repertoires.
    for (encoding, ordering, code, expected) in [
        ("90ms-RKSJ-H", "Japan1", 0x8341, "ア"),
        ("H", "Japan1", 0x2422, "あ"),
        ("EUC-H", "Japan1", 0xa4a2, "あ"),
        ("GBK-EUC-H", "GB1", 0xd6d0, "中"),
        ("B5pc-H", "CNS1", 0xa4a4, "中"),
        ("KSC-EUC-H", "Korea1", 0xb0a1, "가"),
    ] {
        let dictionary = format!("<< /Type /Font /Subtype /Type0 /BaseFont /LegacyBudget /Encoding /{encoding} /DescendantFonts [<< /Type /Font /Subtype /CIDFontType2 /BaseFont /LegacyBudget /CIDSystemInfo << /Registry (Adobe) /Ordering ({ordering}) /Supplement 0 >> >>] >>");
        let (doc, font) = load(pdf(dictionary.as_bytes(), &[]), 1000);
        let font = font.unwrap();
        assert_eq!(
            font.char_to_unicode(code).as_deref(),
            Some(expected),
            "{encoding}"
        );
        let budget = doc.import_budget().unwrap();
        let charged = budget.decoded_bytes();
        assert_eq!(font.char_to_unicode(code).as_deref(), Some(expected));
        assert_eq!(
            budget.decoded_bytes() - charged,
            expected.len() as u64,
            "{encoding}"
        );
        budget.reject("legacy fallback sticky sentinel");
        assert!(font.char_to_unicode(code).is_none());
    }
}

fn glyph_font() -> Vec<u8> {
    // Minimal sfnt with a Format 2 post custom name. Per the TrueType post
    // contract, index 258 addresses the first Pascal string, not a codepoint.
    let mut head = vec![0; 54];
    head[18..20].copy_from_slice(&2048u16.to_be_bytes());
    let mut hhea = vec![0; 36];
    hhea[34..36].copy_from_slice(&1u16.to_be_bytes());
    let mut maxp = vec![0; 32];
    maxp[..4].copy_from_slice(&0x00010000u32.to_be_bytes());
    maxp[4..6].copy_from_slice(&66u16.to_be_bytes());
    let mut post = vec![0; 32];
    post[..4].copy_from_slice(&0x00020000u32.to_be_bytes());
    post.extend(66u16.to_be_bytes());
    for gid in 0..66 {
        post.extend((if gid == 65 { 258u16 } else { 0 }).to_be_bytes());
    }
    post.extend(b"\x06period");
    let tables = [
        (b"head", head),
        (b"hhea", hhea),
        (b"maxp", maxp),
        (b"post", post),
    ];
    let mut output = vec![0; 12 + tables.len() * 16];
    output[..4].copy_from_slice(&0x00010000u32.to_be_bytes());
    output[4..6].copy_from_slice(&(tables.len() as u16).to_be_bytes());
    for (i, (tag, bytes)) in tables.iter().enumerate() {
        let offset = output.len();
        let record = &mut output[12 + i * 16..28 + i * 16];
        record[..4].copy_from_slice(*tag);
        record[8..12].copy_from_slice(&(offset as u32).to_be_bytes());
        record[12..16].copy_from_slice(&(bytes.len() as u32).to_be_bytes());
        output.extend(bytes);
        output.resize(output.len().next_multiple_of(4), 0);
    }
    output
}

#[test]
fn embedded_post_names_are_reserved_before_once_lock_expansion() {
    let bytes = pdf(b"<< /Type /Font /Subtype /TrueType /BaseFont /GlyphBudget /FontDescriptor 6 0 R /ToUnicode 8 0 R >>", &[
        b"<< /Type /FontDescriptor /FontFile2 7 0 R >>".to_vec(), stream(&glyph_font()),
        stream(b"1 beginbfchar <41> <00AC> endbfchar"),
    ]);
    let (doc, font) = load(bytes.clone(), 1000);
    let font = font.unwrap();
    assert_eq!(font.char_to_unicode(65).as_deref(), Some("."));
    assert!(doc.check_import_budget().is_ok());
    let (doc, font) = load(bytes, 65);
    assert!(font.unwrap().char_to_unicode(65).is_none());
    assert_eq!(
        doc.import_budget()
            .unwrap()
            .snapshot()
            .failure
            .unwrap()
            .reason,
        "font metric entries"
    );
}

fn donation_cmap() -> pdf_oxide::fonts::TrueTypeCMap {
    // A one-glyph independent format-12 table: Unicode A -> GID 1.
    let mut data = vec![0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0];
    data.extend(b"cmap");
    data.extend(0u32.to_be_bytes());
    data.extend(28u32.to_be_bytes());
    data.extend(40u32.to_be_bytes());
    data.extend([0, 0, 0, 1, 0, 3, 0, 10]);
    data.extend(12u32.to_be_bytes());
    data.extend([0, 12, 0, 0]);
    for value in [28u32, 0, 1, 65, 65, 1] {
        data.extend(value.to_be_bytes());
    }
    pdf_oxide::fonts::TrueTypeCMap::from_font_data(&data).unwrap()
}

#[test]
fn donated_cmap_rejects_owned_font_copy_before_detaching_shared_recipient() {
    use pdf_oxide::{extractors::TextExtractor, fonts::Encoding};
    use std::sync::Arc;
    for remaining in [16 * 1024, 512 * 1024] {
        let (_, font) = load(plain(), 100_000);
        let mut recipient = font.unwrap();
        let budget = Arc::new(ImportBudget::new(PdfImportLimits::default()).unwrap());
        recipient.import_budget = Some(Arc::clone(&budget));
        recipient.base_font = "AAAAAA+DonatedFace".into();
        recipient.subtype = "Type0".into();
        recipient.encoding = Encoding::Identity;
        recipient.widths = Some(vec![600.; 1024]);
        let mut donor = recipient.clone();
        donor.base_font = "BBBBBB+DonatedFace".into();
        donor.widths = None;
        donor.set_truetype_cmap(Some(donation_cmap()));
        let mut extractor = TextExtractor::new();
        extractor.add_font("Recipient".into(), recipient);
        extractor.add_font("Donor".into(), donor);
        let held_recipient = extractor
            .get_font_set()
            .into_iter()
            .find(|(name, _)| name == "Recipient")
            .unwrap()
            .1;
        assert_eq!(Arc::strong_count(&held_recipient), 2);
        budget
            .restrict_work_limits(budget.decoded_bytes() + remaining, usize::MAX)
            .unwrap();
        extractor.share_truetype_cmaps();
        assert!(
            held_recipient.truetype_cmap().is_none(),
            "sibling context remains untouched"
        );
        if remaining == 16 * 1024 {
            assert!(budget.check().is_err());
            assert_eq!(
                Arc::strong_count(&held_recipient),
                2,
                "no COW before reservation"
            );
            assert_eq!(
                budget.snapshot().failure.unwrap().reason,
                "document decoded bytes"
            );
        } else {
            assert!(budget.check().is_ok());
            assert_eq!(Arc::strong_count(&held_recipient), 1);
            let copied = extractor
                .get_font_set()
                .into_iter()
                .find(|(name, _)| name == "Recipient")
                .unwrap()
                .1;
            assert_eq!(copied.truetype_cmap().unwrap().get_unicode(1), Some('A'));
            assert_eq!(copied.widths.as_ref().unwrap().len(), 1024);
            assert!(
                Arc::ptr_eq(
                    &copied.type0_unicode_memo,
                    &held_recipient.type0_unicode_memo
                ),
                "Unicode memo stays shared while owned widths are copied"
            );
        }
    }
}

#[test]
fn extractor_font_set_copy_reserves_names_and_slots_with_exact_boundary() {
    use pdf_oxide::extractors::TextExtractor;
    use std::sync::Arc;
    let required = std::mem::size_of::<(String, Arc<FontInfo>)>() + 2;
    for remaining in [required - 1, required] {
        let (_, font) = load(plain(), 100_000);
        let mut font = font.unwrap();
        let budget = Arc::new(ImportBudget::new(PdfImportLimits::default()).unwrap());
        font.import_budget = Some(Arc::clone(&budget));
        let mut extractor = TextExtractor::new();
        extractor.add_font("F1".into(), font);
        budget
            .restrict_work_limits(remaining as u64, usize::MAX)
            .unwrap();
        let copy = extractor.get_font_set();
        assert_eq!(copy.len(), usize::from(remaining == required));
        assert_eq!(budget.check().is_ok(), remaining == required);
        if let Some((name, font)) = copy.first() {
            assert_eq!(name, "F1");
            assert_eq!(font.base_font, "Helvetica");
        }
    }
}

#[test]
fn font_cache_failure_is_request_local_and_not_hidden_by_fallback() {
    for entries in [1, 100_000] {
        let doc = PdfDocument::from_bytes_with_limits(
            plain(),
            PdfImportLimits {
                font_entries: entries,
                ..Default::default()
            },
        )
        .unwrap();
        let result = doc.extract_ordered_painted_spans_with_fonts(0);
        if entries == 1 {
            assert!(result.is_err());
            assert!(doc.check_import_budget().is_err());
        } else {
            assert_eq!(
                result
                    .unwrap()
                    .iter()
                    .map(|p| p.span.text.as_str())
                    .collect::<String>(),
                "ABa"
            );
            assert!(doc.check_import_budget().is_ok());
        }
    }
}

#[test]
fn simple_widths_and_replaced_differences_obey_the_same_font_work_limit() {
    let widths = format!("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Widths [{}] /FirstChar 0 /LastChar 1023 >>", "600 ".repeat(1024));
    let differences = format!("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding << /BaseEncoding /WinAnsiEncoding /Differences [{}] >> >>", "65 /A 65 /B ".repeat(1024));
    for (font, tiny) in [(widths.as_bytes(), 2), (differences.as_bytes(), 1024)] {
        let bytes = pdf(font, &[]);
        let (doc, font) = load(bytes.clone(), tiny);
        assert!(font.is_none());
        assert!(doc.check_import_budget().is_err());
        let (doc, font) = load(bytes, 20_000);
        assert!(font.is_some());
        assert!(doc.check_import_budget().is_ok());
        let font = font.unwrap();
        if let Some(widths) = font.widths {
            assert_eq!(widths.len(), 1024);
        } else {
            assert_eq!(font.char_to_unicode(65).as_deref(), Some("B"));
        }
    }
}

#[test]
fn font_metadata_copy_checks_remaining_bytes_before_copying_long_base_name() {
    let name = "A".repeat(4096);
    let bytes = pdf(
        format!("<< /Type /Font /Subtype /Type1 /BaseFont /{name} >>").as_bytes(),
        &[],
    );
    let doc = PdfDocument::from_bytes_with_limits(bytes, PdfImportLimits::default()).unwrap();
    let (_, reference) = pdf_oxide::parser::parse_object(b"5 0 R").unwrap();
    let dict = doc.load_object(reference.as_reference().unwrap()).unwrap();
    let budget = doc.import_budget().unwrap();
    budget
        .restrict_work_limits(budget.decoded_bytes() + 4095, usize::MAX)
        .unwrap();
    assert!(FontInfo::from_dict(&dict, &doc).is_err());
    let failure = budget.snapshot().failure.unwrap();
    assert_eq!(failure.reason, "document decoded bytes");
    assert_eq!(failure.requested, Some(4096));
}

#[test]
fn input_source_copy_is_reserved_before_container_parsing() {
    let bytes = plain();
    let request = std::sync::Arc::new(
        ImportBudget::new(PdfImportLimits {
            decoded_bytes: (bytes.len() - 1) as u64,
            ..Default::default()
        })
        .unwrap(),
    );
    assert!(PdfDocument::from_bytes_with_budget(bytes, std::sync::Arc::clone(&request)).is_err());
    assert!(request.check().is_err());
    assert_eq!(request.snapshot().decoded_bytes, 0);
    let fresh = PdfDocument::from_bytes_with_limits(plain(), PdfImportLimits::default()).unwrap();
    assert_eq!(fresh.page_count().unwrap(), 1);
    assert!(fresh.check_import_budget().is_ok());
}
