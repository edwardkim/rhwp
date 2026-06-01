# Task #1154 Stage 1 완료 보고서 — 진단 정밀화 + baseline 기록

## 1. 목표

- 동일 `bin_data_id` Pic 컨트롤 페어 패턴이 사용되는 모든 sample 식별
- 실제 SVG 출력에서 nested image 가 시각적으로 겹치는 케이스만 추출
- 영향 sample 의 baseline SVG / PNG 보존
- 보호해야 할 의도적 효과(그림자/2중 노출) 케이스와 잔상 케이스 구분

## 2. 1차 IR 스캔 (244 samples → 28 케이스)

`/tmp/scan_bin_id_pairs.py` 로 IR dump 의 같은 문단 내 동일 `bin_id` Pic 컨트롤 2 회 이상 케이스 검출:

| Sample | Para | bin_id | Count |
|---|---|---|---|
| 3-10월_교육_통합_2022.hwp/.hwpx | 0.296 | 7,8/17,18 | 2 |
| KTX.hwp | 0.245 | 3 | 4 |
| basic/BlogForm_BookReview.hwp | 0.0/0.1 | 1 | 7,4 |
| basic/BlogForm_MovieReview.hwp | 0.0 | 1 | 7 |
| basic/BlogForm_Recipe.hwp | 0.0/0.1 | 1,2 | 7,3 |
| **exam_eng.hwp** | 0.104 | 5 | 2 |
| exam_social.hwp | 0.15 | 1 | 2 |
| hwpctl_ParameterSetID_Item_v1.2.hwp | 0.7 | 3-8 | 2 each |
| hwpspec.hwp | 2.52 | 4,5,6 | 2,3,17 |
| pic-in-head-01.hwp / pic-in-table-01.hwp | 0.58 | 3 | 4 |
| pic2-2018.hwp / pic2.hwp / pic2.hwpx | 0.0 | 1,2 | 2 |
| test-image.hwp / .hwpx | 0.0 | 1 | 4 |

총 17 sample / 28 (sample, para, bin_id) 페어.

## 3. 2차 정밀 검증 (실제 SVG 겹침)

IR 스캔은 단순 "같은 문단 내 같은 bin_id 가 2 회 이상" 검출이지만, 실제 렌더링 시점에 시각적 겹침이 있는지는 별개. baseline SVG 생성 후 `/tmp/find_real_overlaps.py` 로 nested `<svg>` / `<image>` 의 bbox 겹침을 검사한 결과 **단 7 SVG 의 25 페어**만 실제 겹침:

| SVG | 페어 수 | 패턴 |
|---|---|---|
| **exam_eng_002.svg** | **1** | **x/width 동일, 세로만 인접 겹침 (대상 이슈)** |
| 3-10월_교육_통합_2022_hwp/010.svg | 6 | x 대각선 오프셋 (의도적) |
| 3-10월_교육_통합_2022_hwp/011.svg | 3 | x 대각선 오프셋 (의도적) |
| 3-10월_교육_통합_2022_hwpx/010.svg | 6 | x 대각선 오프셋 (의도적) |
| 3-10월_교육_통합_2022_hwpx/011.svg | 3 | x 대각선 오프셋 (의도적) |
| test-image_hwp/test-image.svg | 3 | x 다름 (의도적 다중 배치) |
| test-image_hwpx/test-image.svg | 3 | x 다름 (의도적 다중 배치) |

### exam_eng.hwp page 2 의 페어 (대상)

```
LOWER (z=2, drawn first): bbox=(597.15, 243.59, 1005.34, 499.68)
                          x=597.15, width=408.19, height=256.09
                          viewBox=(0, 0, 2532, 1612.77) — src px 0-1612.77
UPPER (z=3, drawn after): bbox=(597.15, 463.17, 1005.34, 533.17)
                          x=597.15, width=408.19, height=70
                          viewBox=(0, 1412.77, 2532, 434.43) — src px 1412.77-1847.20
```

핵심: **x 동일 (597.15) + width 동일 (408.19)** + 세로 겹침 (y 463.17-499.68).

### 의도적 효과 케이스 (보호 대상)

```
test-image.hwp:
  LOWER: bbox=(113.4, 132.3, 325.4, 334.7)  x=113.4, w=212.0
  UPPER: bbox=(242.5, 132.3, 454.5, 334.7)  x=242.5, w=212.0
  → x 다름 (offset 129px), 의도적 다중 배치

3-10월_교육_통합_2022.hwp page 10:
  LOWER: bbox=(736.0, 834.7, 1005.3, 941.4)
  UPPER: bbox=(744.2, 852.7, 1013.5, 959.4)
  → x 살짝 다름 (offset 8.2px), y 도 다름 → 대각선 오프셋 (그림자 효과 추정)
```

## 4. Algorithm 조건 결정 (Stage 2 의 사전 정리)

위 분석에 기반하여 clip algorithm 조건을 **strict** 하게 설정해야 함:

```
모두 만족 시 clip 적용:
1. A.bin_data_id == B.bin_data_id
2. |A.x - B.x| <= 1.0 (수평 위치 동일)
3. |A.width - B.width| <= 1.0 (수평 폭 동일)
4. max(A.y, B.y) < min(A.y+A.height, B.y+B.height) (세로 겹침)
5. A 가 트리 순서상 먼저 (z 작음), A.y < B.y (A 가 위)
```

이 strict 조건은 **exam_eng.hwp 의 정확한 케이스만 매칭**하고, test-image / 3-10월_교육_통합 등 의도적 효과를 그대로 보존.

## 5. Baseline 산출물

- SVG: `output/svg/task1154_baseline/<sample>_<ext>/<file>.svg` (영향 17 sample)
- PNG (영향 페이지 4 개): `output/svg/task1154_baseline/png/`
  - `exam_eng_hwp_exam_eng_002.png`
  - `3-10월_교육_통합_2022_hwp_3-10월_교육_통합_2022_010.png`
  - `3-10월_교육_통합_2022_hwp_3-10월_교육_통합_2022_011.png`
  - `test-image_hwp_test-image.png`

## 6. 다음 단계 (Stage 2)

- `PageRenderTree::clip_overlapping_same_bin_images()` 함수 시그니처 동결
- 위 5 조건 algorithm 의 단위 테스트 우선 작성 (TDD)
- 단위 테스트 commit 만 분리 (구현 미포함)

## 7. 산출물 / 도구

- IR 스캔 스크립트: `/tmp/scan_bin_id_pairs.py`
- 실제 겹침 검출 스크립트: `/tmp/find_real_overlaps.py`
- 실행 로그: `/tmp/scan_results.log`, `/tmp/real_overlaps.log`

승인 후 Stage 2 진행.
