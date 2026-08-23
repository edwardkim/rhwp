//! 폰트 메트릭 조회 facade.
//!
//! Task #4964 W6에서 historical generated data와 measured/manual overlay를 물리적으로 분리했다.
//! 이 파일은 공통 type, 별칭, lookup/index와 두 data 영역의 기존 순서 합성만 소유한다.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::renderer::font_rule_layout_metric_projection::find_font_rule_layout_metric;

#[derive(Debug)]
pub struct HangulMetric {
    pub cho_groups: u8,
    pub jung_groups: u8,
    pub jong_groups: u8,
    pub cho_map: &'static [u8],
    pub jung_map: &'static [u8],
    pub jong_map: &'static [u8],
    pub widths: &'static [u16],
}

#[derive(Debug)]
pub struct FontMetric {
    pub name: &'static str,
    pub bold: bool,
    pub italic: bool,
    pub em_size: u16,
    pub latin_ranges: &'static [LatinRange],
    pub hangul: Option<&'static HangulMetric>,
}

#[derive(Debug)]
pub struct LatinRange {
    pub start: u32,
    pub end: u32,
    pub widths: &'static [u16],
}

impl FontMetric {
    pub fn get_width(&self, ch: char) -> Option<u16> {
        let code = ch as u32;
        // 한글 음절 (U+AC00~U+D7A3)
        if code >= 0xAC00 && code <= 0xD7A3 {
            if let Some(h) = self.hangul {
                let idx = code - 0xAC00;
                let cho = (idx / (21 * 28)) as usize;
                let jung = ((idx % (21 * 28)) / 28) as usize;
                let jong = (idx % 28) as usize;
                let gi = h.cho_map[cho] as usize * h.jung_groups as usize * h.jong_groups as usize
                    + h.jung_map[jung] as usize * h.jong_groups as usize
                    + h.jong_map[jong] as usize;
                return h.widths.get(gi).copied();
            }
            return None;
        }
        // Latin 및 기타 범위
        for range in self.latin_ranges {
            if code >= range.start && code <= range.end {
                let w = range.widths[(code - range.start) as usize];
                return if w > 0 { Some(w) } else { None };
            }
        }
        None
    }
}

/// find_metric의 반환값: 메트릭 + 폴백 정보
pub struct MetricMatch {
    pub metric: &'static FontMetric,
    /// Bold 요청했으나 Bold 메트릭이 없어 Regular로 폴백됨
    /// → Faux Bold 폭 보정이 필요한 경우 true
    pub bold_fallback: bool,
}

/// 내장 메트릭 색인이 선택한 기존 3단 폴백 사다리의 단계.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MetricMatchKind {
    Exact,
    BoldOnly,
    NameFirst,
}

impl MetricMatchKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::BoldOnly => "boldOnly",
            Self::NameFirst => "nameFirst",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct MetricSelection {
    metric: &'static FontMetric,
    bold_fallback: bool,
    match_kind: MetricMatchKind,
    entry_index: usize,
}

/// `find_metric`가 숨겨 온 별칭과 선택 단계. 기존 반환값의 관측용 확장이다.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MetricLookupDecision<'a> {
    pub(crate) requested_name: &'a str,
    pub(crate) alias_resolved_name: &'a str,
    pub(crate) alias_rule_id: Option<&'static str>,
    pub(crate) metric: &'static FontMetric,
    pub(crate) bold_fallback: bool,
    pub(crate) match_kind: MetricMatchKind,
    pub(crate) entry_index: usize,
}

/// W7 전환 전 한국어 폰트 이름 → 내장 메트릭 영문 이름 별칭 오라클.
///
/// runtime은 canonical registry에서 생성한 `layout_metric` projection을 사용한다.
/// 이 표와 근거 주석은 생성 결과의 전건 동등성을 검증하기 위해 test build에만 보존한다.
/// 기존 계층:
/// 1. style_resolver.rs 가 한국어 별칭 → 한국어 정규명 (예: 한양중고딕 → HY중고딕)
/// 2. 본 함수가 한국어 정규명 → 영문 DB 이름 (예: HY중고딕 → HYGothic-Medium)
/// 3. find_metric 이 FONT_METRICS 에서 영문 이름으로 조회
///
/// 본한글/본명조 는 정식 메트릭 DB 엔트리가 없어 Pretendard/Noto Serif KR 로
/// 근사. 근거: 같은 한글 원천 (Source Han Sans KR), 이미 번들, OFL 호환.
/// 한계: Latin 폭 미세 차이, weight 축은 2단계로 근사 (본한글vf 는 wght 중간값을
/// Regular/Bold 중 가까운 쪽으로). CJK 폰트는 weight 별 한글 폭 차이가 작으므로
/// 실무 허용. 정식 DB 엔트리 추가는 별도 이슈.
#[cfg(test)]
fn resolve_metric_alias(name: &str) -> &str {
    match name {
        "함초롬돋움" => "HCR Dotum",
        // [#2279] 한컴돋움/한컴바탕의 실체는 Haansoft Dotum/Batang
        // (HDOTUM.TTF/HBATANG.TTF name table). HCR(함초롬) 계열과 메트릭이
        // 다르므로 ('*' 0.583 vs 0.498em, 한글 음절 1.0 vs 0.97em) 별도 연결.
        "한컴돋움" => "Haansoft Dotum",
        "돋움" => "Dotum",
        "함초롬바탕" => "HCR Batang",
        "한컴바탕" => "Haansoft Batang",
        "바탕" => "Batang", // 윈도우 TTF 바탕
        "맑은 고딕" => "Malgun Gothic",
        "나눔고딕" => "NanumGothic",
        "나눔명조" => "NanumMyeongjo",
        // 윈도우 시스템 폰트 (가변폭)
        "바탕체" => "BatangChe",
        "굴림" => "Gulim",
        "궁서" => "Gungsuh",
        // 윈도우 시스템 폰트 (고정폭)
        "굴림체" => "GulimChe",
        "돋움체" => "DotumChe",
        "궁서체" => "GungsuhChe",
        // 오픈소스 대체 폰트
        "D2Coding" | "D2 Coding" => "D2Coding",
        "고운바탕" | "Gowun Batang" => "Gowun Batang",
        "고운돋움" | "Gowun Dodum" => "Gowun Dodum",
        "Pretendard" | "프리텐다드" => "Pretendard",
        // HY 계열 — 한글 정규명 → 메트릭 DB 영문명 (Issue #259)
        // style_resolver.rs 가 한국어 별칭을 정규명으로 먼저 변환한 뒤 여기로 온다.
        "HY중고딕" => "HYGothic-Medium",
        "HY견고딕" => "HYGothic-Extra",
        "HY헤드라인M" => "HYHeadLine-Medium",
        "HY견명조" => "HYMyeongJo-Extra",
        "HY신명조" => "HYSinMyeongJo-Medium",
        "HY그래픽" => "HYGraphic-Medium",
        "HY궁서" => "HYGungSo-Bold",
        // [#2430] 한양·휴먼 계열 — 한글 실측상 HY 와 ASCII 폭이 다른 별개 페이스.
        "한양신명조" => "HanyangSinMyeongJo",
        "한양중고딕" => "HanyangJungGothic",
        "한양견명조" => "HanyangKyunMyeongJo",
        "한양견고딕" => "HanyangKyunGothic",
        "휴먼명조" => "HumanMyeongJo",
        "신명조" => "HanyangSinMyeongJo",
        // HY 계열 추가 — 한국어 사용명 → DB 단축명 (Task #885)
        // 한컴 폰트 미설치 환경에서 한국어 face 이름으로 들어온 경우 폴백 메트릭 매칭.
        "HY수평선B" => "HYsupB",
        "HY수평선M" => "HYsupM",
        "HY울릉도B" => "HYwulB",
        "HY울릉도M" => "HYwulM",
        "HY태백B" => "HYtbrB",
        "HY동녘B" => "HYdnkB",
        "HY동녘M" => "HYdnkM",
        // HY각헤드라인M: 정확한 메트릭 부재. 헤드라인M으로 근사 (각진/원형 차이로
        // 글리프 모양은 다르나 폭/베이스라인은 유사 — RMSE 가산 감소 목적).
        "HY각헤드라인M" => "HYHeadLine-Medium",
        // Source Han Sans 계열 (본한글 · 본고딕) → Pretendard 근사 (Issue #259)
        // 한글 원천 동일 (Source Han Sans KR), OFL 호환, 이미 번들. 본한글vf 의
        // 임의 weight 도 Pretendard Regular/Bold 중 가까운 쪽으로 근사.
        "본한글" | "본한글vf" | "본한글 Medium" | "본한글M" | "본고딕" | "본고딕vf"
        | "Source Han Sans" | "Source Han Sans K" | "Source Han Sans KR" | "SourceHanSans"
        | "SourceHanSansKR" | "SourceHanSansK" | "Noto Sans CJK KR" => "Pretendard",
        // Source Han Serif 계열 (본명조) → Noto Serif KR 근사 (Issue #259)
        "본명조"
        | "본명조vf"
        | "본명조M"
        | "Source Han Serif"
        | "Source Han Serif K"
        | "Source Han Serif KR"
        | "SourceHanSerif"
        | "SourceHanSerifKR"
        | "SourceHanSerifK"
        | "Noto Serif CJK KR" => "Noto Serif KR",
        _ => name,
    }
}

/// Canonical font rule projection에서 metric alias와 W2 ruleId를 함께 해소한다.
fn resolve_projected_metric_alias(name: &str) -> (&str, Option<&'static str>) {
    match find_font_rule_layout_metric(name) {
        Some(rule) => (rule.target_face_or_policy, Some(rule.rule_id)),
        None => (name, None),
    }
}

/// (bold, italic) → 색인 슬롯 번호. 조합이 4개뿐이라 이름당 배열로 선계산한다.
fn metric_slot_index(bold: bool, italic: bool) -> usize {
    ((bold as usize) << 1) | (italic as usize)
}

/// 이름별 선계산 색인 — 슬롯 값은 (metric, bold_fallback).
///
/// [#4149] 글자 측정마다 find_metric 이 호출되어 600 엔트리 선형 스캔
/// (문자열 비교 × 최대 3단 폴백)이 캐럿 rect 질의 지연의 주요 원인이었다.
/// 이름이 테이블에 있으면 3단(이름만 매칭)이 항상 성공하므로 슬롯에 Option 이
/// 필요 없고, 조회 실패 = 이름 부재 = 기존 None 과 동일하다.
///
/// 키를 (name, bold, italic) 튜플 대신 이름 단독으로 두는 이유: 튜플 키는
/// &'static str 수명 때문에 임의 사용자 폰트명(&str)으로 borrow 조회가 불가능하다.
static METRIC_INDEX: OnceLock<HashMap<&'static str, [MetricSelection; 4]>> = OnceLock::new();

fn metric_index() -> &'static HashMap<&'static str, [MetricSelection; 4]> {
    METRIC_INDEX.get_or_init(|| {
        // 선형 스캔의 "테이블 첫 매칭 우선" 계약 재현: 테이블 순서대로 순회하며
        // (name, bold, italic) 조합별 첫 엔트리와 이름 첫 등장 엔트리만 기록한다.
        struct FirstSeen {
            exact: [Option<(&'static FontMetric, usize)>; 4],
            first: (&'static FontMetric, usize),
        }
        let mut seen: HashMap<&'static str, FirstSeen> = HashMap::new();
        for (entry_index, m) in FONT_METRICS.iter().enumerate() {
            let entry = seen.entry(m.name).or_insert(FirstSeen {
                exact: [None; 4],
                first: (m, entry_index),
            });
            let idx = metric_slot_index(m.bold, m.italic);
            if entry.exact[idx].is_none() {
                entry.exact[idx] = Some((m, entry_index));
            }
        }
        // 슬롯 선주입 순서 = legacy 폴백 사다리와 동일:
        // 1단 정확 매칭 → 2단 bold 매칭(italic 무시) → 3단 이름만 첫 엔트리.
        seen.into_iter()
            .map(|(name, fs)| {
                let mut slots = [MetricSelection {
                    metric: fs.first.0,
                    bold_fallback: false,
                    match_kind: MetricMatchKind::NameFirst,
                    entry_index: fs.first.1,
                }; 4];
                for bold in [false, true] {
                    for italic in [false, true] {
                        slots[metric_slot_index(bold, italic)] =
                            if let Some((metric, entry_index)) =
                                fs.exact[metric_slot_index(bold, italic)]
                            {
                                MetricSelection {
                                    metric,
                                    bold_fallback: false,
                                    match_kind: MetricMatchKind::Exact,
                                    entry_index,
                                }
                            } else if let Some((metric, entry_index)) =
                                fs.exact[metric_slot_index(bold, false)]
                            {
                                MetricSelection {
                                    metric,
                                    bold_fallback: false,
                                    match_kind: MetricMatchKind::BoldOnly,
                                    entry_index,
                                }
                            } else {
                                // bold 요청이었으면 Faux Bold 보정 표시 (legacy 3단과 동일)
                                MetricSelection {
                                    metric: fs.first.0,
                                    bold_fallback: bold,
                                    match_kind: MetricMatchKind::NameFirst,
                                    entry_index: fs.first.1,
                                }
                            };
                    }
                }
                (name, slots)
            })
            .collect()
    })
}

pub fn find_metric(name: &str, bold: bool, italic: bool) -> Option<MetricMatch> {
    let decision = find_metric_decision(name, bold, italic)?;
    Some(MetricMatch {
        metric: decision.metric,
        bold_fallback: decision.bold_fallback,
    })
}

pub(crate) fn find_metric_decision<'a>(
    name: &'a str,
    bold: bool,
    italic: bool,
) -> Option<MetricLookupDecision<'a>> {
    let (alias_resolved_name, alias_rule_id) = resolve_projected_metric_alias(name);
    let slots = metric_index().get(alias_resolved_name)?;
    let selected = slots[metric_slot_index(bold, italic)];
    Some(MetricLookupDecision {
        requested_name: name,
        alias_resolved_name,
        alias_rule_id,
        metric: selected.metric,
        bold_fallback: selected.bold_fallback,
        match_kind: selected.match_kind,
        entry_index: selected.entry_index,
    })
}

/// [#4709] 이 스타일의 글자폭 배치에 실제로 쓰인 내장 메트릭 face 이름.
///
/// `measure_char_width_embedded`(text_measurement.rs)의 해석 순서를 따른다:
/// CSS 체인 첫 face → KoPub 전용 표 → 별칭 해석 → 메트릭 DB. DB 에 없는
/// face(폭이 휴리스틱 추정으로 결정되는 경우)는 None — 주석도 붙이지 않는다.
pub fn layout_metric_face_name(font_family: &str, bold: bool, italic: bool) -> Option<String> {
    let primary = font_family
        .split(',')
        .next()
        .unwrap_or(font_family)
        .trim()
        .trim_matches('\'');
    // KoPub 은 kopub_char_width 전용 표가 find_metric 보다 앞선다 — face 자체가 정체.
    let lower = primary.to_lowercase();
    if primary.contains("KoPub돋움체")
        || lower.contains("kopub dotum")
        || primary.contains("KoPub바탕체")
        || lower.contains("kopub batang")
    {
        return Some(primary.to_string());
    }
    find_metric(primary, bold, italic).map(|m| m.metric.name.to_string())
}

/// 기존 선형 스캔 구현 — 색인 등가성 검증 전용으로 보존 (수정 금지).
#[cfg(test)]
fn legacy_find_metric(name: &str, bold: bool, italic: bool) -> Option<MetricMatch> {
    let name = resolve_metric_alias(name);
    // 정확한 매칭 (name + bold + italic)
    if let Some(m) = FONT_METRICS
        .iter()
        .find(|m| m.name == name && m.bold == bold && m.italic == italic)
    {
        return Some(MetricMatch {
            metric: m,
            bold_fallback: false,
        });
    }
    // bold만 매칭 (italic 무시)
    if let Some(m) = FONT_METRICS
        .iter()
        .find(|m| m.name == name && m.bold == bold && !m.italic)
    {
        return Some(MetricMatch {
            metric: m,
            bold_fallback: false,
        });
    }
    // Regular 폴백 — bold 요청이었으면 bold_fallback 표시
    FONT_METRICS
        .iter()
        .find(|m| m.name == name)
        .map(|m| MetricMatch {
            metric: m,
            bold_fallback: bold,
        })
}

#[cfg(test)]
fn legacy_find_metric_decision<'a>(
    name: &'a str,
    bold: bool,
    italic: bool,
) -> Option<MetricLookupDecision<'a>> {
    let alias_resolved_name = resolve_metric_alias(name);
    let exact = FONT_METRICS
        .iter()
        .enumerate()
        .find(|(_, metric)| {
            metric.name == alias_resolved_name && metric.bold == bold && metric.italic == italic
        })
        .map(|(index, metric)| (metric, index, MetricMatchKind::Exact, false));
    let bold_only = || {
        FONT_METRICS
            .iter()
            .enumerate()
            .find(|(_, metric)| {
                metric.name == alias_resolved_name && metric.bold == bold && !metric.italic
            })
            .map(|(index, metric)| (metric, index, MetricMatchKind::BoldOnly, false))
    };
    let name_first = || {
        FONT_METRICS
            .iter()
            .enumerate()
            .find(|(_, metric)| metric.name == alias_resolved_name)
            .map(|(index, metric)| (metric, index, MetricMatchKind::NameFirst, bold))
    };
    let (metric, entry_index, match_kind, bold_fallback) =
        exact.or_else(bold_only).or_else(name_first)?;
    Some(MetricLookupDecision {
        requested_name: name,
        alias_resolved_name,
        alias_rule_id: None,
        metric,
        bold_fallback,
        match_kind,
        entry_index,
    })
}

include!("font_metrics_generated.rs");
include!("font_metrics_overlays.rs");

/// 기존 600-entry 물리 순서를 노출하는 논리 view.
///
/// generated 595개 뒤에 measured overlay 5개를 연결하며 정렬·dedupe하지 않는다.
struct FontMetrics;

static FONT_METRICS: FontMetrics = FontMetrics;

impl FontMetrics {
    fn iter(&self) -> impl Iterator<Item = &'static FontMetric> {
        GENERATED_FONT_METRICS
            .iter()
            .chain(MEASURED_FONT_METRIC_OVERLAYS.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::font_rule_layout_metric_projection::FONT_RULE_LAYOUT_METRIC_RULES;
    use sha2::{Digest, Sha256};

    const NOTO_SANS_KR_REGULAR: &[u8] =
        include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");

    #[test]
    fn issue_4442_noto_sans_kr_ascii_advances_match_tracked_font() {
        let digest = Sha256::digest(NOTO_SANS_KR_REGULAR);
        assert_eq!(
            &digest[..],
            &[
                0x6e, 0x06, 0xa7, 0xfe, 0x5d, 0x69, 0x6c, 0xa7, 0x19, 0x89, 0x4a, 0x23, 0xf3, 0x6b,
                0xb2, 0xb1, 0xbe, 0x8c, 0x81, 0x6a, 0x59, 0x37, 0xcd, 0x4a, 0xd0, 0xf2, 0x3c, 0xa6,
                0x77, 0x80, 0xdd, 0x74,
            ]
        );

        let face = ttf_parser::Face::parse(NOTO_SANS_KR_REGULAR, 0)
            .expect("tracked bundled font must parse");
        let metric = find_metric("Noto Sans KR", false, false).expect("shared regular metric");
        assert_eq!(metric.metric.em_size, face.units_per_em());
        for ch in ' '..='~' {
            let glyph = face.glyph_index(ch).expect("printable ASCII glyph");
            let advance = face
                .glyph_hor_advance(glyph)
                .expect("printable ASCII horizontal advance");
            assert_eq!(
                metric.metric.get_width(ch),
                Some(advance),
                "shared advance for {ch:?}"
            );
        }
    }

    #[test]
    fn hy_gothic_medium_maps_correctly() {
        let m = find_metric("HY중고딕", false, false);
        assert!(m.is_some(), "HY중고딕 매핑 실패");
        assert_eq!(m.unwrap().metric.name, "HYGothic-Medium");
    }

    #[test]
    fn hy_family_all_map() {
        for (korean, expected_english) in &[
            ("HY중고딕", "HYGothic-Medium"),
            ("HY견고딕", "HYGothic-Extra"),
            ("HY헤드라인M", "HYHeadLine-Medium"),
            ("HY견명조", "HYMyeongJo-Extra"),
            ("HY신명조", "HYSinMyeongJo-Medium"),
            ("HY그래픽", "HYGraphic-Medium"),
            ("HY궁서", "HYGungSo-Bold"),
        ] {
            let m = find_metric(korean, false, false);
            assert!(m.is_some(), "{} 매핑 실패", korean);
            assert_eq!(
                m.unwrap().metric.name,
                *expected_english,
                "{} 이 {} 에 매핑되지 않음",
                korean,
                expected_english
            );
        }
    }

    #[test]
    fn source_han_sans_family_maps_to_pretendard() {
        for name in &[
            "본한글",
            "본한글vf",
            "본한글 Medium",
            "본한글M",
            "본고딕",
            "본고딕vf",
            "Source Han Sans",
            "Source Han Sans K",
            "Source Han Sans KR",
            "SourceHanSans",
            "SourceHanSansKR",
            "SourceHanSansK",
            "Noto Sans CJK KR",
        ] {
            let m = find_metric(name, false, false);
            assert!(m.is_some(), "{} 매핑 실패 (Pretendard 기대)", name);
            assert_eq!(m.unwrap().metric.name, "Pretendard");
        }
    }

    #[test]
    fn source_han_serif_family_maps_to_noto_serif_kr() {
        for name in &[
            "본명조",
            "본명조vf",
            "본명조M",
            "Source Han Serif",
            "Source Han Serif K",
            "Source Han Serif KR",
            "SourceHanSerif",
            "SourceHanSerifKR",
            "SourceHanSerifK",
            "Noto Serif CJK KR",
        ] {
            let m = find_metric(name, false, false);
            assert!(m.is_some(), "{} 매핑 실패 (Noto Serif KR 기대)", name);
            assert_eq!(m.unwrap().metric.name, "Noto Serif KR");
        }
    }

    #[test]
    fn hy_family_bold_fallback() {
        // HY 계열은 DB 에 bold 변형이 없음 (Stage 1 실측 확인).
        // bold=true 요청 시 Regular 폴백 + bold_fallback=true 반환.
        let m = find_metric("HY중고딕", true, false);
        assert!(m.is_some(), "HY중고딕 bold 폴백 실패");
        let mm = m.unwrap();
        assert_eq!(mm.metric.name, "HYGothic-Medium");
        assert!(
            mm.bold_fallback,
            "HY중고딕 bold 요청 시 bold_fallback=true 여야 함"
        );
    }

    #[test]
    fn task885_hy_extra_aliases_resolve() {
        // Task #885 — 한국어 사용명 → DB 단축명 매핑 회귀 테스트.
        // 우변 메트릭이 FONT_METRICS 에 실재해야 하며 (feedback_font_metrics_alias_sync),
        // find_metric 이 Some 반환 + name 일치를 보장한다.
        for (korean, expected_english) in &[
            ("HY수평선B", "HYsupB"),
            ("HY수평선M", "HYsupM"),
            ("HY울릉도B", "HYwulB"),
            ("HY울릉도M", "HYwulM"),
            ("HY태백B", "HYtbrB"),
            ("HY동녘B", "HYdnkB"),
            ("HY동녘M", "HYdnkM"),
            ("HY각헤드라인M", "HYHeadLine-Medium"),
        ] {
            let m = find_metric(korean, false, false);
            assert!(m.is_some(), "{} 별칭 매핑 실패", korean);
            assert_eq!(
                m.unwrap().metric.name,
                *expected_english,
                "{} 이 {} 에 매핑되지 않음",
                korean,
                expected_english
            );
        }
    }

    #[test]
    fn non_korean_font_unchanged() {
        let m = find_metric("함초롬바탕", false, false);
        assert!(m.is_some(), "함초롬바탕 기존 매핑 회귀");
        assert_eq!(m.unwrap().metric.name, "HCR Batang");
    }

    #[test]
    fn index_matches_legacy_linear_scan_exhaustively() {
        // [#4149] O(1) 색인 도입 계약: legacy 선형 스캔(3단 폴백 사다리)과
        // 결과가 100% 동일해야 한다. FONT_METRICS 전체 name + 알려진 alias
        // 전체 + 테이블에 없는 임의 이름 × (bold, italic) 4조합 전수 비교.
        let mut names: Vec<&str> = FONT_METRICS.iter().map(|m| m.name).collect();
        // resolve_metric_alias 좌변 전체 (별칭 추가 시 여기도 갱신).
        names.extend([
            "함초롬돋움",
            "한컴돋움",
            "돋움",
            "함초롬바탕",
            "한컴바탕",
            "바탕",
            "맑은 고딕",
            "나눔고딕",
            "나눔명조",
            "바탕체",
            "굴림",
            "궁서",
            "굴림체",
            "돋움체",
            "궁서체",
            "D2Coding",
            "D2 Coding",
            "고운바탕",
            "Gowun Batang",
            "고운돋움",
            "Gowun Dodum",
            "Pretendard",
            "프리텐다드",
            "HY중고딕",
            "HY견고딕",
            "HY헤드라인M",
            "HY견명조",
            "HY신명조",
            "HY그래픽",
            "HY궁서",
            "한양신명조",
            "한양중고딕",
            "한양견명조",
            "한양견고딕",
            "휴먼명조",
            "신명조",
            "HY수평선B",
            "HY수평선M",
            "HY울릉도B",
            "HY울릉도M",
            "HY태백B",
            "HY동녘B",
            "HY동녘M",
            "HY각헤드라인M",
            "본한글",
            "본한글vf",
            "본한글 Medium",
            "본한글M",
            "본고딕",
            "본고딕vf",
            "Source Han Sans",
            "Source Han Sans K",
            "Source Han Sans KR",
            "SourceHanSans",
            "SourceHanSansKR",
            "SourceHanSansK",
            "Noto Sans CJK KR",
            "본명조",
            "본명조vf",
            "본명조M",
            "Source Han Serif",
            "Source Han Serif K",
            "Source Han Serif KR",
            "SourceHanSerif",
            "SourceHanSerifKR",
            "SourceHanSerifK",
            "Noto Serif CJK KR",
        ]);
        // 테이블에 없는 임의 사용자 폰트명 — 기존과 동일하게 None 이어야 한다.
        names.extend([
            "NoSuchFont-Regular",
            "가상의사용자폰트",
            "Comic Sans MS",
            "",
        ]);

        for name in names {
            for bold in [false, true] {
                for italic in [false, true] {
                    let new = find_metric(name, bold, italic);
                    let old = legacy_find_metric(name, bold, italic);
                    match (new, old) {
                        (None, None) => {}
                        (Some(n), Some(o)) => {
                            assert!(
                                std::ptr::eq(n.metric, o.metric),
                                "metric 불일치: {name:?} bold={bold} italic={italic} \
                                 (new={}/b{}/i{}, old={}/b{}/i{})",
                                n.metric.name,
                                n.metric.bold,
                                n.metric.italic,
                                o.metric.name,
                                o.metric.bold,
                                o.metric.italic,
                            );
                            assert_eq!(
                                n.bold_fallback, o.bold_fallback,
                                "bold_fallback 불일치: {name:?} bold={bold} italic={italic}"
                            );
                        }
                        (n, o) => panic!(
                            "Some/None 불일치: {name:?} bold={bold} italic={italic} \
                             new={} old={}",
                            n.is_some(),
                            o.is_some()
                        ),
                    }
                }
            }
        }

        for rule in FONT_RULE_LAYOUT_METRIC_RULES {
            assert_eq!(rule.source_boundary_id, "rust-metric.metric-alias");
            let source = rule.source_face.expect("metric alias source face");
            let (projected_alias, projected_rule_id) = resolve_projected_metric_alias(source);
            assert_eq!(projected_alias, resolve_metric_alias(source));
            assert_eq!(projected_alias, rule.target_face_or_policy);
            assert_eq!(projected_rule_id, Some(rule.rule_id));

            for bold in [false, true] {
                for italic in [false, true] {
                    let projected = find_metric_decision(source, bold, italic);
                    let legacy = legacy_find_metric_decision(source, bold, italic);
                    match (projected, legacy) {
                        (None, None) => {}
                        (Some(projected), Some(legacy)) => {
                            assert!(std::ptr::eq(projected.metric, legacy.metric));
                            assert_eq!(projected.entry_index, legacy.entry_index);
                            assert_eq!(projected.match_kind, legacy.match_kind);
                            assert_eq!(projected.bold_fallback, legacy.bold_fallback);
                            assert_eq!(projected.alias_resolved_name, legacy.alias_resolved_name);
                            assert_eq!(projected.alias_rule_id, Some(rule.rule_id));
                        }
                        (projected, legacy) => panic!(
                            "metric projection mismatch for {source:?} bold={bold} italic={italic}: projected={}, legacy={}",
                            projected.is_some(),
                            legacy.is_some()
                        ),
                    }
                }
            }
        }

        let sentinel = "__rhwp_w7_unregistered_metric__";
        assert_eq!(resolve_projected_metric_alias(sentinel), (sentinel, None));
        assert!(find_metric_decision(sentinel, false, false).is_none());
    }
}
