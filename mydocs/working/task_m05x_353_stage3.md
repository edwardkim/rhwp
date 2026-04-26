# 단계 3 완료 보고서 — Issue #353

## 한 일

- `src/renderer/typeset.rs::finalize_pages` (TypesetEngine, 기본 경로)
  - 잘못된 NewNumber 적용 블록 제거 (`if nn_pi <= fp { page_num = ... }`)
  - `PageNumberAssigner` 호출로 치환
  - 말미의 `page_num += 1` 도 Assigner 내부 책임으로 이관
- `src/renderer/pagination/engine.rs::finalize_pages` (RHWP_USE_PAGINATOR=1 경로)
  - 동일하게 `PageNumberAssigner` 로 치환
  - `prev_page_last_para` / `page_num_counter` 지역 변수 제거

## 검증

회귀 테스트 (typeset 기본 경로):
```
test gugeo_업무계획_post_new_number_monotonic ... ok
test gugeo_업무계획_max_page_number_close_to_count ... ok
```

회귀 테스트 (RHWP_USE_PAGINATOR=1 경로):
```
RHWP_USE_PAGINATOR=1 cargo test --release --test page_number_propagation
test gugeo_업무계획_post_new_number_monotonic ... ok
test gugeo_업무계획_max_page_number_close_to_count ... ok
```

페이지 시퀀스 (35페이지):
- 변경 전: `[1, 2, 3, 1, 1, 1, ..., 1]`
- 변경 후: `[1, 2, 1, 2, 3, 4, ..., 33]`
  (page 3 에서 NewNumber=1 트리거 — PDF 의 본문 첫 쪽 "- 1 -" 과 일치)
