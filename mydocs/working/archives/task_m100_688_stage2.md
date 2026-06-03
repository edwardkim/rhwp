# Task #688 2단계 완료보고서 — 광범위 회귀 검증

## 검증 1: `cargo test` 전체

```
test result: ok. 1119 passed; 0 failed; 1 ignored
test result: ok. 14 passed; 0 failed
test result: ok. 25 passed; 0 failed
test result: ok. 9 passed; 0 failed
test result: ok. 8 passed; 0 failed
test result: ok. 6 passed; 0 failed (svg_snapshot)
test result: ok. 3 passed; 0 failed
test result: ok. 2 passed; 0 failed
test result: ok. 1 passed; 0 failed (다수)
```

총 1192+ 테스트 모두 통과. 0 failed. SVG 스냅샷 테스트 6개도 회귀 없음.

## 검증 2: 기본 7개 샘플 SVG diff (`scripts/svg_regression_diff.sh`)

```
2010-01-06: total=6 same=6 diff=0
aift: total=77 same=77 diff=0
exam_eng: total=8 same=8 diff=0
exam_kor: total=20 same=20 diff=0
exam_math: total=20 same=20 diff=0
exam_science: total=4 same=4 diff=0
synam-001: total=35 same=35 diff=0
TOTAL: pages=170 same=170 diff=0
```

회귀 0건.

## 검증 3: 광범위 — `samples/` 직속 .hwp + .hwpx 전체

```
Targets: 159
TOTAL: pages=1502 same=1499 diff=3
diff list:
  exam_social__hwp/exam_social_001.svg
  table-vpos-01__hwp/table-vpos-01_005.svg
  table-vpos-01__hwpx/table-vpos-01_005.svg
```

159 샘플 / 1502 페이지 / **diff 3건**.

### diff 3건 분석

| # | 파일 | 분류 | 사유 |
|---|------|------|------|
| 1 | `table-vpos-01__hwpx/_005.svg` | **의도** | 본 타스크 대상 결함 수정 (페이지 5 nested 11×3 그리드 복원) |
| 2 | `table-vpos-01__hwp/_005.svg` | **의도** | 위와 동일 결함 — HWP 변환본도 같은 IR 거치므로 동일 변경 |
| 3 | `exam_social__hwp/_001.svg` | **자연 해소 (회귀 아님)** | 아래 분석 |

#### diff #3: `exam_social.hwp` 페이지 1 자연 해소 분석

- 변경 패턴 (head -200 diff):
  - body-clip width: `1240.12 → 1251.4533` (+11.33px)
  - 셀별로 분리되어 있던 cell-clip 들이 단일 width=411.92 클립으로 통합

- `dump samples/exam_social.hwp -s 0 -p 1` 결과:
  - 문단 0.1 의 1×1 표 (411.9×197.0px, tac=true) 셀[0] paras=**3** ("뜨거워진 한반도, 과일 재배 지도가 바뀐다!" 등)
  - 우리 수정 조건의 정확한 케이스: paragraphs ≠ 1 → unwrap 안 됨

- 수정 전 동작: 외부 표 unwrap → 첫 nested 표만 가져옴 → paragraphs 의 다른 콘텐츠 누락 가능성
- 수정 후 동작: 외부 표 정상 렌더 → 셀 paragraphs 3개 모두 처리 → 콘텐츠 추가 (SVG +2.6KB)
- cell-clip 통합과 width=411.92 (외부 표 권위 width 와 일치) 패턴 → 외부 표 정상화 직접 신호

결론: 본 타스크 수정의 직접 부산 효과. 회귀 아님. PDF 권위본 미보유로 시각 검증은 불가능하나 변경 방향이 정상화.

## DoD 충족 현황 갱신

| DoD | 항목 | 상태 |
|-----|------|------|
| 1 | pi=34 외부 표 외곽 그려짐 | ⚠️ height=57.72px 잔여 의문 — 단계 3 시각 검증 |
| 2 | nested 1×1 헤더 텍스트 | ✅ |
| 3 | nested 11×3 그리드 텍스트 | ✅ |
| 4 | PDF 권위본과 시각 정합 | ⏳ 단계 3 |
| 5 | **회귀 없음** | ✅ — 1502페이지 중 의도된 변경 2건 + 자연 해소 1건, 회귀 0건 |
| 6 | **`cargo test` 통과** | ✅ — 1192+ 테스트 전부 통과 |

## 산출물

- 회귀 검증 결과: `/tmp/svg_diff_full_before/`, `/tmp/svg_diff_full_after/` (1502페이지 SVG)
- 비교 스크립트: `/tmp/regr_full.sh` (광범위 검증용 hand-roll)

## 다음 단계

단계 3: 보조 관찰 측정 (페이지 1 LAYOUT_OVERFLOW 자연 해소 / 페이지 2~3 hwp_used diff) + PDF 시각 정합 검증 + 최종 결과보고서
