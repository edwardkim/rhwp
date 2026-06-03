# Task #295 4단계 — 회귀 테스트 보고서

## 단위 테스트

```
cargo test --release
test result: ok. 983 passed; 0 failed; 1 ignored
test result: ok. 14 passed; 0 failed
test result: ok. 25 passed; 0 failed
test result: ok. 6 passed; 0 failed   (svg_snapshot)
total: 1028 passed, 0 failed
```

## 샘플 회귀 (LAYOUT_OVERFLOW)

| 샘플 | 결과 |
|------|------|
| `samples/exam_math.hwp` (전체) | 0건 |
| `samples/exam_math_no.hwp` (전체) | 0건 |
| `samples/equation-lim.hwp` | 0건 |
| `samples/text-align-2.hwp` | 0건 |

## 주요 페이지 시각 점검 — `exam_math.hwp`

| 페이지 | 패턴 | 결과 |
|--------|------|------|
| 1쪽 | 머리말 표(vert=Page) + 풋터 "1/20" | 정상 (회귀 없음) |
| 12쪽 | 다단 + 페이지 하단 푸터 표 + Square wrap 표 | **본 타스크 수정 대상**, 좌단 정상 흐름 복원 |

## 잔여 별건 (본 타스크 외)

- 머리말 페이지번호 4↔2 불일치 (1단계 보고서에서 분리 권고)
- 12쪽 우측 컬럼 단락 높이 과대로 동전 그림 위치 어긋남 → **이슈 #297로 분리 등록**
  - #295 수정 전·후 모두 우측 좌표 동일(147.4..497.3) — 사전 존재 버그
  - 줄 높이/line_spacing 처리 또는 인라인 수식(tac=true) 메트릭 영향 의심

## 결과

#295 수행 목표(좌단 붕괴 해소, LAYOUT_OVERFLOW 제거, Square wrap 표 호스트 본문 복원) 모두 달성. 회귀 없음.
