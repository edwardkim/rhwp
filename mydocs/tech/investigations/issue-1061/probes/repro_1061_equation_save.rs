// Task #1061 — HWPX → HWP equation 어댑터 정합 reproduce.
//
// 1. Load samples/hwpx/math-001.hwpx
// 2. Export HWP via adapter
// 3. Re-parse and dump equation IR
// 4. Compare with samples/math-001.hwp (정답지)
use rhwp::model::control::Control;
use rhwp::parser::parse_document;

fn dump_equations(label: &str, doc: &rhwp::model::document::Document) {
    let mut idx = 0;
    for (si, section) in doc.sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if let Control::Equation(eq) = ctrl {
                    println!(
                        "[{}] #{} sec {} pi {} ci {}: \
                         attr=0x{:08X} font_name={:?} version_info={:?} script_len={}",
                        label,
                        idx,
                        si,
                        pi,
                        ci,
                        eq.common.attr,
                        eq.font_name,
                        eq.version_info,
                        eq.script.len(),
                    );
                    idx += 1;
                }
            }
        }
    }
}

fn main() {
    let src_hwpx = "samples/hwpx/math-001.hwpx";
    let oracle_hwp = "samples/math-001.hwp";
    let dst = "output/poc/issue_1061/repro_stage1.hwp";

    let bytes = std::fs::read(src_hwpx).expect("read source");
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse hwpx");

    // 어댑터 통한 HWP 저장
    std::fs::create_dir_all("output/poc/issue_1061").ok();
    let out_bytes = doc.export_hwp_with_adapter().expect("export");
    std::fs::write(dst, &out_bytes).expect("write output");
    println!("\n저장: {} ({} bytes)", dst, out_bytes.len());

    // 재파싱
    let re_bytes = std::fs::read(dst).expect("read output");
    let re_doc = parse_document(&re_bytes).expect("re-parse");
    println!("\n== 본 task 저장본 (Stage 1 적용) ==");
    dump_equations("STAGE1", &re_doc);

    // 정답지
    let oracle_bytes = std::fs::read(oracle_hwp).expect("read oracle");
    let oracle_doc = parse_document(&oracle_bytes).expect("parse oracle");
    println!("\n== 정답지 (samples/math-001.hwp) ==");
    dump_equations("ORACLE", &oracle_doc);
}
