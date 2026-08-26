//! Issue #4968 W9-Q3-2: exact font capability detection must be bounded and fail-closed.

#[path = "../../src/renderer/kerning.rs"]
mod kerning;

use kerning::{
    inspect_exact_font_kerning, ExactFontSource, KerningCapability,
    KerningCapabilityFallbackReason, MAX_KERNING_FONT_BYTES,
};

const NOTO_REGULAR: &[u8] =
    include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");
const NO_PAIR_TABLE: &[u8] =
    include_bytes!("../fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf");

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn replace_table_with_legacy_kern(font: &[u8], replace_tag: &[u8; 4]) -> Vec<u8> {
    // OpenType kern v0, horizontal format 0, one pair: glyph 1/glyph 2 => -70.
    let kern_table: [u8; 24] = [
        0, 0, 0, 1, // table version, subtable count
        0, 0, 0, 20, 0, 1, // subtable version, length, horizontal format 0
        0, 1, 0, 6, 0, 0, 0, 0, // nPairs, searchRange, entrySelector, rangeShift
        0, 1, 0, 2, 0xff, 0xba, // glyph pair and -70
    ];
    let mut output = font.to_vec();
    let table_count = usize::from(read_u16(&output, 4));
    let record = (0..table_count)
        .map(|index| 12 + index * 16)
        .find(|offset| &output[*offset..*offset + 4] == replace_tag)
        .expect("replaceable optional table");
    let table_offset = read_u32(&output, record + 8) as usize;
    let table_len = read_u32(&output, record + 12) as usize;
    assert!(table_len >= kern_table.len(), "replacement table is too small");
    output[record..record + 4].copy_from_slice(b"kern");
    output[record + 4..record + 8].fill(0); // parser does not consume checksums
    output[record + 12..record + 16].copy_from_slice(&(kern_table.len() as u32).to_be_bytes());
    output[table_offset..table_offset + kern_table.len()].copy_from_slice(&kern_table);
    output
}

#[test]
fn issue_4968_capability_precedence_and_failure_reasons_are_structured() {
    let gpos = inspect_exact_font_kerning(Some(ExactFontSource {
        bytes: NOTO_REGULAR,
        face_index: 0,
    }));
    assert_eq!(gpos.capability, KerningCapability::GposKern);
    assert_eq!(gpos.fallback_reason, None);
    assert_eq!(gpos.units_per_em, Some(1000));
    assert_eq!(
        gpos.font_source_sha256.as_deref(),
        Some("6e06a7fe5d696ca719894a23f36bb2b1be8c816a5937cd4ad0f23ca67780dd74")
    );

    let legacy_font = replace_table_with_legacy_kern(NO_PAIR_TABLE, b"sbix");
    let legacy = inspect_exact_font_kerning(Some(ExactFontSource {
        bytes: &legacy_font,
        face_index: 0,
    }));
    assert_eq!(legacy.capability, KerningCapability::LegacyKern);
    assert_eq!(legacy.fallback_reason, None);

    let both_font = replace_table_with_legacy_kern(NOTO_REGULAR, b"BASE");
    let both = inspect_exact_font_kerning(Some(ExactFontSource {
        bytes: &both_font,
        face_index: 0,
    }));
    assert_eq!(both.capability, KerningCapability::GposKern);

    let unsupported = inspect_exact_font_kerning(Some(ExactFontSource {
        bytes: NO_PAIR_TABLE,
        face_index: 0,
    }));
    assert_eq!(unsupported.capability, KerningCapability::Unsupported);
    assert_eq!(
        unsupported.fallback_reason,
        Some(KerningCapabilityFallbackReason::PairTableUnsupported)
    );
    let unsupported_json = serde_json::to_value(&unsupported).expect("capability JSON");
    assert_eq!(unsupported_json["capability"], "unsupported");
    assert_eq!(
        unsupported_json["fallbackReason"],
        "pair-table-unsupported"
    );
    assert_eq!(unsupported_json["fontBytes"], NO_PAIR_TABLE.len());
    assert_eq!(unsupported_json["faceIndex"], 0);

    let missing = inspect_exact_font_kerning(None);
    assert_eq!(
        missing.fallback_reason,
        Some(KerningCapabilityFallbackReason::FontSourceUnavailable)
    );

    let malformed = inspect_exact_font_kerning(Some(ExactFontSource {
        bytes: b"not-an-sfnt",
        face_index: 0,
    }));
    assert_eq!(
        malformed.fallback_reason,
        Some(KerningCapabilityFallbackReason::MalformedSfnt)
    );
    assert!(malformed.font_source_sha256.is_some());

    let oversized_bytes = vec![0; MAX_KERNING_FONT_BYTES + 1];
    let oversized = inspect_exact_font_kerning(Some(ExactFontSource {
        bytes: &oversized_bytes,
        face_index: 0,
    }));
    assert_eq!(
        oversized.fallback_reason,
        Some(KerningCapabilityFallbackReason::FontByteLimitExceeded)
    );
    assert_eq!(oversized.font_source_sha256, None);
}
