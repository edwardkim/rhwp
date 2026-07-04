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

struct CliOptions {
    /// 위치 인자(파일/폴더). 배치는 1개(폴더), 자기 라운드트립 1개, 두 파일 비교 2개.
    positionals: Vec<PathBuf>,
    batch: bool,
    via: Via,
    page: Option<u32>,
    max_disp: f64,
    out_dir: PathBuf,
}

fn parse_cli(args: &[String]) -> Result<CliOptions, String> {
    let mut positionals = Vec::new();
    let mut batch = false;
    let mut via = Via::Hwpx;
    let mut page = None;
    let mut max_disp = DEFAULT_MAX_DISP;
    let mut out_dir = PathBuf::from("output/poc/render_diff");

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--batch" => batch = true,
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
        return Err("사용법: rhwp render-diff <파일> [--via hwpx|hwp] [-p N] [--max-disp PX]\n         rhwp render-diff <a> <b> [-p N]\n         rhwp render-diff --batch <폴더> [--via hwpx] [-o 출력폴더]".into());
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
    })
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
    over_pages: usize,
    /// 파일 전체에 걸친 노드 타입별 순증감 (구조 불일치 원인 국소화).
    struct_delta: String,
    elapsed_ms: u128,
    error: String,
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
            return 2;
        }
    };
    if files.is_empty() {
        eprintln!(
            "오류: 처리할 .hwp/.hwpx 파일이 없습니다: {}",
            root.display()
        );
        return 2;
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
                row.over_pages = sum.over_pages;
                row.struct_delta = aggregate_struct_delta(&diff);
            }
            Err(e) => row.error = e,
        }
        row.elapsed_ms = started.elapsed().as_millis();
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
        rows.push(row);
    }

    if let Err(e) = write_batch_tsv(&opts.out_dir, &rows) {
        eprintln!("오류: {e}");
        return 1;
    }
    print_batch_summary(&rows);

    // 게이트 명령이므로 PASS 외 status 는 CI/스크립트에서 실패로 감지되게 한다.
    let hard = rows.iter().any(|r| status_is_hard_failure(&r.status));
    if hard {
        1
    } else {
        0
    }
}

fn write_batch_tsv(out_dir: &Path, rows: &[BatchRow]) -> Result<(), String> {
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
    println!("\nTSV 저장: {}", path.display());
    Ok(())
}

fn print_batch_summary(rows: &[BatchRow]) {
    let count = |s: &str| rows.iter().filter(|r| r.status == s).count();
    let overall_max = rows.iter().map(|r| r.max_disp).fold(0.0_f64, f64::max);
    println!();
    println!("=== render-diff 요약 ===");
    println!("  총 파일         : {}", rows.len());
    println!("  PASS            : {}", count("PASS"));
    println!("  WARN_TEXTRUN    : {}", count("WARN_TEXTRUN"));
    println!("  OVER            : {}", count("OVER"));
    println!("  STRUCT_MISMATCH : {}", count("STRUCT_MISMATCH"));
    println!("  PAGE_MISMATCH   : {}", count("PAGE_MISMATCH"));
    println!("  LOAD_FAIL       : {}", count("LOAD_FAIL"));
    println!("  전체 최대 변위  : {overall_max:.2} px");
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
                eprintln!("오류: A 로드 실패 {}: {e}", a.display());
                return 2;
            }
        };
        let core_b = match fs::read(b)
            .map_err(|e| e.to_string())
            .and_then(|d| DocumentCore::from_bytes(&d).map_err(|e| format!("{e:?}")))
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("오류: B 로드 실패 {}: {e}", b.display());
                return 2;
            }
        };
        match diff_render_geometry(&core_a, &core_b) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("오류: 기하 비교 실패 - {e:?}");
                return 1;
            }
        }
    } else {
        // 자기 라운드트립.
        let f = &opts.positionals[0];
        let data = match fs::read(f) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("오류: 파일 읽기 실패 {}: {e}", f.display());
                return 2;
            }
        };
        match roundtrip_geom(&data, opts.via) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("오류: 라운드트립 비교 실패 - {e:?}");
                return 1;
            }
        }
    };

    let sum = summarize(&diff, opts.page, opts.max_disp);
    let status = status_str(&diff, &sum, opts.max_disp);

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

    if status_is_hard_failure(status) {
        1
    } else {
        0
    }
}

/// `rhwp render-diff` 진입점.
pub fn run(args: &[String]) {
    let opts = match parse_cli(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("오류: {e}");
            std::process::exit(2);
        }
    };
    let code = if opts.batch {
        run_batch(&opts)
    } else {
        run_single(&opts)
    };
    if code != 0 {
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
}
