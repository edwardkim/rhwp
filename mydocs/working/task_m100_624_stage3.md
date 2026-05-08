# Task #624 Stage 3 보고서

## 목적

광범위 fixture sweep 으로 회귀 0 검증 + 최종 보고서 + orders 갱신.

## 광범위 sweep 결과

`samples/` 폴더 전체 **158 fixture / 1,496 페이지** sweep:

| 카테고리 | 페이지 수 | 비율 |
|---|---|---|
| 의도된 정정 | 1 | 0.067% |
| 회귀 | **0** | **0%** |
| byte-identical | 1,495 | 99.933% |

### 의도된 정정 (1 건)

`/tmp/sweep_all_before/exam_science/exam_science_002.svg` ↔ `/tmp/sweep_all_after/exam_science/exam_science_002.svg`:

```diff
-<rect x="117.066" y="213.946" width="62.986" height="22.880" fill="#ffffff" stroke="#000000"/>
-<text x="141.56"  y="229.986" ...>㉠</text>
+<rect x="117.066" y="235.413" width="62.986" height="22.880" fill="#ffffff" stroke="#000000"/>
+<text x="141.56"  y="251.453" ...>㉠</text>
```

- ㉠ 사각형 y: 213.95 → 235.41 (Δ +21.47 px)
- ㉠ 텍스트 y: 229.99 → 251.45 (Δ +21.47 px)
- Δ = (3220 - 1610) HU / 75 = ls[1].vpos - ls[0].vpos / px-conversion 정확

### 회귀 0 검증

| 샘플 카테고리 | 페이지 수 | 변경 페이지 |
|---|---|---|
| `exam_*.hwp` (5 files) | 64 | 1 (의도된 정정) |
| `synam-001.hwp` | 35 | 0 |
| `21_언어_*.hwp` | 15 | 0 |
| `통합재정통계*.hwp` (3 files) | ~30 | 0 |
| `hwpspec-w.hwp` | ~170 | 0 |
| `aift.hwp` / `biz_plan.hwp` / `kps-ai.hwp` | ~60 | 0 |
| 기타 fixture (~140 files) | ~1,120 | 0 |
| **합계** | **1,496** | **1** |

## 최종 검증 체크리스트

- [x] Stage 1 TDD 테스트 RED 확인 (y=213.95 측정)
- [x] Stage 2 정정 적용 후 RED → GREEN
- [x] Stage 2 cargo test --lib 회귀 0 (1135 passed)
- [x] Stage 2 svg_snapshot 6/6 GREEN
- [x] Stage 2 clippy clean
- [x] Stage 2 exam_science p2 ㉠ 사각형 y 시각 측정 (235.41 ≈ 235.65)
- [x] Stage 3 158 fixture sweep — 의도된 변경 외 회귀 0
- [x] Stage 3 exam_math/exam_eng/exam_kor/synam-001/21_언어 무회귀
- [x] Stage 3 최종 보고서 + orders 갱신

## 산출물

- `mydocs/report/task_m100_624_report.md` — 최종 보고서
- `mydocs/orders/20260506.md` — 오늘 할일 갱신 (Task #624 항목 추가)
- `mydocs/working/task_m100_624_stage3.md` — 본 보고서

## 다음 단계

- 작업지시자 승인 후 `local/devel` merge → `devel` 푸시 → Issue #624 close
- 회귀 방지 후속 (선택): cherry-pick base diff 자동 점검 도입 task 제안
