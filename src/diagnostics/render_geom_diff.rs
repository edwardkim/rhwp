//! 렌더 기하(geometry) 비교 — P0-1 시각 정합성 게이트 코어.
//!
//! 두 문서의 페이지별 `RenderNode` bbox(px)를 구조 경로 기준으로 매칭하여, 라운드트립
//! (parse→serialize→reparse)이 유발한 **시각 변위**를 정량화한다. 폰트 래스터화에 의존하지
//! 않는 결정론적 기하 비교이므로 1차 게이트 지표로 사용한다.
//!
//! 범위 한정: 본 비교는 "rhwp가 그린 원본 IR" vs "rhwp가 그린 라운드트립 IR"의 **내부
//! 정합성(회귀 방지)**만 본다. 한컴 정답지 충실도와는 별개다.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::{json, Value};

use crate::provenance;

use crate::document_core::DocumentCore;
use crate::renderer::render_tree::{RenderNode, RenderNodeType};
use crate::HwpError;

/// 라운드트립 경유 포맷.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Via {
    /// 원본 IR → HWPX 직렬화 → 재로드 (hwp 레거시 → hwpx 전환 시각 보존 검증).
    Hwpx,
    /// 원본 IR → HWP 직렬화(어댑터) → 재로드.
    Hwp,
}

/// 한 노드의 bbox 변위.
#[derive(Debug, Clone)]
pub struct NodeDelta {
    /// 구조 경로 (예: `Page/Body/Column0/TextLine2/TextRun0`).
    pub path: String,
    /// 노드 타입 문자열.
    pub node_type: &'static str,
    pub dx: f64,
    pub dy: f64,
    pub dw: f64,
    pub dh: f64,
}

impl NodeDelta {
    /// 변위 크기 = max(|dx|,|dy|,|dw|,|dh|).
    pub fn disp(&self) -> f64 {
        self.dx
            .abs()
            .max(self.dy.abs())
            .max(self.dw.abs())
            .max(self.dh.abs())
    }
}

/// 한 노드 타입의 개수 증감 (구조 불일치 원인 국소화용).
#[derive(Debug, Clone)]
pub struct TypeDelta {
    pub node_type: &'static str,
    pub count_a: usize,
    pub count_b: usize,
}

impl TypeDelta {
    /// 순증감 (b - a). 음수 = 라운드트립에서 손실, 양수 = 추가.
    pub fn net(&self) -> i64 {
        self.count_b as i64 - self.count_a as i64
    }
}

/// 한 페이지의 기하 차이.
#[derive(Debug, Clone)]
pub struct PageGeomDiff {
    pub page: u32,
    pub node_count_a: usize,
    pub node_count_b: usize,
    /// 구조 경로가 어긋난 지점이 있으면 true (변위가 아닌 구조 불일치).
    pub structure_mismatch: bool,
    /// 대응 노드 변위의 최대값(px).
    pub max_disp: f64,
    /// 대응 노드 변위의 평균값(px).
    pub mean_disp: f64,
    /// 변위 큰 순 상위 노드 (보고용, 최대 `TOP_DELTAS`개).
    pub top_deltas: Vec<NodeDelta>,
    /// 노드 타입별 개수 증감 (개수가 다른 타입만). 구조 불일치 원인 국소화용.
    pub type_deltas: Vec<TypeDelta>,
    /// 구조 불일치가 TextRun ±1 (삽입·삭제 각 최대 1개, 다른 타입 없음) 뿐인지.
    /// 레거시 무-컨트롤문자 구역정의 정규화가 유발하는 경미 조성 노이즈 (#1773) —
    /// 게이트에서 WARN_TEXTRUN 으로 분리한다.
    pub struct_textrun_pm1: bool,
}

/// 문서 전체 기하 차이.
#[derive(Debug, Clone)]
pub struct DocGeomDiff {
    pub page_count_a: u32,
    pub page_count_b: u32,
    pub pages: Vec<PageGeomDiff>,
    /// 전 페이지 최대 변위(px).
    pub max_disp: f64,
}

impl DocGeomDiff {
    /// 페이지 수가 달라졌는지 (시각 회귀의 가장 명백한 신호 → 하드 실패).
    pub fn page_count_mismatch(&self) -> bool {
        self.page_count_a != self.page_count_b
    }

    /// 구조 불일치 페이지가 하나라도 있는지.
    pub fn any_structure_mismatch(&self) -> bool {
        self.pages.iter().any(|p| p.structure_mismatch)
    }
}

/// 보고용으로 보관하는 페이지당 상위 변위 노드 수.
const TOP_DELTAS: usize = 20;

/// 노드 타입을 안정 문자열로 매핑 (render_tree write_json 과 동일 규약).
fn node_type_str(t: &RenderNodeType) -> &'static str {
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

fn count_nodes(n: &RenderNode) -> usize {
    1 + n.children.iter().map(count_nodes).sum::<usize>()
}

fn q100(v: f64) -> i64 {
    (v * 100.0).round() as i64
}

/// [#1993] TextLine 의 자식 TextRun 들을 x-구간 합집합(병합 인터벌)으로 요약한다.
/// 라운드트립이 같은 줄의 동일 텍스트를 **다른 경계로 재분할**(char offset/run 조성 변화)해도
/// 커버리지(합집합)는 불변이므로, 재분할은 동일 서명을 낸다. 반대로 run 추가/삭제·이동 등
/// **실제 텍스트 변화는 커버리지를 바꿔** 구분된다(예: 83254 run±1 = 844px 실차이).
fn line_run_coverage(line: &RenderNode) -> Vec<(i64, i64)> {
    let mut ivs: Vec<(i64, i64)> = line
        .children
        .iter()
        .filter(|c| node_type_str(&c.node_type) == "TextRun")
        .map(|c| (q100(c.bbox.x), q100(c.bbox.x + c.bbox.width)))
        .collect();
    ivs.sort_unstable();
    let mut merged: Vec<(i64, i64)> = Vec::new();
    for (s, e) in ivs {
        if let Some(last) = merged.last_mut() {
            if s <= last.1 {
                last.1 = last.1.max(e);
                continue;
            }
        }
        merged.push((s, e));
    }
    merged
}

/// [#1993] 트리 노드를 위치 서명 문자열 멀티셋으로 평탄화(정렬)한다. 두 트리의 이 멀티셋이
/// 동일하면 모든 구조 노드가 같은 위치·같은 줄 run 커버리지를 가지므로 시각적으로 동일하다.
/// 개별 TextRun 은 서명에서 제외하고(재분할 노이즈 회피), 부모 TextLine 서명에 run 커버리지를
/// 포함해 실제 텍스트 변화만 잡는다. 계층 LCS 매칭이 순서 뒤바뀐 동일-위치 노드를 오짝지어
/// 내던 허위 변위(별지 370px, hwp3 변환 리오더)를 원천 차단하면서 실회귀는 보존한다.
fn flatten_pos_set(n: &RenderNode) -> Vec<String> {
    let mut out = Vec::new();
    fn rec(n: &RenderNode, out: &mut Vec<String>) {
        let t = node_type_str(&n.node_type);
        if t != "TextRun" {
            let mut sig = format!(
                "{t}|{}|{}|{}|{}",
                q100(n.bbox.x),
                q100(n.bbox.y),
                q100(n.bbox.width),
                q100(n.bbox.height)
            );
            if t == "TextLine" {
                sig.push_str(&format!("|cov{:?}", line_run_coverage(n)));
            }
            out.push(sig);
        }
        for c in &n.children {
            rec(c, out);
        }
    }
    rec(n, &mut out);
    out.sort_unstable();
    out
}

/// 노드 타입별 개수 히스토그램 (재귀).
fn type_histogram(n: &RenderNode, acc: &mut std::collections::BTreeMap<&'static str, usize>) {
    *acc.entry(node_type_str(&n.node_type)).or_insert(0) += 1;
    for c in &n.children {
        type_histogram(c, acc);
    }
}

/// 두 트리의 타입 히스토그램 차이 — 개수가 다른 타입만 (타입명 정렬).
fn compute_type_deltas(root_a: &RenderNode, root_b: &RenderNode) -> Vec<TypeDelta> {
    let mut ha = std::collections::BTreeMap::new();
    let mut hb = std::collections::BTreeMap::new();
    type_histogram(root_a, &mut ha);
    type_histogram(root_b, &mut hb);
    let keys: std::collections::BTreeSet<&'static str> =
        ha.keys().chain(hb.keys()).copied().collect();
    keys.into_iter()
        .filter_map(|k| {
            let a = ha.get(k).copied().unwrap_or(0);
            let b = hb.get(k).copied().unwrap_or(0);
            (a != b).then_some(TypeDelta {
                node_type: k,
                count_a: a,
                count_b: b,
            })
        })
        .collect()
}

/// 페이지 변위 누적기.
struct PageAccum {
    sum_disp: f64,
    count: usize,
    max_disp: f64,
    structure_mismatch: bool,
    /// 구조 불일치로 기록된 삽입(b 전용) 노드 타입들.
    struct_inserts: Vec<&'static str>,
    /// 구조 불일치로 기록된 삭제(a 전용) 노드 타입들.
    struct_deletes: Vec<&'static str>,
    top: Vec<NodeDelta>,
}

impl PageAccum {
    fn new() -> Self {
        Self {
            sum_disp: 0.0,
            count: 0,
            max_disp: 0.0,
            structure_mismatch: false,
            struct_inserts: Vec::new(),
            struct_deletes: Vec::new(),
            top: Vec::new(),
        }
    }

    fn record(&mut self, delta: NodeDelta) {
        let d = delta.disp();
        self.sum_disp += d;
        self.count += 1;
        if d > self.max_disp {
            self.max_disp = d;
        }
        self.top.push(delta);
    }

    fn finish_top(&mut self) {
        self.top.sort_by(|a, b| {
            b.disp()
                .partial_cmp(&a.disp())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.top.truncate(TOP_DELTAS);
    }
}

/// 두 타입 시퀀스를 LCS(최장공통부분수열)로 정렬한다.
/// 반환: `(Some(i), Some(j))` = 대응, `(Some(i), None)` = a 전용(삭제),
/// `(None, Some(j))` = b 전용(삽입). 입력 순서를 보존한다.
fn lcs_align(sa: &[&str], sb: &[&str]) -> Vec<(Option<usize>, Option<usize>)> {
    let n = sa.len();
    let m = sb.len();
    // dp[i][j] = sa[i..], sb[j..] 의 LCS 길이.
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if sa[i] == sb[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let mut out = Vec::with_capacity(n.max(m));
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if sa[i] == sb[j] {
            out.push((Some(i), Some(j)));
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            out.push((Some(i), None));
            i += 1;
        } else {
            out.push((None, Some(j)));
            j += 1;
        }
    }
    while i < n {
        out.push((Some(i), None));
        i += 1;
    }
    while j < m {
        out.push((None, Some(j)));
        j += 1;
    }
    out
}

/// 타입이 이미 일치한다고 가정하고 두 노드를 비교한다. 자식은 타입 LCS 로 정렬하여
/// 삽입/삭제가 있어도 대응 노드의 변위를 계속 측정한다(구조 불일치는 별도 플래그).
fn walk(a: &RenderNode, b: &RenderNode, path: &str, acc: &mut PageAccum) {
    let ta = node_type_str(&a.node_type);
    acc.record(NodeDelta {
        path: path.to_string(),
        node_type: ta,
        dx: b.bbox.x - a.bbox.x,
        dy: b.bbox.y - a.bbox.y,
        dw: b.bbox.width - a.bbox.width,
        dh: b.bbox.height - a.bbox.height,
    });

    let sa: Vec<&str> = a
        .children
        .iter()
        .map(|c| node_type_str(&c.node_type))
        .collect();
    let sb: Vec<&str> = b
        .children
        .iter()
        .map(|c| node_type_str(&c.node_type))
        .collect();
    for (ia, jb) in lcs_align(&sa, &sb) {
        match (ia, jb) {
            (Some(i), Some(j)) => {
                let child_path = format!("{path}/{}{i}", sa[i]);
                walk(&a.children[i], &b.children[j], &child_path, acc);
            }
            // 한쪽에만 있는 노드 = 삽입/삭제 → 구조 불일치(변위 아님).
            (Some(i), None) => {
                acc.structure_mismatch = true;
                acc.struct_deletes.push(sa[i]);
            }
            (None, Some(j)) => {
                acc.structure_mismatch = true;
                acc.struct_inserts.push(sb[j]);
            }
            (None, None) => unreachable!("lcs_align 은 (None, None) 을 내지 않는다"),
        }
    }
}

/// 구조 불일치 이벤트가 "TextRun ±1" 뿐인지 — 삽입·삭제 모두 TextRun 타입이고
/// 각각 최대 1개일 때만 true. 컨트롤문자-무 구역정의 정규화가 유발하는 run 조성
/// 차이(#1773)를 하드 실패에서 분리하기 위한 판별.
fn is_textrun_pm1(inserts: &[&'static str], deletes: &[&'static str]) -> bool {
    let only_textrun = inserts
        .iter()
        .chain(deletes.iter())
        .all(|t| *t == "TextRun");
    (!inserts.is_empty() || !deletes.is_empty())
        && only_textrun
        && inserts.len() <= 1
        && deletes.len() <= 1
}

/// 한 페이지의 루트 노드 쌍을 비교한다.
pub fn diff_page(page: u32, root_a: &RenderNode, root_b: &RenderNode) -> PageGeomDiff {
    let node_count_a = count_nodes(root_a);
    let node_count_b = count_nodes(root_b);

    // [#1993] 위치 집합이 완전히 동일하면(픽셀 동일, 순서/계층만 상이) 변위·구조
    // 불일치 없음으로 단락. 계층 LCS 매칭이 순서 뒤바뀐 동일-위치 노드를 오짝지어
    // 내던 허위 변위(별지 370px, hwp3 변환 6건 등)를 제거한다.
    if flatten_pos_set(root_a) == flatten_pos_set(root_b) {
        return PageGeomDiff {
            page,
            node_count_a,
            node_count_b,
            structure_mismatch: false,
            max_disp: 0.0,
            mean_disp: 0.0,
            top_deltas: Vec::new(),
            type_deltas: Vec::new(),
            struct_textrun_pm1: false,
        };
    }

    let mut acc = PageAccum::new();

    let ta = node_type_str(&root_a.node_type);
    let tb = node_type_str(&root_b.node_type);
    if ta != tb {
        acc.structure_mismatch = true;
    } else {
        walk(root_a, root_b, ta, &mut acc);
    }
    acc.finish_top();

    let mean_disp = if acc.count > 0 {
        acc.sum_disp / acc.count as f64
    } else {
        0.0
    };
    let struct_textrun_pm1 =
        acc.structure_mismatch && is_textrun_pm1(&acc.struct_inserts, &acc.struct_deletes);
    PageGeomDiff {
        page,
        node_count_a,
        node_count_b,
        structure_mismatch: acc.structure_mismatch,
        max_disp: acc.max_disp,
        mean_disp,
        top_deltas: acc.top,
        type_deltas: compute_type_deltas(root_a, root_b),
        struct_textrun_pm1,
    }
}

/// 두 문서의 페이지별 렌더 기하를 비교한다. 공통 페이지 수까지만 노드 비교하고,
/// 페이지 수 차이는 `page_count_a`/`page_count_b` 로 별도 보존한다.
pub fn diff_render_geometry(a: &DocumentCore, b: &DocumentCore) -> Result<DocGeomDiff, HwpError> {
    let pca = a.page_count();
    let pcb = b.page_count();
    let n = pca.min(pcb);
    let mut pages = Vec::with_capacity(n as usize);
    let mut max_disp = 0.0_f64;
    for p in 0..n {
        let ta = a.build_page_render_tree(p)?;
        let tb = b.build_page_render_tree(p)?;
        let pg = diff_page(p, &ta.root, &tb.root);
        if pg.max_disp > max_disp {
            max_disp = pg.max_disp;
        }
        pages.push(pg);
    }
    Ok(DocGeomDiff {
        page_count_a: pca,
        page_count_b: pcb,
        pages,
        max_disp,
    })
}

/// 원본 바이트를 라운드트립(직렬화→재로드)한 뒤 원본과 기하를 비교한다.
pub fn roundtrip_geom(data: &[u8], via: Via) -> Result<DocGeomDiff, HwpError> {
    let core_a = DocumentCore::from_bytes(data)?;
    let rt_bytes = match via {
        Via::Hwpx => core_a.export_hwpx_native()?,
        Via::Hwp => {
            let mut c = DocumentCore::from_bytes(data)?;
            c.export_hwp_with_adapter()?
        }
    };
    let core_b = DocumentCore::from_bytes(&rt_bytes)?;
    diff_render_geometry(&core_a, &core_b)
}

// ─────────────────────────────────────────────────────────────────────────
// CLI: `rhwp render-diff` (P0-1 2단계)
// ─────────────────────────────────────────────────────────────────────────

/// 기본 변위 임계값(px). Stage 3 에서 실측 분포로 보정한다.
const DEFAULT_MAX_DISP: f64 = 1.0;

/// `--json` 봉투 스키마 버전 — 다른 명령들과 같은 봉투 축이므로 단일
/// 출처([`crate::schema_registry`], #4329)를 별칭으로 쓴다. 필드 추가는 허용,
/// 변경·삭제는 범프한다는 규약도 그쪽이 canonical 이다.
use crate::schema_registry::ENVELOPE_SCHEMA_VERSION as SCHEMA_VERSION;

// 종료 코드 — #2707 사전. 회귀 **검출**(도구는 정상 동작)과 런타임 **실패**를 같은
// 코드로 뭉개면 CI 는 "렌더가 깨졌다"와 "파일을 못 읽었다"를 구분할 수 없다.
const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
/// 검증 단언 실패 — `--verify` 계열과 같은 의미론. `--json` 모드에서만 낸다.
const EXIT_REGRESSION: i32 = 3;

struct CliOptions {
    /// 위치 인자(파일/폴더). 배치는 1개(폴더), 자기 라운드트립 1개, 두 파일 비교 2개.
    positionals: Vec<PathBuf>,
    batch: bool,
    via: Via,
    page: Option<u32>,
    max_disp: f64,
    out_dir: PathBuf,
    /// 기계 계약 모드 — stdout 은 단건 JSON 봉투(배치는 NDJSON)만 싣는다.
    json: bool,
}

fn parse_cli(args: &[String]) -> Result<CliOptions, String> {
    let mut positionals = Vec::new();
    let mut batch = false;
    let mut via = Via::Hwpx;
    let mut page = None;
    let mut max_disp = DEFAULT_MAX_DISP;
    let mut out_dir = PathBuf::from("output/poc/render_diff");
    let mut json_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--batch" => batch = true,
            "--json" => json_mode = true,
            "--via" => {
                i += 1;
                match args.get(i).map(|s| s.as_str()) {
                    Some("hwpx") => via = Via::Hwpx,
                    Some("hwp") => via = Via::Hwp,
                    other => return Err(format!("--via 값은 hwpx|hwp (받음: {other:?})")),
                }
            }
            "-p" | "--page" => {
                i += 1;
                let v = args.get(i).ok_or("-p 다음에 페이지 번호 필요")?;
                page = Some(
                    v.parse()
                        .map_err(|_| format!("페이지 번호 파싱 실패: {v}"))?,
                );
            }
            "--max-disp" => {
                i += 1;
                let v = args.get(i).ok_or("--max-disp 다음에 px 값 필요")?;
                max_disp = v.parse().map_err(|_| format!("max-disp 파싱 실패: {v}"))?;
            }
            "-o" | "--out" => {
                i += 1;
                out_dir = PathBuf::from(args.get(i).ok_or("-o 다음에 출력 폴더 필요")?);
            }
            other if other.starts_with('-') => return Err(format!("알 수 없는 옵션: {other}")),
            other => positionals.push(PathBuf::from(other)),
        }
        i += 1;
    }

    if positionals.is_empty() {
        return Err("사용법: rhwp render-diff <파일> [--via hwpx|hwp] [-p N] [--max-disp PX] [--json]\n         rhwp render-diff <a> <b> [-p N] [--json]\n         rhwp render-diff --batch <폴더> [--via hwpx] [-o 출력폴더] [--json]".into());
    }
    if batch && positionals.len() != 1 {
        return Err("--batch 는 폴더 1개만 지정".into());
    }
    if !batch && positionals.len() > 2 {
        return Err("위치 인자는 최대 2개(두 파일 비교)".into());
    }
    Ok(CliOptions {
        positionals,
        batch,
        via,
        page,
        max_disp,
        out_dir,
        json: json_mode,
    })
}

/// 라운드트립 경유 포맷의 안정 문자열 (봉투 `via` 값).
fn via_str(via: Via) -> &'static str {
    match via {
        Via::Hwpx => "hwpx",
        Via::Hwp => "hwp",
    }
}

/// 유한하지 않은 변위는 JSON `null` 로 낸다.
///
/// NaN·무한대를 0 이나 문자열로 위장시키면 소비자는 "차이 없음"으로 오독한다. 측정
/// 불가는 값이 아니라 값의 **부재**이므로 null 이 유일하게 정직한 표현이다.
fn fnum(v: f64) -> Value {
    if v.is_finite() {
        json!(v)
    } else {
        Value::Null
    }
}

/// `Option<u32>` → 숫자 또는 null.
fn opt_u32(v: Option<u32>) -> Value {
    v.map(Value::from).unwrap_or(Value::Null)
}

/// 페이지 한 장의 봉투 조각.
fn page_json(p: &PageGeomDiff) -> Value {
    let top: Vec<Value> = p
        .top_deltas
        .iter()
        .map(|d| {
            json!({
                "path": d.path,
                "nodeType": d.node_type,
                "disp": fnum(d.disp()),
                "dx": fnum(d.dx),
                "dy": fnum(d.dy),
                "dw": fnum(d.dw),
                "dh": fnum(d.dh),
            })
        })
        .collect();
    let types: Vec<Value> = p
        .type_deltas
        .iter()
        .map(|t| {
            json!({
                "nodeType": t.node_type,
                "countA": t.count_a,
                "countB": t.count_b,
                "net": t.net(),
            })
        })
        .collect();
    json!({
        "page": p.page,
        "nodeCountA": p.node_count_a,
        "nodeCountB": p.node_count_b,
        "maxDisp": fnum(p.max_disp),
        "meanDisp": fnum(p.mean_disp),
        "structureMismatch": p.structure_mismatch,
        "structTextrunPm1": p.struct_textrun_pm1,
        "topDeltas": top,
        "typeDeltas": types,
    })
}

/// 단건(자기 라운드트립·두 파일) 비교의 `--json` 봉투.
fn single_envelope(
    opts: &CliOptions,
    diff: &DocGeomDiff,
    sum: &DiffSummary,
    status: &str,
) -> Value {
    let pair = opts.positionals.len() == 2;
    let pages: Vec<Value> = filtered_pages(diff, opts.page)
        .into_iter()
        .map(page_json)
        .collect();
    provenance::marked(
        json!({
            "schemaVersion": SCHEMA_VERSION,
            // 두 파일 비교에는 "경유 포맷"이 없다 — 라운드트립 축과 섞이지 않게 mode 로 가른다.
            "mode": if pair { "pair" } else { "roundtrip" },
            "sourceA": opts.positionals[0].display().to_string(),
            "sourceB": if pair {
                json!(opts.positionals[1].display().to_string())
            } else {
                Value::Null
            },
            "via": if pair { Value::Null } else { json!(via_str(opts.via)) },
            "pageFilter": opt_u32(opts.page),
            "threshold": fnum(opts.max_disp),
            "pageCountA": diff.page_count_a,
            "pageCountB": diff.page_count_b,
            "pageCountMismatch": diff.page_count_mismatch(),
            "maxDisp": fnum(sum.max_disp),
            "worstPage": opt_u32(sum.worst_page),
            "overPages": sum.over_pages,
            "structPages": sum.struct_pages,
            "hardStructPages": sum.hard_struct_pages,
            "status": status,
            "regression": status_is_hard_failure(status),
            "pages": pages,
        }),
        "render-diff",
    )
}

/// 페이지 목록을 `page` 필터로 좁힌 뷰를 만든다(소유 복제 회피용 참조 벡터).
fn filtered_pages<'a>(diff: &'a DocGeomDiff, page: Option<u32>) -> Vec<&'a PageGeomDiff> {
    diff.pages
        .iter()
        .filter(|p| page.is_none_or(|want| p.page == want))
        .collect()
}

/// 한 문서 비교의 요약 집계.
struct DiffSummary {
    worst_page: Option<u32>,
    max_disp: f64,
    struct_pages: usize,
    /// TextRun ±1 로 설명되지 않는 구조 불일치 페이지 수 (하드 실패 기준).
    hard_struct_pages: usize,
    over_pages: usize,
}

fn summarize(diff: &DocGeomDiff, page: Option<u32>, threshold: f64) -> DiffSummary {
    let pages = filtered_pages(diff, page);
    let mut worst_page = None;
    let mut max_disp = 0.0_f64;
    let mut struct_pages = 0;
    let mut hard_struct_pages = 0;
    let mut over_pages = 0;
    for p in &pages {
        if p.structure_mismatch {
            struct_pages += 1;
            if !p.struct_textrun_pm1 {
                hard_struct_pages += 1;
            }
        }
        if p.max_disp > threshold {
            over_pages += 1;
        }
        if p.max_disp > max_disp {
            max_disp = p.max_disp;
            worst_page = Some(p.page);
        }
    }
    DiffSummary {
        worst_page,
        max_disp,
        struct_pages,
        hard_struct_pages,
        over_pages,
    }
}

fn status_str(diff: &DocGeomDiff, sum: &DiffSummary, threshold: f64) -> &'static str {
    if diff.page_count_mismatch() {
        "PAGE_MISMATCH"
    } else if sum.hard_struct_pages > 0 {
        "STRUCT_MISMATCH"
    } else if sum.max_disp > threshold {
        "OVER"
    } else if sum.struct_pages > 0 {
        // 구조 차이가 TextRun ±1 뿐이고 변위도 임계 이내 — 경미 조성 노이즈 (#1773).
        "WARN_TEXTRUN"
    } else {
        "PASS"
    }
}

fn status_is_hard_failure(status: &str) -> bool {
    !matches!(status, "PASS" | "WARN_TEXTRUN")
}

/// `.hwp`/`.hwpx` 파일을 재귀 수집(정렬).
fn collect_doc_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries =
            fs::read_dir(&dir).map_err(|e| format!("폴더 읽기 실패 {}: {e}", dir.display()))?;
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
    files.sort();
    Ok(files)
}

/// 배치 1건 결과 행.
struct BatchRow {
    rel_path: String,
    status: String,
    pages_a: u32,
    pages_b: u32,
    max_disp: f64,
    worst_page: Option<u32>,
    struct_pages: usize,
    /// TextRun ±1 로 설명되지 않는 구조 불일치 페이지 수 (봉투 `hardStructPages`).
    hard_struct_pages: usize,
    over_pages: usize,
    /// 파일 전체에 걸친 노드 타입별 순증감 (구조 불일치 원인 국소화).
    struct_delta: String,
    elapsed_ms: u128,
    error: String,
}

/// 배치 NDJSON 한 줄.
///
/// 로드 실패도 `error` 를 단 레코드로 남긴다 — 스트림에서 통째로 사라지면 소비자는
/// 처리 누락 자체를 알 수 없다(입력 N건, 출력 N-1건인데 아무도 실패를 보고하지 않는다).
/// `error` 키의 **존재**가 실패 판정이라 성공 레코드에는 넣지 않는다(batch 축 규약과 동일).
fn batch_record(opts: &CliOptions, row: &BatchRow) -> Value {
    let mut rec = json!({
        "schemaVersion": SCHEMA_VERSION,
        "mode": "roundtrip",
        "source": row.rel_path,
        "via": via_str(opts.via),
        "threshold": fnum(opts.max_disp),
        "pageCountA": row.pages_a,
        "pageCountB": row.pages_b,
        "pageCountMismatch": row.pages_a != row.pages_b,
        "maxDisp": fnum(row.max_disp),
        "worstPage": opt_u32(row.worst_page),
        "overPages": row.over_pages,
        "structPages": row.struct_pages,
        "hardStructPages": row.hard_struct_pages,
        "status": row.status,
        // 로드 실패는 "회귀 검출"이 아니라 "측정 실패"다 — 두 축을 겹치지 않게 가른다.
        "regression": row.error.is_empty() && status_is_hard_failure(&row.status),
        "structDelta": row.struct_delta,
        "elapsedMs": row.elapsed_ms as u64,
    });
    if !row.error.is_empty() {
        rec["error"] = json!(row.error);
    }
    provenance::marked(rec, "render-diff")
}

/// 문서 전 페이지의 타입 델타를 타입별 순증감으로 집계 (예: `Line:-4;RawSvg:-1`).
fn aggregate_struct_delta(diff: &DocGeomDiff) -> String {
    let mut net: std::collections::BTreeMap<&'static str, i64> = std::collections::BTreeMap::new();
    for p in &diff.pages {
        for d in &p.type_deltas {
            *net.entry(d.node_type).or_insert(0) += d.net();
        }
    }
    net.iter()
        .filter(|(_, v)| **v != 0)
        .map(|(t, v)| format!("{t}:{v:+}"))
        .collect::<Vec<_>>()
        .join(";")
}

fn run_batch(opts: &CliOptions) -> i32 {
    let root = &opts.positionals[0];
    let files = match collect_doc_files(root) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_USAGE;
        }
    };
    if files.is_empty() {
        eprintln!(
            "오류: 처리할 .hwp/.hwpx 파일이 없습니다: {}",
            root.display()
        );
        return EXIT_USAGE;
    }

    let mut rows = Vec::with_capacity(files.len());
    for path in &files {
        let rel = path
            .strip_prefix(root)
            .map(|r| r.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string_lossy().to_string());
        let started = Instant::now();
        let mut row = BatchRow {
            rel_path: rel.clone(),
            status: "LOAD_FAIL".into(),
            pages_a: 0,
            pages_b: 0,
            max_disp: 0.0,
            worst_page: None,
            struct_pages: 0,
            hard_struct_pages: 0,
            over_pages: 0,
            struct_delta: String::new(),
            elapsed_ms: 0,
            error: String::new(),
        };
        match fs::read(path)
            .map_err(|e| e.to_string())
            .and_then(|data| roundtrip_geom(&data, opts.via).map_err(|e| format!("{e:?}")))
        {
            Ok(diff) => {
                let sum = summarize(&diff, None, opts.max_disp);
                row.status = status_str(&diff, &sum, opts.max_disp).to_string();
                row.pages_a = diff.page_count_a;
                row.pages_b = diff.page_count_b;
                row.max_disp = sum.max_disp;
                row.worst_page = sum.worst_page;
                row.struct_pages = sum.struct_pages;
                row.hard_struct_pages = sum.hard_struct_pages;
                row.over_pages = sum.over_pages;
                row.struct_delta = aggregate_struct_delta(&diff);
            }
            Err(e) => row.error = e,
        }
        row.elapsed_ms = started.elapsed().as_millis();
        if opts.json {
            // 한 줄 한 레코드 — 소비자가 전건 완료를 기다리지 않고 흘려 읽는다.
            println!("{}", batch_record(opts, &row));
        } else {
            println!(
                "[{:>15}] max_disp={:>7.2} struct={} over={} {:>6}ms  {}{}",
                row.status,
                row.max_disp,
                row.struct_pages,
                row.over_pages,
                row.elapsed_ms,
                row.rel_path,
                if row.struct_delta.is_empty() {
                    String::new()
                } else {
                    format!("  [{}]", row.struct_delta)
                }
            );
            if !row.error.is_empty() {
                println!("                  └ {}", row.error);
            }
        }
        rows.push(row);
    }

    if let Err(e) = write_batch_tsv(&opts.out_dir, &rows, opts.json) {
        eprintln!("오류: {e}");
        return EXIT_RUNTIME;
    }
    print_batch_summary(&rows, opts.json);

    if opts.json {
        // 측정 실패(로드)와 회귀 검출은 다른 사건이다. 로드 실패가 하나라도 있으면
        // "전건을 재봤다"고 말할 수 없으므로 런타임 실패(1)가 우선한다.
        if rows.iter().any(|r| !r.error.is_empty()) {
            return EXIT_RUNTIME;
        }
        return if rows.iter().any(|r| status_is_hard_failure(&r.status)) {
            EXIT_REGRESSION
        } else {
            EXIT_OK
        };
    }

    // 사람 모드는 종전 계약 그대로 — 기존 CI 스크립트가 1 을 실패로 읽고 있다.
    let hard = rows.iter().any(|r| status_is_hard_failure(&r.status));
    if hard {
        EXIT_RUNTIME
    } else {
        EXIT_OK
    }
}

fn write_batch_tsv(out_dir: &Path, rows: &[BatchRow], json: bool) -> Result<(), String> {
    fs::create_dir_all(out_dir).map_err(|e| format!("출력 폴더 생성 실패: {e}"))?;
    let path = out_dir.join("geom_inventory.tsv");
    let mut tsv = String::from(
        "sample\tstatus\tpages_a\tpages_b\tmax_disp\tworst_page\tstruct_pages\tover_pages\telapsed_ms\terror\tstruct_delta\n",
    );
    for r in rows {
        tsv.push_str(&format!(
            "{}\t{}\t{}\t{}\t{:.3}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            r.rel_path.replace('\t', " "),
            r.status,
            r.pages_a,
            r.pages_b,
            r.max_disp,
            r.worst_page
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".into()),
            r.struct_pages,
            r.over_pages,
            r.elapsed_ms,
            r.error.replace(['\t', '\n', '\r'], " "),
            r.struct_delta.replace('\t', " "),
        ));
    }
    fs::write(&path, tsv).map_err(|e| format!("TSV 쓰기 실패: {e}"))?;
    // `--json` 의 stdout 은 NDJSON 전용이다 — 산출물 안내는 진단이라 stderr 로 뺀다.
    let note = format!("\nTSV 저장: {}", path.display());
    if json {
        eprintln!("{note}");
    } else {
        println!("{note}");
    }
    Ok(())
}

fn batch_summary_text(rows: &[BatchRow]) -> String {
    let count = |s: &str| rows.iter().filter(|r| r.status == s).count();
    let overall_max = rows.iter().map(|r| r.max_disp).fold(0.0_f64, f64::max);
    let mut out = String::from("\n=== render-diff 요약 ===\n");
    out.push_str(&format!("  총 파일         : {}\n", rows.len()));
    out.push_str(&format!("  PASS            : {}\n", count("PASS")));
    out.push_str(&format!("  WARN_TEXTRUN    : {}\n", count("WARN_TEXTRUN")));
    out.push_str(&format!("  OVER            : {}\n", count("OVER")));
    out.push_str(&format!(
        "  STRUCT_MISMATCH : {}\n",
        count("STRUCT_MISMATCH")
    ));
    out.push_str(&format!("  PAGE_MISMATCH   : {}\n", count("PAGE_MISMATCH")));
    out.push_str(&format!("  LOAD_FAIL       : {}\n", count("LOAD_FAIL")));
    out.push_str(&format!("  전체 최대 변위  : {overall_max:.2} px\n"));
    out
}

fn print_batch_summary(rows: &[BatchRow], json: bool) {
    let text = batch_summary_text(rows);
    if json {
        eprint!("{text}");
    } else {
        print!("{text}");
    }
}

/// 단일/두 파일 비교를 표준출력에 보고하고 종료 코드를 반환한다.
fn run_single(opts: &CliOptions) -> i32 {
    let diff = if opts.positionals.len() == 2 {
        // 두 파일 직접 비교.
        let (a, b) = (&opts.positionals[0], &opts.positionals[1]);
        let core_a = match fs::read(a)
            .map_err(|e| e.to_string())
            .and_then(|d| DocumentCore::from_bytes(&d).map_err(|e| format!("{e:?}")))
        {
            Ok(c) => c,
            Err(e) => {
                // 읽기·파싱 실패는 런타임 오류(1)다. 2는 인자 개수/형식이 틀린
                // 사용법 오류 전용 — 계약(cli_commands.md 종료 코드 표)상 스크립트가
                // "내 호출이 틀렸다"로 오독해 재시도 대신 인자를 고치려 든다.
                eprintln!("오류: A 로드 실패 {}: {e}", a.display());
                return EXIT_RUNTIME;
            }
        };
        let core_b = match fs::read(b)
            .map_err(|e| e.to_string())
            .and_then(|d| DocumentCore::from_bytes(&d).map_err(|e| format!("{e:?}")))
        {
            Ok(c) => c,
            Err(e) => {
                // 읽기·파싱 실패는 런타임 오류(1).
                eprintln!("오류: B 로드 실패 {}: {e}", b.display());
                return EXIT_RUNTIME;
            }
        };
        match diff_render_geometry(&core_a, &core_b) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("오류: 기하 비교 실패 - {e:?}");
                return EXIT_RUNTIME;
            }
        }
    } else {
        // 자기 라운드트립.
        let f = &opts.positionals[0];
        let data = match fs::read(f) {
            Ok(d) => d,
            Err(e) => {
                // 읽기 실패는 런타임 오류(1). 이 갈래도 같은 부류였다.
                eprintln!("오류: 파일 읽기 실패 {}: {e}", f.display());
                return EXIT_RUNTIME;
            }
        };
        match roundtrip_geom(&data, opts.via) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("오류: 라운드트립 비교 실패 - {e:?}");
                return EXIT_RUNTIME;
            }
        }
    };

    // 범위 밖 `-p` 는 사용법 오류(2)다. 조용한 빈 결과로 내면 "필터가 안 맞았다"와
    // "차이가 없다"가 같은 출력이 된다 — 정반대 결론인데 소비자는 후자로 읽는다.
    if let Some(want) = opts.page {
        if !diff.pages.iter().any(|p| p.page == want) {
            eprintln!(
                "오류: -p {want} 는 비교 범위 밖입니다 (비교 대상 페이지 0..{})",
                diff.pages.len()
            );
            return EXIT_USAGE;
        }
    }

    let sum = summarize(&diff, opts.page, opts.max_disp);
    let status = status_str(&diff, &sum, opts.max_disp);

    if opts.json {
        println!("{}", single_envelope(opts, &diff, &sum, status));
        // 회귀 **검출**은 도구의 정상 동작이다. 1(런타임 실패)로 내면 CI 는 "렌더가
        // 깨졌다"와 "파일을 못 읽었다"를 같은 신호로 받는다 — 3 으로 갈라 낸다.
        return if status_is_hard_failure(status) {
            EXIT_REGRESSION
        } else {
            EXIT_OK
        };
    }

    println!("페이지 수: A={} B={}", diff.page_count_a, diff.page_count_b);
    if diff.page_count_mismatch() {
        println!("⚠ 페이지 수 불일치 — 시각 회귀 강신호");
    }
    println!(
        "최대 변위: {:.2} px (page {})",
        sum.max_disp,
        sum.worst_page
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".into())
    );
    println!(
        "임계 초과 페이지: {} / 구조 불일치 페이지: {} (임계 {:.2}px)",
        sum.over_pages, sum.struct_pages, opts.max_disp
    );

    // 페이지별 표 (page 필터 반영).
    for p in filtered_pages(&diff, opts.page) {
        if p.max_disp > opts.max_disp || p.structure_mismatch {
            println!(
                "  page {:>3}: max={:>7.2} mean={:>6.2} nodes={}/{}{}",
                p.page,
                p.max_disp,
                p.mean_disp,
                p.node_count_a,
                p.node_count_b,
                if p.struct_textrun_pm1 {
                    "  [STRUCT:TextRun±1]"
                } else if p.structure_mismatch {
                    "  [STRUCT]"
                } else {
                    ""
                }
            );
            for d in p.top_deltas.iter().take(3) {
                println!("      {:>7.2}px  {}", d.disp(), d.path);
            }
            // 구조 불일치 원인: 노드 타입별 증감 (예: Line: 4→0  RawSvg: 1→0).
            if !p.type_deltas.is_empty() {
                let deltas = p
                    .type_deltas
                    .iter()
                    .map(|d| {
                        format!(
                            "{}: {}→{} ({:+})",
                            d.node_type,
                            d.count_a,
                            d.count_b,
                            d.net()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("  ");
                println!("      Δ {deltas}");
            }
        }
    }
    println!("status: {status}");

    // 사람 모드는 종전 계약 그대로 1 이다 — 이미 1 을 실패로 읽는 CI 스크립트가 있다.
    // 새 의미론(3)은 `--json` 소비자에게만 준다.
    if status_is_hard_failure(status) {
        EXIT_RUNTIME
    } else {
        EXIT_OK
    }
}

/// `rhwp render-diff` 진입점.
pub fn run(args: &[String]) {
    let opts = match parse_cli(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("오류: {e}");
            std::process::exit(EXIT_USAGE);
        }
    };
    let code = if opts.batch {
        run_batch(&opts)
    } else {
        run_single(&opts)
    };
    if code != EXIT_OK {
        std::process::exit(code);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

    fn page_root(children: Vec<RenderNode>) -> RenderNode {
        let mut root = RenderNode::new(
            0,
            RenderNodeType::Body { clip_rect: None },
            BoundingBox::new(0.0, 0.0, 100.0, 100.0),
        );
        root.children = children;
        root
    }

    fn text_run(x: f64, y: f64, w: f64, h: f64) -> RenderNode {
        RenderNode::new(
            1,
            RenderNodeType::TextBox, // 타입 안정성만 필요 — 좌표 비교 검증용
            BoundingBox::new(x, y, w, h),
        )
    }

    #[test]
    fn identical_trees_have_zero_displacement() {
        let a = page_root(vec![text_run(10.0, 20.0, 30.0, 5.0)]);
        let b = page_root(vec![text_run(10.0, 20.0, 30.0, 5.0)]);
        let pg = diff_page(0, &a, &b);
        assert!(!pg.structure_mismatch);
        assert_eq!(pg.max_disp, 0.0);
        assert_eq!(pg.node_count_a, pg.node_count_b);
    }

    #[test]
    fn translation_is_measured() {
        let a = page_root(vec![text_run(10.0, 20.0, 30.0, 5.0)]);
        let b = page_root(vec![text_run(13.0, 20.0, 30.0, 5.0)]); // dx = 3
        let pg = diff_page(0, &a, &b);
        assert!(!pg.structure_mismatch);
        assert!((pg.max_disp - 3.0).abs() < 1e-9);
    }

    #[test]
    fn child_count_mismatch_flags_structure() {
        let a = page_root(vec![text_run(0.0, 0.0, 1.0, 1.0)]);
        let b = page_root(vec![
            text_run(0.0, 0.0, 1.0, 1.0),
            text_run(0.0, 0.0, 1.0, 1.0),
        ]);
        let pg = diff_page(0, &a, &b);
        assert!(pg.structure_mismatch);
        assert_ne!(pg.node_count_a, pg.node_count_b);
    }

    #[test]
    fn size_change_is_measured() {
        let a = page_root(vec![text_run(0.0, 0.0, 30.0, 5.0)]);
        let b = page_root(vec![text_run(0.0, 0.0, 30.0, 9.0)]); // dh = 4
        let pg = diff_page(0, &a, &b);
        assert!((pg.max_disp - 4.0).abs() < 1e-9);
    }

    #[test]
    fn lcs_align_handles_deletion() {
        let sa = ["A", "B", "C"];
        let sb = ["A", "C"];
        let pairs = lcs_align(&sa, &sb);
        assert_eq!(
            pairs,
            vec![(Some(0), Some(0)), (Some(1), None), (Some(2), Some(1))]
        );
    }

    #[test]
    fn displacement_measured_despite_inserted_sibling() {
        // a: [run@(10,20)]  b: [run@(10,20), run@(0,0)] — 첫 노드는 동일 위치, 둘째는 삽입.
        let a = page_root(vec![text_run(10.0, 20.0, 5.0, 5.0)]);
        let b = page_root(vec![
            text_run(10.0, 20.0, 5.0, 5.0),
            text_run(0.0, 0.0, 5.0, 5.0),
        ]);
        let pg = diff_page(0, &a, &b);
        assert!(pg.structure_mismatch); // 삽입 감지
        assert_eq!(pg.max_disp, 0.0); // 대응 노드 변위는 0 으로 정확 측정(가려지지 않음)
    }

    #[test]
    fn type_deltas_report_inserted_node_type() {
        let a = page_root(vec![text_run(0.0, 0.0, 1.0, 1.0)]);
        let b = page_root(vec![
            text_run(0.0, 0.0, 1.0, 1.0),
            text_run(0.0, 0.0, 1.0, 1.0),
        ]);
        let pg = diff_page(0, &a, &b);
        let tb = pg
            .type_deltas
            .iter()
            .find(|d| d.node_type == "TextBox")
            .expect("TextBox 타입 델타가 있어야 함");
        assert_eq!((tb.count_a, tb.count_b), (1, 2));
        assert_eq!(tb.net(), 1);
        // 개수가 같은 타입(Body)은 델타에 없어야 한다.
        assert!(pg.type_deltas.iter().all(|d| d.node_type != "Body"));
    }

    #[test]
    fn non_pass_statuses_are_hard_failures() {
        assert!(!status_is_hard_failure("PASS"));
        // TextRun ±1 경미 조성 노이즈는 게이트 하드 실패에서 제외 (#1773).
        assert!(!status_is_hard_failure("WARN_TEXTRUN"));
        for status in ["OVER", "STRUCT_MISMATCH", "PAGE_MISMATCH", "LOAD_FAIL"] {
            assert!(
                status_is_hard_failure(status),
                "{status} must fail the gate"
            );
        }
    }

    #[test]
    fn textrun_pm1_predicate() {
        // 삽입/삭제 각 1개 이하 + TextRun 만 → true.
        assert!(is_textrun_pm1(&["TextRun"], &[]));
        assert!(is_textrun_pm1(&[], &["TextRun"]));
        assert!(is_textrun_pm1(&["TextRun"], &["TextRun"]));
        // 이벤트 없음 → false (구조 불일치가 아님).
        assert!(!is_textrun_pm1(&[], &[]));
        // 다른 타입 혼입 → false.
        assert!(!is_textrun_pm1(&["TextLine"], &[]));
        assert!(!is_textrun_pm1(&["TextRun"], &["Line"]));
        // 같은 방향 2개 이상 → false.
        assert!(!is_textrun_pm1(&["TextRun", "TextRun"], &[]));
    }

    fn page_diff(
        structure_mismatch: bool,
        struct_textrun_pm1: bool,
        max_disp: f64,
    ) -> PageGeomDiff {
        PageGeomDiff {
            page: 0,
            node_count_a: 1,
            node_count_b: 1,
            structure_mismatch,
            max_disp,
            mean_disp: 0.0,
            top_deltas: Vec::new(),
            type_deltas: Vec::new(),
            struct_textrun_pm1,
        }
    }

    fn doc_diff(pages: Vec<PageGeomDiff>) -> DocGeomDiff {
        let max_disp = pages.iter().map(|p| p.max_disp).fold(0.0_f64, f64::max);
        DocGeomDiff {
            page_count_a: pages.len() as u32,
            page_count_b: pages.len() as u32,
            pages,
            max_disp,
        }
    }

    #[test]
    fn textrun_pm1_struct_downgrades_to_warn() {
        let d = doc_diff(vec![page_diff(true, true, 0.5)]);
        let sum = summarize(&d, None, 1.0);
        assert_eq!(status_str(&d, &sum, 1.0), "WARN_TEXTRUN");
    }

    #[test]
    fn non_textrun_struct_stays_hard() {
        let d = doc_diff(vec![page_diff(true, false, 0.5)]);
        let sum = summarize(&d, None, 1.0);
        assert_eq!(status_str(&d, &sum, 1.0), "STRUCT_MISMATCH");
    }

    #[test]
    fn textrun_pm1_with_over_disp_is_over() {
        // 조성 노이즈가 있어도 변위가 임계를 넘으면 OVER 로 하드 실패 유지.
        let d = doc_diff(vec![page_diff(true, true, 5.0)]);
        let sum = summarize(&d, None, 1.0);
        assert_eq!(status_str(&d, &sum, 1.0), "OVER");
    }

    #[test]
    fn mixed_pages_hard_struct_wins() {
        // 한 페이지는 TextRun ±1, 다른 페이지는 일반 구조 불일치 → STRUCT_MISMATCH.
        let d = doc_diff(vec![
            page_diff(true, true, 0.0),
            page_diff(true, false, 0.0),
        ]);
        let sum = summarize(&d, None, 1.0);
        assert_eq!(status_str(&d, &sum, 1.0), "STRUCT_MISMATCH");
    }

    // ---- [#1993] flatten_pos_set / line_run_coverage 회귀 ----

    fn run_node(x: f64, y: f64, w: f64, h: f64) -> RenderNode {
        use crate::renderer::render_tree::TextRunNode;
        use crate::renderer::TextStyle;
        RenderNode::new(
            3,
            RenderNodeType::TextRun(TextRunNode {
                text: "x".to_string(),
                style: TextStyle::default(),
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
                baseline: h * 0.8,
                field_marker: Default::default(),
                display_text: None,
            }),
            BoundingBox::new(x, y, w, h),
        )
    }

    fn text_line(x: f64, y: f64, w: f64, h: f64, runs: Vec<RenderNode>) -> RenderNode {
        use crate::renderer::render_tree::TextLineNode;
        let mut n = RenderNode::new(
            2,
            RenderNodeType::TextLine(TextLineNode::new(h, h * 0.8)),
            BoundingBox::new(x, y, w, h),
        );
        n.children = runs;
        n
    }

    #[test]
    fn resegmented_runs_same_coverage_is_zero_diff() {
        // 같은 줄의 동일 텍스트를 다른 경계로 재분할 → run 커버리지 합집합 불변 → 허위 변위 없음.
        let a = page_root(vec![text_line(
            0.0,
            0.0,
            60.0,
            10.0,
            vec![
                run_node(0.0, 0.0, 30.0, 10.0),
                run_node(30.0, 0.0, 30.0, 10.0),
            ],
        )]);
        let b = page_root(vec![text_line(
            0.0,
            0.0,
            60.0,
            10.0,
            vec![
                run_node(0.0, 0.0, 20.0, 10.0),
                run_node(20.0, 0.0, 40.0, 10.0),
            ],
        )]);
        let pg = diff_page(0, &a, &b);
        assert!(!pg.structure_mismatch);
        assert_eq!(pg.max_disp, 0.0);
    }

    #[test]
    fn reordered_sibling_lines_same_positions_is_zero_diff() {
        // 위치 동일한 두 줄이 자식 순서만 뒤바뀜(hwp3 리오더/별지 370px 유형) → 계층 LCS 오짝지음이
        // 내던 허위 변위가 flatten 멀티셋 사전차단으로 사라진다.
        let la = text_line(0.0, 0.0, 60.0, 10.0, vec![run_node(0.0, 0.0, 60.0, 10.0)]);
        let lb = text_line(0.0, 20.0, 60.0, 10.0, vec![run_node(0.0, 20.0, 60.0, 10.0)]);
        let a = page_root(vec![la.clone(), lb.clone()]);
        let b = page_root(vec![lb, la]); // 순서만 스왑
        let pg = diff_page(0, &a, &b);
        assert!(!pg.structure_mismatch);
        assert_eq!(pg.max_disp, 0.0);
    }

    #[test]
    fn real_run_coverage_change_is_not_masked() {
        // run 커버리지가 실제로 달라지면(줄 폭 축소 = run 삭제) 사전차단이 발동하지 않고 검출된다.
        let a = page_root(vec![text_line(
            0.0,
            0.0,
            60.0,
            10.0,
            vec![
                run_node(0.0, 0.0, 30.0, 10.0),
                run_node(30.0, 0.0, 30.0, 10.0),
            ],
        )]);
        let b = page_root(vec![text_line(
            0.0,
            0.0,
            30.0,
            10.0,
            vec![run_node(0.0, 0.0, 30.0, 10.0)],
        )]);
        let pg = diff_page(0, &a, &b);
        // 커버리지·줄폭이 다르므로 사전차단 미발동 → 구조 불일치로 검출(위양성 아님).
        assert!(pg.structure_mismatch);
    }

    // ---- `--json` 봉투 ----

    fn opts(positionals: &[&str], page: Option<u32>) -> CliOptions {
        CliOptions {
            positionals: positionals.iter().map(PathBuf::from).collect(),
            batch: false,
            via: Via::Hwpx,
            page,
            max_disp: DEFAULT_MAX_DISP,
            out_dir: PathBuf::from("output/poc/render_diff"),
            json: true,
        }
    }

    #[test]
    fn non_finite_displacement_becomes_null_not_zero() {
        // NaN 을 0 으로 내보내면 소비자는 "차이 없음"으로 오독한다 — 측정 불가는 부재다.
        assert_eq!(fnum(f64::NAN), Value::Null);
        assert_eq!(fnum(f64::INFINITY), Value::Null);
        assert_eq!(fnum(f64::NEG_INFINITY), Value::Null);
        assert_eq!(fnum(0.0), json!(0.0));
        assert_eq!(fnum(-1.5), json!(-1.5));
    }

    #[test]
    fn nan_page_metrics_serialize_as_null() {
        let mut p = page_diff(false, false, f64::NAN);
        p.mean_disp = f64::INFINITY;
        p.top_deltas.push(NodeDelta {
            path: "Body".into(),
            node_type: "Body",
            dx: f64::NAN,
            dy: 0.0,
            dw: 0.0,
            dh: 0.0,
        });
        let v = page_json(&p);
        assert!(v["maxDisp"].is_null(), "{v}");
        assert!(v["meanDisp"].is_null(), "{v}");
        assert!(v["topDeltas"][0]["dx"].is_null(), "{v}");
        assert_eq!(v["topDeltas"][0]["dy"], json!(0.0), "{v}");
    }

    #[test]
    fn roundtrip_envelope_declares_via_and_null_source_b() {
        let o = opts(&["a.hwp"], None);
        let d = doc_diff(vec![page_diff(false, false, 0.0)]);
        let sum = summarize(&d, None, o.max_disp);
        let v = single_envelope(&o, &d, &sum, status_str(&d, &sum, o.max_disp));
        assert_eq!(v["schemaVersion"], SCHEMA_VERSION, "{v}");
        assert_eq!(v["mode"], "roundtrip", "{v}");
        assert_eq!(v["sourceA"], "a.hwp", "{v}");
        assert!(v["sourceB"].is_null(), "{v}");
        assert_eq!(v["via"], "hwpx", "{v}");
        assert!(v["pageFilter"].is_null(), "{v}");
        assert_eq!(v["status"], "PASS", "{v}");
        assert_eq!(v["regression"], false, "{v}");
        assert_eq!(v["pages"].as_array().map(Vec::len), Some(1), "{v}");
    }

    #[test]
    fn pair_envelope_has_no_via_because_there_is_no_roundtrip() {
        let o = opts(&["a.hwp", "b.hwpx"], None);
        let d = doc_diff(vec![page_diff(true, false, 0.0)]);
        let sum = summarize(&d, None, o.max_disp);
        let v = single_envelope(&o, &d, &sum, status_str(&d, &sum, o.max_disp));
        assert_eq!(v["mode"], "pair", "{v}");
        assert_eq!(v["sourceB"], "b.hwpx", "{v}");
        assert!(v["via"].is_null(), "{v}");
        assert_eq!(v["status"], "STRUCT_MISMATCH", "{v}");
        assert_eq!(v["regression"], true, "{v}");
        assert_eq!(v["hardStructPages"], 1, "{v}");
    }

    #[test]
    fn batch_load_failure_record_carries_error_and_is_not_a_regression() {
        let o = opts(&["dir"], None);
        let row = BatchRow {
            rel_path: "x.hwp".into(),
            status: "LOAD_FAIL".into(),
            pages_a: 0,
            pages_b: 0,
            max_disp: 0.0,
            worst_page: None,
            struct_pages: 0,
            hard_struct_pages: 0,
            over_pages: 0,
            struct_delta: String::new(),
            elapsed_ms: 7,
            error: "열 수 없음".into(),
        };
        let v = batch_record(&o, &row);
        assert_eq!(v["source"], "x.hwp", "{v}");
        assert_eq!(v["error"], "열 수 없음", "{v}");
        // 측정 실패는 회귀 검출이 아니다 — 두 축이 겹치면 배치 종료 코드를 못 가른다.
        assert_eq!(v["regression"], false, "{v}");
        assert_eq!(v["elapsedMs"], 7, "{v}");
    }

    #[test]
    fn batch_success_record_omits_the_error_key() {
        let o = opts(&["dir"], None);
        let row = BatchRow {
            rel_path: "ok.hwp".into(),
            status: "OVER".into(),
            pages_a: 2,
            pages_b: 2,
            max_disp: 4.25,
            worst_page: Some(1),
            struct_pages: 0,
            hard_struct_pages: 0,
            over_pages: 1,
            struct_delta: String::new(),
            elapsed_ms: 3,
            error: String::new(),
        };
        let v = batch_record(&o, &row);
        assert!(v.get("error").is_none(), "{v}");
        assert_eq!(v["regression"], true, "{v}");
        assert_eq!(v["worstPage"], 1, "{v}");
        assert_eq!(v["maxDisp"], json!(4.25), "{v}");
    }

    #[test]
    fn json_flag_parses_and_defaults_off() {
        let on = parse_cli(&["a.hwp".to_string(), "--json".to_string()]).expect("파싱");
        assert!(on.json);
        let off = parse_cli(&["a.hwp".to_string()]).expect("파싱");
        assert!(!off.json);
    }
}
