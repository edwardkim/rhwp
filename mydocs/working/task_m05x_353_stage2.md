# 단계 2 완료 보고서 — Issue #353

## 한 일

- `src/renderer/page_number.rs` 신설 — `PageNumberAssigner`
  - `new(new_page_numbers, initial)` / `assign(page) -> u32` / `next_counter()`
  - consumed: `HashSet<usize>` 로 NewNumber 1회만 적용
  - "처음 등장" 판정: PartialParagraph(start_line==0), PartialTable(!is_continuation), 그 외(FullParagraph/Table/Shape) 항상 인정
- `src/renderer/mod.rs` 에 `pub mod page_number;` 추가
- 단위 테스트 6건 작성

## 검증

```
test renderer::page_number::tests::no_new_number_means_monotonic_from_initial ... ok
test renderer::page_number::tests::new_number_applied_once_then_monotonic ... ok
test renderer::page_number::tests::partial_paragraph_first_split_triggers ... ok
test renderer::page_number::tests::partial_paragraph_non_first_split_does_not_trigger ... ok
test renderer::page_number::tests::partial_table_continuation_does_not_trigger ... ok
test renderer::page_number::tests::multiple_new_numbers_each_consumed_once ... ok
test result: ok. 6 passed; 0 failed
```
