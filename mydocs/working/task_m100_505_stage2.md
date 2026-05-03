# Task M100 #505 Stage 2 완료 보고서

## 작업 내용

신규 통합 회귀 테스트 `tests/issue_505.rs` 작성. all-in-one-parser fixture 의 4개 CASES+EQALIGN 중첩 스크립트를 영구 회귀 셈플로 등록.

### 신규 테스트 4건

| 테스트 | 검증 내용 |
|--------|-----------|
| `issue_505_cases_eqalign_height_ratio` | 4 fixture 각각 layout height vs HWP 권위 height 의 scale_y 가 [1/1.30 .. 1.30] 범위 |
| `issue_505_pi165_fraction_recognized` | pi=165 (분수 있음) 의 layout height 가 pi=151 (분수 없음) 보다 ≥10 px 큼 |
| `issue_505_no_internal_overlap` | 4 fixture 모두 CASES 의 인접 행 쌍이 y-overlap 없음 (PR #396 의 row[0]/row[1] 한정 검사를 일반화) |
| `issue_505_eqalign_no_leading_newline_text` | tokenizer skip_spaces 가 \n 을 건너뛰어 EqAlign 행이 Text("\n") 잡음 없음 |

### 측정 결과

```
=== Issue #505 layout vs HWP probe ===
pi=151:  layout=(179.60, 40.34)  HWP=(151.09, 46.87)  scale_y=1.1617
pi=165:  layout=(215.17, 61.47)  HWP=(201.83, 66.27)  scale_y=1.0781
h_diff(layout) = 21.12    h_diff(HWP) = 19.40
```

본 측정에서 `h_diff(layout)` 21.12 vs `h_diff(HWP)` 19.40 — 정정 후 layout 의 fraction 인식이 HWP 권위와 매우 가까운 height 차이 (오차 1.7 px) 를 산출.

## 검증

- `cargo test --test issue_505` — 4 passed, 0 failed
- `cargo test --test issue_418` — 1 passed (회귀 0)
- `cargo test --test issue_501` — 1 passed (회귀 0)
- 전체 `cargo test --lib` 1104 통과

## 비고

svg_snapshot 5/6 실패 발견 — **본 정정과 무관한 사전 회귀** (CRLF/LF 라인엔딩 차이, main 브랜치에서도 동일 실패). 본 task 범위 외.
