//! [#7702] 장평이 있는 공백의 전진폭 — 반각 단위를 먼저 내리고 장평을 곱해 반올림한다.
//!
//! 입력 `samples/issue7702/space_ratio_steps.hwpx` 는 한양신명조 문단 80개다. 문단마다 글자 크기
//! {7, 9, 10, 11, 13}pt × 장평 {75, 85, 90, 95, 97, 98, 110, 120}% × 자간 {0, −5}% 를 주고
//! 「가 가 … 가」 한 줄을 왼쪽 정렬로 둔다. 기대값은 같은 파일을 한/글 2020(MCP engine 2020)
//! Print 경로로 출력한 `pdf/issue7702/space_ratio_steps-2020.pdf` 의 「가」 원점 간격
//! (음절 + 공백) 평균이다.
//!
//! 한/글 공백 전진 단위는 ⌊em비 × ppem × 장평⌋ 이 아니라 반올림(⌊em비 × ppem⌋ × 장평) 이다.
//! 10pt 장평 98% 에서 반각 125 단위는 122.5 가 되는데, 내리면 122(488 HU), 한/글은 123(492 HU)이다.
//! 수정 전 계산은 80문단 중 20문단에서 한 장치 단위(0.053px) 어긋났다(장평 120% 는 넓게, 나머지는 좁게).
//!
//! PDF 좌표는 600dpi 로 양자화되고 배율이 약 1800/1801 이라, 허용 오차를 장치 단위의 절반
//! (0.026px)으로 둔다.

use rhwp::document_core::DocumentCore;

const FIXTURE: &[u8] = include_bytes!("../../samples/issue7702/space_ratio_steps.hwpx");

/// 장치 단위(4 HU = 0.0533px)의 절반.
const TOLERANCE_PX: f64 = 0.026;

/// (글자 크기 HU, 장평 %, 자간 %, 한/글 「가 」 간격 px) — 문단 순서.
const HANGUL_PITCHES: [(u16, u8, i8, f64); 80] = [
    (700, 75, 0, 10.4455),
    (700, 75, -5, 9.9146),
    (700, 85, 0, 11.8336),
    (700, 85, -5, 11.2451),
    (700, 90, 0, 12.5244),
    (700, 90, -5, 11.8848),
    (700, 95, 0, 13.2728),
    (700, 95, -5, 12.6332),
    (700, 97, 0, 13.4839),
    (700, 97, -5, 12.8442),
    (700, 98, 0, 13.6438),
    (700, 98, -5, 12.9530),
    (700, 110, 0, 15.3517),
    (700, 110, -5, 14.5521),
    (700, 120, 0, 16.7333),
    (700, 120, -5, 15.8826),
    (900, 75, 0, 13.4327),
    (900, 75, -5, 12.7931),
    (900, 85, 0, 15.2397),
    (900, 85, -5, 14.4402),
    (900, 90, 0, 16.1512),
    (900, 90, -5, 15.3517),
    (900, 95, 0, 16.9988),
    (900, 95, -5, 16.1512),
    (900, 97, 0, 17.4306),
    (900, 97, -5, 16.5750),
    (900, 98, 0, 17.5905),
    (900, 98, -5, 16.6790),
    (900, 110, 0, 19.7173),
    (900, 110, -5, 18.7578),
    (900, 120, 0, 21.5323),
    (900, 120, -5, 20.4129),
    (1000, 75, 0, 14.9785),
    (1000, 75, -5, 14.2323),
    (1000, 85, 0, 16.9508),
    (1000, 85, -5, 16.0979),
    (1000, 90, 0, 18.0169),
    (1000, 90, -5, 17.1107),
    (1000, 95, 0, 18.9764),
    (1000, 95, -5, 18.0169),
    (1000, 97, 0, 19.3495),
    (1000, 97, -5, 18.3900),
    (1000, 98, 0, 19.6160),
    (1000, 98, -5, 18.6566),
    (1000, 110, 0, 22.0147),
    (1000, 110, -5, 20.8953),
    (1000, 120, 0, 23.9870),
    (1000, 120, -5, 22.7610),
    (1100, 75, 0, 16.4711),
    (1100, 75, -5, 15.6715),
    (1100, 85, 0, 18.5999),
    (1100, 85, -5, 17.6404),
    (1100, 90, 0, 19.7193),
    (1100, 90, -5, 18.7598),
    (1100, 95, 0, 20.8387),
    (1100, 95, -5, 19.7693),
    (1100, 97, 0, 21.2685),
    (1100, 97, -5, 20.1991),
    (1100, 98, 0, 21.4784),
    (1100, 98, -5, 20.4089),
    (1100, 110, 0, 24.1469),
    (1100, 110, -5, 22.9176),
    (1100, 120, 0, 26.3257),
    (1100, 120, -5, 24.9965),
    (1300, 75, 0, 19.4479),
    (1300, 75, -5, 18.4884),
    (1300, 85, 0, 22.0680),
    (1300, 85, -5, 20.9486),
    (1300, 90, 0, 23.3473),
    (1300, 90, -5, 22.1664),
    (1300, 95, 0, 24.6267),
    (1300, 95, -5, 23.3966),
    (1300, 97, 0, 25.1556),
    (1300, 97, -5, 23.8763),
    (1300, 98, 0, 25.4262),
    (1300, 98, -5, 24.1469),
    (1300, 110, 0, 28.5138),
    (1300, 110, -5, 27.0746),
    (1300, 120, 0, 31.1216),
    (1300, 120, -5, 29.5225),
];

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = start + tag[start..].find('"')?;
    Some(&tag[start..end])
}

/// 글자 `<text>` 의 원점. 장평이 100% 가 아니면 `transform="translate(x,y) scale(..)"` 로 쓴다.
fn origin(tag: &str) -> Option<(f64, f64)> {
    if let (Some(x), Some(y)) = (attr(tag, "x"), attr(tag, "y")) {
        return Some((x.parse().ok()?, y.parse().ok()?));
    }
    let transform = attr(tag, "transform")?;
    let args = transform.strip_prefix("translate(")?;
    let args = &args[..args.find(')')?];
    let (x, y) = args.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

/// 모든 쪽의 「가」 줄마다 원점 x 를 위에서 아래로 모은다.
fn ga_lines(core: &DocumentCore) -> Vec<Vec<f64>> {
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        let svg = core.render_page_svg_native(page).expect("SVG");
        let mut lines: Vec<(f64, Vec<f64>)> = Vec::new();
        let mut rest = svg.as_str();
        while let Some(open) = rest.find("<text ") {
            rest = &rest[open..];
            let (Some(tag_end), Some(close)) = (rest.find('>'), rest.find("</text>")) else {
                break;
            };
            if &rest[tag_end + 1..close] == "가" {
                if let Some((x, y)) = origin(&rest[..tag_end]) {
                    let y = (y * 10.0).round() / 10.0;
                    match lines.iter_mut().find(|(line_y, _)| *line_y == y) {
                        Some((_, line)) => line.push(x),
                        None => lines.push((y, vec![x])),
                    }
                }
            }
            rest = &rest[close..];
        }
        lines.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, mut line) in lines {
            line.sort_by(|a, b| a.total_cmp(b));
            out.push(line);
        }
    }
    out
}

#[test]
fn space_advance_with_ratio_rounds_the_floored_half_width_like_hangul() {
    let core = DocumentCore::from_bytes(FIXTURE).expect("시험 문서 열기");
    let lines = ga_lines(&core);
    assert_eq!(
        lines.len(),
        HANGUL_PITCHES.len(),
        "문단마다 한 줄이어야 비교 대상이 맞는다"
    );
    let mut mismatched = Vec::new();
    for ((size, ratio, spacing, hangul), xs) in HANGUL_PITCHES.iter().zip(&lines) {
        let pitch = (xs[xs.len() - 1] - xs[0]) / (xs.len() - 1) as f64;
        if (pitch - hangul).abs() > TOLERANCE_PX {
            mismatched.push(format!(
                "{size} HU · 장평 {ratio}% · 자간 {spacing}%: 「가 」 간격 {pitch:.4}px, 한/글 {hangul:.4}px"
            ));
        }
    }
    assert!(
        mismatched.is_empty(),
        "공백 전진이 한/글과 다르다:\n{}",
        mismatched.join("\n")
    );
}
