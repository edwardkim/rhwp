//! [Issue #1835 진단] TAC 표 common.height 를 1/1.8 로 훼손한 합성 문서 생성.
//! 사용: cargo run --release --example synth_1835 -- <입력.hwp> <출력.hwp>
use rhwp::model::control::Control;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (input, output) = (&args[1], &args[2]);
    let bytes = std::fs::read(input).expect("read input");
    let mut doc = rhwp::parser::parse_hwp(&bytes).expect("parse");
    let mut done = false;
    for section in &mut doc.sections {
        // 재직렬화가 훼손을 반영하도록 원본 스트림 캐시 무효화
        section.raw_stream = None;
        for para in &mut section.paragraphs {
            for ctrl in &mut para.controls {
                if let Control::Table(t) = ctrl {
                    if t.common.treat_as_char && t.row_count >= 4 && !done {
                        let orig = t.common.height;
                        let stale = orig / 18 * 10; // ≈ orig/1.8
                        t.common.height = stale;
                        // 직렬화기는 raw_ctrl_data(CommonObjAttr verbatim)를 우선하므로
                        // raw 바이트의 HEIGHT(16..20 LE u32) 필드도 함께 패치한다.
                        if t.raw_ctrl_data.len() >= 20 {
                            t.raw_ctrl_data[16..20].copy_from_slice(&stale.to_le_bytes());
                        }
                        eprintln!(
                            "TAC 표 {}행×{}열 common.height {} → {} (1/1.8 훼손, raw 포함)",
                            t.row_count, t.col_count, orig, stale
                        );
                        done = true;
                    }
                }
            }
        }
    }
    assert!(done, "4행 이상 TAC 표를 찾지 못함");
    let out = rhwp::serializer::serialize_document(&doc).expect("serialize");
    std::fs::write(output, out).expect("write");
    eprintln!("저장: {output}");
}
