# Task #1749 Stage 1 완료보고서 — 재현 샘플 동결 + 실패 재현

## 수행 내용
- `samples/task1749/saved_bounds_cumulative_vpos.hwpx` 동결 (opengov 결재문서 36371084,
  47KB, PII 방침 A 선례) + README.
- 실패 재현 (upstream/devel + PR #1744/#1746/#1751 적용 베이스):
  ```
  페이지 1: 단 0 (items=20, used=1011.8px)   ← 본문 990.2px 초과 (21.6px overfill)
    FullParagraph  pi=18  " "  vpos=72626
  페이지 2: pi=19 "붙임 1. ..."  vpos=74902
  ```
  한글(OLE) 캐럿은 pi18 을 2쪽 배치.
- FIT 트레이스(조사 시 실증, 이슈 #1749): pi18 `normal_fits=false, saved_single=true`.

## 도입 커밋 보호 케이스 분석 (수행계획서 반영)
- fe6de3ef 합성 테스트 `page_bottom_text_box_fit_keeps_line_even_when_advance_overflows`:
  **문서 마지막 문단** + 저장 좌표가 페이지 하단에 정확히 닿고 트레일링 ls 만 초과 —
  Paginator(engine.rs) 경로. "다음 실줄 없음"으로 결함 케이스와 구분 가능.
- 결함 케이스는 누적좌표 문서(pi19 vpos=74902 > 본문 74265HU, 리셋 없음) — 저장 vpos 가
  페이지 배정을 인코딩하지 않아 bounds 신뢰 전제가 붕괴.

## 상태
완료. Stage 2 (페이지-마지막 증거 조건 + 테스트) 진행.
