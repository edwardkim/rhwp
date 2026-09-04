# Task #1750 Stage 3 완료보고서 — 회귀 검증

## 대상 재현 (한글 OLE 대조)
- `samples/task1750/split_guard_spacing_before.hwp`: **MATCH** (5=5쪽).
- 코퍼스 원본(3024019): PI_MISMATCH(pi22, pi40) → **MATCH** — 연쇄 drift(pi40)까지 해소.
- dump-pages: pi22 → 2쪽 상단 FullParagraph, 2쪽 used 1003.5px = 저장 lineseg 추정 diff 0.0px.

## 회귀 게이트 결과
| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | **2050 passed / 0 failed**, 7 ignored |
| 통합테스트 (issue_1417/546/1486/diag_1042 + 신규 issue_1750) | 2+1+6+1+1 passed / 0 failed |
| byeolpyo1 / byeolpyo4 / 승강기(#1718) / task1700 / task1745 | 4 / 26 / 42 / 1 / 3쪽 무회귀 |
| 코퍼스 mismatch 39건 재검증 | 3024019 만 MATCH 전환, 나머지 38건 판정 변화 없음(악화 0) |
| 코퍼스 MATCH 표본 150건 (seed 42) | **150/150 MATCH 유지** |
| rustfmt(변경 파일) / `cargo clippy --lib` | 통과 (fmt 줄바꿈 1건 정리) |

## 산출물
- `output/poc/hwpdocs_pipage/bad39_recheck_1750.tsv`, `match150_recheck_1750.tsv`

## 상태
완료. 최종보고서 작성 → squash → PR.
