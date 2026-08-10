---
kind: implementation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 81 — upstream PartialTable API rebase 적응

## 발생 조건

Stage 80 완료 뒤 `upstream/devel`을 `e4d07fab7`까지 갱신하고 21개 작업 commit을
rebase했다. 충돌은 없었지만, rebased head의 `cargo check --profile release-test`는 다음
컴파일 오류를 보였다.

- `table_partial.rs`: Stage 78 Center-tail 높이 계산이 CellComposedStore 도입 전의
  `composed_paras` 지역 변수를 참조한다.
- `cursor_rect.rs`: `PageItem::PartialTable`의
  `start_row_height_override`/`end_row_height_override` 필드를 fast-path가 받지 않으며,
  `layout_partial_table()`의 새 25개 인자 순서에도 맞지 않는다.

이는 컴파일러가 잡은 rebase 적응 결함이며, 원본 Stage 80 PDF line-wrap 보정의 시각
결론을 다시 주장하는 근거가 아니다.

## 수정 원칙

1. Center-tail 계산은 non-windowed 경로에서만 호출되므로 `composed_store.eager_slice()`를
   사용한다. windowed probe의 전량 compose 금지 계약을 우회하지 않는다.
2. cursor fast-path는 PageItem에 저장된 start/end physical row-height override를 같은
   순서로 `layout_partial_table()`에 전달한다. `None`으로 버리면 cursor geometry가
   실제 페이지 tree와 달라질 수 있다.
3. 이 적응 뒤에는 compile check를 먼저 통과시키고, #3820 focused test, 전체
   release-test, Native Skia, clippy, WASM을 rebased SHA 기준으로 다시 검증한다.

## 구현

- `src/renderer/layout/table_partial.rs`의 Center-tail visual-height 계산은
  `CellComposedStore::eager_slice()`를 사용하도록 현재 compose-store API에 맞췄다.
  이 분기는 windowed probe가 아닌 eager 조합 경로이므로 lazy/windowed compose 계약을
  무너뜨리지 않는다.
- `src/document_core/queries/cursor_rect.rs`의 PartialTable fast-path는 PageItem에
  저장된 `end_row_height_override`, `start_row_height_override`를 분해해
  `layout_partial_table()`의 현재 인자 순서대로 전달한다. 이 물리 row-height를
  버리면 cursor rectangle과 실제 page tree의 분할 표 geometry가 어긋난다.

## rebased SHA 검증 결과

- `cargo check --profile release-test` 통과.
- `cargo test --profile release-test --test issue_3820_rowbreak_rowspan_band -- --nocapture`:
  2/2 통과.
- `cargo test --profile release-test --tests`: 최종 exit 0. 이 실행 안의
  `overflow_cell_baseline`과 visual baseline도 통과했다.
- Native Skia: `--features native-skia skia --lib` 58/58,
  `issue_2225_missing_picture_placeholder` 2/2,
  `render_p37_direct_pdf_export` 4/4 통과.
- `cargo fmt --check`, `cargo clippy --profile release-test --all-targets -- -D warnings`,
  `wasm-pack build --target web --out-dir pkg` 통과. WASM 브라우저 상호작용 확인은
  사용자의 수동 검증 범위로 남긴다.

## 직접 PDF 대조 증적

`samples/76076_regulatory_analysis.hwp`와 기준 PDF의 35–36쪽을 180 DPI로 다시
비교했다. p35의 `볼 리프트` 문장은 두 줄로 끊기고 표의 다음 physical row는 p36에서
시작한다. p36의 rowspan/분할표 테두리와 row band도 양쪽에서 유지된다.

- [p35 review](../pr/assets/task_m100_3820_stage81_upstream_rebase_partial_table_api/review_035.png)
- [p36 review](../pr/assets/task_m100_3820_stage81_upstream_rebase_partial_table_api/review_036.png)
- [pixel/ink metric](../pr/assets/task_m100_3820_stage81_upstream_rebase_partial_table_api/overlay_metrics.json)
- [sweep summary](../pr/assets/task_m100_3820_stage81_upstream_rebase_partial_table_api/summary.json)

동일 문서의 SVG paint와 PDF raster는 글꼴 paint 차이 때문에 pixel/ink score가 낮게
나올 수 있다. 따라서 이 결과는 p35–36의 줄바꿈·분할표 geometry 보존 근거이며, 전체
문서의 무조건적인 pixel-equality 판정으로 사용하지 않는다.
