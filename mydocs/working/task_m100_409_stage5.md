# Stage 5 보고서 — Task #409 v2

## 작업

1. 전체 회귀 테스트 (`cargo test --release`)
2. 6개 다른 샘플 LAYOUT_OVERFLOW 무회귀 확인
3. 통합 최종 보고서 작성 (`task_m100_409_report_v2.md`)
4. PR 초안 v1+v2 통합 업데이트 (`task_m100_409_pr.md`)

## 결과

### 전체 테스트 (cargo test --release)

11개 스위트 100% 통과 (실패 0):
```
lib:                1023 passed
svg_snapshot:       6 passed
composition_alpha:  14 passed
find_replace_engine: 25 passed
기타 7 suites:      0~25 passed each
```

### 다른 샘플 LAYOUT_OVERFLOW 무회귀

v0/v1/v2 비교 — 6개 샘플 모두 동일 (0/7/0/1/0/4):
- `biz_plan.hwp`, `exam_kor.hwp`, `exam_math.hwp`, `aift.hwp`, `k-water-rfp.hwp`, `kps-ai.hwp`

타겟 샘플: 22 → 4 → **1** (잔여 1건은 page=2 PartialParagraph, 본 작업 무관)

### 21/22 페이지 PDF 대조

| 페이지 | 항목 | v0 | v1 | v2 |
|--------|------|----|----|----|
| 21 | 2x1 표 위치 | 잘림 ❌ | 정상 ✓ | 정상 ✓ |
| 22 | (4) 헤딩 | 누락 ❌ | 누락 ❌ | **정상 ✓** |
| 22 | 10x5 표 | 누락 ❌ | 누락 ❌ | **정상 ✓** |

## 산출물

- 통합 최종 보고서: `mydocs/report/task_m100_409_report_v2.md`
- PR 초안 (v1+v2 통합): `mydocs/report/task_m100_409_pr.md`
- 본 보고서: `mydocs/working/task_m100_409_stage5.md`

## 결론

- chart 관련 overflow 전건 해소
- 6개 다른 샘플 무회귀
- 11개 테스트 스위트 100% 통과
- PR 초안 v1+v2 통합 완료

이슈 클로즈 + 머지 + push + PR 생성 진행 승인 요청.
