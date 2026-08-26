//! 광고 vs 실지원 정직성 표.
//!
//! `BackendCapabilities` 필드가 **최종 산출물이 그 성질을 보존하는가**를
//! 뜻한다. 이 모듈은 어댑터별 기대 광고와, 산출물에서 관찰할 수 있는
//! 증거를 한 표로 묶는다. M06-3 이 기존 단위 시험에 접은 대조를
//! 픽스처·통합 시험이 같은 표로 재사용한다.

use super::backends::{NullBackend, TraceBackend};
use super::caps::{BackendCapabilities, BackendFeature};
use super::png_adapter::PngBackend;
use super::skia_adapter::SkiaBackend;
use super::svg_adapter::SvgBackend;
use super::traits::RenderBackend;

/// 한 백엔드의 정직성 행.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HonestyRow {
    /// `BackendCapabilities::name`.
    pub name: &'static str,
    /// 래스터 전용인가.
    pub raster_only: bool,
    /// 벡터 텍스트.
    pub vector_text: bool,
    /// 폰트 내장.
    pub embedded_fonts: bool,
    /// 그라디언트.
    pub gradients: bool,
    /// 클립.
    pub clipping: bool,
    /// 이미지.
    pub images: bool,
    /// 여러 페이지.
    pub multi_page: bool,
    /// 결정론.
    pub deterministic: bool,
    /// 한 줄 근거.
    pub note: &'static str,
}

impl HonestyRow {
    /// 능력 선언에서 행을 뜬다.
    pub fn from_caps(caps: BackendCapabilities, note: &'static str) -> Self {
        Self {
            name: caps.name,
            raster_only: caps.raster_only,
            vector_text: caps.vector_text,
            embedded_fonts: caps.embedded_fonts,
            gradients: caps.gradients,
            clipping: caps.clipping,
            images: caps.images,
            multi_page: caps.multi_page,
            deterministic: caps.deterministic,
            note,
        }
    }

    /// `is_consistent` 와 같은 불변식.
    pub fn is_consistent(self) -> bool {
        !(self.raster_only && self.vector_text)
    }

    /// 이 행이 `feature` 를 켜는가.
    pub fn supports(self, feature: BackendFeature) -> bool {
        match feature {
            BackendFeature::VectorText => self.vector_text,
            BackendFeature::EmbeddedFonts => self.embedded_fonts,
            BackendFeature::Gradients => self.gradients,
            BackendFeature::Clipping => self.clipping,
            BackendFeature::Images => self.images,
            BackendFeature::MultiPage => self.multi_page,
            BackendFeature::Deterministic => self.deterministic,
        }
    }

    /// 광고 선언과 같은가.
    pub fn matches_caps(self, caps: BackendCapabilities) -> Result<(), String> {
        let live = Self::from_caps(caps, self.note);
        if live.name != self.name {
            return Err(format!("name {} != {}", live.name, self.name));
        }
        for feature in ALL_FEATURES {
            if live.supports(*feature) != self.supports(*feature) {
                return Err(format!(
                    "{} {:?}: 광고 {} 표 {}",
                    self.name,
                    feature,
                    live.supports(*feature),
                    self.supports(*feature)
                ));
            }
        }
        if live.raster_only != self.raster_only {
            return Err(format!(
                "{} raster_only 광고 {} 표 {}",
                self.name, live.raster_only, self.raster_only
            ));
        }
        Ok(())
    }
}

/// 질의 가능한 능력 전체. 순서는 `BackendFeature` match 와 같다.
pub const ALL_FEATURES: &[BackendFeature] = &[
    BackendFeature::VectorText,
    BackendFeature::EmbeddedFonts,
    BackendFeature::Gradients,
    BackendFeature::Clipping,
    BackendFeature::Images,
    BackendFeature::MultiPage,
    BackendFeature::Deterministic,
];

/// SVG 산출물에서 관찰하는 증거.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SvgObservation {
    /// `<text` 가 있고 정직성 문자열이 보이는가.
    pub vector_text: bool,
    /// `linearGradient`/`radialGradient`/`<gradient`.
    pub gradients: bool,
    /// `<image` 또는 `data:image`.
    pub images: bool,
    /// `clipPath` 또는 `clip-path`.
    pub clipping: bool,
    /// `@font-face` 또는 `data:font`.
    pub embedded_fonts: bool,
}

/// SVG 문자열에서 증거를 읽는다.
pub fn observe_svg(svg: &str, honesty_text: &str) -> SvgObservation {
    let visible = svg_visible_text(svg);
    SvgObservation {
        vector_text: svg.contains("<text") && visible.contains(honesty_text),
        gradients: svg.contains("linearGradient")
            || svg.contains("radialGradient")
            || svg.contains("<gradient"),
        images: svg.contains("<image") || svg.contains("data:image"),
        clipping: svg.contains("clipPath") || svg.contains("clip-path"),
        embedded_fonts: svg.contains("@font-face") || svg.contains("data:font"),
    }
}

/// 태그 사이 글자만 이어 붙인다.
pub fn svg_visible_text(svg: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in svg.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag && !ch.is_whitespace() => out.push(ch),
            _ => {}
        }
    }
    out
}

/// PNG 시그니처.
pub const PNG_SIGNATURE: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// PNG 바이트가 시그니처로 시작하는가.
pub fn looks_like_png(bytes: &[u8]) -> bool {
    bytes.starts_with(PNG_SIGNATURE)
}

/// 현재 빌드의 기대 정직성 표.
pub fn expected_honesty_table() -> Vec<HonestyRow> {
    let svg = HonestyRow::from_caps(
        SvgBackend::new().capabilities(),
        "페이지별 SVG. 클립·폰트 내장·다중 페이지는 끈다",
    );
    let null = HonestyRow::from_caps(
        NullBackend::new().capabilities(),
        "계측만. 시각 capability 없음",
    );
    let trace = HonestyRow::from_caps(
        TraceBackend::new().capabilities(),
        "결정론 추적. 시각 capability 없음",
    );
    let png_live = PngBackend::raster_available();
    let png = HonestyRow {
        name: "png",
        raster_only: true,
        vector_text: false,
        embedded_fonts: false,
        gradients: png_live,
        clipping: false,
        images: png_live,
        multi_page: false,
        deterministic: false,
        note: "native-skia 가 있을 때만 이미지·그라디언트",
    };
    let skia_live = SkiaBackend::raster_available();
    let skia = HonestyRow {
        name: "skia",
        raster_only: true,
        vector_text: false,
        embedded_fonts: false,
        gradients: skia_live,
        clipping: false,
        images: skia_live,
        multi_page: false,
        deterministic: false,
        note: "래스터 문서. PNG 어댑터와 같은 가용성",
    };
    vec![svg, null, trace, png, skia]
}

/// 표의 모든 행이 자기모순이 아니고 실광고와 같은지 본다.
pub fn honesty_table_holds() -> Result<(), String> {
    let live = [
        SvgBackend::new().capabilities(),
        NullBackend::new().capabilities(),
        TraceBackend::new().capabilities(),
        PngBackend::new().capabilities(),
        SkiaBackend::new().capabilities(),
    ];
    let table = expected_honesty_table();
    if table.len() != live.len() {
        return Err(format!("표 {}행 vs 실광고 {}개", table.len(), live.len()));
    }
    for (row, caps) in table.iter().zip(live) {
        if !row.is_consistent() {
            return Err(format!("{} 자기모순", row.name));
        }
        if !caps.is_consistent() {
            return Err(format!("{} 광고 자기모순", caps.name));
        }
        row.matches_caps(caps)?;
    }
    if PngBackend::raster_available() != SkiaBackend::raster_available() {
        return Err("png/skia 가용성이 갈리면 안 된다".into());
    }
    Ok(())
}

/// `multi_page` 광고가 두 번째 `begin_page` 판정과 같은지 보는 서술.
pub fn multi_page_contract_summary(caps: BackendCapabilities) -> &'static str {
    if caps.supports(BackendFeature::MultiPage) {
        "두 번째 begin_page 는 성공해야 한다"
    } else {
        "두 번째 begin_page 는 MultiplePagesUnsupported 여야 한다"
    }
}
