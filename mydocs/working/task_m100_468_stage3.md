# Task #468 Stage 3 — 시각/회귀 검증 보고서

## 개요

Stage 2 에서 구현한 cross-column 박스 연속 partial 플래그 보정 로직을 시각·테스트·회귀 측면에서 검증한다.

## 시각 검증 (exam_kor.hwp)

### 페이지 6 col 0 [18~21] passage 박스 (목표)

박스 영역: `x=128.50~529.12, y=1182.48~1417.58`

```
Top    (y=1182.48)  : ✓ stroke 존재 (박스 시작)
Right  (x=529.12)   : ✓ stroke 존재
Left   (x=128.50)   : ✓ stroke 존재
Bottom (y=1417.58)  : ✗ stroke 미렌더 — 목표 달성 (PDF 일치)
```

수정 전: 4면 모두 stroke → 박스가 닫혀 보임  
수정 후: 3면 stroke + 하단 개방 → cross-column 연속 시각화 ✓

### 페이지 6 col 1 박스 (cross-column 연속 검증)

박스 영역: `x=593.38~994.00, y=196.54~1418.04`

```
Top    (y=196.54)  : ✗ stroke 미렌더 (col 0 ← 연속, partial_start)
Bottom (y=1418.04) : ✗ stroke 미렌더 (col 1 → 페이지 7 연속, partial_end)
Left/Right         : ✓ stroke 존재
```

→ 양방향 partial 보정 정상 작동 ✓

### 페이지 7 col 0 박스 (페이지 6 → 7 연속 검증)

박스 영역: `x=128.50~529.12, y=196.54~602.72`

```
Top    (y=196.54)  : ✗ stroke 미렌더 (페이지 6 col 1 ← 연속)
Bottom (y=602.72)  : ✓ stroke 존재 (박스 닫힘)
Left/Right         : ✓ stroke 존재
```

→ 페이지 경계 연속 partial_start 보정 정상 ✓

### 페이지 14 (Task #463 Stage 2 회귀 검증)

페이지 14 stroke-width=0.5 line 27 개. 단일 박스 정상 닫힘 유지 (회귀 0).

## 단위/스냅샷 테스트

```
cargo test --release --lib       : 1069 passed; 0 failed; 1 ignored
cargo test --release --test svg_snapshot : 6 passed; 0 failed
```

## 회귀 검증 (5종 샘플)

| 샘플 | 페이지 수 | 결과 |
|------|----------|------|
| 2010-01-06.hwp | 6 | ✓ 정상 |
| biz_plan.hwp | 6 | ✓ 정상 |
| 21_언어_기출_편집가능본.hwp | 15 | ✓ 정상 |
| exam_eng.hwp | 8 | ✓ 정상 |
| 2022년 국립국어원 업무계획.hwp | 40 | ✓ 정상 |

## 결론

- 목표 달성: exam_kor.hwp 6p 좌측 단 [18~21] passage 박스 하단 stroke 미렌더 (PDF 일치)
- 추가 효과: cross-column 박스 연속 (col→col, page→page) 시각화 정합 확보
- 회귀 0: 단위 1069 + 스냅샷 6 + 5종 샘플 모두 통과
