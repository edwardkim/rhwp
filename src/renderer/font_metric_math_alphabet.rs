//! 확인한 SFNT의 수학 영숫자 cmap·hmtx. 기존 범위의 메트릭은 덮어쓰지 않는다.
//! scripts/extract_verified_math_alphabet_metrics.py로 생성한다.
use super::FontMetric;
// KoPubDotum, KoPubDotum Light, KoPub돋움체, KoPub돋움체 Light; SHA-256 018a0f18613a98d446696ea3d36194e288d5d8004aa58327789c830f6179e23d; 원 프로그램 em 1000
// KoPubDotum, KoPubDotum Medium, KoPub돋움체, KoPub돋움체 Medium; SHA-256 d0adfdcb40b7e73389128cc0222b53a91e5aa0e653f89967485a1f8b46124eb7; 원 프로그램 em 1000
// KoPubDotum, KoPubDotum Bold, KoPub돋움체, KoPub돋움체 Bold; SHA-256 967efe18063dd971c63dde3e8fcb247e0fcf16486a24ba2e284a716e44b90275; 원 프로그램 em 1000
// KoPubBatang, KoPubBatang Light, KoPub바탕체, KoPub바탕체 Light; SHA-256 b7438d61595fe9584706ab2cb3c9aa2acdc486d61c8299374a30b3e77bc2045f; 원 프로그램 em 1000
// KoPubBatang, KoPubBatang Medium, KoPub바탕체, KoPub바탕체 Medium; SHA-256 7a798592beb0d35bb557b010c2794cbcc988e89fcbcee0eb5edbb35161141ab4; 원 프로그램 em 1000
// KoPubBatang, KoPubBatang Bold, KoPub바탕체, KoPub바탕체 Bold; SHA-256 94de26a11aa25619526ac8183fde4f46e74e22c527acb42aea2e93f5313bbc5b; 원 프로그램 em 1000
// HCR Batang Ext, 함초롬바탕 확장; SHA-256 f88d957a26dd5a6360424e37b6d25b4192a83d922b7ac1fa9fb92d34e579afe4; 원 프로그램 em 1000
static PRESENT: [bool; 1024] = [
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
];
static WIDTHS: [u16; 1024] = [
    713, 651, 640, 704, 625, 584, 707, 740, 387, 500, 734, 558, 900, 740, 667, 624, 673, 748, 481,
    652, 728, 738, 1005, 726, 749, 540, 509, 560, 432, 560, 465, 427, 512, 586, 292, 239, 549, 298,
    867, 586, 509, 560, 560, 437, 376, 363, 586, 534, 733, 546, 538, 423, 678, 632, 696, 687, 615,
    601, 695, 691, 335, 454, 708, 549, 842, 691, 697, 611, 697, 698, 514, 686, 703, 739, 1002, 692,
    753, 540, 535, 492, 464, 535, 459, 330, 492, 567, 321, 349, 507, 279, 900, 610, 516, 535, 503,
    485, 460, 346, 581, 518, 750, 543, 541, 459, 718, 641, 647, 700, 611, 571, 708, 729, 354, 464,
    712, 562, 851, 719, 679, 614, 684, 750, 488, 653, 727, 742, 1023, 701, 752, 531, 577, 486, 490,
    555, 487, 359, 496, 648, 366, 321, 589, 335, 920, 647, 519, 507, 536, 487, 406, 450, 628, 527,
    771, 526, 582, 485, 860, 847, 749, 837, 588, 690, 714, 824, 579, 532, 817, 715, 1022, 806, 583,
    710, 693, 893, 700, 634, 872, 674, 956, 778, 707, 689, 510, 425, 364, 502, 363, 297, 490, 543,
    279, 300, 501, 289, 813, 580, 388, 527, 471, 378, 332, 298, 536, 460, 661, 440, 548, 430, 765,
    639, 512, 704, 521, 734, 560, 772, 592, 710, 657, 561, 1033, 908, 716, 657, 732, 762, 609, 714,
    653, 593, 993, 692, 682, 658, 606, 503, 437, 604, 448, 277, 546, 574, 339, 218, 497, 311, 823,
    618, 513, 468, 487, 414, 329, 336, 598, 432, 641, 492, 440, 476, 663, 762, 614, 762, 614, 663,
    762, 663, 614, 614, 663, 614, 910, 712, 762, 762, 762, 712, 762, 663, 663, 762, 910, 614, 712,
    663, 455, 455, 356, 455, 406, 307, 455, 455, 257, 257, 474, 257, 762, 504, 455, 504, 455, 356,
    406, 307, 504, 455, 712, 406, 504, 356, 743, 641, 674, 688, 700, 635, 700, 818, 440, 552, 799,
    660, 972, 752, 729, 589, 729, 756, 550, 667, 763, 772, 1032, 749, 738, 660, 689, 659, 542, 643,
    606, 426, 679, 660, 309, 462, 618, 347, 1025, 685, 625, 649, 658, 500, 609, 461, 687, 688, 972,
    646, 698, 564, 808, 854, 807, 834, 844, 793, 910, 853, 712, 708, 783, 627, 1052, 855, 838, 851,
    977, 879, 793, 744, 721, 855, 1126, 729, 807, 656, 531, 496, 437, 495, 453, 361, 503, 494, 320,
    289, 554, 341, 773, 556, 494, 527, 494, 456, 497, 374, 556, 515, 717, 512, 511, 398, 597, 599,
    610, 674, 552, 501, 679, 657, 365, 395, 605, 485, 738, 654, 689, 526, 689, 607, 597, 538, 640,
    597, 867, 599, 537, 599, 496, 515, 431, 515, 492, 291, 515, 523, 227, 285, 489, 227, 804, 523,
    501, 515, 515, 353, 431, 326, 523, 489, 676, 489, 489, 434, 671, 659, 625, 719, 589, 560, 702,
    726, 468, 476, 667, 548, 823, 733, 736, 633, 736, 677, 613, 588, 703, 660, 982, 660, 636, 597,
    549, 575, 483, 575, 546, 347, 575, 585, 281, 331, 551, 281, 870, 585, 564, 575, 575, 409, 488,
    375, 585, 534, 805, 550, 535, 490, 605, 608, 619, 680, 560, 508, 688, 667, 371, 401, 615, 492,
    749, 664, 699, 534, 699, 617, 606, 546, 649, 605, 881, 607, 545, 607, 503, 515, 431, 519, 492,
    291, 514, 523, 279, 285, 485, 279, 804, 523, 501, 515, 515, 353, 431, 326, 525, 488, 676, 489,
    488, 434, 684, 670, 636, 732, 600, 570, 715, 739, 476, 484, 679, 559, 838, 747, 750, 645, 750,
    689, 625, 599, 716, 672, 1000, 672, 648, 607, 573, 575, 483, 595, 546, 347, 575, 585, 359, 331,
    551, 359, 870, 585, 563, 575, 575, 409, 488, 375, 605, 533, 805, 550, 535, 490, 557, 557, 557,
    557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557,
    557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557,
    557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 557, 321, 349, 603, 603, 713, 651, 558, 582,
    625, 540, 740, 667, 387, 734, 660, 900, 740, 615, 667, 740, 624, 667, 640, 652, 694, 649, 726,
    694, 663, 828, 627, 514, 503, 490, 441, 420, 628, 539, 362, 554, 568, 628, 510, 467, 515, 603,
    524, 461, 560, 500, 614, 586, 539, 648, 685, 640, 468, 704, 522, 666, 522, 844, 678, 632, 576,
    524, 615, 540, 691, 697, 335, 708, 627, 842, 691, 629, 697, 690, 611, 697, 646, 686, 698, 635,
    692, 692, 662, 876, 587, 571, 639, 516, 491, 412, 546, 477, 341, 543, 538, 567, 566, 458, 516,
    592, 522, 451, 579, 513, 518, 592, 557, 666, 673, 570, 453, 614, 479, 589, 516, 718, 718, 641,
    560, 549, 611, 531, 729, 679, 354, 712, 668, 851, 719, 600, 679, 730, 614, 679, 626, 653, 702,
    651, 701, 734, 680, 881, 627, 597, 584, 524, 505, 468, 574, 518, 410, 583, 587, 642, 583, 457,
    519, 631, 552, 436, 639, 572, 520, 688, 657, 678, 661, 657, 420, 590, 575, 664, 519, 709, 671,
    659, 548, 697, 589, 597, 726, 736, 468, 667, 671, 823, 733, 617, 736, 726, 633, 736, 589, 588,
    625, 827, 660, 848, 731, 697, 672, 632, 735, 564, 481, 489, 633, 650, 370, 582, 614, 670, 587,
    478, 564, 691, 598, 462, 625, 508, 717, 751, 598, 834, 754, 575, 498, 579, 607, 808, 598, 832,
    684, 670, 559, 710, 600, 607, 739, 750, 476, 679, 684, 838, 747, 628, 750, 739, 645, 750, 600,
    599, 637, 842, 672, 848, 744, 710, 672, 632, 735, 564, 481, 489, 633, 650, 370, 582, 614, 670,
    587, 478, 564, 691, 598, 534, 625, 508, 717, 751, 598, 834, 754, 575, 498, 579, 607, 808, 598,
    801, 622, 522, 603, 603, 509, 509, 509, 509, 509, 509, 509, 509, 509, 509, 681, 681, 681, 681,
    681, 681, 681, 681, 681, 681, 555, 555, 555, 555, 555, 555, 555, 555, 555, 555, 614, 614, 614,
    614, 614, 614, 614, 614, 614, 614, 555, 555, 555, 555, 572, 555, 555, 555, 555, 555,
];

/// 원 프로그램의 cmap 결측이다. 미수집 DB 구간을 결측으로 추정하지 않는다.
pub(super) fn primary_has_glyph(family: &str, bold: bool, italic: bool, ch: char) -> Option<bool> {
    if bold || italic || !(0x1D400..=0x1D7FF).contains(&(ch as u32)) {
        return None;
    }
    let primary = family
        .split(',')
        .next()?
        .trim()
        .trim_matches('\'')
        .trim_matches('"');
    match primary {
        "KoPubBatang"
        | "KoPubBatang Bold"
        | "KoPubBatang Light"
        | "KoPubBatang Medium"
        | "KoPubDotum"
        | "KoPubDotum Bold"
        | "KoPubDotum Light"
        | "KoPubDotum Medium"
        | "KoPub돋움체"
        | "KoPub돋움체 Bold"
        | "KoPub돋움체 Light"
        | "KoPub돋움체 Medium"
        | "KoPub바탕체"
        | "KoPub바탕체 Bold"
        | "KoPub바탕체 Light"
        | "KoPub바탕체 Medium" => Some(false),
        _ => None,
    }
}

pub(super) fn entry(metric: &FontMetric, ch: char) -> Option<(bool, u16)> {
    if !(0x1D400..=0x1D7FF).contains(&(ch as u32)) {
        return None;
    }
    if (metric.name, metric.em_size, metric.bold, metric.italic)
        != ("HCR Batang Ext", 1000, false, false)
    {
        return None;
    }
    let index = (ch as u32 - 0x1D400) as usize;
    Some((PRESENT[index], WIDTHS[index]))
}

pub(super) fn width(metric: &FontMetric, ch: char) -> Option<u16> {
    let (present, width) = entry(metric, ch)?;
    (present && width > 0).then_some(width)
}
