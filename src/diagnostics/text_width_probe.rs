//! [#2156] 텍스트 폭 프로브 — `estimate_text_width` 를 CLI 로 직독한다.
//!
//! 한글 유효 문자폭 통제 프로브(tools/make_width_ladder.py + probe_width_ladder.py)
//! 의 rhwp 대조축. 동일 문자열의 본 환경 측정 폭을 출력해 클래스별 편차를
//! 정량화한다.
//!
//! 사용:
//!   rhwp measure-width --size 10 [--font "함초롬바탕"] [--ratio 100] <text>...
//!   rhwp measure-width --size 10 --repeat 100 가 0 A a "(" ","
//!
//! 출력(TSV): text(축약) \t chars \t width_px \t per_char_px

use crate::renderer::layout::estimate_text_width_unrounded;
use crate::renderer::TextStyle;

pub fn run(args: &[String]) {
    let mut font = String::from("함초롬바탕");
    let mut size_pt = 10.0f64;
    let mut ratio = 100.0f64;
    let mut repeat = 1usize;
    let mut texts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--font" => {
                i += 1;
                font = args.get(i).cloned().unwrap_or_default();
            }
            "--size" => {
                i += 1;
                size_pt = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(10.0);
            }
            "--ratio" => {
                i += 1;
                ratio = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(100.0);
            }
            "--repeat" => {
                i += 1;
                repeat = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(1);
            }
            t => texts.push(t.to_string()),
        }
        i += 1;
    }
    if texts.is_empty() {
        eprintln!("사용: rhwp measure-width --size 10 [--font 이름] [--repeat N] <text>...");
        return;
    }
    let style = TextStyle {
        font_family: font.clone(),
        font_size: size_pt * 96.0 / 72.0,
        ratio: ratio / 100.0,
        ..TextStyle::default()
    };
    println!(
        "font={font} size={size_pt}pt ({:.3}px) ratio={ratio}%",
        style.font_size
    );
    println!("text\tchars\twidth_px\tper_char_px");
    for t in texts {
        let s = t.repeat(repeat);
        let w = estimate_text_width_unrounded(&s, &style);
        let n = s.chars().count().max(1);
        let label: String = t.chars().take(8).collect();
        println!("{label}\t{n}\t{w:.3}\t{:.4}", w / n as f64);
    }
}
