# Task M100 #363 Stage 4 완료보고서: 문서화 및 최종 검증

## 1. 작업 범위

구현계획서의 Stage 4 범위에 따라 native render tree bridge API의 인덱스 기준을 문서화하고 최종 검증을 수행했다.

## 2. 변경 파일

- `mydocs/tech/hwp_spec_errata.md`
- `mydocs/manual/native_render_tree_bridge_api.md`
- `mydocs/orders/20260427.md`
- `mydocs/working/task_m100_363_stage4.md`
- `mydocs/report/task_m100_363_report.md`

## 3. 문서화 내용

### 3.1 `bin_data_id` 기준 보강

`mydocs/tech/hwp_spec_errata.md`의 `bin_data_id` 섹션에 native bridge API 기준을 추가했다.

- `ImageNode.bin_data_id`는 1-based 참조값
- `DocumentCore::get_bin_data(index)`의 `index`는 0-based `bin_data_content` 배열 인덱스
- render tree의 `bin_data_id`로 bytes를 조회할 때는 일반적으로 `get_bin_data((bin_data_id - 1) as usize)` 사용

### 3.2 native bridge API 매뉴얼 추가

`mydocs/manual/native_render_tree_bridge_api.md`를 추가했다.

문서에는 다음 내용을 정리했다.

- `build_page_render_tree(page_num)` API 목적과 page index 기준
- `get_bin_data(index)` API 목적과 index 기준
- `ImageNode.bin_data_id`에서 실제 bytes를 조회하는 예시
- render tree 직렬화 시 binary payload를 제외하는 기준
- WASM `getPageRenderTree()` 기존 경로와의 관계

## 4. 검증

```bash
cargo build
cargo test
```

결과:

- `cargo build` 통과
- `cargo test` 통과
  - lib tests: 1010 passed, 1 ignored
  - integration tests: `hwpx_roundtrip_integration` 14 passed, `hwpx_to_hwp_adapter` 25 passed, `issue_301` 1 passed, `svg_snapshot` 6 passed, `tab_cross_run` 1 passed
  - doctest: 0 tests
- 기존 테스트 경고 4건이 출력되었으나 이번 변경과 무관한 기존 경고다.

## 5. 다음 단계

작업지시자 승인 후 fork branch push 및 upstream `devel` 대상 PR 생성 절차를 진행한다.
