# Task #317 4단계 완료 보고서: 격리 테스트 재활성화 + 정리

상위: `task_m100_317_impl.md`
선행: `task_m100_317_stage3.md` (어댑터 attr=0 보강)

## 변경

### 1. 격리 테스트 `#[ignore]` 3건 제거 (`tests/hwpx_to_hwp_adapter.rs`)

- `stage4_page_count_recovered_hwpx_h_02`
- `stage5_all_three_samples_recover_via_unified_entry_point`
- `stage6_verify_recovered_for_all_three_samples`

`#[ignore = "TypesetEngine 전환(#313) 후 hwpx-h-02 어댑터 결과 +1쪽..."]` 제거.

### 2. 진단 도구 정리

- `tests/task317_diag.rs` 삭제 (3 임시 진단 테스트)
- `src/renderer/typeset.rs` 의 `RHWP_TYPESET_TRACE` env-gated trace 5블록 제거

## 검증

```
cargo test → 992 lib + 25 어댑터(0 ignored) + 통합 테스트 모두 PASS
```

| 어댑터 테스트 | 결과 |
|---------------|------|
| stage4_page_count_recovered_hwpx_h_02 | ✓ |
| stage5_all_three_samples_recover_via_unified_entry_point | ✓ |
| stage6_verify_recovered_for_all_three_samples | ✓ |

| 4샘플 (release dump-pages) | 기대 | 측정 |
|-----------------------------|------|------|
| 21_언어_기출_편집가능본 | 15쪽 | 15쪽 ✓ |
| exam_math | 20쪽 | 20쪽 ✓ |
| exam_kor | 24쪽 | 24쪽 ✓ |
| exam_eng | 9쪽 | 9쪽 ✓ |

## 산출

- `tests/hwpx_to_hwp_adapter.rs` (수정 — 3건 ignore 제거)
- `src/renderer/typeset.rs` (수정 — trace 제거)
- `tests/task317_diag.rs` (삭제)
- 본 보고서

## 다음 단계

최종 보고서 작성 (`mydocs/report/task_m100_317_report.md`) + 오늘할일 갱신.
