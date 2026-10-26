//! 한글 전진폭으로 대용하지 않는 KoPub 기호의 명시 메트릭.
//!
//! 독립 한컴 2024 Print의 KoPub CIDFont /W에서 굵기별 충돌이 없는 값을 읽었다.
//! 같은 HWP/HWPX 기준 PDF의 SHA-256:
//! 3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593
//! 299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e
//! KoPubWorld는 별도 프로그램이므로 이 표에 이름만으로 포함하지 않는다.

#[path = "kopub_hanja_metrics.rs"]
mod hanja;

pub(super) fn advance_em(dotum: bool, bold: bool, ch: char) -> Option<f64> {
    if let Some(em) = hanja::advance_em(dotum, bold, ch) {
        return Some(em);
    }
    let table = if dotum { DOTUM } else { BATANG };
    table
        .binary_search_by_key(&(ch as u32), |entry| entry.0)
        .ok()
        .map(|index| f64::from(table[index].1) / 1000.0)
}

const BATANG: &[(u32, u16)] = &[
    (0xb7, 312),
    (0xd7, 900),
    (0x2013, 495),
    (0x2018, 312),
    (0x2019, 312),
    (0x201c, 404),
    (0x201d, 404),
    (0x2022, 400),
    (0x2026, 900),
    (0x2027, 900),
    (0x203b, 900),
    (0x2192, 900),
    (0x21d2, 900),
    (0x221a, 900),
    (0x2228, 900),
    (0x223c, 900),
    (0x2299, 900),
    (0x25a0, 900),
    (0x25a1, 900),
    (0x25b2, 900),
    (0x25b6, 900),
    (0x25bc, 900),
    (0x25c0, 900),
    (0x25cb, 900),
    (0x2605, 900),
    (0x2f08, 900),
    (0x2f30, 900),
    (0x2f83, 900),
    (0x2f9d, 900),
    (0x300c, 450),
    // 두 독립 편람 Print의 327개씩과 KoPub 세 굵기 hmtx: 900/1000em.
    (0x318d, 900),
    (0x3262, 900),
    (0x327b, 900),
    (0x339c, 900),
    (0x339d, 900),
    (0x33a1, 900),
    (0xff0a, 900),
    (0xff0b, 900),
    (0xff1a, 900),
    (0xff3b, 900),
    (0xff3d, 900),
];
const DOTUM: &[(u32, u16)] = &[
    (0xb7, 319),
    (0xd7, 900),
    (0x2018, 290),
    (0x2019, 290),
    (0x201c, 410),
    (0x201d, 410),
    (0x2022, 400),
    (0x2026, 900),
    (0x2027, 900),
    (0x203b, 900),
    (0x2191, 900),
    (0x2192, 900),
    (0x2193, 900),
    (0x2194, 900),
    (0x21d2, 900),
    (0x221a, 900),
    (0x2228, 900),
    (0x25a0, 900),
    (0x25a1, 900),
    (0x25b6, 900),
    (0x25c0, 900),
    (0x25cb, 900),
    (0x2605, 900),
    (0x260e, 900),
    (0x261e, 900),
    (0x2f25, 900),
    (0x2f26, 900),
    (0x2f64, 900),
    (0x2f9e, 900),
    (0x2fd2, 900),
    (0x300c, 450),
    (0x318d, 900),
    (0x339c, 900),
    (0x33a1, 900),
    (0xff06, 900),
    (0xff0a, 900),
    (0xff0d, 900),
    (0xff1a, 900),
    (0xff3b, 900),
    (0xff3d, 900),
];
