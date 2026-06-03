# Task M100 #363 구현계획서: native PageRenderTree bridge API

## 1. 조사 결과 요약

최신 `upstream/devel` 기준으로 다음 구조가 확인되었다.

- 내부 렌더 트리 생성 경로: `DocumentCore::build_page_tree(page_num)` (`pub(crate)`)
- 캐시 경로: `DocumentCore::build_page_tree_cached(page_num)` (`pub(crate)`)
- WASM JSON 경로: `HwpDocument::getPageRenderTree(page_num)` → `build_page_tree_cached()` → `tree.root.to_json()`
- 이미지 데이터 조회 관례: `renderer/layout/utils.rs`의 `find_bin_data()`가 `bin_data_id`를 1-based 순번으로 해석하고, 필요 시 `BinDataContent.id` 직접 검색으로 보강한다.

이슈 본문의 검증 커밋 `1e9d78a`는 다음 변경을 포함한다.

- `DocumentCore::build_page_render_tree(page_num)` 추가
- `DocumentCore::get_bin_data(index)` 추가
- render tree 관련 타입에 `serde::Serialize` 추가
- `ImageNode.data`, `PageBackgroundImage.data`는 JSON 직렬화에서 제외

최신 upstream에는 해당 API가 아직 없으므로, 기존 내부 구현을 public API로 노출하는 작은 범위의 변경이 적합하다.

## 2. 구현 원칙

1. `build_page_tree()`의 동작을 복제하지 않고 public wrapper만 추가한다.
2. `PageRenderTree` 타입은 기존 `pub mod renderer` 경로로 이미 외부 접근 가능하므로 타입 재정의는 하지 않는다.
3. render tree JSON 직렬화를 위해 `serde::Serialize`를 추가하되, 이미지 원본 bytes는 render tree JSON에 포함하지 않는다.
4. `ImageNode.bin_data_id`는 1-based 참조값으로 문서화하고, `get_bin_data(index)`는 이슈 제안 시그니처에 맞춰 0-based slice index API로 제공한다.
5. 기존 WASM `getPageRenderTree()`의 수동 JSON 경로는 변경하지 않는다.

## 3. 단계별 구현 계획

### Stage 1 — public API 추가

작업 파일:

- `src/document_core/queries/rendering.rs`

작업 내용:

- `DocumentCore::build_page_render_tree(&self, page_num: u32) -> Result<PageRenderTree, HwpError>` 추가
- `DocumentCore::get_bin_data(&self, index: usize) -> Option<&[u8]>` 추가
- `build_page_render_tree()`는 기존 `build_page_tree(page_num)`를 호출하고, 기존 render API와 동일하게 `layout_engine.take_overflows()`를 정리한다.

검증:

- `cargo build`

완료 산출물:

- `mydocs/working/task_m100_363_stage1.md`
- Stage 1 커밋

### Stage 2 — render tree 직렬화 지원 정리

작업 파일:

- `Cargo.toml`
- `src/renderer/render_tree.rs`
- `src/renderer/mod.rs`
- `src/renderer/composer.rs`
- `src/renderer/layout.rs`
- `src/renderer/equation/ast.rs`
- `src/renderer/equation/layout.rs`
- `src/renderer/equation/symbols.rs`
- `src/model/control.rs`
- `src/model/image.rs`
- `src/model/style.rs`

작업 내용:

- `serde` derive 의존성 추가
- `PageRenderTree`의 public graph를 구성하는 최소 타입에 `Serialize` derive 추가
- `ImageNode.data`, `PageBackgroundImage.data`는 `#[serde(skip)]` 처리
- 컴파일 에러가 요구하는 타입만 추가 반영하고, unrelated model 타입으로 derive 확산을 피한다.

검증:

- `cargo build`
- 필요 시 `cargo test test_page_render_tree`

완료 산출물:

- `mydocs/working/task_m100_363_stage2.md`
- Stage 2 커밋

### Stage 3 — API 동작 테스트 추가

작업 파일 후보:

- `src/document_core/queries/rendering.rs` 또는 관련 테스트 모듈
- 필요 시 `src/wasm_api/tests.rs` 내 기존 fixture 재사용

작업 내용:

- `build_page_render_tree(0)`가 public API로 호출 가능하고 루트 노드를 반환하는 테스트 추가
- `get_bin_data(index)`가 `Document.bin_data_content[index].data` slice를 반환하는 테스트 추가
- 가능하면 작은 synthetic document 또는 기존 테스트 fixture를 사용해 I/O 의존을 줄인다.

검증:

- `cargo test build_page_render_tree`
- `cargo test get_bin_data`
- `cargo test`

완료 산출물:

- `mydocs/working/task_m100_363_stage3.md`
- Stage 3 커밋

### Stage 4 — 문서화 및 최종 검증

작업 파일 후보:

- `mydocs/tech/hwp_spec_errata.md`
- `mydocs/manual/e2e_verification_guide.md` 또는 native bridge API 설명에 적합한 새 문서
- `mydocs/report/task_m100_363_report.md`
- `mydocs/orders/20260427.md`

작업 내용:

- `ImageNode.bin_data_id`는 1-based 참조값임을 명시한다.
- `get_bin_data(index)`는 0-based `bin_data_content` index임을 명시하고, `bin_data_id`를 사용할 때는 보통 `bin_data_id - 1`을 넘긴다는 점을 적는다.
- 전체 검증 결과와 남은 리스크를 최종 보고서에 기록한다.
- 오늘 할일 상태를 완료로 갱신한다.

검증:

- `cargo build`
- `cargo test`
- 문서 링크 및 명칭 확인

완료 산출물:

- `mydocs/working/task_m100_363_stage4.md`
- `mydocs/report/task_m100_363_report.md`
- 최종 커밋

## 4. 테스트 기준

필수:

- `cargo build`
- `cargo test`

선택:

- `cargo test test_page_render_tree`
- `cargo test build_page_render_tree`
- `cargo test get_bin_data`

WASM Docker 빌드는 이번 이슈의 직접 범위가 아니므로 필수 검증에서 제외한다. 단, `Cargo.toml` 의존성 변경이 WASM 빌드에 영향을 줄 가능성이 발견되면 별도 승인 후 WASM 빌드를 수행한다.

## 5. 승인 요청

위 4단계 계획으로 Stage 1 구현을 시작해도 되는지 승인을 요청한다.
