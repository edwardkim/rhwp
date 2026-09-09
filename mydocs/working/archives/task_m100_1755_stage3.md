# Task #1755 Stage 3 완료보고서 — 회귀 검증

## 대상 재현 (한글 OLE 대조)
- `samples/task1753/deferred_takeplace_fill_ahead.hwpx` + 코퍼스 원본(2814765):
  PI_MISMATCH(n=1, pi51) → **완전 MATCH** — host 제목 줄이 9쪽 pre-emit 되어
  pi51 판정까지 해소 (#1753 잔여분 완결).
- SVG: 제목 "투입인원수" 9쪽 렌더 / 11쪽 미렌더(이중 렌더 없음), 총 21쪽 불변.

## 회귀 게이트 결과
| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | **2051 passed / 0 failed** |
| 통합 10크레이트 (1417/546/1486/diag1042/1750/1749/1753/1755/1440/1139) | 103 passed / 0 failed |
| 페이지 게이트 (byeolpyo1/4·승강기·1700/1745/1749/1750/1753·국제고속선기준) | 4/26/42/1/3/5/2/21/**251** 무회귀 |
| 코퍼스 mismatch 재검증 | 대상 1건 완전 MATCH 전환, 나머지 38건 변화 없음(악화 0) |
| 코퍼스 MATCH 표본 150건 | **150/150 유지** |
| rustfmt / clippy --lib | 통과 |

## 산출물
- `output/poc/hwpdocs_pipage/bad39_recheck_1755.tsv`, `match150_recheck_1755.tsv`

## 상태
완료. 최종보고서 작성 → squash → PR.
