//! Probe: does an endnote anchored on page 1 render at the DOCUMENT END (last page)
//! while a footnote stays at its anchor page's bottom?
//!
//! Loads the user's 2-column footnote+endnote demo, inflates the body so it spans
//! multiple pages, and dumps page item placement. Run with:
//!   cargo test --release --test endnote_doc_end_probe -- --nocapture

use rhwp::document_core::DocumentCore;

const DEMO: &str = "/Users/phihu/Downloads/2단_각주_데모.hwp";

#[test]
fn probe_multipage_endnote_placement() {
    let data = std::fs::read(DEMO).expect("read demo");
    let mut core = DocumentCore::from_bytes(&data).expect("parse demo");

    println!("=== BEFORE inflate: {} page(s) ===", core.page_count());
    print!("{}", core.dump_page_items(None));

    // Inflate body AFTER the endnote-anchor paragraph (para 3) so the footnote (para 2)
    // and the endnote anchor (para 3) both stay on PAGE 1. New filler pages follow.
    // Correct Hancom behavior: footnote stays at page-1 bottom; endnote moves to the
    // LAST page (document end).
    // Append a big tail to para 3 (offset ~90 = near its end, AFTER the endnote anchor).
    // Para 3 then spans many pages but STARTS on page 1, keeping footnote(2)+endnote
    // anchor(3) on page 1.
    let filler = "추가 본문 채우기 문장입니다. ".repeat(600);
    let _ = core
        .insert_text_native(0, 3, 90, &filler)
        .expect("insert filler at para 3 tail");

    let pages = core.page_count();
    println!("\n=== AFTER inflate: {} page(s) ===", pages);
    print!("{}", core.dump_page_items(None));
}
