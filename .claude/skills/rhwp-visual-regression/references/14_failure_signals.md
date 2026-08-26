# 14 — 실패 신호 → 처방

| ID | 언제 | 행동 |
| --- | --- | --- |
| F01 | status PASS | 끝. 다음 단 금지 |
| F02 | A==A 가 PASS 아님 | 도구 비결정성. 중단 |
| F03 | STRUCT + 경로가 편집 위치 | 정상. 실패로 읽지 않음 |
| F04 | STRUCT + 경로가 무관 | 진짜 회귀 |
| F05 | PAGE_MISMATCH | dump-pages 로 좁힘 |
| F06 | OVER | worst_page 로 좁힘 |
| F07 | LOAD_FAIL | info 로 그 파일만 |
| F08 | ir-diff --json exit 3 | 차이 검출은 데이터 |
| F09 | 눈 검증 필요 | export-png. thumbnail 은 저장본 |
| F10 | 배치 TSV 혼합 | 행별 status 로 격리 |
| F11 | 질문이 이미 답 | 다음 단 금지 |
| F12 | WARN_TEXTRUN | 하드 실패 아님 |

신호 표는 `fixtures/failure_signals.json` 과 같다.
