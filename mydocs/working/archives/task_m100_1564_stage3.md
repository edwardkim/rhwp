# Task #1564 Stage 3 완료보고서 — 매뉴얼 + #1560 연동 + 최종 보고

## 변경
- `mydocs/manual/opengov_corpus.md`: 말뭉치 구성·두 갈래 게이트(IR 스냅샷/한글 오라클)·갱신 절차.
- `mydocs/report/task_m100_1564_report.md`: 최종 보고.

## #1560 한글 오라클 연동 검증
`tools/verify_hangul_pages.py`(#1560)로 동결 말뭉치 한글 verdict 기록:
- **36382669 OK 8→8** — #1557 secCnt 회귀 가드 실증(붕괴 재발 시 즉시 감지).
- 36383351·36387103 COLLAPSE 2→1(잔여 단일구역 붕괴, secCnt 무관).
- OK 6 / COLLAPSE 2.

## 최종 검증
- `cargo test --test opengov_corpus_snapshot`: 2 passed.
- `cargo test --test hwpx_roundtrip_baseline`: 4 passed(opengov 제외 회귀 없음).
- 말뭉치 8건 동결, 두 갈래 게이트(IR 스냅샷 CI + 한글 오라클 로컬) 동작.

## 결론
고정 실문서 회귀 말뭉치 + 스냅샷 게이트 완성. 측정 재현성 확보, #1557 회귀 상시 감시.
