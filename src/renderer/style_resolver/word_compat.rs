//! [#8582] 호환 대상이 MS 워드인 문서의 줄 높이 배율.
//!
//! 한컴은 문서 설정의 호환 대상이 MS 워드이면 저장 줄 없는 문단의 한 줄 높이를 글자 크기가
//! 아니라 `글자 크기 × H(글꼴)` 로 잡고, 기준선은 그 한 줄 높이의 3/4 에 둔다.
//! 비율 줄 간격은 한 줄 높이에 비율을 곱하고(기준선은 그대로), 고정 줄 간격은 고정값을 피치로
//! 쓰고 기준선을 고정값의 4/5 에 둔다.
//!
//! 한 줄에 글꼴이 섞이면 가장 큰 `크기 × H` 가 한 줄 높이다. 문단 끝 표지는 영문 슬롯 글꼴로
//! 잰다. 마지막 줄에서 표지가 줄 글자들보다 크면 줄 피치에 줄 글자의 아래쪽(한 줄 높이의 1/4)이
//! 줄 간격 뒤에 더해진다.
//!
//! 기준: 데스크톱 한컴 2024 인쇄(Hancom PDF) 의 글자 기준선(600dpi, 12 HU 단위).
//! - 함초롬바탕 11pt 비율 100%: 피치 1860 HU → H = 1.691. 함초롬바탕 10pt 첫 줄 기준선 1270 HU.
//! - 맑은 고딕 11pt 비율 160%: 피치 3043 HU → H = 1.729(= 1.3 × 2724/2048). 기준선 1421 HU.
//! - 함초롬바탕 한글 + 맑은 고딕 영문 슬롯: 빈칸이 있으면 피치 1903 HU(맑은 고딕 한 줄),
//!   한글만이면 2371 HU(맑은 고딕 한 줄 + 함초롬바탕 아래쪽 465 HU).
//! - 고정 1800 HU: 피치 1800, 기준선 1440 HU.
//!
//! 데스크톱 한컴은 HWP5 에도 같은 규칙을 쓴다(같은 합성 문서를 HWP5 로 저장한 판의 2020 엔진
//! 인쇄가 HWPX 2024 인쇄와 기준선까지 같다). 단 HWP5 는 `LAYOUT_COMPATIBILITY` 의 문단 수준
//! 플래그(둘째 UINT32)의 비트 4(0x10)가 켜져 있어야 한다. 문단 수준 비트를 하나씩만 켠 판 13개
//! 중 비트 4 판만 워드 줄 높이로, 나머지 12개와 글자·구역·개체 수준만 켠 판, 플래그 0 판은
//! 한/글 줄 높이로 인쇄된다. HWPX 는 `layoutCompatibility` 가 비어 있어도 워드 줄 높이를
//! 쓴다(합성 HWPX 인쇄).
//!
//! 굴림·Gulim은 같은 TTC를 쓰며 11pt 비율 100%의 한 줄은 1430 HU다. 한컴바탕도 같은
//! 높이다. Arial·Times New Roman은 2024 인쇄에서 라틴 글자 줄과 한글 대체 줄의 피치가 다르다.
//! 라틴은 hhea의 줄 간격을 포함한 2355/2048 em이고 한글은 실제 대체 글꼴의 메트릭을 따른다.
use crate::model::document::DocInfo;
use crate::model::document_compatibility::{CompatibilityTarget, DocumentCompatibility};

/// HWP 5 표 56의 글자 단위 서식. 한컴에서 저장한 Word 문서의 비트를
/// 하나씩 바꿔 인쇄하면 bit 7은 서양 문자 사이 한 칸의 글꼴 전진을,
/// bit 11은 한컴 2024의 공백 자간 조정을 독립적으로 켠다.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct WordSpacing {
    latin_font_space: bool,
    reduce_space: bool,
}
crate::storage::clone_copy!(WordSpacing);

impl WordSpacing {
    pub(super) fn for_document(compat: DocumentCompatibility, hangul_2024: bool) -> Self {
        if compat.target != CompatibilityTarget::MsWord {
            return Self::default();
        }
        let flags = compat.flags[0];
        Self {
            latin_font_space: flags & (1 << 7) != 0,
            reduce_space: hangul_2024 && flags & (1 << 11) != 0,
        }
    }

    pub(crate) fn active(self) -> bool {
        self.latin_font_space || self.reduce_space
    }

    pub(crate) fn metric_at(
        self,
        text: &[char],
        index: usize,
    ) -> Option<crate::model::paragraph::SpaceMetric> {
        if !self.active() || text.get(index) != Some(&' ') {
            return None;
        }
        let western = |ch: &char| {
            !ch.is_whitespace() && !ch.is_control() && super::detect_lang_category(*ch) == 1
        };
        let latin = self.latin_font_space
            && index
                .checked_sub(1)
                .and_then(|i| text.get(i))
                .is_some_and(western)
            && text.get(index + 1).is_some_and(western);
        (latin || self.reduce_space).then_some(crate::model::paragraph::SpaceMetric::Word {
            latin,
            reduced: self.reduce_space,
        })
    }
}

/// 글꼴 이름의 한 줄 높이 배율 `H`(글자 크기 대비).
fn line_height_factor_for_face(face: &str, lang: usize) -> f64 {
    let face = face.trim();
    let family = crate::renderer::font_metrics_data::find_metric(face, false, false)
        .map_or(face, |found| found.metric.name);
    // Times New Roman과 Arial의 cmap에는 한글 음절이 없다. 실제 한컴 PDF의
    // 한글 run에서 확인한 첫 대체 face는 paint의 세리프·고딕 선택과 같다.
    let family = if lang == 0 && matches!(family, "Times New Roman" | "Arial") {
        crate::renderer::generic_fallback(family)
            .split(',')
            .next()
            .unwrap_or(family)
            .trim_matches('\'')
    } else {
        family
    };
    match family {
        "HCR Batang" | "HCR Dotum" => 1.691,
        "Malgun Gothic" => 1.3 * 2724.0 / 2048.0,
        "Times New Roman" | "Arial" => 2355.0 / 2048.0,
        // gulim.ttc의 네 face는 hhea (879, -145), UPEM 1024를 공유한다.
        "Gulim" | "GulimChe" | "Dotum" | "DotumChe" | "Haansoft Batang" => 1.3,
        "휴먼명조" | "Batang" => 1.3 * 1.148,
        _ => 1.3,
    }
}

/// 글자 모양 번호별 7개 언어 슬롯의 한 줄 높이 배율.
pub(crate) fn line_factors(doc_info: &DocInfo) -> Vec<[f64; 7]> {
    doc_info
        .char_shapes
        .iter()
        .map(|shape| {
            let mut factors = [1.3; 7];
            for (lang, factor) in factors.iter_mut().enumerate() {
                if let Some(font) = doc_info
                    .font_faces
                    .get(lang)
                    .and_then(|faces| faces.get(shape.font_ids[lang] as usize))
                {
                    *factor = line_height_factor_for_face(&font.name, lang);
                }
            }
            factors
        })
        .collect()
}
