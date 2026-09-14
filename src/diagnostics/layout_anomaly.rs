//! 레이아웃 이상탐지 — 렌더 트리 하나를 읽어 "정상적인 문서로 보이는가"를 판정한다.
//!
//! # `render-diff` 와의 관계 — 세 번째 층
//!
//! `render_geom_diff`(CLI `render-diff`)는 **두** 렌더(왕복 전/후, 또는 두 파일)의
//! 요소별 bbox를 대응시켜 **변위**(얼마나 움직였나, `maxDisp`)를 잰다. 그 판정은
//! "라운드트립이 원본과 얼마나 같은가"이지 "이 렌더가 정상적인 문서로 보이는가"가
//! 아니다 — 두 렌더가 **똑같이** 망가져 있으면(예: 항상 표가 페이지 밖으로 넘치는
//! 문서) 변위는 0인데 결과물은 여전히 깨져 있다.
//!
//! 이 모듈은 렌더 **한 장**만 입력받아 그 자체의 기하가 말이 되는지 본다:
//! 요소가 본문 여백을 벗어났는가(overflow), 페이지 상자 밖(또는 y<0)에 놓였는가
//! (off-canvas), 겹치면 안 되는 흐름 요소끼리 겹쳤는가(overlap), 보이는 텍스트 런
//! bbox 가 서로 교차하는가(text-overlap), 글자가 자기 저장 줄의 baseline 에서
//! 벗어났는가(stored-line-escape, #7061), 콘텐츠 없는 페이지가 문서 중간에 있는가
//! (empty_page).
//!
//! 앞의 넷은 전부 **"상자를 넘었나 / 둘이 겹쳤나"** 다. 그래서 이탈이 쪽 밖으로
//! 나가지도 다른 상자와 겹치지도 않으면 넷 다 0 이었다 — `#7018` 결함을 되돌린
//! A/B 에서 165px 짜리 배치 이탈이 있으나 없으나 봉투가 한 글자도 다르지 않았다.
//! stored-line-escape 는 기준을 바깥 상자가 아니라 **한/글이 저장해 둔 그 줄
//! 자신**으로 잡아 그 빈틈을 맡는다. 이 축만 렌더 트리 밖(문서 IR)의 값을
//! 필요로 해서 [`scan_page_with_source`] 로 들어온다. overflow 와 off-canvas 는 기준 상자가 다르다 — 본문만 넘치고 쪽 안에
//! 남아 있으면 overflow 만, 쪽 상자(또는 음수 y)를 넘으면 off-canvas. `render-diff`가
//! "달라졌는가"를 묻는다면 이 모듈은 "이상해 보이는가"를 묻는다 — 같은 렌더 기하 축 위의
//! 서로 다른 질문이라 한쪽이 다른 쪽을 대신하지 않는다.
//!
//! # 설계 원칙 — 판정은 데이터, 차단은 소비자 몫
//!
//! 이 저장소의 다른 진단 명령(`render-diff`, `inspect hidden-text` 등)과 같은
//! 철학이다. 탐지 건수가 0이 아니어도 기본 종료 코드는 0이다 — anomaly 발견은
//! 도구의 정상 동작이지 실패가 아니다. 소비자가 실패로 취급하고 싶으면 명시적으로
//! `--strict` 를 준다. `empty_page` 는 특히 오탐 여지가 크다(의도된 표지·구분지
//! 빈 쪽과 회귀를 기하만으로 구분할 수 없다) — 그래서 `--strict` 로도 절대 실패를
//! 유발하지 않는 "가능성 신호"로만 분리해 낸다. `off-canvas` 는 페이지 상자·음수 y,
//! `text-overlap` 은 글자끼리의 bbox 교차라 일반 overlap(표·이미지)과 다른 확정 기하다.
//! 둘 다 `--strict` 에 포함한다. 자세한 배경은 `mydocs/tech/layout_anomaly_detection.md`.
//!
//! # 입력 경계
//!
//! 렌더러가 이미 만들어 내는 [`RenderNode`] 트리를 **읽기만** 한다 — 렌더러·레이아웃
//! 엔진 코드는 건드리지 않는다. `document_core::DocumentCore::build_page_render_tree`
//! 가 유일한 진입점이고, 이 모듈은 그 산출물의 소비자다.
//!
//! # 판정별 스캔 범위
//!
//! overflow·off-canvas·overlap(컨테이너)·empty_page 는 `Body` 서브트리를 순회하고
//! 본문 여백(`Body::bbox`)·페이지 상자를 기준으로 잰다. **text-overlap 만** 범위가
//! 넓다 — `MasterPage`(바탕쪽)·`Header`·`Footer`·`FootnoteArea` 의 글자까지 후보로
//! 모은다. 본문 글자가 바탕쪽 사이드바를 덮으면 사용자에게는 글자 두 개가 겹쳐
//! 보이는데, `Body` 안만 보면 그 짝이 애초에 후보가 아니기 때문이다
//! (편람 69쪽 실측 4건). 자세한 근거는 [`collect_text_outside_body`].

use serde_json::{json, Value};

use crate::document_core::DocumentCore;
use crate::model::shape::TextWrap;
use crate::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use crate::HwpError;

// ─────────────────────────────────────────────────────────────────────────
// 판정 옵션 · 데이터 모델
// ─────────────────────────────────────────────────────────────────────────

/// 스캔 임계값. 둘 다 렌더 트리와 같은 단위(px)다.
#[derive(Debug, Clone)]
pub struct AnomalyOptions {
    /// 요소 bbox가 본문 영역을 이 값(px) 넘게 벗어나야 overflow로 잡는다.
    /// 하위 픽셀 반올림 노이즈를 거르는 목적 — `render-diff` 의 기본 변위
    /// 임계(1.0px)와 같은 자릿수를 쓴다.
    pub overflow_tolerance_px: f64,
    /// 두 요소의 겹침 폭·높이가 **둘 다** 이 값(px)을 넘어야 overlap으로 잡는다.
    /// 모서리가 살짝 스치는 것(안티앨리어싱·반올림)은 정상 조판에서도 흔하다.
    pub overlap_tolerance_px: f64,
    /// overflow·overlap 검사 대상 노드 타입. `None` 이면 기본 검사 대상 전부.
    /// `empty_page` 는 페이지 단위 신호라 이 필터의 영향을 받지 않는다.
    pub type_filter: Option<Vec<&'static str>>,
    /// [#7061] 저장 줄 baseline·줄 높이를 "같다"고 볼 px 허용치.
    ///
    /// 조절 손잡이가 아니라 **부동소수 동등 비교의 여유**다. 저장값은 HWPUNIT 정수를
    /// 96 DPI 로 나눈 값이라 재현될 때 딱 떨어지지 않는다. 실측 분포도 이 값이
    /// 하중을 받지 않음을 보인다 — 줄 높이를 재현한 561,314 줄에서 편차 0.5px 초과가
    /// 625 건, 1.0px 초과가 615 건이라 그 사이에 10 건뿐이고, 10px 초과는 0 건이다.
    pub stored_line_tolerance_px: f64,
}

/// 기본 overflow 허용치(px). `render_geom_diff::DEFAULT_MAX_DISP` 와 같은 자릿수.
pub const DEFAULT_OVERFLOW_TOLERANCE_PX: f64 = 1.0;
/// 기본 overlap 허용치(px, 폭·높이 각각).
pub const DEFAULT_OVERLAP_TOLERANCE_PX: f64 = 2.0;
/// 기본 저장 줄 동등 비교 여유(px) — [`AnomalyOptions::stored_line_tolerance_px`].
pub const DEFAULT_STORED_LINE_TOLERANCE_PX: f64 = 0.5;

impl Default for AnomalyOptions {
    fn default() -> Self {
        Self {
            overflow_tolerance_px: DEFAULT_OVERFLOW_TOLERANCE_PX,
            overlap_tolerance_px: DEFAULT_OVERLAP_TOLERANCE_PX,
            type_filter: None,
            stored_line_tolerance_px: DEFAULT_STORED_LINE_TOLERANCE_PX,
        }
    }
}

/// 요소 하나가 본문 영역(margin box)을 벗어난 사건.
#[derive(Debug, Clone)]
pub struct OverflowAnomaly {
    /// 구조 경로 (예: `Page/Body/Column0/Table2`).
    pub path: String,
    pub node_type: &'static str,
    pub bbox: BoundingBox,
    /// 벗어난 기준이 된 본문 영역(Body 노드의 선언 bbox — 레이아웃이 여백으로
    /// 확정한 콘텐츠 영역이며, overflow 콘텐츠를 반영해 사후 확장되는
    /// `Body::clip_rect` 와는 다르다).
    pub boundary: BoundingBox,
    pub over_left: f64,
    pub over_top: f64,
    pub over_right: f64,
    pub over_bottom: f64,
}

impl OverflowAnomaly {
    /// 네 방향 초과량의 최대값(px) — 보고·정렬용 단일 지표.
    pub fn max_over(&self) -> f64 {
        self.over_left
            .max(self.over_top)
            .max(self.over_right)
            .max(self.over_bottom)
    }
}

/// 요소 하나가 페이지 상자 밖(또는 y<0)에 놓인 사건.
///
/// overflow 는 본문 여백(Body bbox)을 넘은 것이고, off-canvas 는 페이지
/// 상자(Page bbox)를 넘었거나 `y < 0` 인 것이다. 본문만 넘치고 쪽 안에
/// 남아 있으면 overflow 만 난다. `y < 0` 은 표 조각을 표 전체 원점으로
/// 그려 앞 행이 쪽 위로 소실되는 축(#4889)을 잡기 위한 명시 조건이다.
/// `boundary` 는 페이지 상자이고, 허용치는 overflow 와 같은
/// [`AnomalyOptions::overflow_tolerance_px`] 를 쓴다.
pub type OffCanvasAnomaly = OverflowAnomaly;

/// 겹치면 안 되는 두 요소의 bbox가 겹친 사건.
#[derive(Debug, Clone)]
pub struct OverlapAnomaly {
    pub path_a: String,
    pub type_a: &'static str,
    pub bbox_a: BoundingBox,
    pub path_b: String,
    pub type_b: &'static str,
    pub bbox_b: BoundingBox,
    pub overlap_w: f64,
    pub overlap_h: f64,
}

impl OverlapAnomaly {
    pub fn overlap_area(&self) -> f64 {
        self.overlap_w * self.overlap_h
    }
}

/// 보이는 텍스트 런(글리프 묶음) bbox 가 서로 교차한 사건.
///
/// 일반 [`OverlapAnomaly`] 는 표·이미지·문단 줄처럼 **흐름 요소**의 겹침이다.
/// 이쪽은 텍스트끼리만 본다 — 표와 그림이 겹쳐도 여기엔 안 잡힌다. 렌더 트리에
/// 글자 단위 글리프 bbox 는 없으므로, 레이아웃이 이미 나눠 둔 `TextRun` 노드
/// bbox 를 글리프 묶음으로 쓴다. 한컴 글자겹침(`char_overlap`) 런은 의도된
/// 겹침이라 후보에서 뺀다.
pub type TextOverlapAnomaly = OverlapAnomaly;

/// 글자가 자기 저장 줄(`LINE_SEG`)을 벗어나 **같은 문단의 다른 저장 줄** baseline 에
/// 앉은 사건.
///
/// 다른 다섯 축은 전부 **"상자를 넘었나 / 둘이 겹쳤나"** 다. 이탈이 쪽 밖으로 나가지도,
/// 다른 상자와 겹치지도 않으면 다섯 축이 전부 0 이다(#7061). `#7018` 결함을 되돌린 A/B
/// 에서 165px 짜리 배치 이탈이 있으나 없으나 봉투가 한 글자도 다르지 않았다.
///
/// # 왜 "저장값과 다르다"가 아닌가 — 정본이 그 판정을 반증한다
///
/// 처음 세운 판정은 "렌더 baseline 이 저장 `baseline_distance` 와 다르면 이탈"이었다.
/// **한/글 자신이 저장값으로 그리지 않는다.**
///
/// | 문서 | rhwp | 한/글 2020 정본 | 저장 `baseline_distance` |
/// | --- | ---: | ---: | ---: |
/// | `exam_eng.hwp` 1쪽 `①` | 494.76 | **494.08** (차 0.68) | 490.16 (차 3.92) |
/// | `pr-1674.hwp` 23쪽 `1.` | 172.67 | **173.60** (차 0.93) | 167.47 (차 6.13) |
/// | `tac-case-003.hwp` 1쪽 `표 다음` | 214.72 | **214.49** (차 0.23) | 238.91 (차 24.42) |
/// | `issue6181/…` 6쪽 `회피` | 158.43 | **158.40** (차 0.03) | 161.58 (차 3.18) |
///
/// 네 문서 모두 rhwp 가 정본과 1px 안에서 맞고 저장값 쪽이 3~24px 틀렸다. samples 961 건
/// 전수로도 같은 말이 나온다 — 렌더 baseline 이 저장값과 다른 줄이 **38%**(90.1만 중
/// 34.0만)이고, 비로 봐도 단일 배율이 아니라 줄마다 갈린다(1.00 이 62.3%). 저장값과의
/// 불일치는 신호가 아니라 **배경**이다.
///
/// # 그래서 무엇을 신고하나
///
/// 배경을 걷어내고 남는 것은 **줄 귀속이 바뀐 경우**다. 네 조건을 모두 만족해야 한다.
///
/// 1. 그 글자가 속한 저장 줄을 서수 추정 없이 짚을 수 있다
///    (줄 노드 경로는 `line_index`+`vertical_pos`, 줄 노드 없는 경로는 런이 저장 줄
///    하나 **안에 온전히** 들어간다).
/// 2. 그 문단의 저장 `text_start` 색인 공간이 IR 문단 `text` 와 맞물린다
///    ([`stored_text_space_matches`]).
/// 3. 렌더 baseline 이 **자기 저장 줄 상자 밖**이다 — 이슈 제목 그대로다.
/// 4. 그 baseline 이 같은 문단의 **다른** 저장 줄 baseline 과 일치한다.
///
/// 조건 3·4 가 빠지면 위 표의 네 문서가 전부 위양성으로 올라온다(각각 145·14·1·27 건).
/// 다 걸면 devel + samples 961 건에서 **0 건**이고, `#7018` 수정 두 커밋을 되돌린
/// 상태에서 **정확히 1 건**이 나온다 — 표가 자기 저장 줄을 가진 문단에서 앞 텍스트가
/// 표 줄의 baseline(13,423 HU = 178.97px)을 받아 자기 줄(1,020 HU = 13.60px)에서
/// 165.37px 아래로 떨어진 그 결함이다.
///
/// # 덮지 않는 것
///
/// `#6928` 이 보인 "표가 그림 위에 그려진다"는 글자 대 **그림**의 겹침이라 이 축이 아니다.
/// 그쪽은 여전히 다섯 축도 이 축도 보지 못한다.
#[derive(Debug, Clone)]
pub struct StoredLineEscapeAnomaly {
    /// 구조 경로 (예: `Page/Body/Column0/TextLine3`).
    pub path: String,
    pub section: usize,
    pub para: usize,
    /// 문단 안 저장 줄 서수 — 이 글자가 **속한** 줄.
    pub line_seg_index: usize,
    /// 이 글자가 실제로 앉은 줄의 서수 — 같은 문단의 **다른** 저장 줄.
    pub took_line_seg_index: usize,
    /// 줄 노드가 있었나 — `false` 면 줄 노드 없는 경로다.
    pub from_line_node: bool,
    /// 렌더가 실제로 쓴 baseline (줄 상자 위에서 px).
    pub rendered_baseline: f64,
    /// 저장 `LINE_SEG.baseline_distance` (px).
    pub stored_baseline: f64,
    /// 저장 `LINE_SEG.line_height` (px).
    pub stored_line_height: f64,
    /// 그 줄의 첫 글자 일부 (보고용).
    pub text: String,
}

impl StoredLineEscapeAnomaly {
    /// 저장 baseline 에서 벗어난 거리(px).
    pub fn deviation(&self) -> f64 {
        (self.rendered_baseline - self.stored_baseline).abs()
    }
}

/// 문서 중간에서 콘텐츠 없는 페이지를 만난 사건. 의도된 빈 페이지(표지 뒷면,
/// 장 구분지 등)와 기하만으로 구분할 수 없으므로 이 자체가 곧 "가능성 신호"다
/// (별도 severity 플래그를 두지 않는다 — 존재 자체가 이미 낮은 신뢰도를 뜻한다).
#[derive(Debug, Clone, Copy)]
pub struct EmptyPageAnomaly {
    pub page: u32,
}

/// 한 페이지의 이상탐지 결과.
#[derive(Debug, Clone, Default)]
pub struct PageAnomalies {
    pub page: u32,
    pub overflow: Vec<OverflowAnomaly>,
    pub off_canvas: Vec<OffCanvasAnomaly>,
    pub overlap: Vec<OverlapAnomaly>,
    pub text_overlap: Vec<TextOverlapAnomaly>,
    pub stored_line_escape: Vec<StoredLineEscapeAnomaly>,
    pub empty_page: Option<EmptyPageAnomaly>,
}

impl PageAnomalies {
    pub fn is_empty(&self) -> bool {
        self.overflow.is_empty()
            && self.off_canvas.is_empty()
            && self.overlap.is_empty()
            && self.text_overlap.is_empty()
            && self.stored_line_escape.is_empty()
            && self.empty_page.is_none()
    }

    /// `--strict` 가 실패로 셀 만한 확정 신호가 있는가.
    /// overflow·off-canvas·overlap·text-overlap·stored-line-escape 는 확정 신호.
    /// `empty_page` 는 가능성 신호일 뿐이라 제외한다. off-canvas 는 페이지 상자 밖·음수 y,
    /// text-overlap 은 글자 bbox 교차, stored-line-escape 는 저장 줄을 재현한 줄에서의
    /// baseline 불일치라 빈 쪽처럼 기하만으로 애매하지 않다.
    pub fn has_signal(&self) -> bool {
        !self.overflow.is_empty()
            || !self.off_canvas.is_empty()
            || !self.overlap.is_empty()
            || !self.text_overlap.is_empty()
            || !self.stored_line_escape.is_empty()
    }
}

/// 문서 전체의 이상탐지 결과. `pages` 는 anomaly가 있는 페이지만 담는다
/// (전 페이지를 매번 싣는 `render_geom_diff::DocGeomDiff` 와 달리, 여기선 페이지당
/// 형태가 가변적이고 보통 대다수 페이지가 깨끗하므로 압축한다).
#[derive(Debug, Clone)]
pub struct DocAnomalies {
    pub page_count: u32,
    pub pages: Vec<PageAnomalies>,
}

impl DocAnomalies {
    pub fn overflow_count(&self) -> usize {
        self.pages.iter().map(|p| p.overflow.len()).sum()
    }

    pub fn off_canvas_count(&self) -> usize {
        self.pages.iter().map(|p| p.off_canvas.len()).sum()
    }

    pub fn overlap_count(&self) -> usize {
        self.pages.iter().map(|p| p.overlap.len()).sum()
    }

    pub fn text_overlap_count(&self) -> usize {
        self.pages.iter().map(|p| p.text_overlap.len()).sum()
    }

    pub fn stored_line_escape_count(&self) -> usize {
        self.pages.iter().map(|p| p.stored_line_escape.len()).sum()
    }

    pub fn empty_page_count(&self) -> usize {
        self.pages.iter().filter(|p| p.empty_page.is_some()).count()
    }

    /// `--strict` 가 실패로 셀 확정 신호가 문서 어디든 있는가.
    pub fn has_signal(&self) -> bool {
        self.pages.iter().any(|p| p.has_signal())
    }
}

// ─────────────────────────────────────────────────────────────────────────
// 코어 스캔
// ─────────────────────────────────────────────────────────────────────────

/// 노드 타입을 안정 문자열로 매핑. `render_geom_diff::node_type_str` /
/// `RenderNode::write_json` 과 같은 매핑을 각 소비자가 독립적으로 들고 있는
/// 이 저장소의 기존 관례를 따른다(모듈 간 결합을 만들지 않는다).
fn node_type_label(t: &RenderNodeType) -> &'static str {
    match t {
        RenderNodeType::Page(_) => "Page",
        RenderNodeType::PageBackground(_) => "PageBg",
        RenderNodeType::MasterPage => "MasterPage",
        RenderNodeType::Header => "Header",
        RenderNodeType::Footer => "Footer",
        RenderNodeType::Body { .. } => "Body",
        RenderNodeType::Column(_) => "Column",
        RenderNodeType::FootnoteArea => "FootnoteArea",
        RenderNodeType::TextLine(_) => "TextLine",
        RenderNodeType::TextRun(_) => "TextRun",
        RenderNodeType::Table(_) => "Table",
        RenderNodeType::TableCell(_) => "Cell",
        RenderNodeType::Image(_) => "Image",
        RenderNodeType::TextBox => "TextBox",
        RenderNodeType::Equation(_) => "Equation",
        RenderNodeType::Line(_) => "Line",
        RenderNodeType::Rectangle(_) => "Rect",
        RenderNodeType::Ellipse(_) => "Ellipse",
        RenderNodeType::Path(_) => "Path",
        RenderNodeType::Group(_) => "Group",
        RenderNodeType::FormObject(_) => "Form",
        RenderNodeType::FootnoteMarker(_) => "FnMarker",
        RenderNodeType::Placeholder(_) => "Placeholder",
        RenderNodeType::RawSvg(_) => "RawSvg",
    }
}

/// overflow·overlap 판정 대상이 되는 "요소" 타입인가.
///
/// 표·이미지·글상자·수식·묶음·도형류·문단 줄(TextLine)만 검사한다. 그 아래
/// 자손(표 셀, TextRun)은 검사 대상 노드에 들어간 순간부터 더 내려가지 않는다
/// (`walk`의 `suppress`) — 표 하나가 넘치면 그 표에 딸린 모든 줄을 중복으로
/// 보고하는 대신, 표 자체를 한 번만 보고한다.
fn is_checkable(t: &RenderNodeType) -> bool {
    matches!(
        t,
        RenderNodeType::Table(_)
            | RenderNodeType::Image(_)
            | RenderNodeType::TextBox
            | RenderNodeType::Equation(_)
            | RenderNodeType::Group(_)
            | RenderNodeType::FormObject(_)
            | RenderNodeType::Placeholder(_)
            | RenderNodeType::RawSvg(_)
            | RenderNodeType::Line(_)
            | RenderNodeType::Rectangle(_)
            | RenderNodeType::Ellipse(_)
            | RenderNodeType::Path(_)
            | RenderNodeType::TextLine(_)
    )
}

/// overflow·overlap 기본 검사 대상의 안정 라벨 (`--types` 가 받는 이름).
const CHECKABLE_TYPE_LABELS: &[&str] = &[
    "Table",
    "Image",
    "TextBox",
    "Equation",
    "Group",
    "Form",
    "Placeholder",
    "RawSvg",
    "Line",
    "Rect",
    "Ellipse",
    "Path",
    "TextLine",
];

fn type_allowed(label: &str, opts: &AnomalyOptions) -> bool {
    match &opts.type_filter {
        None => true,
        Some(allowed) => allowed.contains(&label),
    }
}

/// 이 노드가 "겹치면 안 되는" overlap 후보인가. `node` 는 TextLine의 자식(TextRun)
/// 검사에 쓴다.
///
/// 표·문단 줄(TextLine)은 흐름 콘텐츠라 후보다 — 정상 조판은 절대 두 문단 줄이나
/// 표를 같은 자리에 겹쳐 놓지 않는다. 단 TextLine 은 **보이는 글자가 있을 때만**
/// 후보로 본다 — 표·묶음 개체를 문단에 앵커링하는 "운반용" 줄은 빈 TextRun(`text:
/// ""`) 하나만 자식으로 두고 그 개체와 정확히 같은 좌상단에 찍힌다(실측: 380쪽
/// 분량 표본에서 표-줄 겹침 43건이 전부 이 패턴). 실제 텍스트가 없는 줄은 애초에
/// 화면에 아무것도 그리지 않으므로 겹침이 아니다. 그 밖의(이미지·도형류) 개체는
/// 배치된 text_wrap 이 "겹침을 배제하는" 종류(Square/Tight/TopAndBottom — 텍스트를
/// 밀어내는 wrap)일 때만 후보로 본다. BehindText/InFrontOfText 는 애초에 다른
/// 콘텐츠와 겹치라고 있는 wrap 이라 후보에서 뺀다. 바탕쪽(master page) 유래
/// 개체도 항상 배경에 깔릴 뿐이라 제외한다.
/// 흐름 겹침 판정에 쓰는 상자.
///
/// `TextLine` 은 줄 상자(줄높이)를 bbox 로 갖는다. 줄높이는 줄 간격을 포함하므로,
/// 줄 간격이 줄높이보다 좁은 문단에서는 **연속된 두 줄의 상자가 자동으로 겹친다** —
/// 글자는 안 겹치는데도. 수식이 섞인 줄에서 특히 잦다(위첨자·분수가 줄 상자를 키운다).
///
/// 실측(2026-08-28): `3-11월_실전_통합_2022.hwp` 의 겹침 8건이 전부 연속 줄 쌍이고,
/// 줄 상자 27.6px 에 줄 간격 20.4px 이라 7.2px 이 겹친다. 렌더에는 겹친 글자가 없다.
///
/// 그래서 `TextLine` 은 자식 `TextRun` 들의 **글자 상자 합집합**으로 잰다. 글자가 실제로
/// 차지하는 범위이고, `text_overlap` 이 런 단위로 쓰는 것과 같은 기준이다 — 같은 질문에
/// 두 개의 답을 두지 않는다. 줄이 정말로 포개지면 그 합집합도 겹치므로 검출은 잃지 않는다.
///
/// `TextLine` 이 아닌 노드(표·그림·도형)는 자기 bbox 가 곧 차지하는 범위다.
fn flow_extent_bbox(node: &RenderNode) -> BoundingBox {
    if !matches!(node.node_type, RenderNodeType::TextLine(_)) {
        return node.bbox;
    }
    let mut union: Option<BoundingBox> = None;
    for child in &node.children {
        if !matches!(&child.node_type, RenderNodeType::TextRun(tr) if has_visible_text(&tr.text)) {
            continue;
        }
        let band = glyph_band_bbox(child);
        union = Some(match union {
            None => band,
            Some(u) => {
                let x = u.x.min(band.x);
                let y = u.y.min(band.y);
                let right = (u.x + u.width).max(band.x + band.width);
                let bottom = (u.y + u.height).max(band.y + band.height);
                BoundingBox::new(x, y, right - x, bottom - y)
            }
        });
    }
    // 보이는 런이 없으면 줄 상자를 그대로 쓴다 — 판정 대상에서 조용히 빠지지 않게 한다.
    union.unwrap_or(node.bbox)
}

fn is_overlap_candidate(node: &RenderNode) -> bool {
    match &node.node_type {
        RenderNodeType::TextLine(_) => node.children.iter().any(
            |c| matches!(&c.node_type, RenderNodeType::TextRun(tr) if has_visible_text(&tr.text)),
        ),
        RenderNodeType::Table(_) => true,
        _ => {
            let Some(l) = node.layer else { return false };
            if l.master_page {
                return false;
            }
            matches!(
                l.text_wrap,
                Some(TextWrap::Square) | Some(TextWrap::Tight) | Some(TextWrap::TopAndBottom)
            )
        }
    }
}

fn has_visible_text(s: &str) -> bool {
    s.chars().any(|c| !c.is_whitespace())
}

/// Empty paragraph carriers reserve cursor/line height without painting content.
/// Keep whitespace, display substitutions, shading, borders and object children checked.
fn is_empty_text_carrier(node: &RenderNode) -> bool {
    matches!(node.node_type, RenderNodeType::TextLine(_))
        && !node.children.is_empty()
        && node.children.iter().all(|child| {
            matches!(&child.node_type, RenderNodeType::TextRun(run)
                if run.text.is_empty()
                    && run.display_or_text().is_empty()
                    && run.border_fill_id == 0
                    && crate::model::color::char_shade(run.style.shade_color).is_none()
                    && run.style.tab_leaders.is_empty())
                && child.children.is_empty()
        })
}

/// `TextRun` bbox 를 **글자 상자**로 좁힌다.
///
/// 노드 bbox 는 줄 상자(전진폭 × 줄높이)이지 글리프 잉크가 아니다. 줄높이는 줄 간격을
/// 포함하므로, 행 간격이 줄높이보다 좁은 표에서는 위아래 줄의 **상자**가 자동으로 겹친다 —
/// 글자는 안 겹치는데도.
///
/// 실측(2026-08-28): `hwpx/hancom-hwp/hwpx-02.hwp` 2쪽은 이 검출기가 71건을 보고하는데
/// 렌더에는 겹친 글자가 하나도 없다. 예 — 줄 상자 높이 16.0px, 행 간격 11.73px,
/// 보고된 세로 겹침 4.27px. 그 4.27 은 줄 간격이지 글자가 아니다.
///
/// 보이는 세로 범위는 글리프의 em 상자이고 그 높이는 `font_size` 다. 가로는 렌더러의
/// replay 위치에서 앞뒤 공백의 전진폭만 뺀다. 비공백 끝과 유효한 전진폭이 없는 런은
/// 원상자를 유지한다.
///
/// [#7023] 그 em 상자를 **baseline 에 맞춰** 놓는다. 종전에는 줄 상자 **중앙**에
/// 놓았고, 그 선택의 이유로 "렌더 트리에 baseline 필드가 없다" 를 적어 두었다.
/// 그런데 `TextRunNode::baseline` 은 존재하고(`bbox.y` 로부터의 거리), SVG 는 바로
/// 그 값으로 글자를 찍는다.
///
/// 중앙 가정은 방향이 한쪽이 아니다 — 줄 상자가 부풀수록 띠가 잉크에서 `(h − em)/2`
/// 만큼 멀어져 **위음성을 만든다**. 2769535 2쪽 실측: 상자 `h=179.0` · `em=16` 인
/// 줄에서 띠는 `y 687.3..703.3` 인데 실제 잉크는 `y 771.2..784.8` 이라 81.5px 어긋난다.
///
/// em 상자의 세로 교차만으로 실제 글자 겹침을 단정하지 않는다. 앞뒤 공백과 장평·자간을
/// 반영한 가로 범위도 겹쳐야 후보가 된다. baseline 위치와 공백 전진폭의 경계는
/// `layout_anomaly_glyph_band` 계약 테스트에서 각각 확인한다.
///
/// baseline 아래로 두는 몫은 `1 − 0.85` 다 — 글꼴 기준으로 baseline 을 복원하는
/// `renderer::corrected_line_baseline_for_source` 가 쓰는 것과 같은 비율이고,
/// 같은 문서들의 실측(정상 줄 `baseline = y + 0.85·h`, 오차 0.03px)과 맞는다.
///
/// 이것은 여전히 근사다. 정확히 하려면 폰트 메트릭의 ascent/descent 로 잉크 상자를
/// 계산해야 한다. 다만 "줄 간격을 글자 겹침으로 세지 않는다"(원래 목적,
/// `hwpx/hancom-hwp/hwpx-02.hwp` 2쪽 71건 위양성)는 그대로 유지된다.
fn glyph_band_bbox(node: &RenderNode) -> BoundingBox {
    let RenderNodeType::TextRun(run) = &node.node_type else {
        return node.bbox;
    };

    let em = run.style.font_size;
    // NaN·비유한 font_size 는 종전 `!(em > 0.0)` 처럼 원상자 유지로 처리한다.
    if !em.is_finite() || em <= 0.0 {
        return node.bbox;
    }
    let (x, width) = glyph_band_horizontal(run, node.bbox.x, node.bbox.width, em);
    // 세로 좁히기는 줄 상자가 em 보다 클 때만 뜻이 있다 — 가로 다듬기는 그와
    // 무관하게 적용한다(빈칸 전진폭은 줄 높이와 상관없이 잉크 밖이다).
    if em >= node.bbox.height {
        return BoundingBox::new(x, node.bbox.y, width, node.bbox.height);
    }
    // baseline 이 없거나(0.0 기본값) 상자 밖이면 **종전 중앙 기준**으로 떨어진다.
    // 근거가 없을 때 좁히기를 아예 끄면 이 함수가 막으려던 줄 간격 오탐이 되살아난다
    // (`hwpx-02.hwp` 2쪽 71건). 중앙은 틀린 자리지만 `h ≈ em` 인 흔한 줄에서는
    // baseline 기준과 거의 같고, 어긋남이 커지는 구간은 `baseline` 이 있는 실제
    // 렌더 트리가 덮는다.
    let baseline = run.baseline;
    let top = if baseline.is_finite() && baseline > 0.0 && baseline <= node.bbox.height {
        node.bbox.y + baseline - em * GLYPH_BAND_ASCENT_RATIO
    } else {
        node.bbox.y + (node.bbox.height - em) / 2.0
    };
    BoundingBox::new(x, top, width, em)
}

/// 렌더러의 replay 위치로 앞뒤 공백의 전진폭만 제거한다.
/// 장평·자간·탭·유효한 layout_positions를 같은 경로로 해석한다.
/// 표시 문자열이 달라진 필드도 backend와 같은 검증/fallback을 거친다.
fn glyph_band_horizontal(
    run: &crate::renderer::render_tree::TextRunNode,
    x: f64,
    width: f64,
    _em: f64,
) -> (f64, f64) {
    let text = run.display_or_text();
    let is_space = |c: char| matches!(c, ' ' | '\u{3000}');
    let start = text.chars().take_while(|&c| is_space(c)).count();
    let end = text.chars().count() - text.chars().rev().take_while(|&c| is_space(c)).count();
    if start >= end || (start == 0 && end == text.chars().count()) {
        return (x, width);
    }
    let positions = run.replay_positions_for(text);
    let Some((&left, &right)) = positions.get(start).zip(positions.get(end)) else {
        return (x, width);
    };
    // bbox와 전진폭의 계약이 맞지 않을 때 근거 없는 축소를 하지 않는다.
    if !left.is_finite() || !right.is_finite() || left < 0.0 || right <= left || left >= width {
        return (x, width);
    }
    // 공백 없는 쪽 끝은 원상자를 유지한다. 비공백 advance 차이까지 줄이지 않는다.
    let left = if start == 0 { 0.0 } else { left };
    let right = if end == text.chars().count() {
        width
    } else {
        right.min(width)
    };
    if right <= left {
        return (x, width);
    }
    (x + left, right - left)
}

/// em 상자에서 baseline 위쪽이 차지하는 몫.
///
/// `renderer::corrected_line_baseline_for_source` 가 글꼴 기준 baseline 을
/// `max_fs * 0.85` 로 복원하는 것과 같은 비율이다.
const GLYPH_BAND_ASCENT_RATIO: f64 = 0.85;

/// text-overlap 후보 — 보이는 글자가 있는 `TextRun` 이고, 한컴 글자겹침
/// 컨트롤이 아니며, 면적이 있는 bbox 를 가진다. 표·이미지·도형은 여기
/// 들어오지 않는다(그건 일반 overlap).
fn is_text_overlap_candidate(node: &RenderNode) -> bool {
    match &node.node_type {
        RenderNodeType::TextRun(tr) => {
            tr.char_overlap.is_none()
                && has_visible_text(tr.display_or_text())
                && node.bbox.width > 0.0
                && node.bbox.height > 0.0
        }
        _ => false,
    }
}

/// 페이지 트리에서 첫 `Body` 노드를 찾는다(전위 순회).
fn find_body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(find_body)
}

fn intersection(a: &BoundingBox, b: &BoundingBox) -> Option<(f64, f64)> {
    let x0 = a.x.max(b.x);
    let y0 = a.y.max(b.y);
    let x1 = (a.x + a.width).min(b.x + b.width);
    let y1 = (a.y + a.height).min(b.y + b.height);
    if x1 > x0 && y1 > y0 {
        Some((x1 - x0, y1 - y0))
    } else {
        None
    }
}

fn check_overflow(
    bbox: &BoundingBox,
    path: &str,
    node_type: &'static str,
    boundary: &BoundingBox,
    opts: &AnomalyOptions,
    out: &mut Vec<OverflowAnomaly>,
) {
    let over_left = (boundary.x - bbox.x).max(0.0);
    let over_top = (boundary.y - bbox.y).max(0.0);
    let over_right = (bbox.x + bbox.width - (boundary.x + boundary.width)).max(0.0);
    let over_bottom = (bbox.y + bbox.height - (boundary.y + boundary.height)).max(0.0);
    let max_over = over_left.max(over_top).max(over_right).max(over_bottom);
    if max_over > opts.overflow_tolerance_px {
        out.push(OverflowAnomaly {
            path: path.to_string(),
            node_type,
            bbox: *bbox,
            boundary: *boundary,
            over_left,
            over_top,
            over_right,
            over_bottom,
        });
    }
}

/// [#5586] 노드와 그 가시 자손 전체의 **세로** 범위를 합친 bbox.
///
/// off-canvas 판정용 — 컨테이너(표 등)가 선언 크기로 배치되고 내용이 그 아래로
/// 흘러나온 경우, 자기 bbox 는 쪽 안이라도 심층 세로 범위는 쪽 밖이다(00365).
/// 가로는 자기 bbox 를 유지한다 — TextRun 의 말미 공백 advance 등 측정 폭이
/// 쪽 우측을 스치는 무해한 초과가 흔해(표본 100문서에서 62건 위양성 실측),
/// 가로까지 합치면 검출기의 신호가 잠긴다. 이 결함군(#5586·#4889)의 본질은
/// 세로 유출이다.
fn deep_vertical_union_bbox(node: &RenderNode) -> BoundingBox {
    fn vertical_extent(node: &RenderNode, min_y: &mut f64, max_bottom: &mut f64) {
        for child in &node.children {
            if !child.visible || child.editor_only {
                continue;
            }
            if child.bbox.height > 0.0 && child.bbox.width > 0.0 {
                *min_y = min_y.min(child.bbox.y);
                *max_bottom = max_bottom.max(child.bbox.y + child.bbox.height);
            }
            vertical_extent(child, min_y, max_bottom);
        }
    }
    let mut min_y = node.bbox.y;
    let mut max_bottom = node.bbox.y + node.bbox.height;
    vertical_extent(node, &mut min_y, &mut max_bottom);
    BoundingBox::new(node.bbox.x, min_y, node.bbox.width, max_bottom - min_y)
}

/// 페이지 상자 밖이거나 y<0 이면 off-canvas. overflow 와 같은 허용치를 쓰되
/// 기준 상자는 Page bbox 이다. `y < 0` 은 page.y 가 0 이 아니어도 쪽 위로
/// 소실된 노드를 놓치지 않기 위한 명시 조건이다.
fn check_off_canvas(
    bbox: &BoundingBox,
    path: &str,
    node_type: &'static str,
    page: &BoundingBox,
    opts: &AnomalyOptions,
    out: &mut Vec<OffCanvasAnomaly>,
) {
    let over_left = (page.x - bbox.x).max(0.0);
    let over_top = (page.y - bbox.y).max(0.0).max((-bbox.y).max(0.0));
    let over_right = (bbox.x + bbox.width - (page.x + page.width)).max(0.0);
    let over_bottom = (bbox.y + bbox.height - (page.y + page.height)).max(0.0);
    let max_over = over_left.max(over_top).max(over_right).max(over_bottom);
    if max_over > opts.overflow_tolerance_px {
        out.push(OffCanvasAnomaly {
            path: path.to_string(),
            node_type,
            bbox: *bbox,
            boundary: *page,
            over_left,
            over_top,
            over_right,
            over_bottom,
        });
    }
}

/// 페이지 루트의 쪽 상자. Page 노드 bbox 가 비어 있으면 PageNode 치수로
/// (0,0,w,h) 를 쓴다.
fn page_box(root: &RenderNode) -> BoundingBox {
    match &root.node_type {
        RenderNodeType::Page(p) if root.bbox.width <= 0.0 || root.bbox.height <= 0.0 => {
            BoundingBox::new(0.0, 0.0, p.width, p.height)
        }
        _ => root.bbox,
    }
}

/// overlap 후보 — 겹침 판정을 같은 단(column) 안에서만 짝짓기 위해 열 인덱스를
/// 함께 들고 다닌다. 서로 다른 단은 애초에 x축이 나뉘어 있어 정상 조판에서도
/// 나란히 배치되므로 후보 짝짓기에서 제외한다.
struct FlowCandidate {
    path: String,
    node_type: &'static str,
    bbox: BoundingBox,
    column: Option<u16>,
}

#[allow(clippy::too_many_arguments)]
fn walk(
    node: &RenderNode,
    path: String,
    column: Option<u16>,
    suppress: bool,
    off_canvas_suppress: bool,
    boundary: &BoundingBox,
    page: &BoundingBox,
    opts: &AnomalyOptions,
    overflow_out: &mut Vec<OverflowAnomaly>,
    off_canvas_out: &mut Vec<OffCanvasAnomaly>,
    flow_out: &mut Vec<FlowCandidate>,
    text_out: &mut Vec<FlowCandidate>,
    has_content: &mut bool,
) {
    if !node.visible || node.editor_only {
        return;
    }

    match &node.node_type {
        RenderNodeType::TextRun(tr) if has_visible_text(&tr.text) => *has_content = true,
        RenderNodeType::Image(_)
        | RenderNodeType::Table(_)
        | RenderNodeType::Equation(_)
        | RenderNodeType::TextBox
        | RenderNodeType::Line(_)
        | RenderNodeType::Rectangle(_)
        | RenderNodeType::Ellipse(_)
        | RenderNodeType::Path(_)
        | RenderNodeType::Group(_)
        | RenderNodeType::FormObject(_)
        | RenderNodeType::Placeholder(_)
        | RenderNodeType::RawSvg(_) => *has_content = true,
        _ => {}
    }

    let column = if let RenderNodeType::Column(c) = &node.node_type {
        Some(*c)
    } else {
        column
    };

    // 표·글상자 안으로도 내려가 런 bbox 를 모은다. 일반 overlap 의 suppress 와
    // 무관 — 표 안의 글자가 서로 겹치는 것은 표-표 겹침이 아니다.
    if is_text_overlap_candidate(node) {
        text_out.push(FlowCandidate {
            path: path.clone(),
            node_type: "TextRun",
            bbox: glyph_band_bbox(node),
            column,
        });
    }

    let mut next_suppress = suppress;
    let mut next_off_canvas_suppress = off_canvas_suppress;
    if is_checkable(&node.node_type) {
        let label = node_type_label(&node.node_type);
        if !off_canvas_suppress {
            // [#5586] 자기 bbox 가 아니라 **자손까지 합친 심층 세로 범위**로 판정한다.
            // 선언 높이를 신뢰해 배치된 1×1 래퍼 표는 자기 bbox(선언값)는 쪽 안인데
            // 중첩 내용이 쪽 밖(00365: 최하단 1181 > 쪽 1122.5)으로 넘친다 — 자손
            // 검사를 접는(suppress) 설계라 자기 bbox 만 보면 이 결함군 전체가
            // 침묵한다. 보고는 종전처럼 컨테이너당 1회다.
            let deep = deep_vertical_union_bbox(node);
            check_off_canvas(&deep, &path, label, page, opts, off_canvas_out);
            next_off_canvas_suppress = true;
        }
        if !suppress && type_allowed(label, opts) {
            if !is_empty_text_carrier(node) {
                check_overflow(&node.bbox, &path, label, boundary, opts, overflow_out);
            }
            if is_overlap_candidate(node) {
                flow_out.push(FlowCandidate {
                    path: path.clone(),
                    node_type: label,
                    bbox: flow_extent_bbox(node),
                    column,
                });
            }
            // 필터에 걸린 컨테이너만 자손을 접는다. `--types TextLine` 은 표 안의
            // 줄까지 내려가야 하므로, 제외된 Table 은 suppress 하지 않는다.
            next_suppress = true;
        }
    }

    for (i, child) in node.children.iter().enumerate() {
        let child_path = format!("{path}/{}{i}", node_type_label(&child.node_type));
        walk(
            child,
            child_path,
            column,
            next_suppress,
            next_off_canvas_suppress,
            boundary,
            page,
            opts,
            overflow_out,
            off_canvas_out,
            flow_out,
            text_out,
            has_content,
        );
    }
}

/// 흐름 요소(표·이미지 등)의 짝짓기 규칙 — 단이 다르면 짝짓지 않는다.
/// 단 밖(`None`)도 서로 다른 값으로 취급하는 종전 동작을 그대로 둔다.
fn flow_columns_can_overlap(a: Option<u16>, b: Option<u16>) -> bool {
    a == b
}

/// 글자 짝짓기 규칙 — "다른 단"과 "단 밖"을 구분한다.
///
/// 서로 다른 단은 x 축이 나뉘어 있어 정상 조판에서도 나란히 놓이므로 제외한다.
/// 그러나 **단 밖**(`None`) 은 다른 단이 아니라 단 개념이 없는 자리다 — 바탕쪽
/// 사이드바·머리말·꼬리말처럼 쪽에 고정된 글자가 여기 해당하고, 이들은 어느 단의
/// 본문과도 같은 자리에 놓일 수 있다. 종전 규칙(`a.column != b.column`)은 이 짝을
/// 통째로 버려서, 본문 글자가 바탕쪽 글자를 덮어도 신호가 0 이었다.
fn text_columns_can_overlap(a: Option<u16>, b: Option<u16>) -> bool {
    match (a, b) {
        (Some(x), Some(y)) => x == y,
        _ => true,
    }
}

fn find_overlaps(candidates: &[FlowCandidate], opts: &AnomalyOptions) -> Vec<OverlapAnomaly> {
    find_overlaps_with(candidates, opts, flow_columns_can_overlap)
}

fn find_text_overlaps(candidates: &[FlowCandidate], opts: &AnomalyOptions) -> Vec<OverlapAnomaly> {
    find_overlaps_with(candidates, opts, text_columns_can_overlap)
}

fn find_overlaps_with(
    candidates: &[FlowCandidate],
    opts: &AnomalyOptions,
    pair_allowed: fn(Option<u16>, Option<u16>) -> bool,
) -> Vec<OverlapAnomaly> {
    let mut out = Vec::new();
    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            let a = &candidates[i];
            let b = &candidates[j];
            if !pair_allowed(a.column, b.column) {
                continue;
            }
            if let Some((ow, oh)) = intersection(&a.bbox, &b.bbox) {
                if ow > opts.overlap_tolerance_px && oh > opts.overlap_tolerance_px {
                    out.push(OverlapAnomaly {
                        path_a: a.path.clone(),
                        type_a: a.node_type,
                        bbox_a: a.bbox,
                        path_b: b.path.clone(),
                        type_b: b.node_type,
                        bbox_b: b.bbox,
                        overlap_w: ow,
                        overlap_h: oh,
                    });
                }
            }
        }
    }
    out
}

/// 본문(`Body`) **밖** 영역에서 글자 겹침 후보만 모은다.
///
/// 페이지 트리의 직계 자식은 `Body` 하나가 아니다 — `MasterPage`(바탕쪽),
/// `Header`, `Footer`, `FootnoteArea` 가 함께 있고, 이들이 그리는 글자도 본문과
/// 같은 종이 위에 놓인다. 본문 글자가 바탕쪽 사이드바를 덮으면 사용자에게는 두 글자가
/// 겹쳐 보이는데, 종전에는 `Body` 서브트리만 순회해 이 짝이 애초에 후보가 아니었다.
///
/// 모으는 것은 **`TextRun` 후보뿐**이다. 컨테이너 overlap 후보(`flow`)에는 넣지 않는다 —
/// 바탕쪽은 전면 배경 이미지를 갖는 일이 흔해(편람 `Image x=0..740.8 y=0..1014.4`)
/// 컨테이너로 넣으면 그 이미지가 모든 것과 겹치는 오탐이 된다. overflow·off-canvas 의
/// 기준 상자(본문 여백·페이지 상자)도 그대로 둔다 — 이 함수는 판정 기준을 바꾸지 않고
/// 겹침 후보의 수집 범위만 넓힌다.
///
/// 단(column) 은 본문 개념이라 여기서 모은 후보는 모두 `column: None` 이다.
/// 짝짓기는 [`text_columns_can_overlap`] 이 "단 밖은 어느 단과도 짝이 된다"로 받는다.
fn collect_text_outside_body(node: &RenderNode, path: String, out: &mut Vec<FlowCandidate>) {
    if !node.visible || node.editor_only {
        return;
    }
    if is_text_overlap_candidate(node) {
        out.push(FlowCandidate {
            path: path.clone(),
            node_type: "TextRun",
            bbox: node.bbox,
            column: None,
        });
    }
    for (i, child) in node.children.iter().enumerate() {
        let child_path = format!("{path}/{}{i}", node_type_label(&child.node_type));
        collect_text_outside_body(child, child_path, out);
    }
}

/// 글자 겹침 후보를 모으는 본문 밖 영역인가.
///
/// `PageBackground` 는 종이 자체의 배경·테두리라 글자를 담지 않으므로 제외한다.
fn is_outside_body_text_area(t: &RenderNodeType) -> bool {
    matches!(
        t,
        RenderNodeType::MasterPage
            | RenderNodeType::Header
            | RenderNodeType::Footer
            | RenderNodeType::FootnoteArea
    )
}

/// [#6344] 페이지 어디든 보이는 내용이 있는가 — `empty_page` 판정 전용.
///
/// `walk` 의 `has_content` 는 `Body` 서브트리만 본다. 용지 기준으로 배치된 표·도형은
/// 페이지 직계 자식이라 그쪽에 잡히지 않으므로, 빈 쪽 판정에서만 페이지 전체를 훑는다.
/// 바탕쪽(`MasterPage`)은 제외한다 — 배경·장식은 모든 쪽에 있으므로 그걸 내용으로 세면
/// 빈 쪽 판정 자체가 무의미해진다. 머리말·꼬리말도 같은 이유로 제외한다(쪽번호만 있는
/// 빈 쪽을 "내용 있음" 으로 볼 수 없다).
fn page_has_visible_content(node: &RenderNode) -> bool {
    if !node.visible || node.editor_only {
        return false;
    }
    if matches!(
        node.node_type,
        RenderNodeType::MasterPage | RenderNodeType::Header | RenderNodeType::Footer
    ) {
        return false;
    }
    let self_has = match &node.node_type {
        RenderNodeType::TextRun(tr) => has_visible_text(&tr.text),
        RenderNodeType::Image(_)
        | RenderNodeType::Table(_)
        | RenderNodeType::Equation(_)
        | RenderNodeType::TextBox
        | RenderNodeType::Line(_)
        | RenderNodeType::Rectangle(_)
        | RenderNodeType::Ellipse(_)
        | RenderNodeType::Path(_)
        | RenderNodeType::Group(_)
        | RenderNodeType::FormObject(_)
        | RenderNodeType::Placeholder(_)
        | RenderNodeType::RawSvg(_) => true,
        _ => false,
    };
    self_has || node.children.iter().any(page_has_visible_content)
}

/// 한 페이지 렌더 트리를 스캔한다. `page_count` 는 `empty_page` 가 "문서 중간"인지
/// 판정하는 데만 쓴다(첫·마지막 쪽은 의도된 빈 쪽이 흔해 애초에 검사하지 않는다).
/// 저장 줄 baseline 과 렌더 baseline 을 px 로 맞추는 단위. `hwpunit_to_px` 한 곳만 쓴다.
fn stored_px(hwpunit: i32) -> f64 {
    crate::renderer::hwpunit_to_px(hwpunit, crate::renderer::DEFAULT_DPI)
}

/// 이 줄이 글자를 담고 있나 — 빈 줄에는 이탈을 물을 대상이 없다.
fn has_visible_run(node: &RenderNode) -> bool {
    if !node.visible || node.editor_only {
        return false;
    }
    if let RenderNodeType::TextRun(r) = &node.node_type {
        if has_visible_text(r.display_or_text()) {
            return true;
        }
    }
    node.children.iter().any(has_visible_run)
}

fn first_visible_text(node: &RenderNode) -> String {
    if !node.visible || node.editor_only {
        return String::new();
    }
    if let RenderNodeType::TextRun(r) = &node.node_type {
        let t = r.display_or_text();
        if has_visible_text(t) {
            return t.chars().take(24).collect();
        }
    }
    for c in &node.children {
        let t = first_visible_text(c);
        if !t.is_empty() {
            return t;
        }
    }
    String::new()
}

/// 저장 `LINE_SEG.text_start` 가 이 문단의 글자 색인 공간과 맞물리나.
///
/// `text_start` 는 저장 당시의 UTF-16 서수이고 `TextRunNode::char_start` 는 IR 문단
/// `text` 의 서수다. 확장 컨트롤을 세는 방식이 달라 둘이 어긋나는 문단이 있다 —
/// `samples/tac-case-003.hwp` 의 TAC host 문단은 `text` 가 35자인데 마지막 저장 줄이
/// `text_start=39` 에서 시작한다고 말한다. 그런 문단에서 서수로 줄을 고르면 엉뚱한
/// 줄을 짚는다(그 문서에서 실제로 정본과 0.23px 안에서 맞는 렌더가 24.19px 이탈로
/// 신고됐다). 색인 공간이 맞물리는 문단만 판정한다.
fn stored_text_space_matches(para: &crate::model::paragraph::Paragraph) -> bool {
    let len = para.text.encode_utf16().count() as u64;
    para.line_segs
        .last()
        .is_some_and(|seg| u64::from(seg.text_start) <= len)
}

/// 같은 문단의 **다른** 저장 줄 중 이 baseline 과 일치하는 줄의 서수.
///
/// 이 축이 신고하는 것은 "저장값과 다르다"가 아니라 "**남의 저장 줄에 앉았다**"이다.
/// 그 좁힘은 정본이 강요한 것이다 — [`StoredLineEscapeAnomaly`] 주석의 표를 본다.
fn baseline_belongs_to_another_line(
    para: &crate::model::paragraph::Paragraph,
    own: usize,
    rendered_baseline: f64,
    tol: f64,
) -> Option<usize> {
    use crate::model::paragraph::LineSeg;
    // 자기 저장 줄 **상자 안**에 있으면 이탈이 아니다.
    //
    // 이 조건이 없으면 "다른 줄과 값이 같다"가 우연으로 성립한다 — 한 문단의 여러 줄이
    // 같은 `baseline_distance` 를 갖는 것은 흔하므로, 몇 px 어긋난 값이 옆 줄 값과
    // 맞아떨어지는 일이 생긴다. `samples/issue6181/…` 27 건이 그 모양이었고, 정본은
    // 그 줄들에서 rhwp 가 **0.03px 안에서 맞다**고 말한다(저장값 쪽이 3.18px 틀렸다).
    let own_seg = para.line_segs.get(own)?;
    let own_height = stored_px(own_seg.line_height);
    if rendered_baseline >= -tol && rendered_baseline <= own_height + tol {
        return None;
    }
    para.line_segs.iter().enumerate().find_map(|(k, seg)| {
        (k != own
            && seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            && (rendered_baseline - stored_px(seg.baseline_distance)).abs() <= tol)
            .then_some(k)
    })
}

/// [#7061] 여섯째 축 — 글자가 자기 저장 줄의 baseline 에서 벗어났나.
///
/// 두 경로의 게이트와 실측 근거는 [`StoredLineEscapeAnomaly`] 주석에 있다.
fn check_stored_line_escape(
    node: &RenderNode,
    path: String,
    line: Option<(u32, i32, f64, f64)>,
    doc: &crate::model::document::Document,
    tol: f64,
    seen: &mut std::collections::BTreeSet<(usize, usize, usize)>,
    out: &mut Vec<StoredLineEscapeAnomaly>,
) {
    use crate::model::paragraph::LineSeg;

    if !node.visible || node.editor_only {
        return;
    }
    let mut cur = line;

    if let RenderNodeType::TextLine(l) = &node.node_type {
        cur = match (l.line_index, l.vpos) {
            (Some(li), Some(vp)) => Some((li, vp, node.bbox.height, l.baseline)),
            // 저장 줄을 가리키지 않는 줄 노드는 아래 런들의 판정 근거가 되지 못한다.
            _ => None,
        };
        if let (Some(si), Some(pi), Some((li, vp, rendered_h, rendered_base))) =
            (l.section_index, l.para_index, cur)
        {
            let para = doc.sections.get(si).and_then(|sec| sec.paragraphs.get(pi));
            if let Some((para, seg)) =
                para.and_then(|pa| pa.line_segs.get(li as usize).map(|sg| (pa, sg)))
            {
                let stored_h = stored_px(seg.line_height);
                let stored_base = stored_px(seg.baseline_distance);
                // 게이트: 합성 줄이 아니고, 그 저장 줄을 가리키며(vpos), 줄 높이까지 재현했다.
                let took = if seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                    && seg.vertical_pos == vp
                    && (rendered_h - stored_h).abs() <= tol
                    && (rendered_base - stored_base).abs() > tol
                {
                    baseline_belongs_to_another_line(para, li as usize, rendered_base, tol)
                } else {
                    None
                };
                if let Some(took) = took {
                    if has_visible_run(node) && seen.insert((si, pi, li as usize)) {
                        out.push(StoredLineEscapeAnomaly {
                            path: path.clone(),
                            section: si,
                            para: pi,
                            line_seg_index: li as usize,
                            took_line_seg_index: took,
                            from_line_node: true,
                            rendered_baseline: rendered_base,
                            stored_baseline: stored_base,
                            stored_line_height: stored_h,
                            text: first_visible_text(node),
                        });
                    }
                }
            }
        }
    }

    if let RenderNodeType::TextRun(r) = &node.node_type {
        // 줄 노드 없는 경로 — 상자가 `줄 위 ~ baseline` 형상일 때만 본다.
        if cur.is_none()
            && node.bbox.height > 0.0
            && (r.baseline - node.bbox.height).abs() <= tol
            && has_visible_text(r.display_or_text())
        {
            if let (Some(si), Some(pi), Some(cs)) = (r.section_index, r.para_index, r.char_start) {
                if let Some(para) = doc
                    .sections
                    .get(si)
                    .and_then(|s| s.paragraphs.get(pi))
                    .filter(|pa| stored_text_space_matches(pa))
                {
                    // 런이 저장 줄 **하나 안에 온전히** 들어갈 때만 그 줄의 것으로 본다.
                    //
                    // 시작 글자만 보면 줄 경계를 걸친 런을 앞 줄에 붙이게 된다.
                    // `samples/tac-case-002.hwp` 의 TAC host 문단이 그 모양이었다 — 런
                    // `tacglkj 표 다음`(23..34)이 저장 경계 31 을 넘는데 시작만 보면 표가
                    // 있는 첫 줄(baseline 35.52px)에 속한 것으로 읽혀, 실제로는 정본과
                    // 0.23px 안에서 맞는 렌더가 24.19px 이탈로 신고됐다.
                    let run_end = cs as u64 + r.text.encode_utf16().count() as u64;
                    let owner = para.line_segs.iter().enumerate().find(|(k, seg)| {
                        let next = para
                            .line_segs
                            .get(k + 1)
                            .map(|n| u64::from(n.text_start))
                            .unwrap_or(u64::MAX);
                        cs as u64 >= u64::from(seg.text_start) && run_end <= next
                    });
                    if let Some((k, seg)) = owner {
                        let stored_base = stored_px(seg.baseline_distance);
                        let took = if seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                            && (r.baseline - stored_base).abs() > tol
                        {
                            baseline_belongs_to_another_line(para, k, r.baseline, tol)
                        } else {
                            None
                        };
                        if let Some(took) = took {
                            if seen.insert((si, pi, k)) {
                                out.push(StoredLineEscapeAnomaly {
                                    path: path.clone(),
                                    section: si,
                                    para: pi,
                                    line_seg_index: k,
                                    took_line_seg_index: took,
                                    from_line_node: false,
                                    rendered_baseline: r.baseline,
                                    stored_baseline: stored_base,
                                    stored_line_height: stored_px(seg.line_height),
                                    text: r.display_or_text().chars().take(24).collect(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    for (i, child) in node.children.iter().enumerate() {
        let child_path = format!("{path}/{}{i}", node_type_label(&child.node_type));
        check_stored_line_escape(child, child_path, cur, doc, tol, seen, out);
    }
}

pub fn scan_page(
    page: u32,
    root: &RenderNode,
    page_count: u32,
    opts: &AnomalyOptions,
) -> PageAnomalies {
    scan_page_with_source(page, root, page_count, opts, None)
}

/// [#7061] `scan_page` 에 문서 IR 을 함께 준 판정.
///
/// 저장 줄 이탈 축만 IR 이 필요하다 — 렌더 트리에는 저장 `LINE_SEG.baseline_distance`
/// 가 없다. `source` 가 `None` 이면 그 축만 비고 나머지 다섯은 같다.
pub fn scan_page_with_source(
    page: u32,
    root: &RenderNode,
    page_count: u32,
    opts: &AnomalyOptions,
    source: Option<&crate::model::document::Document>,
) -> PageAnomalies {
    let mut overflow = Vec::new();
    let mut off_canvas = Vec::new();
    let mut flow = Vec::new();
    let mut text = Vec::new();
    let mut has_content = false;
    let page_boundary = page_box(root);

    if let Some(body) = find_body(root) {
        let boundary = body.bbox;
        walk(
            body,
            "Page/Body".to_string(),
            None,
            false,
            false,
            &boundary,
            &page_boundary,
            opts,
            &mut overflow,
            &mut off_canvas,
            &mut flow,
            &mut text,
            &mut has_content,
        );
    }

    // 본문 밖(바탕쪽·머리말·꼬리말·각주 영역)의 글자도 겹침 후보에 넣는다.
    // 본문 서브트리는 위에서 이미 순회했으므로 여기서 건너뛴다.
    for (i, child) in root.children.iter().enumerate() {
        if is_outside_body_text_area(&child.node_type) {
            let path = format!("Page/{}{i}", node_type_label(&child.node_type));
            collect_text_outside_body(child, path, &mut text);
        }
    }

    let overlap = find_overlaps(&flow, opts);
    let text_overlap = find_text_overlaps(&text, opts);

    let mut stored_line_escape = Vec::new();
    if let Some(doc) = source {
        let mut seen = std::collections::BTreeSet::new();
        check_stored_line_escape(
            root,
            "Page".to_string(),
            None,
            doc,
            opts.stored_line_tolerance_px,
            &mut seen,
            &mut stored_line_escape,
        );
    }

    // [#6344] 콘텐츠 유무는 **페이지 전체**로 판정한다.
    //
    // 위 `walk` 는 `Body` 서브트리만 도는데, 용지 기준으로 배치된 표·도형은 `Body` 가 아니라
    // **페이지 직계 자식**으로 그려진다. `Body` 만 보면 그 쪽이 통째로 비어 보인다.
    //
    // 실측(`samples/table-ipc.hwp`): 10쪽 문서의 8쪽이 빈 쪽으로 잡혔는데, 그 쪽들은
    // `Table`(181칸, 글자 154개)과 쪽번호 `Rect` 를 페이지 직계로 갖고 `Body` 는 비어 있다.
    // `export-text` 는 같은 쪽에서 700~860자를 뽑고 한컴 정답지도 10쪽 모두 861~1,026자다.
    //
    // overflow·off-canvas·overlap 의 기준 상자는 그대로다 — 이 보정은 "이 쪽에 내용이
    // 있는가" 하나만 고친다.
    if !has_content {
        has_content = page_has_visible_content(root);
    }

    // 문서 중간(첫·마지막 제외)이고 콘텐츠가 전혀 없을 때만 "가능성 신호"로 남긴다.
    let empty_page = if page_count >= 3 && page > 0 && page < page_count - 1 && !has_content {
        Some(EmptyPageAnomaly { page })
    } else {
        None
    };

    PageAnomalies {
        page,
        overflow,
        off_canvas,
        overlap,
        text_overlap,
        stored_line_escape,
        empty_page,
    }
}

/// 문서 전 페이지를 스캔한다. `render_geom_diff::diff_render_geometry` 와 같은 배선
/// (`build_page_render_tree` 를 페이지마다 호출)을 쓴다.
pub fn scan_document(core: &DocumentCore, opts: &AnomalyOptions) -> Result<DocAnomalies, HwpError> {
    let page_count = core.page_count();
    let mut pages = Vec::new();
    for p in 0..page_count {
        let tree = core.build_page_render_tree(p)?;
        let pa = scan_page_with_source(p, &tree.root, page_count, opts, Some(core.document()));
        if !pa.is_empty() {
            pages.push(pa);
        }
    }
    Ok(DocAnomalies { page_count, pages })
}

// ─────────────────────────────────────────────────────────────────────────
// CLI: `rhwp layout-anomaly`
// ─────────────────────────────────────────────────────────────────────────

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::schema_registry::ENVELOPE_SCHEMA_VERSION as SCHEMA_VERSION;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
/// `--strict` 가 확정 신호(overflow·off-canvas·overlap·text-overlap·stored-line-escape)를 하나라도
/// 찾았을 때 내는 코드. `render_geom_diff::EXIT_REGRESSION` 과 같은 값 — "검출은
/// 도구의 정상 동작"이라는 같은 계약이다. off-canvas 와 text-overlap 을 확정에 포함하는
/// 선택은 모듈 머리말·`PageAnomalies::has_signal` 주석과 같다.
const EXIT_ANOMALY: i32 = 3;

fn usage() -> String {
    "사용법: rhwp layout-anomaly <파일.hwp|파일.hwpx> [-p N] [--json] [--strict] \
     [--types Type,...] [--overflow-tolerance PX] [--overlap-tolerance PX]\n         \
     rhwp layout-anomaly --batch <폴더> [-p N] [--json] [--strict] \
     [--types Type,...] [--overflow-tolerance PX] [--overlap-tolerance PX] \n     [--stored-line-tolerance PX]"
        .to_string()
}

struct CliOptions {
    path: PathBuf,
    batch: bool,
    page: Option<u32>,
    json: bool,
    strict: bool,
    anomaly_opts: AnomalyOptions,
}

fn parse_type_filter(raw: &str) -> Result<Vec<&'static str>, String> {
    let mut out = Vec::new();
    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let found = CHECKABLE_TYPE_LABELS
            .iter()
            .copied()
            .find(|label| label.eq_ignore_ascii_case(part));
        match found {
            Some(label) => {
                if !out.contains(&label) {
                    out.push(label);
                }
            }
            None => {
                return Err(format!(
                    "--types 알 수 없는 노드 타입: {part} (허용: {})",
                    CHECKABLE_TYPE_LABELS.join(", ")
                ));
            }
        }
    }
    if out.is_empty() {
        return Err("--types 뒤에 Table,Image 같은 타입 목록이 필요합니다".into());
    }
    Ok(out)
}

fn parse_cli(args: &[String]) -> Result<CliOptions, String> {
    let mut path: Option<PathBuf> = None;
    let mut batch = false;
    let mut page = None;
    let mut json = false;
    let mut strict = false;
    let mut anomaly_opts = AnomalyOptions::default();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--strict" => strict = true,
            "--batch" => batch = true,
            "-p" | "--page" => {
                i += 1;
                let v = args.get(i).ok_or("-p 다음에 페이지 번호 필요")?;
                page = Some(
                    v.parse()
                        .map_err(|_| format!("페이지 번호 파싱 실패: {v}"))?,
                );
            }
            "--overflow-tolerance" => {
                i += 1;
                let v = args
                    .get(i)
                    .ok_or("--overflow-tolerance 다음에 px 값 필요")?;
                anomaly_opts.overflow_tolerance_px = v
                    .parse()
                    .map_err(|_| format!("--overflow-tolerance 파싱 실패: {v}"))?;
            }
            "--overlap-tolerance" => {
                i += 1;
                let v = args.get(i).ok_or("--overlap-tolerance 다음에 px 값 필요")?;
                anomaly_opts.overlap_tolerance_px = v
                    .parse()
                    .map_err(|_| format!("--overlap-tolerance 파싱 실패: {v}"))?;
            }
            "--stored-line-tolerance" => {
                i += 1;
                let v = args
                    .get(i)
                    .ok_or("--stored-line-tolerance 다음에 px 값 필요")?;
                anomaly_opts.stored_line_tolerance_px = v
                    .parse()
                    .map_err(|_| format!("--stored-line-tolerance 파싱 실패: {v}"))?;
            }
            "--types" => {
                i += 1;
                let v = args.get(i).ok_or("--types 다음에 타입 목록 필요")?;
                anomaly_opts.type_filter = Some(parse_type_filter(v)?);
            }
            other if other.starts_with('-') => return Err(format!("알 수 없는 옵션: {other}")),
            other => {
                if path.replace(PathBuf::from(other)).is_some() {
                    return Err(if batch {
                        "--batch 는 폴더 1개만 지정".into()
                    } else {
                        "입력 파일은 하나만 지정할 수 있습니다".into()
                    });
                }
            }
        }
        i += 1;
    }

    let path = path.ok_or_else(usage)?;
    Ok(CliOptions {
        path,
        batch,
        page,
        json,
        strict,
        anomaly_opts,
    })
}

fn bbox_json(b: &BoundingBox) -> Value {
    json!({ "x": b.x, "y": b.y, "w": b.width, "h": b.height })
}

fn overflow_json(o: &OverflowAnomaly) -> Value {
    json!({
        "path": o.path,
        "nodeType": o.node_type,
        "bbox": bbox_json(&o.bbox),
        "boundary": bbox_json(&o.boundary),
        "overLeft": o.over_left,
        "overTop": o.over_top,
        "overRight": o.over_right,
        "overBottom": o.over_bottom,
        "maxOver": o.max_over(),
    })
}

fn overlap_json(o: &OverlapAnomaly) -> Value {
    json!({
        "pathA": o.path_a,
        "typeA": o.type_a,
        "bboxA": bbox_json(&o.bbox_a),
        "pathB": o.path_b,
        "typeB": o.type_b,
        "bboxB": bbox_json(&o.bbox_b),
        "overlapW": o.overlap_w,
        "overlapH": o.overlap_h,
        "overlapArea": o.overlap_area(),
    })
}

fn page_json(p: &PageAnomalies) -> Value {
    json!({
        "page": p.page,
        "overflow": p.overflow.iter().map(overflow_json).collect::<Vec<_>>(),
        "offCanvas": p.off_canvas.iter().map(overflow_json).collect::<Vec<_>>(),
        "overlap": p.overlap.iter().map(overlap_json).collect::<Vec<_>>(),
        "textOverlap": p.text_overlap.iter().map(overlap_json).collect::<Vec<_>>(),
        "storedLineEscape": p.stored_line_escape.iter().map(stored_line_escape_json).collect::<Vec<_>>(),
        "emptyPage": p.empty_page.is_some(),
    })
}

fn stored_line_escape_json(a: &StoredLineEscapeAnomaly) -> Value {
    json!({
        "path": a.path,
        "section": a.section,
        "para": a.para,
        "lineSegIndex": a.line_seg_index,
        "tookLineSegIndex": a.took_line_seg_index,
        "fromLineNode": a.from_line_node,
        "renderedBaseline": a.rendered_baseline,
        "storedBaseline": a.stored_baseline,
        "storedLineHeight": a.stored_line_height,
        "deviation": a.deviation(),
        "text": a.text,
    })
}

fn types_json(opts: &CliOptions) -> Value {
    match &opts.anomaly_opts.type_filter {
        Some(types) => json!(types),
        None => Value::Null,
    }
}

/// [#6348] `-p` 가 주어지면 그 쪽만 남긴 결과를 돌려준다.
///
/// 종전에는 `envelope` 이 `pages` 배열만 걸러내고 카운트·`hasSignal` 은 필터 이전의
/// 문서 전체 값을 실었다. 그래서 신호가 0 인 쪽을 지정해도 `overflowCount: 163`,
/// `hasSignal: true` 가 나오고 `--strict` 가 종료코드 3 을 냈다 — 배열과 카운트가 서로
/// 모순되고, 쪽 단위 판정에 쓸 수 없었다.
///
/// `page_count` 는 문서 전체 쪽수라 필터와 무관한 메타데이터다. `pageFilter` 와 함께
/// 읽으면 "전체 N 쪽 중 M 쪽" 이 되므로 그대로 둔다.
fn filtered_for_page(doc: &DocAnomalies, page: Option<u32>) -> Cow<'_, DocAnomalies> {
    let Some(want) = page else {
        return Cow::Borrowed(doc);
    };
    Cow::Owned(DocAnomalies {
        page_count: doc.page_count,
        pages: doc
            .pages
            .iter()
            .filter(|p| p.page == want)
            .cloned()
            .collect(),
    })
}

fn envelope(source: &str, doc: &DocAnomalies, opts: &CliOptions) -> Value {
    // 카운트·hasSignal 과 pages 는 반드시 같은 집합에서 나와야 한다. 필터를 여기서
    // 다시 적용하는 것은 이미 걸러진 값을 받아도 무해하며(idempotent), 호출자가
    // 하나라도 빠뜨렸을 때 JSON 안에서 서로 모순되는 값이 나가는 것을 막는다.
    let doc = &filtered_for_page(doc, opts.page);
    let pages: Vec<Value> = doc.pages.iter().map(page_json).collect();
    crate::provenance::marked(
        json!({
            "schemaVersion": SCHEMA_VERSION,
            "mode": if opts.batch { "batch" } else { "single" },
            "source": source,
            "pageCount": doc.page_count,
            "pageFilter": opts.page,
            "overflowTolerancePx": opts.anomaly_opts.overflow_tolerance_px,
            "overlapTolerancePx": opts.anomaly_opts.overlap_tolerance_px,
            "types": types_json(opts),
            "strict": opts.strict,
            "overflowCount": doc.overflow_count(),
            "offCanvasCount": doc.off_canvas_count(),
            "overlapCount": doc.overlap_count(),
            "textOverlapCount": doc.text_overlap_count(),
            "storedLineEscapeCount": doc.stored_line_escape_count(),
            "emptyPageCount": doc.empty_page_count(),
            "hasSignal": doc.has_signal(),
            "pages": pages,
        }),
        "layout-anomaly",
    )
}

fn error_envelope(source: &str, error: &str, opts: &CliOptions, elapsed_ms: u128) -> Value {
    let mut rec = crate::provenance::marked(
        json!({
            "schemaVersion": SCHEMA_VERSION,
            "mode": "batch",
            "source": source,
            "pageCount": 0,
            "pageFilter": opts.page,
            "overflowTolerancePx": opts.anomaly_opts.overflow_tolerance_px,
            "overlapTolerancePx": opts.anomaly_opts.overlap_tolerance_px,
            "types": types_json(opts),
            "strict": opts.strict,
            "overflowCount": 0,
            "overlapCount": 0,
            "emptyPageCount": 0,
            "hasSignal": false,
            "pages": [],
            "elapsedMs": elapsed_ms as u64,
        }),
        "layout-anomaly",
    );
    rec["error"] = json!(error);
    rec
}

fn load_and_scan(path: &Path, opts: &AnomalyOptions) -> Result<DocAnomalies, String> {
    let data =
        std::fs::read(path).map_err(|e| format!("파일 읽기 실패 {}: {e}", path.display()))?;
    let core = DocumentCore::from_bytes(&data)
        .map_err(|e| format!("문서 로드 실패 {}: {e:?}", path.display()))?;
    scan_document(&core, opts).map_err(|e| format!("렌더 트리 생성 실패 - {e:?}"))
}

fn run_single(opts: &CliOptions) -> i32 {
    let doc = match load_and_scan(&opts.path, &opts.anomaly_opts) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_RUNTIME;
        }
    };

    if let Some(want) = opts.page {
        if want >= doc.page_count {
            eprintln!(
                "오류: -p {want} 는 문서 범위 밖입니다 (쪽 0..{})",
                doc.page_count
            );
            return EXIT_USAGE;
        }
    }

    // -p 는 여기서 한 번만 적용한다. 이후의 카운트·hasSignal·종료코드는 모두
    // 이 걸러진 집합에서 나오므로 출력끼리 서로 모순되지 않는다.
    let doc = filtered_for_page(&doc, opts.page);

    if opts.json {
        println!("{}", envelope(&opts.path.display().to_string(), &doc, opts));
        return if opts.strict && doc.has_signal() {
            EXIT_ANOMALY
        } else {
            EXIT_OK
        };
    }

    let shown: Vec<&PageAnomalies> = doc.pages.iter().collect();

    println!(
        "쪽 수: {}  overflow: {}  off-canvas: {}  overlap: {}  text-overlap: {}           stored-line-escape: {}  empty_page(가능성): {}",
        doc.page_count,
        doc.overflow_count(),
        doc.off_canvas_count(),
        doc.overlap_count(),
        doc.text_overlap_count(),
        doc.stored_line_escape_count(),
        doc.empty_page_count()
    );
    if shown.is_empty() {
        println!("이상 신호 없음: {}", opts.path.display());
    }
    for p in &shown {
        for o in &p.overflow {
            println!(
                "  [OVERFLOW] page {:>3}  {:>7.2}px  {} ({})",
                p.page,
                o.max_over(),
                o.path,
                o.node_type
            );
        }
        for o in &p.off_canvas {
            println!(
                "  [OFF-CANVAS] page {:>3}  {:>7.2}px  {} ({})",
                p.page,
                o.max_over(),
                o.path,
                o.node_type
            );
        }
        for o in &p.overlap {
            println!(
                "  [OVERLAP]  page {:>3}  {:.2}x{:.2}px  {} ({}) x {} ({})",
                p.page, o.overlap_w, o.overlap_h, o.path_a, o.type_a, o.path_b, o.type_b
            );
        }
        for o in &p.text_overlap {
            println!(
                "  [TEXT-OVERLAP] page {:>3}  {:.2}x{:.2}px  {} ({}) x {} ({})",
                p.page, o.overlap_w, o.overlap_h, o.path_a, o.type_a, o.path_b, o.type_b
            );
        }
        if p.empty_page.is_some() {
            println!(
                "  [EMPTY_PAGE?] page {:>3}  콘텐츠 없음 (가능성 신호 — 의도된 빈 쪽일 수 있음)",
                p.page
            );
        }
    }
    println!(
        "status: {}",
        if doc.has_signal() { "ANOMALY" } else { "CLEAN" }
    );

    if opts.strict && doc.has_signal() {
        EXIT_ANOMALY
    } else {
        EXIT_OK
    }
}

/// `.hwp`/`.hwpx` 파일을 재귀 수집한 뒤 상대 경로(슬래시) 기준으로 정렬한다.
/// 정렬 키가 안정적이어야 같은 폴더를 다시 돌려도 보고서 순서가 같다.
fn collect_doc_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    if !root.is_dir() {
        return Err(format!(
            "--batch 는 폴더만 지정할 수 있습니다: {}",
            root.display()
        ));
    }
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .map_err(|e| format!("폴더 읽기 실패 {}: {e}", dir.display()))?;
        for entry in entries {
            let path = entry.map_err(|e| format!("항목 읽기 실패: {e}"))?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("hwp") || ext.eq_ignore_ascii_case("hwpx")
            }) {
                files.push(path);
            }
        }
    }
    files.sort_by_key(|a| rel_slash(root, a));
    Ok(files)
}

fn rel_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

struct BatchRow {
    rel_path: String,
    status: String,
    overflow: usize,
    overlap: usize,
    empty_page: usize,
    has_signal: bool,
    elapsed_ms: u128,
    error: String,
    envelope: Option<Value>,
}

fn batch_record(opts: &CliOptions, row: &BatchRow) -> Value {
    if let Some(env) = &row.envelope {
        let mut rec = env.clone();
        rec["elapsedMs"] = json!(row.elapsed_ms as u64);
        rec
    } else {
        error_envelope(&row.rel_path, &row.error, opts, row.elapsed_ms)
    }
}

fn print_batch_summary(rows: &[BatchRow], json: bool) {
    let count = |s: &str| rows.iter().filter(|r| r.status == s).count();
    let text = format!(
        "\n=== layout-anomaly 요약 ===\n  총 파일         : {}\n  CLEAN           : {}\n  ANOMALY         : {}\n  LOAD_FAIL       : {}\n",
        rows.len(),
        count("CLEAN"),
        count("ANOMALY"),
        count("LOAD_FAIL"),
    );
    if json {
        eprint!("{text}");
    } else {
        print!("{text}");
    }
}

fn run_batch(opts: &CliOptions) -> i32 {
    let files = match collect_doc_files(&opts.path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_USAGE;
        }
    };
    if files.is_empty() {
        eprintln!(
            "오류: 처리할 .hwp/.hwpx 파일이 없습니다: {}",
            opts.path.display()
        );
        return EXIT_USAGE;
    }

    let mut rows = Vec::with_capacity(files.len());
    for path in &files {
        let rel = rel_slash(&opts.path, path);
        let started = Instant::now();
        let mut row = BatchRow {
            rel_path: rel.clone(),
            status: "LOAD_FAIL".into(),
            overflow: 0,
            overlap: 0,
            empty_page: 0,
            has_signal: false,
            elapsed_ms: 0,
            error: String::new(),
            envelope: None,
        };
        match load_and_scan(path, &opts.anomaly_opts) {
            Ok(doc) => {
                if let Some(want) = opts.page {
                    if want >= doc.page_count {
                        row.error =
                            format!("-p {want} 는 문서 범위 밖입니다 (쪽 0..{})", doc.page_count);
                    } else {
                        fill_ok_row(&mut row, &doc, opts);
                    }
                } else {
                    fill_ok_row(&mut row, &doc, opts);
                }
            }
            Err(e) => row.error = e,
        }
        row.elapsed_ms = started.elapsed().as_millis();
        if opts.json {
            println!("{}", batch_record(opts, &row));
        } else {
            println!(
                "[{:>15}] overflow={:<4} overlap={:<4} empty={:<3} {:>6}ms  {}",
                row.status, row.overflow, row.overlap, row.empty_page, row.elapsed_ms, row.rel_path
            );
            if !row.error.is_empty() {
                println!("                  └ {}", row.error);
            }
        }
        rows.push(row);
    }

    print_batch_summary(&rows, opts.json);

    // 로드 실패는 측정 실패(1)다. 전건을 재봤다고 말할 수 없으면 --strict
    // 이상 검출(3)보다 런타임 실패가 우선한다. 기본은 이상 신호가 있어도 0.
    if rows.iter().any(|r| !r.error.is_empty()) {
        return EXIT_RUNTIME;
    }
    if opts.strict && rows.iter().any(|r| r.has_signal) {
        EXIT_ANOMALY
    } else {
        EXIT_OK
    }
}

fn fill_ok_row(row: &mut BatchRow, doc: &DocAnomalies, opts: &CliOptions) {
    let doc = &filtered_for_page(doc, opts.page);
    row.overflow = doc.overflow_count();
    row.overlap = doc.overlap_count();
    row.empty_page = doc.empty_page_count();
    row.has_signal = doc.has_signal();
    row.status = if row.has_signal {
        "ANOMALY".into()
    } else {
        "CLEAN".into()
    };
    row.envelope = Some(envelope(&row.rel_path, doc, opts));
}

/// `rhwp layout-anomaly` 진입점.
pub fn run(args: &[String]) -> i32 {
    let opts = match parse_cli(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_USAGE;
        }
    };
    if opts.batch {
        run_batch(&opts)
    } else {
        run_single(&opts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::render_tree::{
        BoundingBox, RectangleNode, RenderLayerInfo, RenderNode, RenderNodeType, TableCellNode,
        TableNode, TextLineNode, TextRunNode,
    };
    use crate::renderer::ShapeStyle;

    fn page_root(width: f64, height: f64, body: RenderNode) -> RenderNode {
        let mut root = RenderNode::new(
            0,
            RenderNodeType::Page(crate::renderer::render_tree::PageNode {
                page_index: 0,
                width,
                height,
                section_index: 0,
            }),
            BoundingBox::new(0.0, 0.0, width, height),
        );
        root.children.push(body);
        root
    }

    fn body_node(bbox: BoundingBox, children: Vec<RenderNode>) -> RenderNode {
        let mut n = RenderNode::new(1, RenderNodeType::Body { clip_rect: None }, bbox);
        n.children = children;
        n
    }

    fn text_line(x: f64, y: f64, w: f64, h: f64) -> RenderNode {
        RenderNode::new(
            99,
            RenderNodeType::TextLine(TextLineNode::new(h, h * 0.8)),
            BoundingBox::new(x, y, w, h),
        )
    }

    fn text_run(text: &str) -> RenderNode {
        RenderNode::new(
            100,
            RenderNodeType::TextRun(TextRunNode {
                text: text.to_string(),
                style: crate::renderer::TextStyle::default(),
                char_shape_id: None,
                para_shape_id: None,
                section_index: None,
                para_index: None,
                char_start: None,
                cell_context: None,
                is_para_end: false,
                is_line_break_end: false,
                rotation: 0.0,
                is_vertical: false,
                char_overlap: None,
                border_fill_id: 0,
                baseline: 0.0,
                field_marker: Default::default(),
                layout_positions: None,
                display_text: None,
            }),
            BoundingBox::new(0.0, 0.0, 10.0, 10.0),
        )
    }

    fn text_run_at(text: &str, x: f64, y: f64, w: f64, h: f64) -> RenderNode {
        let mut n = text_run(text);
        n.bbox = BoundingBox::new(x, y, w, h);
        n
    }

    fn table(x: f64, y: f64, w: f64, h: f64, children: Vec<RenderNode>) -> RenderNode {
        let mut n = RenderNode::new(
            2,
            RenderNodeType::Table(TableNode {
                row_count: 1,
                col_count: 1,
                border_fill_id: 0,
                section_index: None,
                para_index: None,
                control_index: None,
                cell_context: None,
            }),
            BoundingBox::new(x, y, w, h),
        );
        n.children = children;
        n
    }

    /// 겹침 후보 테스트용 floating 도형(Rectangle) — 실제 Image 대신 필드가 적은
    /// Rectangle 로 같은 `layer`/`text_wrap` 조합을 재현한다(overlap 판정은 노드
    /// 타입이 아니라 `layer.text_wrap` 을 본다).
    fn floating_shape(x: f64, y: f64, w: f64, h: f64, wrap: TextWrap) -> RenderNode {
        RenderNode::new(
            3,
            RenderNodeType::Rectangle(RectangleNode::new(0.0, ShapeStyle::default(), None)),
            BoundingBox::new(x, y, w, h),
        )
        .with_layer(RenderLayerInfo::new(Some(wrap), 0, 0))
    }

    #[test]
    fn clean_document_has_no_anomalies() {
        let mut line = text_line(10.0, 10.0, 50.0, 12.0);
        line.children.push(text_run("hello"));
        let body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 200.0), vec![line]);
        let root = page_root(100.0, 200.0, body);
        let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
        assert!(pa.is_empty());
    }

    #[test]
    fn table_wider_than_body_is_flagged_overflow() {
        // 좁은 본문(폭 100) 안에 폭 200짜리 표 — 명백한 스캐폴딩 케이스.
        // 쪽 폭은 300 이라 본문만 넘치고 페이지 상자 안에는 남는다 → overflow
        // 만, off-canvas 는 아님.
        let t = table(0.0, 0.0, 200.0, 50.0, vec![]);
        let body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 300.0), vec![t]);
        let root = page_root(300.0, 300.0, body);
        let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
        assert_eq!(pa.overflow.len(), 1);
        assert_eq!(pa.overflow[0].node_type, "Table");
        assert!((pa.overflow[0].over_right - 100.0).abs() < 1e-9);
        assert!(
            pa.off_canvas.is_empty(),
            "본문만 넘치고 쪽 안에 있으면 off-canvas 가 아니다: {:?}",
            pa.off_canvas
        );
        assert!(pa.has_signal());
    }

    #[test]
    fn overflow_within_tolerance_is_not_flagged() {
        let t = table(0.0, 0.0, 100.5, 50.0, vec![]); // 0.5px 초과 — 기본 허용치(1.0px) 이내
        let body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 300.0), vec![t]);
        let root = page_root(100.0, 300.0, body);
        let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
        assert!(pa.overflow.is_empty());
    }

    #[test]
    fn nested_lines_inside_overflowing_table_are_not_double_reported() {
        let mut cell_line = text_line(0.0, 0.0, 30.0, 10.0);
        cell_line.children.push(text_run("x"));
        let cell = RenderNode::new(
            5,
            RenderNodeType::TableCell(TableCellNode {
                col: 0,
                row: 0,
                col_span: 1,
                row_span: 1,
                border_fill_id: 0,
                text_direction: 0,
                clip: false,
                page_fragment: false,
                model_cell_index: None,
            }),
            BoundingBox::new(0.0, 0.0, 30.0, 10.0),
        );
        let mut cell = cell;
        cell.children.push(cell_line);
        let t = table(0.0, 0.0, 200.0, 50.0, vec![cell]);
        let body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 300.0), vec![t]);
        let root = page_root(300.0, 300.0, body);
        let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
        // 표 자신만 한 번 보고 — 내부 줄이 별도로 다시 잡히지 않는다.
        assert_eq!(pa.overflow.len(), 1);
        assert_eq!(pa.overflow[0].node_type, "Table");
        assert!(pa.off_canvas.is_empty());
    }

    #[test]
    fn two_overlapping_lines_are_flagged() {
        let mut line_a = text_line(10.0, 10.0, 50.0, 12.0);
        line_a
            .children
            .push(text_run_at("a", 10.0, 10.0, 50.0, 12.0));
        let mut line_b = text_line(15.0, 12.0, 50.0, 12.0); // line_a 와 상당 부분 겹침
        line_b
            .children
            .push(text_run_at("b", 15.0, 12.0, 50.0, 12.0));
        let body = body_node(
            BoundingBox::new(0.0, 0.0, 200.0, 300.0),
            vec![line_a, line_b],
        );
        let root = page_root(200.0, 300.0, body);
        let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
        assert_eq!(pa.overlap.len(), 1);
        assert_eq!(pa.text_overlap.len(), 1);
        assert_eq!(pa.text_overlap[0].type_a, "TextRun");
        assert_eq!(pa.text_overlap[0].type_b, "TextRun");
        assert!(pa.has_signal());
    }

    #[test]
    fn adjacent_non_overlapping_lines_are_clean() {
        let mut line_a = text_line(10.0, 10.0, 50.0, 12.0);
        line_a
            .children
            .push(text_run_at("a", 10.0, 10.0, 50.0, 12.0));
        let mut line_b = text_line(10.0, 22.0, 50.0, 12.0); // 바로 아래로 이어짐, 안 겹침
        line_b
            .children
            .push(text_run_at("b", 10.0, 22.0, 50.0, 12.0));
        let body = body_node(
            BoundingBox::new(0.0, 0.0, 200.0, 300.0),
            vec![line_a, line_b],
        );
        let root = page_root(200.0, 300.0, body);
        let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
        assert!(pa.overlap.is_empty());
        assert!(pa.text_overlap.is_empty());
    }

    #[test]
    fn behind_text_floats_do_not_count_as_overlap() {
        // BehindText 는 텍스트와 겹치라고 있는 wrap — 후보에서 제외되어야 한다.
        let mut line = text_line(10.0, 10.0, 50.0, 12.0);
        line.children.push(text_run("a"));
        let img = floating_shape(10.0, 10.0, 50.0, 12.0, TextWrap::BehindText);
        let body = body_node(BoundingBox::new(0.0, 0.0, 200.0, 300.0), vec![line, img]);
        let root = page_root(200.0, 300.0, body);
        let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
        assert!(pa.overlap.is_empty());
    }

    #[test]
    fn empty_carrier_line_does_not_falsely_overlap_its_own_table() {
        // 실측(samples/2025 행정업무운영 편람(최종).hwpx, 380쪽): 표를 문단에
        // 앵커링하는 "운반용" TextLine 은 빈 TextRun(text: "") 하나만 자식으로 두고
        // 그 표와 정확히 같은 좌상단에 찍힌다 — is_overlap_candidate 가 이를
        // TextLine 으로 취급해 후보에 넣으면 표 하나마다 겹침 오탐이 하나씩
        // 생긴다(실측 43건/380쪽). 보이는 글자가 없는 줄은 후보에서 빠져야 한다.
        let mut carrier = text_line(98.3, 267.3, 13.3, 13.3);
        carrier.children.push(text_run("")); // 빈 텍스트 — 화면에 아무것도 안 그림
        let t = table(98.3, 267.3, 529.1, 221.6, vec![]);
        let body = body_node(BoundingBox::new(0.0, 0.0, 700.0, 900.0), vec![t, carrier]);
        let root = page_root(700.0, 900.0, body);
        let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
        assert!(
            pa.overlap.is_empty(),
            "빈 운반용 줄이 표와 겹침으로 오탐되면 안 된다: {:?}",
            pa.overlap
        );
    }

    #[test]
    fn square_wrap_floats_overlapping_each_other_are_flagged() {
        let img_a = floating_shape(10.0, 10.0, 40.0, 40.0, TextWrap::Square);
        let img_b = floating_shape(20.0, 20.0, 40.0, 40.0, TextWrap::Square);
        let body = body_node(BoundingBox::new(0.0, 0.0, 200.0, 300.0), vec![img_a, img_b]);
        let root = page_root(200.0, 300.0, body);
        let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
        assert_eq!(pa.overlap.len(), 1);
        assert!(
            pa.text_overlap.is_empty(),
            "도형끼리 겹침은 일반 overlap 이지 text-overlap 이 아니다: {:?}",
            pa.text_overlap
        );
    }

    #[test]
    fn middle_empty_page_is_possible_signal_but_not_hard_signal() {
        let body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 200.0), vec![]);
        let root = page_root(100.0, 200.0, body);
        let pa = scan_page(1, &root, 3, &AnomalyOptions::default());
        assert!(pa.empty_page.is_some());
        // empty_page 는 has_signal() (strict 하드 실패 대상) 에 안 들어간다.
        assert!(!pa.has_signal());
    }

    #[test]
    fn first_and_last_empty_pages_are_not_flagged() {
        let body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 200.0), vec![]);
        let root_first = page_root(100.0, 200.0, body.clone());
        let pa_first = scan_page(0, &root_first, 3, &AnomalyOptions::default());
        assert!(pa_first.empty_page.is_none());

        let root_last = page_root(100.0, 200.0, body);
        let pa_last = scan_page(2, &root_last, 3, &AnomalyOptions::default());
        assert!(pa_last.empty_page.is_none());
    }

    #[test]
    fn single_page_document_never_flags_empty_page() {
        let body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 200.0), vec![]);
        let root = page_root(100.0, 200.0, body);
        let pa = scan_page(0, &root, 1, &AnomalyOptions::default());
        assert!(pa.empty_page.is_none());
    }

    #[path = "layout_anomaly_tests.rs"]
    mod behavioral;
}
