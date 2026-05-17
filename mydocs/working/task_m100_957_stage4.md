# Task #957 Stage 4 — 다중 sample 회귀 검증

## 1. cargo test --release --lib

**결과**: **1288 passed, 0 failed, 2 ignored**

→ golden SVG diff + 모든 unit test 회귀 0.

## 2. sample16 page 18 (Fix A 검증)

### Before fix
```
TAC_CURSOR Shape pi=394 ci=1 y_in=767.3 y_out=1197.9 dy=430.6
TAC_CURSOR FullPara pi=395 y_in=1197.9 ...
```
pi=395~401 본문이 body 외 (y=1220+) 위치 → 시각 누락

### After fix
```
TAC_CURSOR Shape pi=394 ci=1 y_in=767.3 y_out=767.3 dy=0.0 ✓
TAC_CURSOR FullPara pi=395 y_in=759.8 y_out=795.0 dy=35.3
... pi=395~401 y_out=973.6 (body 안)
```
pi=395~401 본문 같은 페이지 정상 표시 ✓

## 3. caption text 보유 sample 검증 (hwp3-sample14)

### Page 2 (다수 empty caption + non-empty "Cut&Paste 할 영역" 혼재)

| Caption type | 위치 | 결과 |
|--------------|------|------|
| Non-empty "Cut&Paste 할 영역" | image 바로 아래 | ✓ 정상 |
| Empty captions (text="" 4개) | invisible | ✓ phantom advance 제거됨 |
| 후속 본문 ("이번에는 마크블록..." 등) | caption 아래 정상 흐름 | ✓ |

→ **non-empty caption 정상 동작 + empty caption 만 advance skip** 확인.

## 4. 다른 sample 회귀 점검

| Sample | 결과 |
|--------|------|
| hwp3-sample10 page 1 | 정상 emit |
| hwp3-sample11 page 1 | 정상 emit |
| hwp3-sample13 page 1 | 정상 emit (808 texts) |
| exam_kor page 1 | 정상 emit (1497 texts) |
| exam_math page 1 | 정상 emit (241 texts) |
| hwp3-sample14 page 1, 2, 1+ | caption 정상 위치 |

## 5. PR #956 (Issue 1 외곽선 fix) 와의 호환성

본 task #957 의 fix 는 `layout_shape_item` 의 caption advance 분기 만 영향. PR #956 의 `paper_based = true` 변경 (라인 770 영역) 와 독립적. 양쪽 merge 시 충돌 없음.

## 6. 평가

- 단위 검증 (sample16 page 18): ✓
- cargo test 전체: ✓ (1288/0/2)
- 다중 sample 시각 회귀: ✓ 0
- caption text 보유 sample 회귀: ✓ 0

→ Stage 5 진행 (시각 검증 + 최종 보고서 + PR).
