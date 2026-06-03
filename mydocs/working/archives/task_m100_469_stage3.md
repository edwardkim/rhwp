# Task #469 Stage 3 완료 보고서

## 단계 목표

전체 회귀 테스트 통과 + 골든 SVG 영향 검토.

## 검증 결과

### 전체 cargo test

```
test result: ok. 1070 passed; 0 failed; 1 ignored; 0 measured (lib unit tests)
test result: ok. 14 passed (exam_eng_multicolumn)
test result: ok. 25 passed (hwpx_roundtrip_integration)
test result: ok. 1 passed (hwpx_to_hwp_adapter)
test result: ok. 1 passed (issue_301)
test result: ok. 1 passed (issue_418)
test result: ok. 2 passed (page_number_propagation)
test result: ok. 6 passed (svg_snapshot)
test result: ok. 1 passed (tab_cross_run)
```

총 **1121 PASS / 0 FAIL**. 신규 추가 테스트 `test_469_partial_start_box_does_not_cross_col_top` 포함.

### 골든 SVG 영향

`tests/svg_snapshot.rs` 의 6개 골든 모두 통과 — 골든 SVG 갱신 불필요.

기존 워킹 디렉토리에 있던 `tests/golden_svg/*.actual.svg` 5건은 이전 작업(Task #468) 의 잔여물이며 본 변경과 무관 (해당 페이지에 cross-column partial border 박스 없음).

### 기존 clippy 오류

`cargo clippy` 의 `panicking_unwrap` 오류 2건은 `src/document_core/commands/object_ops.rs:298` 의 사전 존재 이슈 (`git stash` 후 동일 발생 확인). 본 작업 범위 외.

### 시각 확인

`samples/exam_kor.hwp` 페이지 2 SVG 재생성:

```bash
./target/release/rhwp export-svg samples/exam_kor.hwp -p 1 -o /tmp/p1_after/
```

- 우측 단 (나) 박스 세로선 y1: `196.55` → `211.65` (헤더선과 15.1 px 분리)
- 좌측 단 (가) 박스 세로선 y1: `242.41` (변동 없음 — partial_start 아님)

## 추가 영향 분석

`is_partial_end` 케이스도 함께 처리됨:
- partial_end 시 `bot_pad` 미적용 → `rect_h` 가 col_bot 아래로 확장되지 않음
- 꼬리말선과 충돌 방지 (대칭성 확보)

기존 단일 paragraph 박스(non-partial) 는 `is_partial_start = is_partial_end = false` 라 `effective_*_pad = top_pad/bot_pad` 로 동작 → 영향 없음.

## 결론

분석에서 진단한 근본 원인을 정확히 수정. 기존 회귀 0건. 다음 단계: 최종 보고서 작성.
