// Measured/manual metric overlay region — Task #4964 W6.
//
// These five entries are reconstructed from the tracked #2430 Hancom COM ladder evidence.
// They intentionally reuse selected Latin/Hangul tables from the historical generated region.
// Do not move them ahead of the generated entries or alter their order.

pub // [#2430] 한양신명조 실측 ASCII (한글 COM 무신축 래더 2026-07-20, 93/95 실측·2 보간 med=0.942).
// 한글은 한양신명조 를 HYSinMyeongJo-Medium 와 다른 실폭으로 렌더한다 — LATIN_0 만 교체, 이외 범위·한글 메트릭은 HYSinMyeongJo-Medium 공유.
static HANYANGSINMYEONGJO_LATIN_0: [u16; 95] = [518,333,401,465,614,939,833,241,404,386,535,579,237,588,246,404,509,509,509,509,509,509,509,509,509,509,281,298,746,588,754,526,939,816,719,711,763,693,667,763,798,342,465,807,658,983,798,763,640,772,719,667,790,790,798,1061,772,790,614,368,404,360,439,509,395,518,570,500,553,491,360,605,570,281,325,570,281,877,579,526,544,561,412,500,360,579,596,860,605,588,483,430,290,430,509];
static HANYANGSINMYEONGJO_LATIN_RANGES: [LatinRange; 7] = [
    LatinRange {
        start: 0x0020,
        end: 0x007E,
        widths: &HANYANGSINMYEONGJO_LATIN_0,
    },
    LatinRange {
        start: 0x00A0,
        end: 0x00FF,
        widths: &FONT_276_LATIN_1,
    },
    LatinRange {
        start: 0x2000,
        end: 0x206F,
        widths: &FONT_276_LATIN_2,
    },
    LatinRange {
        start: 0x2200,
        end: 0x22FF,
        widths: &FONT_276_LATIN_3,
    },
    LatinRange {
        start: 0x3000,
        end: 0x303F,
        widths: &FONT_276_LATIN_4,
    },
    LatinRange {
        start: 0x3130,
        end: 0x318F,
        widths: &FONT_276_LATIN_5,
    },
    LatinRange {
        start: 0xFF00,
        end: 0xFF5E,
        widths: &FONT_276_LATIN_6,
    },
];

// [#2430] 한양중고딕 실측 ASCII (한글 COM 무신축 래더 2026-07-20, 93/95 실측·2 보간 med=0.871).
// 한글은 한양중고딕 를 HYGothic-Medium 와 다른 실폭으로 렌더한다 — LATIN_0 만 교체, 이외 범위·한글 메트릭은 HYGothic-Medium 공유.
static HANYANGJUNGGOTHIC_LATIN_0: [u16; 95] = [
    518, 254, 371, 544, 544, 886, 658, 223, 298, 290, 377, 526, 254, 526, 254, 272, 509, 509, 509,
    509, 509, 509, 509, 509, 509, 509, 254, 254, 596, 535, 596, 535, 1026, 649, 640, 719, 711, 667,
    605, 772, 702, 254, 483, 649, 553, 825, 711, 763, 649, 772, 711, 649, 596, 702, 649, 921, 632,
    632, 596, 263, 290, 263, 456, 509, 263, 544, 535, 491, 544, 544, 263, 544, 544, 202, 202, 483,
    202, 825, 544, 553, 544, 544, 316, 491, 263, 544, 491, 711, 483, 474, 500, 316, 246, 316, 509,
];
static HANYANGJUNGGOTHIC_LATIN_RANGES: [LatinRange; 7] = [
    LatinRange {
        start: 0x0020,
        end: 0x007E,
        widths: &HANYANGJUNGGOTHIC_LATIN_0,
    },
    LatinRange {
        start: 0x00A0,
        end: 0x00FF,
        widths: &FONT_267_LATIN_1,
    },
    LatinRange {
        start: 0x2000,
        end: 0x206F,
        widths: &FONT_267_LATIN_2,
    },
    LatinRange {
        start: 0x2200,
        end: 0x22FF,
        widths: &FONT_267_LATIN_3,
    },
    LatinRange {
        start: 0x3000,
        end: 0x303F,
        widths: &FONT_267_LATIN_4,
    },
    LatinRange {
        start: 0x3130,
        end: 0x318F,
        widths: &FONT_267_LATIN_5,
    },
    LatinRange {
        start: 0xFF00,
        end: 0xFF5E,
        widths: &FONT_267_LATIN_6,
    },
];

// [#2430] 한양견명조 실측 ASCII (한글 COM 무신축 래더 2026-07-20, 93/95 실측·2 보간 med=0.911).
// 한글은 한양견명조 를 HYMyeongJo-Extra 와 다른 실폭으로 렌더한다 — LATIN_0 만 교체, 이외 범위·한글 메트릭은 HYMyeongJo-Extra 공유.
static HANYANGKYUNMYEONGJO_LATIN_0: [u16; 95] = [
    509, 412, 544, 474, 649, 974, 860, 388, 412, 395, 544, 649, 290, 658, 325, 430, 579, 579, 579,
    579, 579, 579, 579, 579, 579, 579, 342, 351, 754, 658, 754, 579, 956, 833, 825, 772, 816, 737,
    719, 833, 851, 386, 526, 877, 711, 1009, 833, 816, 693, 833, 763, 702, 842, 833, 825, 1097,
    816, 816, 667, 404, 430, 395, 430, 509, 395, 561, 605, 535, 596, 535, 447, 640, 614, 316, 333,
    614, 316, 912, 632, 570, 596, 596, 456, 535, 395, 614, 640, 886, 649, 623, 526, 465, 316, 465,
    509,
];
static HANYANGKYUNMYEONGJO_LATIN_RANGES: [LatinRange; 7] = [
    LatinRange {
        start: 0x0020,
        end: 0x007E,
        widths: &HANYANGKYUNMYEONGJO_LATIN_0,
    },
    LatinRange {
        start: 0x00A0,
        end: 0x00FF,
        widths: &FONT_271_LATIN_1,
    },
    LatinRange {
        start: 0x2000,
        end: 0x206F,
        widths: &FONT_271_LATIN_2,
    },
    LatinRange {
        start: 0x2200,
        end: 0x22FF,
        widths: &FONT_271_LATIN_3,
    },
    LatinRange {
        start: 0x3000,
        end: 0x303F,
        widths: &FONT_271_LATIN_4,
    },
    LatinRange {
        start: 0x3130,
        end: 0x318F,
        widths: &FONT_271_LATIN_5,
    },
    LatinRange {
        start: 0xFF00,
        end: 0xFF5E,
        widths: &FONT_271_LATIN_6,
    },
];

// [#2430] 한양견고딕 실측 ASCII (한글 COM 무신축 래더 2026-07-20, 93/95 실측·2 보간 med=0.905).
// 한글은 한양견고딕 를 HYGothic-Extra 와 다른 실폭으로 렌더한다 — LATIN_0 만 교체, 이외 범위·한글 메트릭은 HYGothic-Extra 공유.
static HANYANGKYUNGOTHIC_LATIN_0: [u16; 95] = [
    509, 342, 540, 596, 570, 930, 728, 308, 377, 386, 430, 535, 333, 526, 333, 351, 579, 579, 579,
    579, 579, 579, 579, 579, 579, 579, 333, 333, 737, 535, 719, 623, 1026, 737, 737, 737, 737, 684,
    623, 798, 737, 281, 570, 737, 623, 851, 737, 798, 684, 790, 737, 684, 623, 737, 684, 965, 684,
    684, 623, 342, 351, 342, 439, 509, 272, 570, 623, 570, 623, 570, 342, 623, 623, 281, 281, 570,
    281, 912, 623, 623, 623, 623, 395, 570, 342, 623, 570, 798, 570, 570, 509, 412, 307, 412, 509,
];
static HANYANGKYUNGOTHIC_LATIN_RANGES: [LatinRange; 7] = [
    LatinRange {
        start: 0x0020,
        end: 0x007E,
        widths: &HANYANGKYUNGOTHIC_LATIN_0,
    },
    LatinRange {
        start: 0x00A0,
        end: 0x00FF,
        widths: &FONT_266_LATIN_1,
    },
    LatinRange {
        start: 0x2000,
        end: 0x206F,
        widths: &FONT_266_LATIN_2,
    },
    LatinRange {
        start: 0x2200,
        end: 0x22FF,
        widths: &FONT_266_LATIN_3,
    },
    LatinRange {
        start: 0x3000,
        end: 0x303F,
        widths: &FONT_266_LATIN_4,
    },
    LatinRange {
        start: 0x3130,
        end: 0x318F,
        widths: &FONT_266_LATIN_5,
    },
    LatinRange {
        start: 0xFF00,
        end: 0xFF5E,
        widths: &FONT_266_LATIN_6,
    },
];

// [#2430] 휴먼명조 실측 ASCII (한글 COM 무신축 래더 2026-07-20, 93/95 실측·2 보간 med=0.854).
// 한글은 휴먼명조 를 HYSinMyeongJo-Medium 와 다른 실폭으로 렌더한다 — LATIN_0 만 교체, 이외 범위·한글 메트릭은 HYSinMyeongJo-Medium 공유.
static HUMANMYEONGJO_LATIN_0: [u16; 95] = [
    518, 211, 364, 675, 518, 772, 790, 219, 316, 316, 509, 509, 272, 509, 272, 316, 509, 509, 509,
    509, 509, 509, 509, 509, 509, 509, 272, 272, 509, 509, 509, 439, 798, 675, 614, 658, 693, 640,
    596, 719, 693, 263, 404, 658, 588, 798, 711, 728, 588, 702, 667, 509, 640, 693, 675, 956, 728,
    684, 614, 325, 316, 325, 368, 509, 333, 518, 553, 500, 518, 535, 404, 526, 570, 254, 272, 535,
    254, 816, 561, 526, 553, 553, 412, 421, 351, 553, 553, 737, 500, 526, 465, 298, 211, 298, 509,
];
static HUMANMYEONGJO_LATIN_RANGES: [LatinRange; 7] = [
    LatinRange {
        start: 0x0020,
        end: 0x007E,
        widths: &HUMANMYEONGJO_LATIN_0,
    },
    LatinRange {
        start: 0x00A0,
        end: 0x00FF,
        widths: &FONT_276_LATIN_1,
    },
    LatinRange {
        start: 0x2000,
        end: 0x206F,
        widths: &FONT_276_LATIN_2,
    },
    LatinRange {
        start: 0x2200,
        end: 0x22FF,
        widths: &FONT_276_LATIN_3,
    },
    LatinRange {
        start: 0x3000,
        end: 0x303F,
        widths: &FONT_276_LATIN_4,
    },
    LatinRange {
        start: 0x3130,
        end: 0x318F,
        widths: &FONT_276_LATIN_5,
    },
    LatinRange {
        start: 0xFF00,
        end: 0xFF5E,
        widths: &FONT_276_LATIN_6,
    },
];

static MEASURED_FONT_METRIC_OVERLAYS: [FontMetric; 5] = [
    FontMetric {
        name: "HanyangSinMyeongJo",
        bold: false,
        italic: false,
        em_size: 1024,
        latin_ranges: &HANYANGSINMYEONGJO_LATIN_RANGES,
        hangul: Some(&FONT_276_HANGUL),
    },
    FontMetric {
        name: "HanyangJungGothic",
        bold: false,
        italic: false,
        em_size: 1024,
        latin_ranges: &HANYANGJUNGGOTHIC_LATIN_RANGES,
        hangul: Some(&FONT_267_HANGUL),
    },
    FontMetric {
        name: "HanyangKyunMyeongJo",
        bold: false,
        italic: false,
        em_size: 1024,
        latin_ranges: &HANYANGKYUNMYEONGJO_LATIN_RANGES,
        hangul: Some(&FONT_271_HANGUL),
    },
    FontMetric {
        name: "HanyangKyunGothic",
        bold: false,
        italic: false,
        em_size: 1024,
        latin_ranges: &HANYANGKYUNGOTHIC_LATIN_RANGES,
        hangul: Some(&FONT_266_HANGUL),
    },
    FontMetric {
        name: "HumanMyeongJo",
        bold: false,
        italic: false,
        em_size: 1024,
        latin_ranges: &HUMANMYEONGJO_LATIN_RANGES,
        hangul: Some(&FONT_276_HANGUL),
    },
];
