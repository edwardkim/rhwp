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

/// 페이지 변위 누적기.
struct PageAccum {
    sum_disp: f64,
    count: usize,
    max_disp: f64,
    structure_mismatch: bool,
    top: Vec<NodeDelta>,
}

impl PageAccum {
    fn new() -> Self {
        Self {
            sum_disp: 0.0,
            count: 0,
            max_disp: 0.0,
            structure_mismatch: false,
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
            _ => acc.structure_mismatch = true,
        }
    }
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
    PageGeomDiff {
        page,
        node_count_a,
        node_count_b,
        structure_mismatch: acc.structure_mismatch,
        max_disp: acc.max_disp,
        mean_disp,
        top_deltas: acc.top,
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
    over_pages: usize,
}

fn summarize(diff: &DocGeomDiff, page: Option<u32>, threshold: f64) -> DiffSummary {
    let pages = filtered_pages(diff, page);
    let mut worst_page = None;
    let mut max_disp = 0.0_f64;
    let mut struct_pages = 0;
    let mut over_pages = 0;
    for p in &pages {
        if p.structure_mismatch {
            struct_pages += 1;
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
        over_pages,
    }
}

fn status_str(diff: &DocGeomDiff, sum: &DiffSummary, threshold: f64) -> &'static str {
    if diff.page_count_mismatch() {
        "PAGE_MISMATCH"
    } else if sum.struct_pages > 0 {
        "STRUCT_MISMATCH"
    } else if sum.max_disp > threshold {
        "OVER"
    } else {
        "PASS"
    }
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
    elapsed_ms: u128,
    error: String,
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
            }
            Err(e) => row.error = e,
        }
        row.elapsed_ms = started.elapsed().as_millis();
        println!(
            "[{:>15}] max_disp={:>7.2} struct={} over={} {:>6}ms  {}",
            row.status,
            row.max_disp,
            row.struct_pages,
            row.over_pages,
            row.elapsed_ms,
            row.rel_path
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

    // 하드 실패(로드/RT 오류, 페이지 수 불일치)가 있으면 비정상 종료(회귀 검출용).
    let hard = rows
        .iter()
        .any(|r| r.status == "LOAD_FAIL" || r.status == "PAGE_MISMATCH");
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
        "sample\tstatus\tpages_a\tpages_b\tmax_disp\tworst_page\tstruct_pages\tover_pages\telapsed_ms\terror\n",
    );
    for r in rows {
        tsv.push_str(&format!(
            "{}\t{}\t{}\t{}\t{:.3}\t{}\t{}\t{}\t{}\t{}\n",
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
                if p.structure_mismatch {
                    "  [STRUCT]"
                } else {
                    ""
                }
            );
            for d in p.top_deltas.iter().take(3) {
                println!("      {:>7.2}px  {}", d.disp(), d.path);
            }
        }
    }
    println!("status: {status}");

    match status {
        "PASS" => 0,
        "OVER" | "STRUCT_MISMATCH" => 0, // 측정 단계 — 정보성, 비정상 종료 아님
        _ => 1,                          // PAGE_MISMATCH 등 하드 신호
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
}
