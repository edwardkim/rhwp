# 단계 완료 보고 — Task M100 #2064 3단계: 추출 2 (P8 split 방출)

- 작성일: 2026-07-08 / 브랜치: `local/task2064`

## 수행 내용

P8 블록(split 후보 확정 시 PartialParagraph 방출, 133줄)을 `emit_endnote_split`으로 추출
(동작 불변, 제어 흐름 0 — break/continue/return 없음 사전 확인).

- mut 1(`split_endnote_emitted`)은 **반환 bool**로 치환, caller `if ... { flag = true }`.
- 직접 파라미터 21개 (단일 호출부라 vars struct 불요 — emit_line_runs 전례).
- 컴파일러 타입 확정 3건(tac_rewind Option<f64>/between_gap bool/paragraphs 추가).

## 게이트 (전수 통과)

fmt ✓ / clippy **0** / `--tests` **2,943/0** / issue_1116 **13/13** / OVR 5/5 회귀 0.

## 계측 (라운드 7 누적, 분기 지표)

| 함수 | 라운드 시작 | 현재 |
|---|---|---|
| `typeset_endnote_paragraphs` | 4,397줄 · 1,792 | **4,227줄 · 1,717** |
| 신규: `compute_endnote_metrics` / `emit_endnote_split` | — | 161줄·46 / 159줄·30 |

## 다음 단계

4단계 — 재평가(`--snapshot r7 --no-coverage`) + 잔여 국면 지도(P2/P4~P7) 인계 + 보고.
