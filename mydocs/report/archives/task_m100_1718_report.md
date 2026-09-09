# Task #1718 최종 보고서 — 대형 RowBreak 셀 over-fill under-pagination 수정

## 요약
hwpdocs 3,249건 rhwp vs 한글(OLE) 페이지·PI 회귀에서 발견된 대형 표 under-pagination의
근본 원인(페이지 채움 grace 과적용)을 규명·수정. `.any()`→`.all()` 1-라인 의미 정정.

## 원인
- 대표: `[별표27] 경사형 휠체어리프트 안전기준.hwp` — rhwp 40쪽 vs 한글 48쪽(−8).
- 문서 = 단일 5×1 RowBreak 표, 마지막 거대 셀 654문단/~1,200라인 + 그림14 + 중첩표.
- 정량 배제: 줄 pitch 동일(19.2pt) · 한글 하단여백까지 채움 · 그림 크기 아님.
- 확정: `advance_row_cut`/`advance_row_block_cut` 의 `visible_tail_before_spacer` grace 가
  `units[j+1..].any(|u| u.empty_spacer)` — 뒤에 빈문단 spacer 가 하나라도 있으면 avail+120px
  오버플로 라인을 수용. 거대 셀엔 spacer 가 흩어져 있어 연속 텍스트 중간에서도 grace → over-fill.

## 수정
`src/renderer/layout/table_layout.rs` 2곳: `.any()` → `.all()`.
- 의도(변수명 `visible_tail_before_spacer`)대로 "뒤가 전부 spacer 인 진짜 꼬리줄"에만 grace.
- 연속 텍스트 → grace 거부 → 정상 capacity break(한글 정합).
- 진짜 tail → grace 유지 → byeolpyo1/4 over-pagination 방지 무회귀.

## 검증
| 항목 | 결과 |
|------|------|
| 승강기 대표 | 40 → 42쪽 (텍스트 밀도 over-fill 해소; 한글 48) |
| byeolpyo1 / byeolpyo4 | 4 / 26 무회귀 |
| cargo test --lib | 2042 + 신규 2 = 전부 통과 |
| 코퍼스 mismatch 재검증 | 25건 개선(→MATCH), 신규 회귀 0 |
| 코퍼스 match 표본(300) | 222 MATCH 유지, 회귀 0 |

## 한계 / 후속
- 승강기 잔여 42 vs 48(−6): 이미지/개체(그림·중첩표) 배치 밀도 차이 영역. 별도 이슈 후보.
- 한글 COM 배치 크래시로 전수 재검증은 완료분 기준(harness 한계).

## 산출물
- 소스: `src/renderer/layout/table_layout.rs` (+ 단위테스트 2)
- 재현: `samples/task1718/table_giant_cell_overfill.hwp` + README
- 조사 산출: `output/poc/hwpdocs_pipage/` (TSV, 한글 PDF, 측정 스크립트)
