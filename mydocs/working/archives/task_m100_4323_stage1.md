# task_m100_4323 Stage 1 — 셀 합치기 후 병합 셀 문단 재배치

- **이슈**: [#4323](https://github.com/edwardkim/rhwp/issues/4323)
- **PR**: [#4363](https://github.com/edwardkim/rhwp/pull/4363)
- **브랜치**: `fix/issue-4323-merge-cell-reflow`
- **분기 기준**: `upstream/devel` `e48fe8694`
- **상태**: 로컬 전체 검증 통과, PR 게시
- **기록일**: 2026-08-09 KST

## 1. 결함

셀을 합치면 주 셀 폭이 흡수된 셀들만큼 넓어진다. 그런데 `merge_table_cells_native`
(`table_ops.rs:637`)는 `recompose_section` 과 `paginate_if_needed` 만 부르고
`reflow_cell_paragraph` 를 부르지 않았다.

`compose` 는 저장된 `line_segs` 를 그대로 쓰므로, 병합 셀 텍스트가 **합치기 전 좁은 폭 그대로**
줄바꿈된 채 남는다.

## 2. 형제 명령과의 대조

폭을 바꾸는 형제는 모두 부른다:

| 명령 | 위치 | reflow 호출 |
|---|---|---|
| `set_table_column_widths_native` | `table_ops.rs:2211-2234` | 있음 |
| `resize_table_cells_native` | `:2172` 부근 | 있음 |
| `merge_table_cells_native` | `:637` | **없음** |

`split_table_cell_native` 는 별도 헬퍼 `reflow_stale_cells_after_split` 를 쓰는데, 그건 "너무
넓어진 세그먼트만" 거르는 필터라 폭이 **넓어지는** 병합에는 그대로 쓸 수 없다.

## 3. 구현 — 형제와 같은 규약

`recompose_section` 호출 직전에 병합 결과 셀의 모든 문단에 대해 `reflow_cell_paragraph` 를 부른다.
호출 순서(reflow → `raw_stream = None` → `recompose_section` → `paginate_if_needed`)는 형제와
동일하다.

한 가지만 다르다 — `Table::merge_cells` 가 병합 뒤 `sort_by_key((row, col))` 로 셀을 재정렬하므로
옛 인덱스가 무효다. 주 셀을 `(row == start_row && col == start_col)` 로 다시 찾는다. 새 방식을
발명한 것이 아니라 병합의 인덱스 재배치 특성에 맞춘 조회 방식 차이다.

형제가 전체 셀을 도는 것과 달리 주 셀 하나만 도는데, 병합에서는 흡수된 셀이 제거되고 나머지 폭은
바뀌지 않으므로 최소이자 정확하다. `table.dirty = true` 는 이미 세우고 있어 인접 결함이 없다.

## 4. 검증 (완료)

- `tests/issue_4323_merge_cell_reflow.rs` 신설 2건. 좁은 폭(4200 HWPUNIT) 1×3 표 첫 셀에 42자를
  넣어 14줄을 만들고 (0,0)~(0,2) 병합(폭 3배) 후 줄 수 감소를 확인, 병합 범위 밖 셀 불변도 확인.
- 수정 전 코드에서 `14 -> 14` 로 실패함을 확인했다.
- 인접 회귀(`issue_4138_split_cell_stale_linesegs`, `issue_2342_cell_merge_para_meta`,
  `issue_2724_passthrough_invalidation_guard`) 통과.
- `cargo test --profile release-test --tests` 전체 통과.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` 통과.

남은 미래 조건은 GitHub Actions 와 작업지시자 승인, merge 다.
