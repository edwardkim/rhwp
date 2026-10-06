# 단계 완료 보고 — Task M100 #2064 2단계: 추출 1 (P3 메트릭 클로저 fn 승격)

- 작성일: 2026-07-08 / 브랜치: `local/task2064`

## 수행 내용

`typeset_endnote_paragraphs`의 `compute_en_metrics` 클로저(127줄, #1363 SSOT 메트릭)를
`compute_endnote_metrics` 메서드로 승격 (본문 무변경).

- **`EnMetricsVars`** (Copy 26필드) — 캡처 명시화. **두 호출부(P3/P7)가 멀리 떨어져
  있어 호출부마다 신선하게 구성** — 클로저의 call-time 캡처 의미 보존.
- **스캐너 사각지대 6호 발견**: 클로저가 실제로는 FnMut — `current_endnote_had_inline_
  object_vpos_overestimate`를 변이(컴파일러 E0384 검출, 사전 스캔 미검출). `&mut bool`
  파라미터로 분리. → 체크리스트 6종으로 갱신.
- 타입 확정 2건(col_count u16 / between_notes_px Option<f64>)과 doc 재배치(고아 doc을
  원 주인 fn 위로) — 컴파일러/clippy 검출.

## 게이트 결과 (전수 통과)

fmt ✓ / clippy **0** / `--tests` **2,943/0** / issue_1116 **13/13** / OVR 5샘플 회귀 0.

## 계측

| 함수 | 이전 | 이후 |
|---|---|---|
| `typeset_endnote_paragraphs` | 4,397줄 · 분기 1,792 | **4,334줄 · 1,746** (호출부 vars 구성 2곳 추가분 상쇄) |
| `compute_endnote_metrics` (신규) | — | 161줄 · 분기 46 |

## 다음 단계

3단계 — 추출 2: P8 split 방출(133줄, reads 23/muts 1). 승인 후 착수.
