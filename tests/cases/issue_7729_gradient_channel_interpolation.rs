//! 검사 유형: 고정형 — 독립 한컴 Print의 RGB 기대값이며 px 좌표 고정은 아니다.
//! 주석 갱신: 2026-11-14T23:18:39+09:00.
//! 측정: 2026-11-14, 정본 Visual Sweep PNG Print / compat2022 / 96DPI / raw2px / 문턱32.
//! 독립 색상 표본 6쪽: Native/fresh WASM 각각 전쪽 100.00%; 기존 원형 표본 6쪽도 각각 100.00%.
//! 실제 권고안 원본 22쪽: Native 최저98.69%, fresh WASM 최저98.68%(각7쪽), 대상4쪽 양쪽99.96%.
//! 원본4쪽의 배경 채널 보정 전 양쪽97.44%에서 독립 색상 원인을 분리했다.
//! 증거: mydocs/pr/assets/issues7727_7732_20261114/gradient-channel/cause.json 및 전쪽 TSV.
//! 양방향7·50·255단계 대조군: tests/fixtures/issue7729/gradient-band-color-probe.hwpx.
//! MCP job6c8ae335-f880-41ed-9f83-0687d078a1c4, 한컴2020 Print11.0.0.9136, method0.
//! PDF: pdf/issue7729/gradient-band-color-probe-2020.pdf.
//! PDF SHA256: 99b077124213a4f042ba2ab2a4ec5fc1910fb8d37cfca0f4f63d38d05487a0a3.
//! 기대 RGB는 PDF 동심원 fill이며 같은 중앙색의 중복 원은 하나의 띠로 본다.
//! 글꼴 설치나 PNG 문턱에 의존하지 않고 원본 색상 채널을 보호한다.

use rhwp::renderer::expand_gradient_steps;

fn color(rgb: (u8, u8, u8)) -> u32 {
    u32::from(rgb.0) | (u32::from(rgb.1) << 8) | (u32::from(rgb.2) << 16)
}

#[test]
fn mixed_channel_ramps_keep_the_independent_print_colors_in_both_directions() {
    let first = (210, 100, 50);
    let last = (40, 130, 200);
    // 쪽3·4의 중복 중앙 원을 제외한 독립 PDF fill 색상(안쪽→바깥쪽).
    for (from, to, expected) in [
        (
            first,
            last,
            [
                (210, 100, 50),
                (182, 105, 75),
                (154, 110, 100),
                (125, 115, 125),
                (97, 120, 150),
                (69, 125, 175),
                (40, 130, 200),
            ],
        ),
        (
            last,
            first,
            [
                (40, 130, 200),
                (68, 125, 175),
                (96, 120, 150),
                (125, 115, 125),
                (153, 110, 100),
                (181, 105, 75),
                (210, 100, 50),
            ],
        ),
    ] {
        let (colors, _) = expand_gradient_steps(&[color(from), color(to)], &[0.0, 1.0], 7, 50);
        let observed: Vec<_> = colors.iter().step_by(2).copied().collect();
        assert_eq!(
            observed,
            expected.map(color),
            "최종 RGB 반올림은 독립 Print의 채널을 바꾼다"
        );
    }
}

#[test]
fn fine_gray_ramps_keep_the_print_endpoint_side_of_fractional_changes() {
    // 쪽1·2의 중심부 네 번째 띠는 각각255·215다. 최종값 반올림은254·216을 만든다.
    for (from, to, expected) in [(255, 215, 255), (215, 255, 215)] {
        let (colors, _) = expand_gradient_steps(
            &[color((from, from, from)), color((to, to, to))],
            &[0.0, 1.0],
            255,
            50,
        );
        assert_eq!(colors[4 * 2], color((expected, expected, expected)));
        assert_eq!(colors.first().copied(), Some(color((from, from, from))));
        assert_eq!(colors.last().copied(), Some(color((to, to, to))));
    }
}
