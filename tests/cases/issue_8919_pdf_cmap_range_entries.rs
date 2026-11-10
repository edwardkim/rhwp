//! #8919: ToUnicode `bfrange`/`notdefrange` entries are read by token, not by line.
//! Synthetic CMap sources, not PDF/Hancom layout baselines.
#![cfg(feature = "pdf-import")]

use pdf_oxide::fonts::cmap::{parse_tounicode_cmap, parse_tounicode_cmap_with_budget};
use pdf_oxide::import_limits::{ImportBudget, PdfImportLimits};

fn default_budget() -> ImportBudget {
    ImportBudget::new(PdfImportLimits::default()).unwrap()
}

/// Acrobat Distiller wraps a destination array over several lines. The second
/// line here starts with a BMP string and a surrogate pair. Read as a line of
/// its own it is the range 0x2135..=0xD835DC9C, 3.6 billion codes, and the
/// whole import stopped with `PDF_RESOURCE_LIMIT: CMap entries` on a table of
/// eight.
#[test]
fn bfrange_array_wrapped_over_lines_maps_its_own_codes() {
    let source = b"1 begincodespacerange\n<00><FF>endcodespacerange\n\
1 beginbfrange\n\
<01><08>[<00B7><00D7><2217>\n\
<2135><D835DC9C><212C>\n\
<D835DC9E><2130>]endbfrange";
    let expected = [
        "\u{00B7}",
        "\u{00D7}",
        "\u{2217}",
        "\u{2135}",
        "\u{1D49C}",
        "\u{212C}",
        "\u{1D49E}",
        "\u{2130}",
    ];

    let budget = default_budget();
    let bounded = parse_tounicode_cmap_with_budget(source, &budget).unwrap();
    budget.check().unwrap();
    let unbounded = parse_tounicode_cmap(source).unwrap();
    for cmap in [&bounded, &unbounded] {
        for (i, value) in expected.iter().enumerate() {
            let code = i as u32 + 1;
            assert_eq!(cmap.get(&code).as_deref(), Some(*value), "code {code:#x}");
        }
        // The strings of the array body are destinations. Before, the first
        // line mapped 0xB7..=0xD7 and the third 0xD835DC9E onward.
        assert!(cmap.get(&0xB7).is_none());
        assert!(cmap.get(&0x2135).is_none());
    }
}

/// A wrapped array still costs one entry per element, no more and no less.
#[test]
fn wrapped_array_charges_one_entry_per_element() {
    let source = b"beginbfrange\n<01><04>[<0041><0042>\n<0043><0044>]\nendbfrange";
    for (cap, fits) in [(3, false), (4, true)] {
        let budget = ImportBudget::new(PdfImportLimits {
            cmap_entries: cap,
            ..PdfImportLimits::default()
        })
        .unwrap();
        let result = parse_tounicode_cmap_with_budget(source, &budget);
        assert_eq!(result.is_ok(), fits, "cap {cap}");
        if let Ok(cmap) = result {
            assert_eq!(cmap.get(&0x04).as_deref(), Some("D"));
        }
    }
}

/// A line break inside an entry and several entries on one line are both plain
/// white space, in `bfrange` and in `notdefrange`.
#[test]
fn range_entries_do_not_depend_on_line_breaks() {
    let source = b"beginbfrange\n\
<10>\n<12> <0041> <20> <21>\n[<0058>\n<0059>] <30> <31>\n<0061>\n\
endbfrange\n\
beginnotdefrange\n<40>\n<41>\n<FFFD>\nendnotdefrange";
    let budget = default_budget();
    for cmap in [
        parse_tounicode_cmap_with_budget(source, &budget).unwrap(),
        parse_tounicode_cmap(source).unwrap(),
    ] {
        for (code, value) in [
            (0x10, "A"),
            (0x11, "B"),
            (0x12, "C"),
            (0x20, "X"),
            (0x21, "Y"),
            (0x30, "a"),
            (0x31, "b"),
            (0x40, "\u{FFFD}"),
            (0x41, "\u{FFFD}"),
        ] {
            assert_eq!(cmap.get(&code).as_deref(), Some(value), "code {code:#x}");
        }
    }
    budget.check().unwrap();
}

/// An array the array form does not take (an empty destination here) is
/// dropped whole. Its body is not read as `<lo> <hi> <dst>`, which for the
/// first three strings is again a range of 3.6 billion codes.
#[test]
fn unreadable_array_is_not_read_as_a_sequential_range() {
    let source =
        b"beginbfrange\n<01> <04> [<2135> <D835DC9C> <212C> <>]\n<05> <06> <0041>\nendbfrange";
    let budget = default_budget();
    let cmap = parse_tounicode_cmap_with_budget(source, &budget).unwrap();
    budget.check().unwrap();
    assert!(cmap.get(&0x01).is_none());
    assert!(cmap.get(&0x2135).is_none());
    assert_eq!(cmap.get(&0x05).as_deref(), Some("A"));
    assert_eq!(cmap.get(&0x06).as_deref(), Some("B"));
}

/// Brackets and strings that belong to no entry do not shift the entries after
/// them, and an array or a string left open maps nothing.
#[test]
fn stray_brackets_and_open_entries_map_nothing() {
    for source in [
        "beginbfrange\n<01> [<0041> <0042>] <05> <06> <0043>\nendbfrange",
        "beginbfrange\n<01> <02> ] <05> <06> <0043>\nendbfrange",
    ] {
        let budget = default_budget();
        let cmap = parse_tounicode_cmap_with_budget(source.as_bytes(), &budget).unwrap();
        budget.check().unwrap();
        assert!(cmap.get(&0x01).is_none(), "{source}");
        assert!(cmap.get(&0x41).is_none(), "{source}");
        assert_eq!(cmap.get(&0x05).as_deref(), Some("C"), "{source}");
        assert_eq!(cmap.get(&0x06).as_deref(), Some("D"), "{source}");
    }
    for source in [
        "beginbfrange\n<01> <02> [<0041> <0042>\nendbfrange",
        "beginbfrange\n<01> <02> <00\nendbfrange",
    ] {
        let budget = default_budget();
        let cmap = parse_tounicode_cmap_with_budget(source.as_bytes(), &budget).unwrap();
        budget.check().unwrap();
        assert!(cmap.is_empty(), "{source}");
    }
}

/// The limit this patch stops tripping by mistake still holds for a range
/// that really is that large, on one line or across two.
#[test]
fn a_really_huge_range_is_still_rejected() {
    for source in [
        "beginbfrange\n<00000000> <FFFFFFFF> <0041>\nendbfrange",
        "beginbfrange\n<00000000>\n<FFFFFFFF> <0041>\nendbfrange",
        "beginbfrange\n<00000000> <FFFFFFFF>\n[<0041>\n<0042>]\nendbfrange",
        "beginnotdefrange\n<00000000>\n<FFFFFFFF>\n<0041>\nendnotdefrange",
    ] {
        let budget = default_budget();
        let result = parse_tounicode_cmap_with_budget(source.as_bytes(), &budget);
        assert!(
            matches!(
                result,
                Err(pdf_oxide::error::Error::ImportResourceLimit("CMap entries"))
            ),
            "{source}"
        );
    }
}
