# 구현계획서 — Task #1499: render-diff 렌더 기하 정합성 게이트

- **이슈**: edwardkim/rhwp#1499 (M100) · **브랜치**: `local/task1499`
- **수행계획서**: [`task_m100_1499.md`](task_m100_1499.md)

---

## 확정된 통합 경로 (조사 결과)

- 렌더 트리: `HwpDocument::{page_count(&self), build_page_render_tree(&self, page) -> PageRenderTree}`
  (둘 다 `&self`). `PageRenderTree.root: RenderNode`.
- `RenderNode { node_type: RenderNodeType(24 variant), bbox: BoundingBox{x,y,width,height: f64}, children: Vec<RenderNode> }`.
- 자기 라운드트립 바이트 생성: `parser::hwpx::parse_hwpx(&[u8]) -> Document`,
  `serializer::hwpx::serialize_hwpx(&Document) -> Vec<u8>` (선례: `hwpx_roundtrip_batch::roundtrip_one`).
- 렌더는 항상 `HwpDocument::from_bytes(bytes)` 로 수행 → 원본/RT 모두 동일 경로.

---

## 단계 구성 (4단계)

### 단계 1 — 비교 코어 모듈 `diagnostics/render_geom_diff.rs`

순수 비교 로직 (I/O·CLI 무관, 단위테스트 가능).

- `fn type_tag(&RenderNodeType) -> &'static str` — variant 판별자 → 안정 태그.
- `fn flatten_page(&RenderNode, out: &mut Vec<FlatNode>)` — 전위순회 평탄화.
  `FlatNode { tag: &'static str, bbox: BoundingBox }`.
- `fn lcs_match(&[FlatNode], &[FlatNode]) -> Vec<(Option<usize>, Option<usize>)>` —
  태그 시퀀스 LCS (DP). 매칭쌍 + 삽입/삭제 인덱스 산출.
- `fn diff_page(a: &[FlatNode], b: &[FlatNode]) -> PageGeomDiff` —
  `{ matched: usize, inserted: usize, deleted: usize, max_disp: f64, worst: Option<(tag, disp)> }`.
  변위 = `max(|Δx|,|Δy|,|Δw|,|Δh|)`.
- `fn diff_documents_geom(a: &HwpDocument, b: &HwpDocument, threshold: f64) -> DocGeomDiff` —
  페이지 수 비교 → 페이지별 `diff_page` → `{ pages_a, pages_b, page_mismatch: bool,
  total_inserted, total_deleted, max_disp, verdict }`.
- `enum Verdict { Pass, StructMismatch, DispOver }`, 등급 함수.
- **단위테스트**: (1) 동일 트리 → Pass·disp 0, (2) 1px 평행이동 → disp 1.0,
  (3) 노드 1개 삽입 → StructMismatch, (4) 페이지 수 불일치 → StructMismatch.

> 모듈은 `diagnostics/` 내부에만 둔다 (공통 모듈 무수정).

### 단계 2 — `render-diff` CLI 3모드

- `diagnostics/render_geom_diff.rs` 에 `pub fn run(args: &[String])` 추가 (배치/단일 분기).
- `main.rs` dispatch: `Some("render-diff") => rhwp::diagnostics::render_geom_diff::run(&args[2..])`.
- 모드:
  - `render-diff <a.hwpx>` — 자기 라운드트립 (orig vs serialize→reparse). 사람 가독 요약 출력.
  - `render-diff <a.hwpx> <b.hwpx>` — 두 파일 비교.
  - `render-diff --batch <dir> [-o out]` — `.hwpx` 재귀 수집, 각자 자기 라운드트립,
    `{out}/geom_inventory.tsv` (sample, pages_a, pages_b, inserted, deleted, max_disp, verdict, elapsed_ms, error).
- `--threshold <px>` 옵션 (기본 0.5).
- `--help` 한 줄 + `print_help()` 에 항목 추가.

### 단계 3 — 회귀 게이트 `tests/visual_roundtrip_baseline.rs`

- `samples/hwpx` 재귀 전수 (기존 baseline 과 동일한 수집·신규 자동 포함).
- 각 샘플 자기 라운드트립 → `Verdict::Pass` 단언 (페이지 보존 ∧ 삽입삭제 0 ∧ max_disp ≤ 임계).
- `XFAIL`/`EXCLUDED` 등급 상수 (사유 문자열). `xfail_entries_still_fail` 가드 미러링.
- `LARGE` 분리 (`#[ignore]` 대용량) — 기존 baseline 관례 준수.

### 단계 4 — 임계 실측 확정 + 등급 정리 + 문서화

- 단계 2 CLI `--batch samples/hwpx` 실측 → max_disp 분포 확인 → 임계 0.5px 확정 또는 조정.
- 초과/구조 불일치 샘플은 사유와 함께 `XFAIL`/`EXCLUDED` 등록.
- `mydocs/manual/render_diff_command.md` 작성 + `CLAUDE.md` 명령 표에 한 줄.
- 전체 `cargo test` 회귀 (메모리 룰 준수).

---

## 산출물

| 파일 | 단계 |
|------|------|
| `src/diagnostics/render_geom_diff.rs` (신설) | 1·2 |
| `src/diagnostics/mod.rs` (모듈 등록) | 1 |
| `src/main.rs` (dispatch + help) | 2 |
| `tests/visual_roundtrip_baseline.rs` (신설) | 3 |
| `mydocs/manual/render_diff_command.md`, `CLAUDE.md` | 4 |

각 단계: 소스 커밋 + `_stage{N}.md` 보고서. 최종 `_report.md` + 오늘할일 갱신.
