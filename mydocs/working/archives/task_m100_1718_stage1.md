# Task #1718 Stage 1 — 원인 확정 + 베이스라인

## 베이스라인 (수정 전, 통합베이스 15a0253e = upstream/devel + PR #1717·#1715·#1714)
| 파일 | rhwp | 한글 | 비고 |
|------|------|------|------|
| 승강기 [별표27] 경사형 휠체어리프트 | 40 | 48 | 목표 개선 |
| samples/byeolpyo1 | 4 | 4 | 무회귀 게이트 |
| samples/byeolpyo4 | 26 | 27 | 무회귀 게이트 |

- `cargo test --lib` green (2042 passed) — baseline 확인.

## 원인 정량 규명
- 문서 = 단일 5×1 RowBreak 표, 마지막 거대 셀 654문단/~1,200라인 + 그림 14 + 중첩표.
- 기본 줄 pitch 한글 19.2pt = rhwp 1920 HU 동일 / 한글 하단여백까지 꽉 채움 / 그림 총합 2,369pt — 배제.
- 확정: `advance_row_cut`·`advance_row_block_cut` 의 `visible_tail_before_spacer` grace 가
  `units[j+1..].any(|u| u.empty_spacer)` — 뒤에 spacer 하나라도 있으면 avail+120px 오버플로 수용.
  거대 셀의 흩어진 빈문단 spacer 때문에 연속텍스트 중간에서도 grace → 페이지당 +1~5줄 over-fill.
