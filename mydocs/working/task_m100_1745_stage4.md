# Task #1745 Stage 4 완료보고서 — 회귀 검증

## 대상 재현 (한글 OLE 대조)
- `samples/task1745/table_text_anchor_wrap.hwp`: PI_MISMATCH(pi1 rhwp p3 ↔ 한글 p1) → **MATCH** (3쪽=3쪽).

## 회귀 게이트 결과
| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | **2050 passed / 0 failed** (신규 2 포함), 7 ignored |
| wrap 통합테스트 (issue_546 / issue_1440 / issue_1139) | 85 + 4 + 1 passed / 0 failed |
| byeolpyo1 / byeolpyo4 (#1658 게이트) | 4쪽 / 26쪽 무회귀 |
| task1718 승강기 / task1700 단일쪽 wrap | 42쪽 / 1쪽 무회귀 |
| 코퍼스 mismatch 39건 재검증 (한글 OLE) | 수정 대상 1건만 MATCH 전환, 나머지 38건 판정 변화 없음(악화 0) |
| 코퍼스 MATCH 표본 150건 (seed 42) | **150/150 MATCH 유지** (PI_MISMATCH/PAGE_DELTA/ERR 0) |
| rustfmt(변경 3파일, newline 제외) / `cargo clippy --lib` | 통과 / 경고·오류 0 |

## 비고
- 사례 A(자리차지 다쪽 표 anchor 캐럿, 17991519)는 검증 도구 측정 의미 차이로 잔존 —
  수행계획서의 한계 항목대로 범위 밖.
- 산출 TSV: `output/poc/hwpdocs_pipage/bad39_recheck_1745.tsv`, `match150_recheck_1745.tsv`.

## 상태
완료. 최종보고서 작성.
