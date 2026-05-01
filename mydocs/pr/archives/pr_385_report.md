# PR #385 처리 보고서 — 정상 머지 (cherry-pick)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#385](https://github.com/edwardkim/rhwp/pull/385) |
| 작성자 | [@postmelee](https://github.com/postmelee) (Taegyu Lee) |
| 이슈 | [#363](https://github.com/edwardkim/rhwp/issues/363) |
| 처리 결정 | **정상 머지 (cherry-pick)** |
| 처리 일자 | 2026-04-28 |

## 결함 요약

`PageRenderTree` 생성 경로가 내부 구조로만 존재하여 native bridge (Rust core 직접 사용) 에서 안정적으로 사용할 public API 부재.

## 변경 내용

### `DocumentCore` public API 2개 (`src/document_core/queries/rendering.rs`)

```rust
pub fn build_page_render_tree(&self, page_num: u32) -> Result<PageRenderTree, HwpError>
pub fn get_bin_data(&self, index: usize) -> Option<&[u8]>
```

기존 `build_page_tree` (pub(crate)) 를 감싸는 thin wrapper. 새 로직 없음.

### `serde::Serialize` derive 추가

`PageRenderTree` + 관련 render node 타입 (TextRun, ImageNode, ShapeNode, TableCellNode 등) 에 derive 추가. 이미지 bytes (`ImageNode.data`, `PageBackgroundImage.data`) 는 `#[serde(skip)]`.

### Cargo.toml 신규 의존성

```toml
serde = { version = "1", features = ["derive"] }
```

### 문서

- `mydocs/manual/native_render_tree_bridge_api.md` (신규)
- `mydocs/tech/hwp_spec_errata.md` 보강
- task_m100_363 5개 문서 (수행/구현 계획서 + stage1-4 + report)

### 테스트

- 신규 단위 테스트 2건 (build_page_render_tree, get_bin_data)
- 1014 → **1016 passed**

## 처리 절차

### Stage 0: PR 댓글 정정
이전 댓글 (정책 검토 항목 4개 제기) 을 정정하여 정상 머지 결정 안내.

### Stage 1: cherry-pick
- `local/pr385` 브랜치 (`local/devel` 분기)
- PR 의 4 commit (Stage 1~4) cherry-pick — postmelee author 보존
  - `45d376d` Stage 1 native render tree API
  - `e23aac7` Stage 2 serialize render tree
  - `c740b5b` Stage 3 test native bridge APIs
  - `d3bafc1` Stage 4 document native bridge API

### Stage 2: 충돌 해결
- `mydocs/orders/20260427.md`: 수동 통합 (Task #361, #362, #370, #372 + #363 항목 통합)
- 기타 파일: 자동 머지

### Stage 3: 자동 회귀 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | 통과 (serde 신규 의존성) |
| `cargo test --lib` | **1016 passed, 0 failed** (이전 1014 → +2 신규 테스트) |
| `cargo test --test svg_snapshot` | 6/6 통과 |
| `cargo test --test issue_301` | 1/1 통과 |
| `cargo test --test page_number_propagation` | 2/2 통과 (PR #366 효과 유지) |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |

### Stage 4: 머지 + close + push

- `local/pr385` → `local/devel` (no-ff merge)
- `local/devel` → `devel` (FF) push
- PR #385 댓글 + close
- 이슈 #363 close

## 작성자 기여

@postmelee (Taegyu Lee) — 기존 활발한 컨트리뷰터:
- Firefox 확장 + AMO 등록 + Safari + Chrome 옵션 등 8 머지 PR
- 본 PR (Task #363): native bridge API 노출 + alhangeul-macos use case 검증

## 부수 정책 결정 (작업지시자)

- PageRenderTree public API 노출 진행 — 향후 호환성은 작업지시자 책임 영역
- semver: 본 PR 단독으로는 patch (v0.7.8 후보), v0.7.x 의 정정 누적과 함께 minor 결정 가능
- `serde` 직접 의존성 추가 — 정책 문제 없음
- alhangeul-macos use case 의 1차 지원

## 참고

- 검토 문서: `mydocs/pr/pr_385_review.md`
- 구현계획서: `mydocs/pr/pr_385_review_impl.md`
- 작성자 외부 프로젝트: [alhangeul-macos](https://github.com/postmelee/alhangeul-macos)
