# 구현계획 — task_m100_4252

- **Issue**: #4252
- **수행계획**: [task_m100_4252.md](task_m100_4252.md)
- **대상 브랜치**: `fix/issue-4252-nested-table-selection-path`
- **기준 commit**: `fcc3b2135fa782699b66b583ddf11fe9f748306e`

## 1. RED 회귀 테스트

`tests/issue_4252_nested_partial_table_cell_path.rs`를 추가한다.

- 실제 HWP fixture를 `HwpDocument`로 로드하고 17개 PageRenderTree를 순회한다.
- 물리 5쪽의 `구 분` TextRun을 찾아 현재 합성 경로가 원본 IR에 resolve되지 않음을 RED로 증명한다.
- 수정 후 대표 경로가 다음 계층을 보존하는지 고정한다.

```text
section[0].paragraph[7]
  controls[1] outer table / cell[0] / paragraph[0]
  controls[2] wrapper table / cell[0] / paragraph[12]
  controls[0] child table / selected child cell
```

- 전 페이지의 중첩 표 run에서 고유 `(section, parentPara, path)`를 수집하고, 각 경로의 중간 엔트리를
  통해 셀 문단으로 이동한 뒤 마지막 컨트롤이 `Table`인지 검사한다.
- 대표 자식 표 경로 JSON을 `get_table_cell_bboxes_by_path()`에 전달해 Studio와 같은 조회가 성공하고
  기대 셀 bbox를 반환하는지 확인한다.

## 2. renderer 경로 전달

`src/renderer/layout/table_partial.rs`를 다음과 같이 정정한다.

1. `layout_partial_table()`에 `enclosing_cell_ctx: Option<&CellContext>`를 추가한다.
2. 인자를 `layout_partial_table_cells()`까지 전달한다.
3. 셀 문단 컨텍스트 생성 시:
   - enclosing context가 없으면 현재처럼 실제 최상위 `para_index/control_index`로 루트 경로를 만든다.
   - 있으면 context를 복제하고 마지막 path entry의 `cell_index`, `cell_para_index`,
     `text_direction`만 현재 자식 셀에 맞게 갱신한다.
4. 재귀 `nested_host` 부분 표 호출에는 이미 계산한 `nested_ctx.as_ref()`를 전달한다. 합성 `(0, 0)`은 자식
   `Table` 조회에만 사용한다.
5. 캡션 context도 enclosing context가 있으면 같은 실제 경로를 유지하고 caption sentinel만 적용한다.

최상위 호출부 `src/renderer/layout.rs`는 enclosing context로 `None`을 전달한다. 일반
`layout_table()` 경로와 hit-test/선택 API 계약은 변경하지 않는다.

## 3. 브라우저 hot path 제한

- `rhwp-studio/src/engine/input-handler.ts`, `cursor.ts`, `requestIdleCallback` 등록 코드는 원칙적으로
  변경하지 않는다.
- path 유효성 검사는 회귀 테스트에서 수행하고 제품의 커서 이동·선택 이벤트에 넣지 않는다.
- 함수 인자는 borrowed context를 사용해 재귀 진입 자체의 불필요한 `Vec<CellPathEntry>` 복제를 막는다.
- 수정 전·후 release WASM에서 다음을 같은 조건으로 기록한다.
  - fixture 로드부터 `initDoc` 완료까지의 반복 측정 중앙값
  - 물리 5쪽 대표 자식 표의 `getTableCellBboxesByPath()` 반복 측정 중앙값과 반환 셀 수
  - `probe-input-perf-issue3137.mjs`의 mutation·cursor update·focused repaint·long-task 계약
- 측정 산출물은 `output/4252/perf-before.json`, `output/4252/perf-after.json`에 두며 Git에는 포함하지
  않는다.

## 4. GREEN·인접 검증

- `cargo test --profile release-test --test issue_4252_nested_partial_table_cell_path`
- `cargo test --profile release-test --test issue_2007_nested_cell_pagination`
- `cargo test --profile release-test --test issue_2212_nested_cell_path_bbox`
- #4159 Canvas2D E2E 및 필요 시 #4252 WASM bbox 계약
- `npm --prefix rhwp-studio run e2e:issue-3137-perf`
- `cargo fmt --all -- --check`
- 변경 파일 대상 Clippy 또는 승인된 범위 게이트
- `git diff --check`

RED→GREEN 결과, geometry 불변과 브라우저 성능 전후 근거는
`mydocs/working/task_m100_4252_stage1.md`에 기록한다.

## 5. 사용자 재검증 후 추가 GREEN

초기 구현 뒤 실제 Studio 키 입력 로그로 드러난 두 번째 결함층은 다음 래칫과 최소 변경으로 고정한다.

- 전 17쪽 TextRun/TableCell 중심 hit-test가 반환한 경로를 원본 IR에 재적용한다: 181 RED → 0 GREEN.
- `effective_cell_context()`는 traversal context의 depth가 raw context보다 클 때만 보완한다.
- 재귀 부분 표 `TableNode`에는 실제 포함 셀 문단과 현재 table control provenance를 기록한다.
- 표만 있는 부모 셀 문단의 두 번째 `Esc`는 기존 RenderTree fallback에서 caret anchor를 찾는다.
- `table-object-selection-changed` 이벤트 뒤 직접 렌더를 제거하고 E2E에서 렌더 1회를 고정한다.
- `get_table_cell_bboxes_by_path_native()`는 `build_page_tree_cached()`를 사용해 기존 page tree를
  재사용한다. 새 순회나 조회 범위 확장은 없다.

최종 E2E는 선택·부모 caret 관련 경고 0건, 선택 renderer 1회·1.2ms, 물리 5쪽 55-cell bbox
중앙값 0.4ms를 확인한다. 변경하지 않은 idle prefetch·visible-page rAF 경로는 별도 추적 대상으로
남긴다.
