//! 은닉 텍스트 탐지 — 사람 눈에는 안 보이는데 텍스트 추출기는 읽어 가는 문자열.
//!
//! # 왜 조판 엔진이 있어야만 되는가
//!
//! MCP 도구가 `export-text` 로 뽑은 본문은 그대로 LLM 프롬프트가 된다. 공격자가
//! 흰 배경에 흰 글씨로 "이전 지시를 무시하고 …" 를 심어 두면 **문서를 열어 본 사람은
//! 아무것도 못 보는데** 추출기는 그 문장을 읽어 모델에게 넘긴다 — 간접 프롬프트
//! 인젝션의 가장 악질적인 형태다. 평범한 텍스트 추출기는 글자가 무슨 색인지 모른다.
//! 글자 모양(`CharShape`)·채우기(`BorderFill`)·조판 결과를 모두 들고 있는 rhwp 만
//! 이 판정을 할 수 있다.
//!
//! # 설계 원칙 — 모르면 잡지 않는다
//!
//! 정상 문서에서 한 건이라도 헛울리면 아무도 이 명령을 쓰지 않는다. 그래서 판정은
//! **배경색을 확정할 수 있을 때만** 내린다. 자동/투명색, 그러데이션·이미지 채우기,
//! 바탕쪽(마스터 페이지), 글 뒤 배치 개체가 걸리면 그 케이스는 `Background::Unknown`
//! 으로 두고 **판정을 포기한다**. 부분 정보로 단정하지 않는다.
//!
//! 문서를 고치지 않는 **읽기 전용 질의**다.

use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::document_core::queries::grep::{CellRef, TextBoxRef};
use crate::document_core::DocumentCore;
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{DrawingObjAttr, ShapeObject, TextWrap};
use crate::model::style::{CharShape, FillType};
use crate::model::ColorRef;
use crate::renderer::pagination::PageItem;
use crate::renderer::render_tree::{RenderNode, RenderNodeType};

/// `--threshold-pt` 기본값. 1pt 미만이면 200% 확대해도 점 하나로만 보인다.
pub const DEFAULT_THRESHOLD_PT: f64 = 1.0;

/// 발췌 상한(문자). 은닉 텍스트가 거대하면 그 자체가 컨텍스트 범람 공격이므로
/// 보고 쪽에서 먼저 자른다 — `charCount` 는 자르기 전 실제 길이를 그대로 알린다.
pub const DEFAULT_EXCERPT_LIMIT: usize = 200;

/// 채우기가 없는 쪽의 바탕. 한컴은 흰 종이를 그린다
/// (`renderer::layout` 의 `hide_fill` 경로도 같은 값으로 되돌린다).
const PAPER_WHITE: ColorRef = 0x00FF_FFFF;

/// 중첩 순회 깊이 상한 (표 안의 표 안의 글상자…). 악성 입력의 스택 폭주 방지.
const MAX_NEST_DEPTH: usize = 8;

/// 탐지 옵션.
#[derive(Debug, Clone, Copy)]
pub struct HiddenTextOptions {
    /// 쪽 밖 배치(`off_page`) 탐지 여부. 기본 꺼짐 — 조판 좌표 판정이라 오탐 여지가 있다.
    pub include_off_page: bool,
    /// `near_invisible` 임계(pt). 실효 글자 크기가 이 값 **미만**이면 잡는다.
    pub threshold_pt: f64,
    /// 발췌 상한(문자).
    pub excerpt_limit: usize,
}

impl Default for HiddenTextOptions {
    fn default() -> Self {
        Self {
            include_off_page: false,
            threshold_pt: DEFAULT_THRESHOLD_PT,
            excerpt_limit: DEFAULT_EXCERPT_LIMIT,
        }
    }
}

/// 은닉 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HiddenKind {
    /// 글자색이 배경(글자 음영·문단/셀 채우기·쪽 바탕)과 같다.
    SameAsBackground,
    /// 실효 글자 크기가 임계 미만이다.
    NearInvisible,
    /// 실효 글자 크기가 0이다.
    ZeroSize,
    /// 조판 결과 쪽 경계 **완전히** 밖에 놓였다.
    OffPage,
}

/// 배경색의 출처 — 소비자가 판정 근거를 되짚을 수 있도록 함께 보고한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BackgroundSource {
    /// 글자 음영색(`CharShape.shade_color`).
    CharShade,
    /// 문단 배경(`ParaShape.border_fill_id`).
    Paragraph,
    /// 표 셀 배경(`Cell.border_fill_id`).
    TableCell,
    /// 글상자 채우기.
    TextBox,
    /// 쪽 바탕(쪽 테두리/배경 또는 흰 종이).
    Page,
}

/// 배경색 판정 결과.
///
/// `Unknown` 은 "배경이 없다"가 아니라 **"확정할 수 없다"** 다. 이 값이 나오면 색
/// 기반 판정을 하지 않는다 — 부분 정보로 단정하면 그것이 곧 오탐이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Background {
    Known {
        color: ColorRef,
        source: BackgroundSource,
    },
    Unknown,
}

/// 문자 한 개에 대한 판정.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Verdict {
    ZeroSize {
        effective_pt: f64,
    },
    SameAsBackground {
        text_color: ColorRef,
        background: ColorRef,
        source: BackgroundSource,
    },
    NearInvisible {
        effective_pt: f64,
    },
}

impl Verdict {
    pub fn kind(&self) -> HiddenKind {
        match self {
            Verdict::ZeroSize { .. } => HiddenKind::ZeroSize,
            Verdict::SameAsBackground { .. } => HiddenKind::SameAsBackground,
            Verdict::NearInvisible { .. } => HiddenKind::NearInvisible,
        }
    }
}

/// 판정 근거 수치. 종류에 따라 채워지는 필드가 다르므로 전부 선택 필드다.
#[derive(Debug, Clone, Default, Serialize)]
pub struct HiddenDetail {
    #[serde(rename = "textColor", skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
    #[serde(rename = "shadeColor", skip_serializing_if = "Option::is_none")]
    pub shade_color: Option<String>,
    #[serde(rename = "backgroundColor", skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(rename = "backgroundSource", skip_serializing_if = "Option::is_none")]
    pub background_source: Option<BackgroundSource>,
    #[serde(rename = "effectivePt", skip_serializing_if = "Option::is_none")]
    pub effective_pt: Option<f64>,
    #[serde(rename = "thresholdPt", skip_serializing_if = "Option::is_none")]
    pub threshold_pt: Option<f64>,
    #[serde(rename = "bbox", skip_serializing_if = "Option::is_none")]
    pub bbox: Option<BBox>,
    #[serde(rename = "pageSize", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<PageSize>,
}

/// 쪽 밖 판정에 쓰인 조판 사각형 (px, 쪽 내 절대 좌표).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct BBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// 쪽 크기 (px).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct PageSize {
    pub w: f64,
    pub h: f64,
}

/// 탐지 1건.
#[derive(Debug, Clone, Serialize)]
pub struct HiddenTextFinding {
    pub kind: HiddenKind,
    /// 구역 인덱스.
    pub section: usize,
    /// 본문 문단 인덱스 (표 셀·글상자 안이면 그 컨트롤을 담은 본문 문단).
    pub paragraph: usize,
    /// 0부터 시작하는 글로벌 쪽 번호. 조판에 배치되지 않았으면 생략된다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// 표 셀 안의 탐지면 셀 좌표 (`search --json` 과 같은 주소 어휘).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell: Option<CellRef>,
    /// 글상자 안의 탐지면 글상자 좌표.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub textbox: Option<TextBoxRef>,
    /// 은닉 문자열 발췌 (제어문자 제거, `excerpt_limit` 상한).
    pub excerpt: String,
    /// 은닉 문자 수 (제어문자 제외, 자르기 **전** 실제 길이).
    #[serde(rename = "charCount")]
    pub char_count: usize,
    pub detail: HiddenDetail,
}

/// 탐지 보고 봉투 본문. `schemaVersion`·`source` 는 CLI 봉투가 덧붙인다.
#[derive(Debug, Clone, Serialize)]
pub struct HiddenTextReport {
    #[serde(rename = "hiddenText")]
    pub hidden_text: Vec<HiddenTextFinding>,
    #[serde(rename = "hiddenCharCount")]
    pub hidden_char_count: usize,
    pub clean: bool,
}

// ── 색 해석 ────────────────────────────────────────────────────────────────

/// `ColorRef`(0x00BBGGRR) 를 `#RRGGBB` 로.
fn hex(color: ColorRef) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        color & 0xFF,
        (color >> 8) & 0xFF,
        (color >> 16) & 0xFF
    )
}

// [#4155] `opaque_rgb`·`char_shade` 는 여기서 정의하지 않는다. 같은 질문을 렌더 백엔드
// 5곳이 각자 재구현하고 있었고 그중 둘은 서로 어긋나 있었다 — 정본을 `model::color` 로
// 올렸다. 판정기와 렌더러가 **글자 그대로 같은 술어**를 쓴다는 이 모듈의 원칙은 이제
// 문서가 아니라 타입으로 강제된다.
use crate::model::color::{char_shade, opaque_rgb};

// ── 판정 코어 (합성 입력으로 단위 테스트 가능) ─────────────────────────────

/// 실효 글자 크기(pt) = `base_size` / 100. **`relative_sizes` 를 곱하지 않는다.**
///
/// # 왜 스펙대로 곱하지 않는가
///
/// HWP 스펙상 실효 크기는 `base_size × relative_sizes[언어] / 100` 이 맞다. 그런데
/// **이 엔진은 그 곱을 하지 않는다** — `renderer::style_resolver` 의 글자 모양 해소는
/// `hwpunit_to_px(cs.base_size, dpi)` 로 크기를 정하고, 같은 함수가 장평(`ratios`)과
/// 자간(`spacings`)은 반영하면서 `relative_sizes` 는 쓰지 않는다. 실제로 `src/` 전체에서
/// `relative_sizes` 를 읽는 곳은 파서·모델·직렬화·편집뿐이고 렌더 경로는 0건이다.
///
/// 판정기가 렌더러보다 글자를 **작게** 계산하면, 화면에는 멀쩡히 보이는 글자를 은닉으로
/// 보고하게 된다. 은닉 판정의 기준은 "이 도구가 그려 내는 결과"여야 하므로 렌더러와
/// 같은 식을 쓴다.
///
/// 현실 문서에서 차이는 없다 — HWPX 60개 문서 4,298개 `charPr` 실측에서 `relSz != 100`
/// 은 0건이었다.
///
/// # 이력 (#4141)
///
/// 종전 이 주석은 "`CharShape::default()` 의 `relative_sizes = [0; 7]` 을 0% 로 오독해
/// 모든 글자를 0pt 로 보고하는 사고도 구조적으로 불가능해진다"를 부수 논거로 들었다.
/// 그 전제는 사라졌다 — `CharShape::default()` 는 이제 스펙 기본값 `[100; 7]` 이다
/// (`model/style.rs`). **결론은 그대로다**: 렌더러가 안 곱하므로 판정기도 안 곱한다.
/// 사라진 것은 근거 하나뿐이고, 아래 회귀 테스트가 0 입력에 대한 안전성을 직접 고정한다.
pub fn effective_pt(shape: &CharShape) -> f64 {
    shape.base_size.max(0) as f64 / 100.0
}

/// 문자 한 개를 판정한다. 보이면 `None`.
///
/// 우선순위는 **확실한 것부터**다: `ZeroSize`(아무것도 안 그려짐) →
/// `SameAsBackground`(색이 정확히 일치) → `NearInvisible`(임계 휴리스틱). 한 문자가
/// 여러 조건에 걸려도 한 종류로만 보고하므로 `hiddenCharCount` 는 중복 집계되지 않는다.
pub fn classify_char(
    shape: &CharShape,
    container: Background,
    threshold_pt: f64,
) -> Option<Verdict> {
    let pt = effective_pt(shape);
    if pt <= 0.0 {
        return Some(Verdict::ZeroSize { effective_pt: pt });
    }

    // 글자 음영이 있으면 그것이 바로 뒤 배경이다. 없으면 바깥 컨테이너 배경으로 내려간다.
    let background = match char_shade(shape.shade_color) {
        Some(color) => Background::Known {
            color,
            source: BackgroundSource::CharShade,
        },
        None => container,
    };
    if let (Some(text_color), Background::Known { color, source }) =
        (opaque_rgb(shape.text_color), background)
    {
        if text_color == color {
            return Some(Verdict::SameAsBackground {
                text_color,
                background: color,
                source,
            });
        }
    }

    if pt < threshold_pt {
        return Some(Verdict::NearInvisible { effective_pt: pt });
    }
    None
}

/// 보고 대상 문자인가 — 제어문자는 조판부호/필드 마커라 은닉의 대상이 아니다.
fn is_reportable(ch: char) -> bool {
    !ch.is_control()
}

/// 내용이 있는 문자인가 — 공백만 있는 런은 "숨길 것이 없다".
fn is_content(ch: char) -> bool {
    !ch.is_control() && !ch.is_whitespace()
}

// ── 배경 해소 ──────────────────────────────────────────────────────────────

/// `border_fill_id`(1-based, 0=없음) → 배경.
///
/// - `id == 0` 또는 채우기 없음 → `None` (바깥 층으로 내려간다)
/// - 단색 → `Some(Known)`
/// - 그러데이션·이미지·무늬 → `Some(Unknown)` (그 위 글자는 판정하지 않는다)
fn fill_background(
    doc: &DocumentCore,
    border_fill_id: u16,
    source: BackgroundSource,
) -> Option<Background> {
    if border_fill_id == 0 {
        return None;
    }
    let idx = (border_fill_id as usize).saturating_sub(1);
    let style = doc.styles.border_styles.get(idx)?;
    if style.gradient.is_some() || style.image_fill.is_some() || style.pattern.is_some() {
        return Some(Background::Unknown);
    }
    style
        .fill_color
        .and_then(opaque_rgb)
        .map(|color| Background::Known { color, source })
}

/// 그리기 개체(글상자 등) 채우기 → 배경.
///
/// 채우기가 아예 없으면 뒤에 무엇이 있는지 알 수 없으므로 `Unknown` 이다 — 쪽 바탕으로
/// 내려가면 "투명 글상자 뒤가 흰 종이"라고 단정하게 되는데, 그림 위에 놓인 글상자에서
/// 그 단정은 그대로 오탐이 된다.
fn shape_background(drawing: &DrawingObjAttr) -> Background {
    match drawing.fill.fill_type {
        FillType::Solid => match drawing.fill.solid.as_ref() {
            Some(solid) if solid.pattern_type == 0 => match opaque_rgb(solid.background_color) {
                Some(color) => Background::Known {
                    color,
                    source: BackgroundSource::TextBox,
                },
                None => Background::Unknown,
            },
            _ => Background::Unknown,
        },
        _ => Background::Unknown,
    }
}

/// `ShapeObject` 에서 공통 그리기 속성을 꺼낸다
/// (`document_core::helpers::get_textbox_from_shape` 와 같은 범위).
fn drawing_of(shape: &ShapeObject) -> Option<&DrawingObjAttr> {
    match shape {
        ShapeObject::Rectangle(s) => Some(&s.drawing),
        ShapeObject::Ellipse(s) => Some(&s.drawing),
        ShapeObject::Polygon(s) => Some(&s.drawing),
        ShapeObject::Curve(s) => Some(&s.drawing),
        _ => None,
    }
}

impl DocumentCore {
    /// 구역의 쪽 바탕색.
    ///
    /// 바탕쪽(마스터 페이지)이 있거나 "글 뒤" 배치 개체가 하나라도 있으면 본문 글자 뒤에
    /// 무엇이 깔리는지 알 수 없으므로 `Unknown` 이다. 어두운 배경 위 흰 글씨는 **보이는**
    /// 글씨인데, 이를 구별하지 못한 채 흰 종이를 가정하면 표지 문서가 통째로 오탐이 된다.
    fn page_background(&self, sec_idx: usize) -> Background {
        let Some(section) = self.document.sections.get(sec_idx) else {
            return Background::Unknown;
        };
        let def = &section.section_def;
        if !def.master_pages.is_empty() && !def.hide_master_page {
            return Background::Unknown;
        }
        if section.paragraphs.iter().any(|para| {
            para.controls.iter().any(|ctrl| match ctrl {
                Control::Shape(shape) => shape.common().text_wrap == TextWrap::BehindText,
                Control::Picture(pic) => pic.common.text_wrap == TextWrap::BehindText,
                _ => false,
            })
        }) {
            return Background::Unknown;
        }
        // 배경 감추기면 렌더러가 흰 종이로 되돌린다 (`renderer::layout` 의 hide_fill 경로).
        if def.hide_fill {
            return Background::Known {
                color: PAPER_WHITE,
                source: BackgroundSource::Page,
            };
        }
        fill_background(
            self,
            def.page_border_fill.border_fill_id,
            BackgroundSource::Page,
        )
        .unwrap_or(Background::Known {
            color: PAPER_WHITE,
            source: BackgroundSource::Page,
        })
    }

    /// 그림·그래픽이 놓인 쪽 번호 집합.
    ///
    /// # 왜 필요한가 — 실측으로 드러난 오탐
    ///
    /// 흰 글씨가 흰 종이 위에 있으면 안 보이지만, **사진 위에 있으면 잘 보인다.**
    /// `samples/tac-img-02.hwp` 6쪽의 흰 숫자는 `x 75.6~721.9, y 175.6~208.9` 를 덮는
    /// JPEG 배너 위에 얹힌 캡션 번호다(SVG 렌더로 좌표 대조 확인). 쪽 바탕만 보고
    /// "흰 종이 위 흰 글씨"로 단정하면 이런 정상 문서가 통째로 오탐이 된다.
    ///
    /// 그래서 **쪽 바탕(`BackgroundSource::Page`)을 근거로 한 판정에 한해** 그림이 놓인
    /// 쪽에서는 판정을 포기한다. 글자 음영·문단 배경·셀 배경은 글자 바로 뒤에 칠해지는
    /// 불투명 면이므로 이 제약을 받지 않는다.
    ///
    /// 조판 결과가 아니라 IR 로만 계산한다 — 쪽마다 렌더 트리를 세우는 비용 없이
    /// 앵커 문단이 **차지하는 모든 쪽**으로 근사하고, 개체가 다음 쪽으로 넘칠 수 있으므로
    /// 각 쪽의 **다음 쪽까지** 함께 표시해 안전 쪽으로 기운다.
    ///
    /// 첫 쪽만 표시하면 안 되는 이유도 실측이다 — `samples/tac-img-02.hwp` 는 수십 쪽에
    /// 걸치는 표의 셀 안에 배너 그림을 담는다. 표 호스트 문단의 첫 쪽만 표시하면 23쪽
    /// 배너 위 흰 숫자가 그대로 오탐으로 남는다(실측: 그림 `y 468.5~501.7` 안의 흰 '2').
    fn image_bearing_pages(&self, pages_of: &HashMap<(usize, usize), Vec<u32>>) -> HashSet<u32> {
        fn paints_over_text(ctrl: &Control) -> bool {
            match ctrl {
                Control::Picture(_) => true,
                Control::Shape(shape) => match shape.as_ref() {
                    // 그림/차트/OLE/그룹은 면을 덮는다.
                    ShapeObject::Picture(_)
                    | ShapeObject::Chart(_)
                    | ShapeObject::Ole(_)
                    | ShapeObject::Group(_) => true,
                    // 나머지 도형은 채우기가 있을 때만 뒤를 가린다.
                    other => drawing_of(other).is_some_and(|d| d.fill.fill_type != FillType::None),
                },
                _ => false,
            }
        }

        /// 문단이 (중첩 포함) 면을 덮는 개체를 품고 있는가.
        fn hosts_graphic(para: &Paragraph, depth: usize) -> bool {
            if depth >= MAX_NEST_DEPTH {
                return false;
            }
            para.controls.iter().any(|ctrl| {
                if paints_over_text(ctrl) {
                    return true;
                }
                match ctrl {
                    Control::Table(table) => table
                        .cells
                        .iter()
                        .any(|c| c.paragraphs.iter().any(|p| hosts_graphic(p, depth + 1))),
                    Control::Shape(shape) => drawing_of(shape)
                        .and_then(|d| d.text_box.as_ref())
                        .is_some_and(|tb| {
                            tb.paragraphs.iter().any(|p| hosts_graphic(p, depth + 1))
                        }),
                    _ => false,
                }
            })
        }

        let mut pages = HashSet::new();
        for (sec_idx, section) in self.document.sections.iter().enumerate() {
            for (para_idx, para) in section.paragraphs.iter().enumerate() {
                if !hosts_graphic(para, 0) {
                    continue;
                }
                for page in pages_of
                    .get(&(sec_idx, para_idx))
                    .into_iter()
                    .flatten()
                    .copied()
                {
                    pages.insert(page);
                    pages.insert(page + 1);
                }
            }
        }
        pages
    }

    /// `(구역, 본문 문단) → 그 문단이 놓인 **모든** 쪽`.
    ///
    /// 표처럼 여러 쪽에 걸치는 항목의 실제 점유 범위를 알아야 하는 곳(그림 쪽 표시)에서
    /// 쓴다. 보고용 주소는 첫 쪽 하나면 충분하므로 [`Self::hidden_text_page_index`] 와
    /// 용도를 나눠 둔다.
    fn hidden_text_paragraph_pages(&self) -> HashMap<(usize, usize), Vec<u32>> {
        let mut index: HashMap<(usize, usize), Vec<u32>> = HashMap::new();
        let mut global_offset = 0u32;
        for (sec_idx, pr) in self.pagination.iter().enumerate() {
            for (local_i, page) in pr.pages.iter().enumerate() {
                let global_page = global_offset + local_i as u32;
                for col in &page.column_contents {
                    for item in &col.items {
                        let para_index = match item {
                            PageItem::FullParagraph { para_index }
                            | PageItem::PartialParagraph { para_index, .. }
                            | PageItem::Table { para_index, .. }
                            | PageItem::PartialTable { para_index, .. }
                            | PageItem::Shape { para_index, .. } => Some(*para_index),
                            _ => None,
                        };
                        if let Some(p) = para_index {
                            let slot = index.entry((sec_idx, p)).or_default();
                            if !slot.contains(&global_page) {
                                slot.push(global_page);
                            }
                        }
                    }
                }
            }
            global_offset += pr.pages.len() as u32;
        }
        index
    }

    /// `(구역, 본문 문단) → 글로벌 쪽` 인덱스.
    ///
    /// 문단이 여러 쪽에 걸치면 **처음 등장한 쪽**을 쓴다(`grep` 과 같은 규약).
    fn hidden_text_page_index(&self) -> HashMap<(usize, usize), u32> {
        let mut index: HashMap<(usize, usize), u32> = HashMap::new();
        let mut global_offset = 0u32;
        for (sec_idx, pr) in self.pagination.iter().enumerate() {
            for (local_i, page) in pr.pages.iter().enumerate() {
                let global_page = global_offset + local_i as u32;
                for col in &page.column_contents {
                    for item in &col.items {
                        let para_index = match item {
                            PageItem::FullParagraph { para_index }
                            | PageItem::PartialParagraph { para_index, .. }
                            | PageItem::Table { para_index, .. }
                            | PageItem::PartialTable { para_index, .. }
                            | PageItem::Shape { para_index, .. } => Some(*para_index),
                            _ => None,
                        };
                        if let Some(p) = para_index {
                            index.entry((sec_idx, p)).or_insert(global_page);
                        }
                    }
                }
            }
            global_offset += pr.pages.len() as u32;
        }
        index
    }

    /// 쪽 경계 **완전히** 밖에 놓인 본문 문단 → (쪽, 사각형, 쪽 크기).
    ///
    /// "겹친다"가 아니라 "완전히 밖"만 잡는다. 경계에 살짝 걸치는 것은 정상 조판에서도
    /// 흔해서(테두리·머리말 여백) 임계 판정으로 만들면 오탐이 쏟아진다. 반대로 y = -5000
    /// 같은 은닉 배치는 언제나 완전히 밖이다.
    fn off_page_paragraphs(&self) -> HashMap<(usize, usize), (u32, BBox, PageSize)> {
        let mut found: HashMap<(usize, usize), (u32, BBox, PageSize)> = HashMap::new();
        for page in 0..self.page_count() {
            let Ok(tree) = self.build_page_render_tree(page) else {
                continue;
            };
            let size = PageSize {
                w: tree.root.bbox.width,
                h: tree.root.bbox.height,
            };
            if size.w <= 0.0 || size.h <= 0.0 {
                continue;
            }
            let mut stack: Vec<&RenderNode> = vec![&tree.root];
            while let Some(node) = stack.pop() {
                if !node.visible || node.editor_only {
                    continue;
                }
                stack.extend(node.children.iter());
                let RenderNodeType::TextRun(run) = &node.node_type else {
                    continue;
                };
                // 표 셀·수식 조각은 부모 좌표계를 따로 쓰는 경로가 있어 본문 런만 본다.
                if run.cell_context.is_some() || !run.text.chars().any(is_content) {
                    continue;
                }
                let (Some(sec), Some(para)) = (run.section_index, run.para_index) else {
                    continue;
                };
                let b = &node.bbox;
                let outside =
                    b.x + b.width <= 0.0 || b.y + b.height <= 0.0 || b.x >= size.w || b.y >= size.h;
                if !outside {
                    continue;
                }
                found.entry((sec, para)).or_insert((
                    page,
                    BBox {
                        x: b.x,
                        y: b.y,
                        w: b.width,
                        h: b.height,
                    },
                    size,
                ));
            }
        }
        found
    }

    /// 문서에서 은닉 텍스트를 찾아 보고한다. **문서를 수정하지 않는다.**
    pub fn detect_hidden_text(&self, opts: &HiddenTextOptions) -> HiddenTextReport {
        let page_index = self.hidden_text_page_index();
        let off_page = if opts.include_off_page {
            self.off_page_paragraphs()
        } else {
            HashMap::new()
        };

        let image_pages = self.image_bearing_pages(&self.hidden_text_paragraph_pages());

        let mut out: Vec<HiddenTextFinding> = Vec::new();
        for (sec_idx, section) in self.document.sections.iter().enumerate() {
            let section_bg = self.page_background(sec_idx);
            for (para_idx, para) in section.paragraphs.iter().enumerate() {
                let page = page_index.get(&(sec_idx, para_idx)).copied();

                // 쪽 바탕을 근거로 삼을 수 있는가. 조판에 배치되지 않았거나(쪽을 모른다)
                // 그림이 놓인 쪽이면 글자 뒤에 무엇이 깔리는지 확정할 수 없다.
                let page_bg = match page {
                    Some(p) if !image_pages.contains(&p) => section_bg,
                    _ => Background::Unknown,
                };

                // 쪽 밖 문단은 그 한 건으로 보고하고 색·크기 재판정을 하지 않는다
                // (같은 문자를 두 번 세지 않기 위함).
                if let Some((off_page_num, bbox, size)) = off_page.get(&(sec_idx, para_idx)) {
                    let (excerpt, count) = excerpt_of(&para.text, opts.excerpt_limit);
                    if count > 0 {
                        out.push(HiddenTextFinding {
                            kind: HiddenKind::OffPage,
                            section: sec_idx,
                            paragraph: para_idx,
                            page: Some(*off_page_num),
                            cell: None,
                            textbox: None,
                            excerpt,
                            char_count: count,
                            detail: HiddenDetail {
                                bbox: Some(*bbox),
                                page_size: Some(*size),
                                ..Default::default()
                            },
                        });
                        continue;
                    }
                }

                self.scan_paragraph(
                    sec_idx, para_idx, page, para, page_bg, None, None, opts, 0, &mut out,
                );
            }
        }

        let hidden_char_count = out.iter().map(|f| f.char_count).sum();
        HiddenTextReport {
            clean: out.is_empty(),
            hidden_text: out,
            hidden_char_count,
        }
    }

    /// 문단 하나(본문·셀·글상자 공통)를 훑고 중첩 컨트롤로 내려간다.
    #[allow(clippy::too_many_arguments)]
    fn scan_paragraph(
        &self,
        sec_idx: usize,
        body_para_idx: usize,
        page: Option<u32>,
        para: &Paragraph,
        container: Background,
        cell: Option<CellRef>,
        textbox: Option<TextBoxRef>,
        opts: &HiddenTextOptions,
        depth: usize,
        out: &mut Vec<HiddenTextFinding>,
    ) {
        // 문단 배경(문단 테두리/배경)이 있으면 컨테이너보다 안쪽이므로 우선한다.
        let para_bg = self
            .document
            .doc_info
            .para_shapes
            .get(para.para_shape_id as usize)
            .and_then(|ps| fill_background(self, ps.border_fill_id, BackgroundSource::Paragraph))
            .unwrap_or(container);

        for (verdict, text) in self.paragraph_runs(para, para_bg, opts.threshold_pt) {
            let (excerpt, count) = excerpt_of(&text, opts.excerpt_limit);
            if count == 0 {
                continue;
            }
            out.push(HiddenTextFinding {
                kind: verdict.kind(),
                section: sec_idx,
                paragraph: body_para_idx,
                page,
                cell: cell.clone(),
                textbox: textbox.clone(),
                excerpt,
                char_count: count,
                detail: detail_of(&verdict, opts.threshold_pt),
            });
        }

        if depth >= MAX_NEST_DEPTH {
            return;
        }
        for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
            match ctrl {
                Control::Table(table) => {
                    // 표 배경은 세 겹이다. `renderer::layout::table_layout` 이 표 전체 →
                    // 영역(zone) → 셀 순으로 칠하므로 글자 바로 뒤에 오는 것은 셀이고,
                    // 셀에 채우기가 없으면 영역, 그다음이 표 전체다.
                    //
                    // 영역을 빠뜨리면 안 되는 이유는 실측이다 — `samples/tac-img-02.hwp`
                    // 의 구역 제목 막대는 셀 `bf=5`(채우기 없음) + 영역 `bf=18`(색)로
                    // 되어 있고, 그 위의 흰 번호는 **잘 보인다**. 영역을 보지 않으면
                    // 쪽 바탕(흰 종이)까지 흘러내려가 그대로 오탐이 된다.
                    let table_bg =
                        fill_background(self, table.border_fill_id, BackgroundSource::TableCell);
                    for (cell_idx, table_cell) in table.cells.iter().enumerate() {
                        // 겹치는 영역이 여럿이면 나중에 칠해진 것이 위에 온다.
                        let zone_bg = table
                            .zones
                            .iter()
                            .filter(|z| {
                                (z.start_row..=z.end_row).contains(&table_cell.row)
                                    && (z.start_col..=z.end_col).contains(&table_cell.col)
                            })
                            .filter_map(|z| {
                                fill_background(self, z.border_fill_id, BackgroundSource::TableCell)
                            })
                            .next_back();
                        let cell_bg = fill_background(
                            self,
                            table_cell.border_fill_id,
                            BackgroundSource::TableCell,
                        )
                        .or(zone_bg)
                        .or(table_bg)
                        .unwrap_or(para_bg);
                        for (cp_idx, cp) in table_cell.paragraphs.iter().enumerate() {
                            self.scan_paragraph(
                                sec_idx,
                                body_para_idx,
                                page,
                                cp,
                                cell_bg,
                                Some(CellRef {
                                    control: ctrl_idx,
                                    cell: cell_idx,
                                    paragraph: cp_idx,
                                }),
                                textbox.clone(),
                                opts,
                                depth + 1,
                                out,
                            );
                        }
                    }
                }
                Control::Shape(shape) => {
                    let Some(drawing) = drawing_of(shape) else {
                        continue;
                    };
                    let Some(tb) = drawing.text_box.as_ref() else {
                        continue;
                    };
                    let tb_bg = shape_background(drawing);
                    for (tp_idx, tp) in tb.paragraphs.iter().enumerate() {
                        self.scan_paragraph(
                            sec_idx,
                            body_para_idx,
                            page,
                            tp,
                            tb_bg,
                            cell.clone(),
                            Some(TextBoxRef {
                                control: ctrl_idx,
                                paragraph: tp_idx,
                            }),
                            opts,
                            depth + 1,
                            out,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    /// 문단 텍스트를 글자 모양 단위로 판정해 **연속 구간**으로 묶는다.
    ///
    /// 판정이 같은 문자끼리만 한 건으로 합쳐지므로, 문단 일부만 흰 글씨인 경우에도
    /// 그 구간의 문자열만 발췌된다.
    fn paragraph_runs(
        &self,
        para: &Paragraph,
        container: Background,
        threshold_pt: f64,
    ) -> Vec<(Verdict, String)> {
        let chars: Vec<char> = para.text.chars().collect();
        if chars.is_empty() || para.char_shapes.is_empty() {
            return Vec::new();
        }
        // char_offsets 가 텍스트와 어긋난 문서(파서 경로에 따라 비어 있을 수 있다)에서는
        // UTF-16 오프셋을 직접 누적해 쓴다.
        let use_stored = para.char_offsets.len() == chars.len();

        let mut runs: Vec<(Verdict, String)> = Vec::new();
        let mut current: Option<(Verdict, String)> = None;
        let mut utf16_pos: u32 = 0;
        for (i, ch) in chars.iter().enumerate() {
            let offset = if use_stored {
                para.char_offsets[i]
            } else {
                utf16_pos
            };
            utf16_pos += ch.len_utf16() as u32;

            let verdict = self
                .char_shape_at(para, offset)
                .and_then(|shape| classify_char(shape, container, threshold_pt));
            match verdict {
                Some(v) => match current.as_mut() {
                    Some((cur, buf)) if *cur == v => buf.push(*ch),
                    _ => {
                        if let Some(done) = current.take() {
                            runs.push(done);
                        }
                        current = Some((v, ch.to_string()));
                    }
                },
                None => {
                    if let Some(done) = current.take() {
                        runs.push(done);
                    }
                }
            }
        }
        if let Some(done) = current.take() {
            runs.push(done);
        }
        runs
    }

    /// UTF-16 오프셋에 적용되는 글자 모양. 참조가 깨졌으면 `None` — 모르면 판정하지 않는다.
    fn char_shape_at(&self, para: &Paragraph, utf16_offset: u32) -> Option<&CharShape> {
        let mut chosen: Option<u32> = None;
        for cs in &para.char_shapes {
            if cs.start_pos <= utf16_offset {
                chosen = Some(cs.char_shape_id);
            } else {
                break;
            }
        }
        // 첫 CharShapeRef 가 0보다 뒤에서 시작하는 문서에서는 그 앞 글자를 판정하지 않는다.
        let id = chosen?;
        self.document.doc_info.char_shapes.get(id as usize)
    }
}

/// 판정 근거를 봉투용 detail 로.
fn detail_of(verdict: &Verdict, threshold_pt: f64) -> HiddenDetail {
    match verdict {
        Verdict::ZeroSize { effective_pt } => HiddenDetail {
            effective_pt: Some(*effective_pt),
            ..Default::default()
        },
        Verdict::SameAsBackground {
            text_color,
            background,
            source,
        } => HiddenDetail {
            text_color: Some(hex(*text_color)),
            shade_color: matches!(source, BackgroundSource::CharShade).then(|| hex(*background)),
            background_color: Some(hex(*background)),
            background_source: Some(*source),
            ..Default::default()
        },
        Verdict::NearInvisible { effective_pt } => HiddenDetail {
            effective_pt: Some(*effective_pt),
            threshold_pt: Some(threshold_pt),
            ..Default::default()
        },
    }
}

/// 발췌와 실제 문자 수. 제어문자는 빼고, 내용이 하나도 없으면 0을 돌려준다
/// (공백만 있는 런은 "숨길 것이 없다").
fn excerpt_of(text: &str, limit: usize) -> (String, usize) {
    if !text.chars().any(is_content) {
        return (String::new(), 0);
    }
    let kept: Vec<char> = text.chars().filter(|c| is_reportable(*c)).collect();
    let count = kept.len();
    if count <= limit {
        return (kept.into_iter().collect(), count);
    }
    let mut excerpt: String = kept.into_iter().take(limit).collect();
    excerpt.push('…');
    (excerpt, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 음영 없음(흰색 sentinel) — 대다수 HWP5 문서의 실제 값.
    /// `model::color::char_shade` 가 "없음"으로 보는 세 값 중 하나다 (#4155).
    const NO_SHADE: ColorRef = 0x00FF_FFFF;

    fn shape(base_size: i32, text_color: ColorRef, shade_color: ColorRef) -> CharShape {
        CharShape {
            base_size,
            text_color,
            shade_color,
            relative_sizes: [100; 7],
            ..Default::default()
        }
    }

    const PAGE_WHITE: Background = Background::Known {
        color: 0x00FF_FFFF,
        source: BackgroundSource::Page,
    };

    #[test]
    fn white_text_on_white_page_is_caught() {
        let cs = shape(1000, 0x00FF_FFFF, NO_SHADE);
        let v = classify_char(&cs, PAGE_WHITE, DEFAULT_THRESHOLD_PT);
        assert!(
            matches!(v, Some(Verdict::SameAsBackground { source, .. }) if source == BackgroundSource::Page),
            "{v:?}"
        );
    }

    #[test]
    fn black_text_on_white_page_is_clean() {
        let cs = shape(1000, 0x0000_0000, NO_SHADE);
        assert_eq!(classify_char(&cs, PAGE_WHITE, DEFAULT_THRESHOLD_PT), None);
    }

    #[test]
    fn black_shade_sentinel_is_not_a_black_background() {
        // 회귀: CharShape::default()/HML/HWPX 미지정이 남기는 shade_color=0 을 "검정 음영"
        // 으로 읽으면 검정 글자 = 검정 배경이 되어 정상 문서가 통째로 오탐된다
        // (실측 351 표본 중 HWP3 17개에서 31,907건). 렌더러도 0은 칠하지 않는다.
        let cs = shape(1000, 0x0000_0000, 0x0000_0000);
        assert_eq!(char_shade(cs.shade_color), None);
        assert_eq!(classify_char(&cs, PAGE_WHITE, DEFAULT_THRESHOLD_PT), None);
    }

    #[test]
    fn white_shade_sentinel_is_not_a_white_background() {
        // 흰색 sentinel 은 "음영 없음"이므로 음영 근거로는 잡지 않는다. 다만 쪽 바탕이
        // 흰 종이로 확정되면 그 근거(source=page)로 잡힌다 — 근거가 뒤바뀌지 않아야 한다.
        let cs = shape(1000, 0x00FF_FFFF, NO_SHADE);
        assert_eq!(char_shade(cs.shade_color), None);
        let v = classify_char(&cs, PAGE_WHITE, DEFAULT_THRESHOLD_PT);
        assert!(
            matches!(v, Some(Verdict::SameAsBackground { source, .. }) if source == BackgroundSource::Page),
            "{v:?}"
        );
        // 배경을 모르면 흰 글씨라도 판정하지 않는다.
        assert_eq!(
            classify_char(&cs, Background::Unknown, DEFAULT_THRESHOLD_PT),
            None
        );
    }

    #[test]
    fn text_equal_to_char_shade_is_caught_regardless_of_page() {
        // 노란 형광펜 위 노란 글씨 — 쪽 바탕이 무엇이든 보이지 않는다.
        let cs = shape(1000, 0x0000_FFFF, 0x0000_FFFF);
        let v = classify_char(&cs, Background::Unknown, DEFAULT_THRESHOLD_PT);
        assert!(
            matches!(v, Some(Verdict::SameAsBackground { source, .. }) if source == BackgroundSource::CharShade),
            "{v:?}"
        );
    }

    #[test]
    fn white_text_on_unknown_background_is_not_judged() {
        // 바탕쪽·그러데이션·글 뒤 개체 등으로 배경을 확정 못 하면 잡지 않는다.
        let cs = shape(1000, 0x00FF_FFFF, NO_SHADE);
        assert_eq!(
            classify_char(&cs, Background::Unknown, DEFAULT_THRESHOLD_PT),
            None
        );
    }

    #[test]
    fn white_text_on_dark_cell_is_visible() {
        let cs = shape(1000, 0x00FF_FFFF, NO_SHADE);
        let dark = Background::Known {
            color: 0x0000_0000,
            source: BackgroundSource::TableCell,
        };
        assert_eq!(classify_char(&cs, dark, DEFAULT_THRESHOLD_PT), None);
    }

    #[test]
    fn auto_color_is_never_judged_same_as_background() {
        // 0xFFFFFFFF = CLR_INVALID/자동. 흰색으로 단정하면 그것이 곧 오탐이다.
        let cs = shape(1000, 0xFFFF_FFFF, NO_SHADE);
        assert_eq!(classify_char(&cs, PAGE_WHITE, DEFAULT_THRESHOLD_PT), None);
    }

    #[test]
    fn tiny_and_zero_sizes_are_separated() {
        let tiny = shape(50, 0x0000_0000, NO_SHADE); // 0.5pt
        assert!(matches!(
            classify_char(&tiny, PAGE_WHITE, DEFAULT_THRESHOLD_PT),
            Some(Verdict::NearInvisible { .. })
        ));
        let zero = shape(0, 0x0000_0000, NO_SHADE);
        assert!(matches!(
            classify_char(&zero, PAGE_WHITE, DEFAULT_THRESHOLD_PT),
            Some(Verdict::ZeroSize { .. })
        ));
    }

    #[test]
    fn effective_size_ignores_relative_sizes_like_the_renderer_does() {
        // 스펙상 실효 크기는 base_size × relSz/100 이지만, 이 엔진의
        // `renderer::style_resolver` 는 base_size 만 픽셀로 환산하고 relSz 는 쓰지 않는다
        // (렌더 경로 참조 0건). 판정기가 렌더러보다 작게 계산하면 화면에 보이는 글자를
        // 은닉으로 보고하게 되므로 렌더러와 같은 식을 쓴다.
        let mut cs = shape(1000, 0x0000_0000, NO_SHADE);
        cs.relative_sizes = [10; 7]; // 스펙대로면 1pt, 렌더러 기준으로는 10pt
        assert!((effective_pt(&cs) - 10.0).abs() < 1e-9);
        assert_eq!(classify_char(&cs, PAGE_WHITE, DEFAULT_THRESHOLD_PT), None);
        assert_eq!(classify_char(&cs, PAGE_WHITE, 2.0), None);
    }

    #[test]
    fn default_relative_sizes_can_never_cause_a_zero_size_misjudgment() {
        // relSz=0 은 OWPML 유효범위 10~250 밖이지만 외부 파일에는 실제로 들어온다 —
        // #4141 이전 rhwp 가 만든 HWP3 변환본이 전부 그렇다(CHAR_SHAPE 68,744개 전건).
        // 그 값을 0% 로 곱했다면 문서 전체가 0pt 로 보고됐을 것이다. relSz 를 아예
        // 보지 않으므로 그 사고가 구조적으로 불가능하다.
        //
        // (#4141 이후 `CharShape::default()` 자체는 [100; 7] 이다. 그래서 여기서 0 을
        //  명시 대입한다 — 이 테스트가 지키는 것은 기본값이 아니라 *입력 내성*이다.)
        let mut cs = shape(1000, 0x0000_0000, NO_SHADE);
        cs.relative_sizes = [0; 7];
        assert!((effective_pt(&cs) - 10.0).abs() < 1e-9);
        assert_eq!(classify_char(&cs, PAGE_WHITE, DEFAULT_THRESHOLD_PT), None);
    }

    #[test]
    fn excerpt_is_capped_and_reports_true_length() {
        let long = "무".repeat(500);
        let (excerpt, count) = excerpt_of(&long, DEFAULT_EXCERPT_LIMIT);
        assert_eq!(count, 500);
        assert_eq!(excerpt.chars().count(), DEFAULT_EXCERPT_LIMIT + 1);
        assert!(excerpt.ends_with('…'));
    }

    #[test]
    fn whitespace_only_run_is_dropped() {
        assert_eq!(excerpt_of("   \t ", DEFAULT_EXCERPT_LIMIT).1, 0);
        assert_eq!(excerpt_of("\u{0002}\u{0003}", DEFAULT_EXCERPT_LIMIT).1, 0);
    }

    #[test]
    fn control_chars_are_excluded_from_excerpt() {
        let (excerpt, count) = excerpt_of("가\u{0003}나", DEFAULT_EXCERPT_LIMIT);
        assert_eq!(excerpt, "가나");
        assert_eq!(count, 2);
    }

    #[test]
    fn hex_follows_bgr_layout() {
        // ColorRef 는 0x00BBGGRR — 빨강은 0x0000_00FF.
        assert_eq!(hex(0x0000_00FF), "#FF0000");
        assert_eq!(hex(0x00FF_0000), "#0000FF");
        assert_eq!(hex(0x00FF_FFFF), "#FFFFFF");
    }
}
