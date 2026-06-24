//! Task #1499 — HWPX 라운드트립 **렌더 기하 정합성** 비교 코어 + `render-diff` CLI.
//!
//! 기존 `hwpx-roundtrip` baseline 은 IR 뼈대(구조) 보존만 검증한다. IrDiff 0 이어도
//! 라운드트립(parse→serialize→reparse)이 **렌더 결과**(페이지 수, 렌더 노드 삽입/삭제,
//! bbox 변위)를 바꾸는 공백을 메우기 위해, 두 문서의 페이지별 `RenderNode` bbox 를
//! **타입 LCS 매칭**으로 비교하여 시각 회귀를 정량화한다.
//!
//! 폰트 래스터화에 의존하지 않는 **결정론적 기하 비교**가 1차 지표다. 픽셀/SSIM 보조
//! 비교는 폰트·플랫폼 잡음이 커 후속(P1)으로 분리한다.
//!
//! 본 게이트는 "원본 IR 렌더 vs 라운드트립 IR 렌더" 의 **내부 정합성(회귀 방지)** 만
//! 보장하며, 한컴 정답지 충실도와는 별개다(자기 roundtrip 통과 ≠ 한컴 호환).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::parser::hwpx::parse_hwpx;
use crate::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use crate::serializer::hwpx::serialize_hwpx;
use crate::wasm_api::HwpDocument;

/// 라운드트립 변위 기본 임계 (px). 실측 PASS 샘플 매칭 노드 변위 0.0px → 임계 0.5px.
pub const DEFAULT_THRESHOLD_PX: f64 = 0.5;

/// 평탄화된 렌더 노드 — LCS 매칭의 단위. (타입 태그 + bbox 만, 텍스트/스타일 무시)
#[derive(Debug, Clone)]
pub struct FlatNode {
    pub tag: &'static str,
    pub bbox: BoundingBox,
}

/// 한 페이지의 기하 비교 결과.
#[derive(Debug, Clone, Default)]
pub struct PageGeomDiff {
    /// LCS 로 대응된 노드쌍 수.
    pub matched: usize,
    /// b 에만 있는 노드 수 (삽입).
    pub inserted: usize,
    /// a 에만 있는 노드 수 (삭제).
    pub deleted: usize,
    /// 매칭쌍 중 최대 변위 (px).
    pub max_disp: f64,
    /// 최대 변위 노드의 (태그, 변위).
    pub worst: Option<(&'static str, f64)>,
}

/// 판정 등급.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// 페이지 수 일치 ∧ 삽입/삭제 0 ∧ max_disp ≤ 임계.
    Pass,
    /// 페이지 수 불일치 또는 렌더 노드 삽입/삭제 존재 (하드 실패).
    StructMismatch,
    /// 매칭 노드 변위가 임계 초과 (하드 실패).
    DispOver,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::StructMismatch => "STRUCT_MISMATCH",
            Verdict::DispOver => "DISP_OVER",
        }
    }

    pub fn is_pass(self) -> bool {
        matches!(self, Verdict::Pass)
    }
}

/// 문서 전체의 기하 비교 결과.
#[derive(Debug, Clone)]
pub struct DocGeomDiff {
    pub pages_a: u32,
    pub pages_b: u32,
    pub page_mismatch: bool,
    pub total_inserted: usize,
    pub total_deleted: usize,
    /// 전 페이지 통틀어 최대 매칭 변위 (px).
    pub max_disp: f64,
    /// 최대 변위 (페이지, 태그, 변위).
    pub worst: Option<(u32, &'static str, f64)>,
    pub threshold: f64,
    pub verdict: Verdict,
}

impl DocGeomDiff {
    pub fn verdict(&self) -> Verdict {
        self.verdict
    }
}

/// `RenderNodeType` variant 판별자 → 안정 태그. (텍스트 내용/스타일은 의도적으로 무시)
pub fn type_tag(t: &RenderNodeType) -> &'static str {
    match t {
        RenderNodeType::Page(_) => "Page",
        RenderNodeType::PageBackground(_) => "PageBackground",
        RenderNodeType::MasterPage => "MasterPage",
        RenderNodeType::Header => "Header",
        RenderNodeType::Footer => "Footer",
        RenderNodeType::Body { .. } => "Body",
        RenderNodeType::Column(_) => "Column",
        RenderNodeType::FootnoteArea => "FootnoteArea",
        RenderNodeType::TextLine(_) => "TextLine",
        RenderNodeType::TextRun(_) => "TextRun",
        RenderNodeType::Table(_) => "Table",
        RenderNodeType::TableCell(_) => "TableCell",
        RenderNodeType::Line(_) => "Line",
        RenderNodeType::Rectangle(_) => "Rectangle",
        RenderNodeType::Ellipse(_) => "Ellipse",
        RenderNodeType::Path(_) => "Path",
        RenderNodeType::Image(_) => "Image",
        RenderNodeType::Group(_) => "Group",
        RenderNodeType::TextBox => "TextBox",
        RenderNodeType::Equation(_) => "Equation",
        RenderNodeType::FormObject(_) => "FormObject",
        RenderNodeType::FootnoteMarker(_) => "FootnoteMarker",
        RenderNodeType::Placeholder(_) => "Placeholder",
        RenderNodeType::RawSvg(_) => "RawSvg",
    }
}

/// 렌더 노드 트리를 전위순회로 평탄화한다 (루트 포함).
pub fn flatten_page(node: &RenderNode, out: &mut Vec<FlatNode>) {
    out.push(FlatNode {
        tag: type_tag(&node.node_type),
        bbox: node.bbox,
    });
    for child in &node.children {
        flatten_page(child, out);
    }
}

/// 두 bbox 의 변위 = max(|Δx|, |Δy|, |Δw|, |Δh|).
fn displacement(a: &BoundingBox, b: &BoundingBox) -> f64 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    let dw = (a.width - b.width).abs();
    let dh = (a.height - b.height).abs();
    dx.max(dy).max(dw).max(dh)
}

/// 태그 시퀀스 LCS (DP). 결과는 a/b 인덱스 정렬 페어:
/// `(Some, Some)` = 매칭, `(Some, None)` = 삭제(a만), `(None, Some)` = 삽입(b만).
pub fn lcs_match(a: &[FlatNode], b: &[FlatNode]) -> Vec<(Option<usize>, Option<usize>)> {
    let n = a.len();
    let m = b.len();
    // dp[i][j] = a[i..], b[j..] 의 LCS 길이.
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i].tag == b[j].tag {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let mut out = Vec::with_capacity(n.max(m));
    let (mut i, mut j) = (0usize, 0usize);
    while i < n && j < m {
        if a[i].tag == b[j].tag {
            out.push((Some(i), Some(j)));
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            out.push((Some(i), None)); // 삭제
            i += 1;
        } else {
            out.push((None, Some(j))); // 삽입
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

/// 한 페이지의 평탄 시퀀스 두 개를 비교한다.
pub fn diff_page(a: &[FlatNode], b: &[FlatNode]) -> PageGeomDiff {
    let mut d = PageGeomDiff::default();
    for pair in lcs_match(a, b) {
        match pair {
            (Some(ia), Some(jb)) => {
                d.matched += 1;
                let disp = displacement(&a[ia].bbox, &b[jb].bbox);
                if disp > d.max_disp {
                    d.max_disp = disp;
                    d.worst = Some((a[ia].tag, disp));
                }
            }
            (Some(_), None) => d.deleted += 1,
            (None, Some(_)) => d.inserted += 1,
            (None, None) => {}
        }
    }
    d
}

/// 두 문서를 페이지별로 렌더 트리 생성 후 기하 비교한다.
pub fn diff_documents_geom(a: &HwpDocument, b: &HwpDocument, threshold: f64) -> DocGeomDiff {
    let pages_a = a.page_count();
    let pages_b = b.page_count();
    let page_mismatch = pages_a != pages_b;

    let mut total_inserted = 0usize;
    let mut total_deleted = 0usize;
    let mut max_disp = 0.0f64;
    let mut worst: Option<(u32, &'static str, f64)> = None;

    // 공통 페이지만 노드 단위 비교 (페이지 수 불일치는 page_mismatch 로 별도 분류).
    let common = pages_a.min(pages_b);
    for p in 0..common {
        let (mut fa, mut fb) = (Vec::new(), Vec::new());
        if let Ok(ta) = a.build_page_render_tree(p) {
            flatten_page(&ta.root, &mut fa);
        }
        if let Ok(tb) = b.build_page_render_tree(p) {
            flatten_page(&tb.root, &mut fb);
        }
        let pd = diff_page(&fa, &fb);
        total_inserted += pd.inserted;
        total_deleted += pd.deleted;
        if pd.max_disp > max_disp {
            max_disp = pd.max_disp;
            if let Some((tag, disp)) = pd.worst {
                worst = Some((p, tag, disp));
            }
        }
    }

    let verdict = if page_mismatch || total_inserted > 0 || total_deleted > 0 {
        Verdict::StructMismatch
    } else if max_disp > threshold {
        Verdict::DispOver
    } else {
        Verdict::Pass
    };

    DocGeomDiff {
        pages_a,
        pages_b,
        page_mismatch,
        total_inserted,
        total_deleted,
        max_disp,
        worst,
        threshold,
        verdict,
    }
}

// ───────────────────────── CLI (`render-diff`) ─────────────────────────

/// 자기 라운드트립: HWPX 바이트 → 직렬화→재parse 바이트 생성.
fn self_roundtrip_bytes(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let doc = parse_hwpx(bytes).map_err(|e| format!("HWPX 파싱 실패: {e}"))?;
    serialize_hwpx(&doc).map_err(|e| format!("HWPX 직렬화 실패: {e}"))
}

fn load_doc(bytes: &[u8]) -> Result<HwpDocument, String> {
    HwpDocument::from_bytes(bytes).map_err(|e| format!("문서 로드 실패: {e}"))
}

/// 한 파일의 배치 측정 결과.
struct GeomRow {
    rel: String,
    diff: Option<DocGeomDiff>,
    elapsed_ms: u128,
    error: String,
}

impl GeomRow {
    fn verdict_str(&self) -> &'static str {
        match &self.diff {
            Some(d) => d.verdict.as_str(),
            None => "ERROR",
        }
    }
}

/// 자기 라운드트립 기하 비교: 원본 IR 렌더 vs `serialize_hwpx`→재parse IR 렌더.
/// 게이트 테스트(`tests/visual_roundtrip_baseline.rs`)와 CLI 가 공유한다.
pub fn self_roundtrip_diff(bytes: &[u8], threshold: f64) -> Result<DocGeomDiff, String> {
    let orig = load_doc(bytes)?;
    let rt_bytes = self_roundtrip_bytes(bytes)?;
    let rt = load_doc(&rt_bytes).map_err(|e| format!("재조립 {e}"))?;
    Ok(diff_documents_geom(&orig, &rt, threshold))
}

/// 자기 라운드트립 1건 측정.
fn measure_self_roundtrip(path: &Path, rel: &str, threshold: f64) -> GeomRow {
    let started = Instant::now();
    let mk = |error: String| GeomRow {
        rel: rel.to_string(),
        diff: None,
        elapsed_ms: started.elapsed().as_millis(),
        error,
    };

    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => return mk(format!("읽기 실패: {e}")),
    };
    match self_roundtrip_diff(&bytes, threshold) {
        Ok(diff) => GeomRow {
            rel: rel.to_string(),
            diff: Some(diff),
            elapsed_ms: started.elapsed().as_millis(),
            error: String::new(),
        },
        Err(e) => mk(e),
    }
}

fn print_doc_diff(label: &str, d: &DocGeomDiff) {
    println!("[{:>14}] {}", d.verdict.as_str(), label);
    println!(
        "                 pages: {} → {}{}",
        d.pages_a,
        d.pages_b,
        if d.page_mismatch {
            "  (불일치!)"
        } else {
            ""
        }
    );
    println!(
        "                 노드 삽입 {} 삭제 {} · 최대변위 {:.3}px (임계 {:.3})",
        d.total_inserted, d.total_deleted, d.max_disp, d.threshold
    );
    if let Some((page, tag, disp)) = d.worst {
        println!(
            "                 최대변위 위치: page {} {} {:.3}px",
            page, tag, disp
        );
    }
}

fn collect_hwpx_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries =
            fs::read_dir(&dir).map_err(|e| format!("폴더 읽기 실패 {}: {e}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("폴더 항목 읽기 실패: {e}"))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("hwpx"))
            {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn write_inventory(out_dir: &Path, rows: &[GeomRow]) -> Result<PathBuf, String> {
    fs::create_dir_all(out_dir).map_err(|e| format!("출력 폴더 생성 실패: {e}"))?;
    let path = out_dir.join("geom_inventory.tsv");
    let mut tsv = String::from(
        "sample\tverdict\tpages_a\tpages_b\tinserted\tdeleted\tmax_disp\telapsed_ms\terror\n",
    );
    for r in rows {
        let (pa, pb, ins, del, disp) = match &r.diff {
            Some(d) => (
                d.pages_a.to_string(),
                d.pages_b.to_string(),
                d.total_inserted.to_string(),
                d.total_deleted.to_string(),
                format!("{:.3}", d.max_disp),
            ),
            None => ("-".into(), "-".into(), "-".into(), "-".into(), "-".into()),
        };
        tsv.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            r.rel.replace('\t', " "),
            r.verdict_str(),
            pa,
            pb,
            ins,
            del,
            disp,
            r.elapsed_ms,
            r.error.replace(['\t', '\n'], " ")
        ));
    }
    fs::write(&path, tsv).map_err(|e| format!("TSV 쓰기 실패: {e}"))?;
    Ok(path)
}

fn print_batch_summary(rows: &[GeomRow]) {
    let mut pass = 0;
    let mut struct_mm = 0;
    let mut disp_over = 0;
    let mut err = 0;
    for r in rows {
        match &r.diff {
            Some(d) => match d.verdict {
                Verdict::Pass => pass += 1,
                Verdict::StructMismatch => struct_mm += 1,
                Verdict::DispOver => disp_over += 1,
            },
            None => err += 1,
        }
    }
    println!(
        "\n총 {}건: PASS {} · STRUCT_MISMATCH {} · DISP_OVER {} · ERROR {}",
        rows.len(),
        pass,
        struct_mm,
        disp_over,
        err
    );
}

struct Options {
    inputs: Vec<PathBuf>,
    batch: bool,
    out_dir: PathBuf,
    threshold: f64,
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut inputs: Vec<PathBuf> = Vec::new();
    let mut batch = false;
    let mut out_dir = PathBuf::from("output/poc/task1499");
    let mut threshold = DEFAULT_THRESHOLD_PX;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--batch" => batch = true,
            "-o" | "--out" => {
                i += 1;
                let v = args
                    .get(i)
                    .ok_or_else(|| "-o 다음에 출력 폴더가 필요합니다".to_string())?;
                out_dir = PathBuf::from(v);
            }
            "--threshold" => {
                i += 1;
                let v = args
                    .get(i)
                    .ok_or_else(|| "--threshold 다음에 px 값이 필요합니다".to_string())?;
                threshold = v
                    .parse::<f64>()
                    .map_err(|_| format!("임계값이 올바르지 않습니다: {v}"))?;
            }
            other if other.starts_with('-') => {
                return Err(format!("알 수 없는 옵션: {other}"));
            }
            other => inputs.push(PathBuf::from(other)),
        }
        i += 1;
    }

    if inputs.is_empty() {
        return Err(
            "사용법: rhwp render-diff <a.hwpx> [b.hwpx] | --batch <폴더> [-o 출력] [--threshold px]"
                .to_string(),
        );
    }
    if batch && inputs.len() != 1 {
        return Err("--batch 모드는 폴더 1개만 지정합니다".to_string());
    }
    if !batch && inputs.len() > 2 {
        return Err("입력은 최대 2개(자기 라운드트립 1개 / 두 파일 비교 2개)입니다".to_string());
    }
    Ok(Options {
        inputs,
        batch,
        out_dir,
        threshold,
    })
}

pub fn run(args: &[String]) {
    let opts = match parse_args(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("오류: {e}");
            std::process::exit(2);
        }
    };

    // 배치: 폴더 전수 자기 라운드트립.
    if opts.batch {
        let files = match collect_hwpx_files(&opts.inputs[0]) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("오류: {e}");
                std::process::exit(2);
            }
        };
        if files.is_empty() {
            eprintln!("오류: .hwpx 파일이 없습니다: {}", opts.inputs[0].display());
            std::process::exit(2);
        }
        let mut rows = Vec::with_capacity(files.len());
        for path in &files {
            let rel = path
                .strip_prefix(&opts.inputs[0])
                .map(|r| r.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());
            let row = measure_self_roundtrip(path, &rel, opts.threshold);
            println!(
                "[{:>14}] {:>6}ms  {}{}",
                row.verdict_str(),
                row.elapsed_ms,
                row.rel,
                if row.error.is_empty() {
                    String::new()
                } else {
                    format!("  └ {}", row.error)
                }
            );
            rows.push(row);
        }
        match write_inventory(&opts.out_dir, &rows) {
            Ok(p) => println!("\nTSV 저장: {}", p.display()),
            Err(e) => {
                eprintln!("오류: {e}");
                std::process::exit(1);
            }
        }
        print_batch_summary(&rows);
        let hard = rows.iter().any(|r| {
            r.diff
                .as_ref()
                .map(|d| !d.verdict.is_pass())
                .unwrap_or(true)
        });
        if hard {
            std::process::exit(1);
        }
        return;
    }

    // 두 파일 비교.
    if opts.inputs.len() == 2 {
        let read = |p: &Path| fs::read(p).map_err(|e| format!("읽기 실패 {}: {e}", p.display()));
        let result = (|| -> Result<DocGeomDiff, String> {
            let ba = read(&opts.inputs[0])?;
            let bb = read(&opts.inputs[1])?;
            let da = load_doc(&ba)?;
            let db = load_doc(&bb)?;
            Ok(diff_documents_geom(&da, &db, opts.threshold))
        })();
        match result {
            Ok(d) => {
                let label = format!(
                    "{} vs {}",
                    opts.inputs[0].display(),
                    opts.inputs[1].display()
                );
                print_doc_diff(&label, &d);
                if !d.verdict.is_pass() {
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("오류: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    // 자기 라운드트립 (단일 파일).
    let rel = opts.inputs[0]
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| opts.inputs[0].to_string_lossy().to_string());
    let row = measure_self_roundtrip(&opts.inputs[0], &rel, opts.threshold);
    match &row.diff {
        Some(d) => {
            print_doc_diff(&format!("{} (자기 라운드트립)", rel), d);
            println!("                 소요 {}ms", row.elapsed_ms);
            if !d.verdict.is_pass() {
                std::process::exit(1);
            }
        }
        None => {
            eprintln!("오류: {}", row.error);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bb(x: f64, y: f64) -> BoundingBox {
        BoundingBox::new(x, y, 100.0, 20.0)
    }

    fn fnode(tag: &'static str, x: f64, y: f64) -> FlatNode {
        FlatNode {
            tag,
            bbox: bb(x, y),
        }
    }

    #[test]
    fn identical_sequences_pass_with_zero_disp() {
        let a = vec![fnode("Page", 0.0, 0.0), fnode("TextLine", 10.0, 20.0)];
        let b = a.clone();
        let d = diff_page(&a, &b);
        assert_eq!(d.matched, 2);
        assert_eq!(d.inserted, 0);
        assert_eq!(d.deleted, 0);
        assert_eq!(d.max_disp, 0.0);
    }

    #[test]
    fn one_pixel_shift_yields_disp_one() {
        let a = vec![fnode("Page", 0.0, 0.0), fnode("TextLine", 10.0, 20.0)];
        let b = vec![fnode("Page", 0.0, 0.0), fnode("TextLine", 10.0, 21.0)];
        let d = diff_page(&a, &b);
        assert_eq!(d.matched, 2);
        assert!((d.max_disp - 1.0).abs() < 1e-9, "max_disp={}", d.max_disp);
        assert_eq!(d.worst.map(|w| w.0), Some("TextLine"));
    }

    #[test]
    fn inserted_node_counts_as_structural() {
        let a = vec![fnode("Page", 0.0, 0.0), fnode("TextLine", 10.0, 20.0)];
        let b = vec![
            fnode("Page", 0.0, 0.0),
            fnode("TextLine", 10.0, 20.0),
            fnode("Image", 30.0, 40.0),
        ];
        let d = diff_page(&a, &b);
        assert_eq!(d.matched, 2);
        assert_eq!(d.inserted, 1);
        assert_eq!(d.deleted, 0);
    }

    #[test]
    fn deleted_node_counts_as_structural() {
        let a = vec![
            fnode("Page", 0.0, 0.0),
            fnode("Table", 5.0, 5.0),
            fnode("TextLine", 10.0, 20.0),
        ];
        let b = vec![fnode("Page", 0.0, 0.0), fnode("TextLine", 10.0, 20.0)];
        let d = diff_page(&a, &b);
        assert_eq!(d.matched, 2);
        assert_eq!(d.deleted, 1);
        assert_eq!(d.inserted, 0);
    }

    #[test]
    fn lcs_aligns_around_inserted_node() {
        // 중간에 삽입된 노드가 있어도 앞뒤 매칭이 유지되어 변위 오탐이 없어야 한다.
        let a = vec![fnode("Page", 0.0, 0.0), fnode("TextLine", 10.0, 20.0)];
        let b = vec![
            fnode("Page", 0.0, 0.0),
            fnode("Image", 99.0, 99.0),
            fnode("TextLine", 10.0, 20.0),
        ];
        let d = diff_page(&a, &b);
        assert_eq!(d.matched, 2);
        assert_eq!(d.inserted, 1);
        assert_eq!(d.max_disp, 0.0, "매칭 노드는 변위 0 이어야 함");
    }
}
