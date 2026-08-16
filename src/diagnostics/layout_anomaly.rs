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
//! 요소가 페이지 여백을 벗어났는가(overflow), 겹치면 안 되는 요소끼리 겹쳤는가
//! (overlap), 콘텐츠 없는 페이지가 문서 중간에 있는가(empty_page). `render-diff`가
//! "달라졌는가"를 묻는다면 이 모듈은 "이상해 보이는가"를 묻는다 — 같은 렌더 기하
//! 축 위의 서로 다른 질문이라 한쪽이 다른 쪽을 대신하지 않는다.
//!
//! # 설계 원칙 — 판정은 데이터, 차단은 소비자 몫
//!
//! 이 저장소의 다른 진단 명령(`render-diff`, `inspect hidden-text` 등)과 같은
//! 철학이다. 탐지 건수가 0이 아니어도 기본 종료 코드는 0이다 — anomaly 발견은
//! 도구의 정상 동작이지 실패가 아니다. 소비자가 실패로 취급하고 싶으면 명시적으로
//! `--strict` 를 준다. `empty_page` 는 특히 오탐 여지가 크다(의도된 표지·구분지
//! 빈 쪽과 회귀를 기하만으로 구분할 수 없다) — 그래서 `--strict` 로도 절대 실패를
//! 유발하지 않는 "가능성 신호"로만 분리해 낸다. 자세한 배경은
//! `mydocs/tech/layout_anomaly_detection.md`.
//!
//! # 입력 경계
//!
//! 렌더러가 이미 만들어 내는 [`RenderNode`] 트리를 **읽기만** 한다 — 렌더러·레이아웃
//! 엔진 코드는 건드리지 않는다. `document_core::DocumentCore::build_page_render_tree`
//! 가 유일한 진입점이고, 이 모듈은 그 산출물의 소비자다.

use serde_json::{json, Value};

use crate::document_core::DocumentCore;
use crate::model::shape::TextWrap;
use crate::renderer::render_tree::{BoundingBox, RenderLayerInfo, RenderNode, RenderNodeType};
use crate::HwpError;

// ─────────────────────────────────────────────────────────────────────────
// 판정 옵션 · 데이터 모델
// ─────────────────────────────────────────────────────────────────────────

/// 스캔 임계값. 둘 다 렌더 트리와 같은 단위(px)다.
#[derive(Debug, Clone, Copy)]
pub struct AnomalyOptions {
    /// 요소 bbox가 본문 영역을 이 값(px) 넘게 벗어나야 overflow로 잡는다.
    /// 하위 픽셀 반올림 노이즈를 거르는 목적 — `render-diff` 의 기본 변위
    /// 임계(1.0px)와 같은 자릿수를 쓴다.
    pub overflow_tolerance_px: f64,
    /// 두 요소의 겹침 폭·높이가 **둘 다** 이 값(px)을 넘어야 overlap으로 잡는다.
    /// 모서리가 살짝 스치는 것(안티앨리어싱·반올림)은 정상 조판에서도 흔하다.
    pub overlap_tolerance_px: f64,
}

/// 기본 overflow 허용치(px). `render_geom_diff::DEFAULT_MAX_DISP` 와 같은 자릿수.
pub const DEFAULT_OVERFLOW_TOLERANCE_PX: f64 = 1.0;
/// 기본 overlap 허용치(px, 폭·높이 각각).
pub const DEFAULT_OVERLAP_TOLERANCE_PX: f64 = 2.0;

impl Default for AnomalyOptions {
    fn default() -> Self {
        Self {
            overflow_tolerance_px: DEFAULT_OVERFLOW_TOLERANCE_PX,
            overlap_tolerance_px: DEFAULT_OVERLAP_TOLERANCE_PX,
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
    pub overlap: Vec<OverlapAnomaly>,
    pub empty_page: Option<EmptyPageAnomaly>,
}

impl PageAnomalies {
    pub fn is_empty(&self) -> bool {
        self.overflow.is_empty() && self.overlap.is_empty() && self.empty_page.is_none()
    }

    /// `--strict` 가 실패로 셀 만한 확정 신호(overflow·overlap)가 있는가.
    /// `empty_page` 는 가능성 신호일 뿐이라 제외한다.
    pub fn has_signal(&self) -> bool {
        !self.overflow.is_empty() || !self.overlap.is_empty()
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

    pub fn overlap_count(&self) -> usize {
        self.pages.iter().map(|p| p.overlap.len()).sum()
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
    boundary: &BoundingBox,
    opts: &AnomalyOptions,
    overflow_out: &mut Vec<OverflowAnomaly>,
    flow_out: &mut Vec<FlowCandidate>,
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

    let mut next_suppress = suppress;
    if !suppress && is_checkable(&node.node_type) {
        let label = node_type_label(&node.node_type);
        check_overflow(&node.bbox, &path, label, boundary, opts, overflow_out);
        if is_overlap_candidate(node) {
            flow_out.push(FlowCandidate {
                path: path.clone(),
                node_type: label,
                bbox: node.bbox,
                column,
            });
        }
        next_suppress = true;
    }

    for (i, child) in node.children.iter().enumerate() {
        let child_path = format!("{path}/{}{i}", node_type_label(&child.node_type));
        walk(
            child,
            child_path,
            column,
            next_suppress,
            boundary,
            opts,
            overflow_out,
            flow_out,
            has_content,
        );
    }
}

fn find_overlaps(candidates: &[FlowCandidate], opts: &AnomalyOptions) -> Vec<OverlapAnomaly> {
    let mut out = Vec::new();
    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            let a = &candidates[i];
            let b = &candidates[j];
            if a.column != b.column {
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

/// 한 페이지 렌더 트리를 스캔한다. `page_count` 는 `empty_page` 가 "문서 중간"인지
/// 판정하는 데만 쓴다(첫·마지막 쪽은 의도된 빈 쪽이 흔해 애초에 검사하지 않는다).
pub fn scan_page(
    page: u32,
    root: &RenderNode,
    page_count: u32,
    opts: &AnomalyOptions,
) -> PageAnomalies {
    let mut overflow = Vec::new();
    let mut flow = Vec::new();
    let mut has_content = false;

    if let Some(body) = find_body(root) {
        let boundary = body.bbox;
        walk(
            body,
            "Page/Body".to_string(),
            None,
            false,
            &boundary,
            opts,
            &mut overflow,
            &mut flow,
            &mut has_content,
        );
    }

    let overlap = find_overlaps(&flow, opts);

    // 문서 중간(첫·마지막 제외)이고 콘텐츠가 전혀 없을 때만 "가능성 신호"로 남긴다.
    let empty_page = if page_count >= 3 && page > 0 && page < page_count - 1 && !has_content {
        Some(EmptyPageAnomaly { page })
    } else {
        None
    };

    PageAnomalies {
        page,
        overflow,
        overlap,
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
        let pa = scan_page(p, &tree.root, page_count, opts);
        if !pa.is_empty() {
            pages.push(pa);
        }
    }
    Ok(DocAnomalies { page_count, pages })
}

// ─────────────────────────────────────────────────────────────────────────
// CLI: `rhwp layout-anomaly`
// ─────────────────────────────────────────────────────────────────────────

use crate::schema_registry::ENVELOPE_SCHEMA_VERSION as SCHEMA_VERSION;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
/// `--strict` 가 확정 신호(overflow·overlap)를 하나라도 찾았을 때 내는 코드.
/// `render_geom_diff::EXIT_REGRESSION` 과 같은 값 — "검출은 도구의 정상 동작"
/// 이라는 같은 계약이다.
const EXIT_ANOMALY: i32 = 3;

struct CliOptions {
    path: std::path::PathBuf,
    page: Option<u32>,
    json: bool,
    strict: bool,
    anomaly_opts: AnomalyOptions,
}

fn parse_cli(args: &[String]) -> Result<CliOptions, String> {
    let mut path: Option<std::path::PathBuf> = None;
    let mut page = None;
    let mut json = false;
    let mut strict = false;
    let mut anomaly_opts = AnomalyOptions::default();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--strict" => strict = true,
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
            other if other.starts_with('-') => return Err(format!("알 수 없는 옵션: {other}")),
            other => {
                if path.replace(std::path::PathBuf::from(other)).is_some() {
                    return Err("입력 파일은 하나만 지정할 수 있습니다".into());
                }
            }
        }
        i += 1;
    }

    let path = path.ok_or_else(|| {
        "사용법: rhwp layout-anomaly <파일.hwp|파일.hwpx> [-p N] [--json] [--strict] \
         [--overflow-tolerance PX] [--overlap-tolerance PX]"
            .to_string()
    })?;
    Ok(CliOptions {
        path,
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
        "overlap": p.overlap.iter().map(overlap_json).collect::<Vec<_>>(),
        "emptyPage": p.empty_page.is_some(),
    })
}

fn envelope(source: &str, doc: &DocAnomalies, opts: &CliOptions) -> Value {
    let pages: Vec<Value> = doc
        .pages
        .iter()
        .filter(|p| opts.page.is_none_or(|want| p.page == want))
        .map(page_json)
        .collect();
    crate::provenance::marked(
        json!({
            "schemaVersion": SCHEMA_VERSION,
            "source": source,
            "pageCount": doc.page_count,
            "pageFilter": opts.page,
            "overflowTolerancePx": opts.anomaly_opts.overflow_tolerance_px,
            "overlapTolerancePx": opts.anomaly_opts.overlap_tolerance_px,
            "strict": opts.strict,
            "overflowCount": doc.overflow_count(),
            "overlapCount": doc.overlap_count(),
            "emptyPageCount": doc.empty_page_count(),
            "hasSignal": doc.has_signal(),
            "pages": pages,
        }),
        "layout-anomaly",
    )
}

fn run_single(opts: &CliOptions) -> i32 {
    let data = match std::fs::read(&opts.path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일 읽기 실패 {}: {e}", opts.path.display());
            return EXIT_RUNTIME;
        }
    };
    let core = match DocumentCore::from_bytes(&data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("오류: 문서 로드 실패 {}: {e:?}", opts.path.display());
            return EXIT_RUNTIME;
        }
    };
    let doc = match scan_document(&core, &opts.anomaly_opts) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 렌더 트리 생성 실패 - {e:?}");
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

    if opts.json {
        println!("{}", envelope(&opts.path.display().to_string(), &doc, opts));
        return if opts.strict && doc.has_signal() {
            EXIT_ANOMALY
        } else {
            EXIT_OK
        };
    }

    let shown: Vec<&PageAnomalies> = doc
        .pages
        .iter()
        .filter(|p| opts.page.is_none_or(|want| p.page == want))
        .collect();

    println!(
        "쪽 수: {}  overflow: {}  overlap: {}  empty_page(가능성): {}",
        doc.page_count,
        doc.overflow_count(),
        doc.overlap_count(),
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
        for o in &p.overlap {
            println!(
                "  [OVERLAP]  page {:>3}  {:.2}x{:.2}px  {} ({}) x {} ({})",
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

/// `rhwp layout-anomaly` 진입점.
pub fn run(args: &[String]) -> i32 {
    let opts = match parse_cli(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_USAGE;
        }
    };
    run_single(&opts)
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
                display_text: None,
            }),
            BoundingBox::new(0.0, 0.0, 10.0, 10.0),
        )
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
        let t = table(0.0, 0.0, 200.0, 50.0, vec![]);
        let body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 300.0), vec![t]);
        let root = page_root(100.0, 300.0, body);
        let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
        assert_eq!(pa.overflow.len(), 1);
        assert_eq!(pa.overflow[0].node_type, "Table");
        assert!((pa.overflow[0].over_right - 100.0).abs() < 1e-9);
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
                model_cell_index: None,
            }),
            BoundingBox::new(0.0, 0.0, 30.0, 10.0),
        );
        let mut cell = cell;
        cell.children.push(cell_line);
        let t = table(0.0, 0.0, 200.0, 50.0, vec![cell]);
        let body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 300.0), vec![t]);
        let root = page_root(100.0, 300.0, body);
        let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
        // 표 자신만 한 번 보고 — 내부 줄이 별도로 다시 잡히지 않는다.
        assert_eq!(pa.overflow.len(), 1);
        assert_eq!(pa.overflow[0].node_type, "Table");
    }

    #[test]
    fn two_overlapping_lines_are_flagged() {
        let mut line_a = text_line(10.0, 10.0, 50.0, 12.0);
        line_a.children.push(text_run("a"));
        let mut line_b = text_line(15.0, 12.0, 50.0, 12.0); // line_a 와 상당 부분 겹침
        line_b.children.push(text_run("b"));
        let body = body_node(
            BoundingBox::new(0.0, 0.0, 200.0, 300.0),
            vec![line_a, line_b],
        );
        let root = page_root(200.0, 300.0, body);
        let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
        assert_eq!(pa.overlap.len(), 1);
        assert!(pa.has_signal());
    }

    #[test]
    fn adjacent_non_overlapping_lines_are_clean() {
        let mut line_a = text_line(10.0, 10.0, 50.0, 12.0);
        line_a.children.push(text_run("a"));
        let mut line_b = text_line(10.0, 22.0, 50.0, 12.0); // 바로 아래로 이어짐, 안 겹침
        line_b.children.push(text_run("b"));
        let body = body_node(
            BoundingBox::new(0.0, 0.0, 200.0, 300.0),
            vec![line_a, line_b],
        );
        let root = page_root(200.0, 300.0, body);
        let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
        assert!(pa.overlap.is_empty());
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

    #[test]
    fn doc_anomalies_omit_clean_pages() {
        let clean_body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 200.0), vec![]);
        // 페이지 0: 콘텐츠 있음(첫 쪽이라 애초에 empty 후보도 아님).
        let mut line = text_line(10.0, 10.0, 20.0, 10.0);
        line.children.push(text_run("x"));
        let mut body0 = clean_body.clone();
        body0.children.push(line);
        let pa0 = scan_page(
            0,
            &page_root(100.0, 200.0, body0),
            3,
            &AnomalyOptions::default(),
        );
        // 페이지 1: 비어 있음 (중간 쪽 → possible 신호).
        let pa1 = scan_page(
            1,
            &page_root(100.0, 200.0, clean_body.clone()),
            3,
            &AnomalyOptions::default(),
        );
        // 페이지 2: 콘텐츠 있음, 깨끗.
        let mut body2 = clean_body;
        let mut line2 = text_line(10.0, 10.0, 20.0, 10.0);
        line2.children.push(text_run("y"));
        body2.children.push(line2);
        let pa2 = scan_page(
            2,
            &page_root(100.0, 200.0, body2),
            3,
            &AnomalyOptions::default(),
        );

        let pages: Vec<PageAnomalies> = vec![pa0, pa1, pa2]
            .into_iter()
            .filter(|p| !p.is_empty())
            .collect();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].page, 1);
    }
}
