//! 임시 도구 (Task #519 Stage 1): 모든 샘플의 Picture 컨트롤 중
//! horz_flip / vert_flip / rotation_angle 가 0 이 아닌 것을 출력한다.
//! Stage 3 종료 시 삭제.

use rhwp::model::control::Control;
use rhwp::parser::parse_hwp;

fn scan(path: &std::path::Path) {
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(_) => return,
    };
    let doc = match parse_hwp(&data) {
        Ok(d) => d,
        Err(_) => return,
    };
    let mut header_printed = false;
    for (sec_idx, section) in doc.sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if let Control::Picture(pic) = ctrl {
                    let sa = &pic.shape_attr;
                    if sa.horz_flip || sa.vert_flip || sa.rotation_angle != 0 {
                        if !header_printed {
                            println!("\n=== {} ===", path.display());
                            header_printed = true;
                        }
                        println!("  [s{} p{} c{}] bin_id={} flip=(h={},v={}) rot={} cur={}×{} M=[{:.3},{:.3},{:.0};{:.3},{:.3},{:.0}]",
                            sec_idx, pi, ci, pic.image_attr.bin_data_id,
                            sa.horz_flip, sa.vert_flip, sa.rotation_angle,
                            sa.current_width, sa.current_height,
                            sa.render_sx, sa.render_b, sa.render_tx,
                            sa.render_c, sa.render_sy, sa.render_ty,
                        );
                    }
                }
            }
        }
    }
}

fn main() {
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    for entry in std::fs::read_dir("samples").unwrap() {
        let p = entry.unwrap().path();
        if p.extension().and_then(|s| s.to_str()) == Some("hwp") {
            paths.push(p);
        }
    }
    paths.sort();
    println!("스캔 대상: {} 파일", paths.len());
    for p in &paths {
        scan(p);
    }
    println!("\n=== 완료 ===");
}
