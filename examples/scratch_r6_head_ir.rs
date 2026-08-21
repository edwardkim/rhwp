//! scratch: 파싱된 Header/Footer 내부 문단 컨트롤 확인 (RHWP_HP_SRC).
use rhwp::model::control::Control;

fn main() {
    let src = std::env::var("RHWP_HP_SRC").unwrap();
    let data = std::fs::read(&src).unwrap();
    let doc = rhwp::parser::parse_document(&data).unwrap();
    for (si, sec) in doc.sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                let (kind, paras) = match ctrl {
                    Control::Header(h) => ("head", &h.paragraphs),
                    Control::Footer(f) => ("foot", &f.paragraphs),
                    _ => continue,
                };
                for (hpi, hp) in paras.iter().enumerate() {
                    let ctrls: Vec<String> = hp
                        .controls
                        .iter()
                        .map(|c| format!("{c:?}").chars().take(20).collect())
                        .collect();
                    println!(
                        "s{si} p{pi} c{ci} {kind} inner[{hpi}]: text={:?} cc={} ctrls={:?}",
                        hp.text, hp.char_count, ctrls
                    );
                }
            }
        }
    }
}
