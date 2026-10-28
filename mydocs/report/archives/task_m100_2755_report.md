# Task M100 #2755 결과보고서 — 표 셀 by_path 편집 경로의 셀 폭 리플로우 누락

- 이슈: [#2755](https://github.com/edwardkim/rhwp/issues/2755)
- 브랜치: `task/m100-2755-cell-bypath-reflow`
- 기준 브랜치: `origin/devel`
- 대상 파일: `src/document_core/commands/text_editing.rs`, `src/document_core/commands/formatting.rs`
- 작성일: 2026-07-22

## 1. 문제와 유입 경로

표 셀 편집이 flat `*_in_cell_native` 계열에서 `*_in_cell_by_path` 계열로 라우팅을 옮기면서,
flat 계열이 가지고 있던 **셀 폭 기준 LineSeg 리플로우**가 `by_path` 계열에는 없어 편집 후 줄
나눔이 갱신되지 않는다.

- `e5c81421`(#2452): `DeleteSelectionCommand` 를 `deleteRangeInCellByPath` 로, 분기 조건은
  `isCell`(= `parentParaIndex !== undefined`, 즉 모든 셀).
- `6399dc3e`(#2453): `ApplyCharFormatCommand.execute`/`restoreCharShapeIds` 를
  `applyCharFormatInCellByPath`/`setCharShapeIdInCellByPath` 로, 역시 `isCell`.

두 PR이 고친 결함은 중첩 셀(`isNestedCell`, depth>1)의 축 불일치인데 분기 조건으로 `isCell` 을
써서 **깊이 1 셀까지 전부** `by_path` 로 넘어갔다. `6399dc3e` 본문의 근거
"셀 리플로우는 rebuild_section 전체 재조판이 담당" 은 코드상 성립하지 않는다(§2).

## 2. 반증 사슬 (현재 devel 에서 직접 확인)

1. `compose_lines`(`src/renderer/composer.rs:472`)는 `line_segs` 가 비어 있을 때만 45자 휴리스틱을
   쓰고, 있으면 저장된 경계로 줄을 나눈다(`:549-563`). → `line_segs` 미갱신 시 줄 수 불변.
2. `recompose_for_cell_width`(`composer.rs:1633`)는 권위 `line_segs` 앞에서 즉시 반환(`:1645-1647`)
   하고, `para` 가 `&Paragraph`(불변)이라 애초에 편집 결과를 되돌릴 수 없다. 이 함수는 셀 높이 측정
   (`height_measurer.rs`)·표 레이아웃(`table_layout.rs`) 양쪽에서 쓰인다.
3. `rebuild_section`(`queries/rendering.rs:4760`) → `recompose_section` → `compose_section`
   (`composer.rs:108`)은 `section.paragraphs` 만 순회한다. 셀 문단은 대상이 아니다.
   `delete_range_in_cell_by_path` 는 `rebuild_section` 을 호출조차 안 하고 `mark_section_dirty`
   (`rendering.rs:2785`, 주석: "재조판 없이")만 한다.
4. 편집 경로에서 셀 문단 `line_segs` 를 쓰는 유일한 기록자는 `reflow_line_segs`
   (`composer/line_breaking.rs:1056`)이고, 셀 편집에서 이를 호출하는 헬퍼는
   `reflow_cell_paragraph`(`text_editing.rs:1136`)뿐이다.
5. 같은 저장소의 flat `apply_char_format_in_cell_native`(`formatting.rs:1103`)은
   `[자체 발견]` 주석과 함께 `reflow_cell_paragraph` **와** `rebuild_section` 을 둘 다 호출한다.
   `rebuild_section` 하나로 충분하다면 이 코드는 존재할 이유가 없다.

## 3. 결함 목록 및 수정

| # | 심각도 | 위치 | 결함 | 수정 |
|---|---|---|---|---|
| 1 | High | `delete_range_in_cell_by_path` (`text_editing.rs`) | 리플로우 없음 | 삭제 후 `reflow_cell_paragraph_by_path(start_para)` |
| 2 | High | `apply_char_format_in_cell_by_path` / `set_char_shape_id_in_cell_by_path` (`formatting.rs`) | 리플로우 없음 | 깊이1 위임 + 깊이≥2 리플로우(apply 는 `char_shape_mods_affect_text_flow` 게이팅) |
| 3 | Medium | `delete_text_in_cell_by_path` | 깊이1 위임 가드 없음 | 깊이1 `delete_text_in_cell_native` 위임 + 깊이≥2 리플로우+vpos |
| 4 | Low | `split_/merge_paragraph_in_cell_by_path` | 빈 경로 `path.last().unwrap()` 패닉 | `Err` 반환 |
| root | High | 모든 by_path 변형 | 깊이≥2 최내곽 셀 리플로우 불가 | `reflow_cell_paragraph_by_path` 신설 |
| 5 | Medium | `split_/merge_paragraph_in_cell_by_path` | 리플로우 없음(깊이≥2 유일 도달) | flat split/merge 처럼 리플로우 후 shift/recalc |

### 수정 설계

- **공유 헬퍼**: 컨트롤+cell_idx → 셀 폭·패딩을 해석하는 `cell_metrics_for_control` 를 추출해
  flat `reflow_cell_paragraph` 와 신설 `reflow_cell_paragraph_by_path` 가 공유한다(중복 제거).
- **깊이≥2 리플로우**: `resolve_innermost_cell_metrics` 가 path 사슬을 따라 **최내곽** 셀 폭을
  해석하고, `reflow_cell_paragraph_by_path` 가 그 폭으로 최내곽 문단을 재래핑한다. 깊이 1 에서는
  flat 형제와 동일 결과.
- **깊이 1 처리 방식 차이(의도적)**:
  - 단일 문단 연산(`delete_text`/`apply`/`set`)은 flat native 로 **위임**(모든 컨테이너 처리,
    `insert_text_in_cell_by_path:3389` 위임 선례와 동형).
  - `delete_range` 는 위임하지 않고 by_path 본문 + 리플로우로 처리한다. flat
    `delete_range_native` 의 다중 문단 분기는 `get_cell_mut` 이 **표 전용**이라, 최상위
    글상자/그림 캡션의 다중 문단 선택 삭제(hit-test 가 length-1 cellPath+isTextBox 를 주므로
    도달 가능)를 `Err` 로 만든다. by_path 본문은 세 컨테이너를 모두 처리하므로 위임 대신
    리플로우만 보강해 회귀를 피했다.
- **split/merge**: flat 형제 순서(reflow → shift_vpos_origin → recalc)를 그대로 이식.
  리플로우가 `line_segs` 를 재작성하므로 shift/recalc 를 리플로우 뒤로 옮겼다.
- **paint-only 보존**: `apply_char_format_in_cell_native` 의 게이팅을 그대로 타므로 밑줄/색 변경은
  리플로우하지 않는다. `paint_only_char_shape_changes_do_not_require_text_reflow` 통과 확인.

## 4. 왜 기존 테스트가 못 잡았는가

기존 `by_path` 테스트(`delete_range_in_cell_by_path_deletes_within_resolved_cell` 등)는 대상 축만
검증하고 줄 나눔은 보지 않았으며, 픽스처가 `Paragraph::default()` 라 `line_segs` 가 비어 §2-2
early-return 조건에 걸리지 않았다. 실제 파일에서 파싱한 셀 문단은 권위 `line_segs` 를 가진다.
본 PR 은 flat 쪽에 이미 있던 `cell_reflow_width_tests::core_with_narrow_cell`(셀 폭 200 HWPUNIT +
권위 `line_segs`) 픽스처를 by_path 계열에 처음으로 적용하고, 깊이 2 중첩 픽스처를 추가했다.

## 5. 검증 (red→green 실측)

증거 수준: 본 결함의 런타임 근거는 아래 Rust 테스트의 red→green 이다(CLI 편집 명령·빌드된 wasm
부재로 브라우저 왕복은 미수행).

### RED

- **1차(수정 전 코드, 테스트만 존재)** — 깊이 1 위임/빈 경로 테스트 5건 전부 실패:
  ```
  delete_range_in_cell_by_path_reflows_depth1_cell_line_segs ... FAILED (left: 2, right: 1)
  delete_text_in_cell_by_path_reflows_depth1_cell_line_segs  ... FAILED (left: 2, right: 1)
  char_format_by_path_reflow_uses_cell_width...              ... FAILED (실제 1줄)
  set_char_shape_id_by_path_reflow_uses_cell_width...        ... FAILED (실제 1줄)
  cell_paragraph_ops_by_path_reject_empty_path_with_error   ... FAILED (panic @ path.last().unwrap())
  test result: FAILED. 0 passed; 5 failed
  ```
- **2차(깊이≥2 헬퍼 `reflow_cell_paragraph_by_path` 무력화)** — 리플로우 의존 7건 실패,
  위임 기반 깊이1 3건은 flat native 경유라 통과:
  ```
  delete_range_in_cell_by_path_reflows_depth1_cell_line_segs        ... FAILED (left: 2, right: 1)
  delete_range_in_nested_cell_by_path_reflows_inner_cell            ... FAILED
  delete_text_in_nested_cell_by_path_reflows_inner_cell             ... FAILED
  split_paragraph_in_nested_cell_by_path_reflows_inner_cell         ... FAILED (실제 1줄)
  merge_paragraph_in_nested_cell_by_path_reflows_inner_cell         ... FAILED (실제 1줄)
  char_format_by_path_reflow_reaches_nested_inner_cell             ... FAILED (실제 1줄)
  set_char_shape_id_by_path_reflow_reaches_nested_inner_cell       ... FAILED (실제 1줄)
  (통과: delete_text_d1, char_format_d1, set_d1 — flat native 위임)
  test result: FAILED. 3 passed; 7 failed
  ```

### GREEN

수정 적용 후 신규 12건 + 핀 6건 모두 통과:
```
test result: ok. 18 passed; 0 failed; 0 ignored; 2475 filtered out
```
`paint_only_char_shape_changes_do_not_require_text_reflow` 포함 통과.

### CI

- `cargo clippy --all-targets --profile release-test -- -D warnings`: (아래 커밋 시점 기록)
- `cargo test --tests --profile release-test --no-fail-fast`: (기록)
- 변경 `.rs` `rustfmt --edition 2021` 후 `git diff --name-only`: (기록)

## 6. 범위 밖 (잔여)

- **`rhwp-studio` 라우팅 정리**: Rust 쪽 깊이1 처리가 들어가 `isCell`→`isNestedCell` 축소는
  기능상 불필요해졌으나, 불필요한 by_path 우회를 없애는 정리 가치는 남는다(스튜디오는 형제
  에이전트 소유 — 본 PR 범위 밖).
- **글상자/그림 캡션의 깊이≥2 중첩**: `reflow_cell_paragraph_by_path` 는 표/글상자/캡션 폭을 모두
  해석하나, 테스트는 표 중첩(가장 흔한 경로)만 커버했다.
