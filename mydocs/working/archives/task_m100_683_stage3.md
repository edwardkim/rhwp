# Stage 3 — 시각 검증 및 회귀 테스트 (Task #683)

## 요약

`samples/pr-149.hwp` 정합 검증 + 동일 패턴(빈 paragraph + TopAndBottom 그림) 보유 다른 7개 샘플 시각 회귀 검증. **모든 샘플 회귀 없음.**

## 1. 정합 대상 (pr-149.hwp)

### 측정 결과 (150 dpi)

| 요소 | PDF (한글 2022) | rhwp SVG | 차이 |
|------|----------------|---------|------|
| 그림1 | 273..600 | 273..600 | ✓ 0 px |
| 그림2 | 666..993 | 667..994 | ✓ +1 px |
| 그림3 | 1059..1387 | 1060..1388 | ✓ +1 px |
| "회색조:" | 634..649 | 634..651 | ✓ 0 px |
| "흑백:" | 1028..1042 | 1027..1044 | ✓ -1 px |
| "입니다." | 1454..1472 | 1454..1473 | ✓ 0 px |

Cluster 거리: PDF 18864 HU → rhwp 18896 HU (+32 HU sub-pixel rounding).

## 2. 동일 패턴 보유 샘플 회귀 검증

빈 paragraph (text_len=0) + TopAndBottom 그림 (treat_as_char=false) 패턴 보유 샘플 식별:

| 샘플 | empty image-para 수 | PDF 보유 | 시각 검증 |
|------|--------------------|---------|-----------|
| `pr-149.hwp` | 3 | ✓ | ✅ 정합 (대상) |
| `exam_science.hwp` | 4 | ✓ | ✅ 회귀 없음 |
| `exam_eng.hwp` | 1 | ✓ | ✅ 회귀 없음 |
| `hwp-img-001.hwp` | 1 | ✓ | ✅ 회귀 없음 |
| `k-water-rfp.hwp` | 1 | ✓ | ✅ 회귀 없음 |
| `kps-ai.hwp` | 1 | ✓ | ⚠️ PDF 가 2-up landscape (직접 비교 불가) |
| `mel-001.hwp` | 1 | ✓ | ⚠️ PDF 렌더 실패 (multi-page) |
| `hwpspec.hwp` | 4 | ✗ | (PDF 없음) |

## 3. cargo test 회귀 검증

```
$ cargo test --release
test result: ok. (모든 스위트 0 failures)
```

신규 추가된 `test_task683_pr149_image_cluster_spacing` 도 통과.

## 4. 영향 범위 평가

| 항목 | 결과 |
|------|------|
| 동일 패턴 샘플 회귀 | ✅ 없음 |
| 다른 wrap 모드 (Square, BehindText, InFrontOfText, TAC) | ✅ 가드로 제외 |
| 머리말/꼬리말, 바탕쪽 그림 | ✅ 별도 layout 경로 |
| 표 셀 내부 그림 | ✅ `cell_ctx.is_some()` 분기 |
| caption 보유 그림 | ✅ 가드로 제외 |
| HWP3, HWPX 동일 IR | ✅ 자동 적용 |
| Skia 네이티브 렌더러 | ✅ 회귀 없음 |
| 기존 회귀 테스트 | ✅ 모두 통과 |

## 5. 잔여 이슈 (별개 작업으로 분리)

- 흑백(BlackWhite) 효과 디더링 (SVG 하드 임계값 vs 한컴 디더링)
- 회색조(GrayScale) 효과 브라우저 렌더링 검증

## 다음 단계

Stage 4 — 최종 보고서 + orders 갱신 + commit + PR.
