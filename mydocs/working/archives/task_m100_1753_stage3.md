# Task #1753 Stage 3 완료보고서 — 회귀 검증

## 대상 재현 (한글 OLE 대조)
- `samples/task1753/deferred_takeplace_fill_ahead.hwpx` + 코퍼스 원본(2814765):
  n_mismatch **3 → 1** — pi52/53 이 9쪽 선행 채움(한글 정합)으로 해소.
  잔여 1건(pi51 rhwp_p10|hwp_p9)은 캐럿-개체 분리(도구 한계, 수행계획서 범위 밖 명시).
- 총 21쪽 불변, pi54(후속 표)는 마지막 fragment 뒤 11쪽 유지.

## 회귀 게이트 결과
| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | **2051 passed / 0 failed** |
| 통합 9크레이트 (1417/546/1486/diag1042/1750/1749/1440/1139 + 신규 1753) | 102 passed / 0 failed |
| 페이지 게이트 (byeolpyo1/4·승강기·1700/1745/1749/1750·국제고속선기준) | 4/26/42/1/3/5/2/**251** 무회귀 |
| 코퍼스 mismatch 재검증 | 대상 1건 n=3→1 개선, 나머지 38건 변화 없음(악화 0) |
| 코퍼스 MATCH 표본 150건 | **150/150 유지** |
| rustfmt / clippy --lib | 통과 |

## 산출물
- `output/poc/hwpdocs_pipage/bad39_recheck_1753.tsv`, `match150_recheck_1753.tsv`

## 상태
완료. 최종보고서 작성 → squash → PR.
