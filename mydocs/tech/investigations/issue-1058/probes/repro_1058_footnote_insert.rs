// Reproduce Task #1058 reopen Round 5 — insert footnote + type text + save.
//
// Simulates rhwp-studio behavior:
// 1. Load samples/footnote-01.hwp
// 2. insert_footnote_native at section 0, paragraph 5, char_offset 4 (between text)
// 3. Apply user input "기술이란?" via insert_text_in_footnote at fnCharOffset=2
// 4. Save HWP to output/poc/issue_1058/repro_round5.hwp
// 5. Re-parse and dump newest footnote inner_para
use rhwp::model::control::Control;
use rhwp::parser::parse_document;

fn main() {
    let src = "samples/footnote-01.hwp";
    let dst = "output/poc/issue_1058/repro_round5.hwp";

    let bytes = std::fs::read(src).expect("read source");
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse source");

    // 1. Insert footnote at paragraph 5, char_offset 4
    doc.insert_footnote_native(0, 5, 4)
        .expect("insert_footnote_native");

    // Find the new footnote — inner_para containing only placeholders
    let (sec_idx, para_idx, ctrl_idx, inner_idx) = {
        let document = doc.document();
        let mut found = None;
        'outer: for (si, section) in document.sections.iter().enumerate() {
            for (pi, para) in section.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Footnote(fn_) = ctrl {
                        if let Some(inner) = fn_.paragraphs.first() {
                            if inner.text == "  "
                                && inner.controls.len() == 1
                                && matches!(inner.controls[0], Control::AutoNumber(_))
                            {
                                found = Some((si, pi, ci, 0));
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
        found.expect("신규 footnote (text='  ') 찾기")
    };

    println!(
        "신규 footnote: section={} para={} ctrl={} inner={}",
        sec_idx, para_idx, ctrl_idx, inner_idx
    );

    // 2. Insert user text at fnCharOffset=2 (after two placeholder spaces)
    doc.insert_text_in_footnote_native(sec_idx, para_idx, ctrl_idx, inner_idx, 2, "기술이란?")
        .expect("insert_text_in_footnote");

    // 3. Dump pre-save state
    let document = doc.document();
    let inner = match &document.sections[sec_idx].paragraphs[para_idx].controls[ctrl_idx] {
        Control::Footnote(fn_) => fn_.paragraphs.first().unwrap(),
        _ => panic!("footnote not found"),
    };
    println!("== 저장 전 inner_para ==");
    println!("  text={:?}", inner.text);
    println!("  char_offsets={:?}", inner.char_offsets);
    println!("  char_count={}", inner.char_count);

    // 4. Save
    std::fs::create_dir_all("output/poc/issue_1058").ok();
    let out_bytes = doc.export_hwp_with_adapter().expect("export");
    std::fs::write(dst, &out_bytes).expect("write output");
    println!("\n저장: {} ({} bytes)", dst, out_bytes.len());

    // 5. Re-parse and dump
    let re_bytes = std::fs::read(dst).expect("read output");
    let re_doc = parse_document(&re_bytes).expect("re-parse");
    let mut footnote_idx = 0usize;
    for (si, section) in re_doc.sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for ctrl in &para.controls {
                if let Control::Footnote(fn_) = ctrl {
                    if let Some(inner) = fn_.paragraphs.first() {
                        if inner.text.contains("기술") {
                            println!(
                                "\n== 재파싱 신규 footnote (#{}, section {} pi {}) ==",
                                footnote_idx, si, pi
                            );
                            println!("  text={:?}", inner.text);
                            println!("  char_offsets={:?}", inner.char_offsets);
                            println!("  char_count={}", inner.char_count);
                            println!("  text_chars={}", inner.text.chars().count());
                            println!("  style_id={}", inner.style_id);
                            println!("  controls={}", inner.controls.len());
                            println!("  control_mask=0x{:X}", inner.control_mask);
                        }
                        footnote_idx += 1;
                    }
                }
            }
        }
    }
}
