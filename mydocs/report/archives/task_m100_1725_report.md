# Task #1725 최종 보고서 — 흐름 텍스트 각주 tail 문단 over-pagination 수정

## 요약
흐름 텍스트 법령의 over-pagination(한글보다 페이지 많음)을 각주 예약 버퍼 관점에서 규명·수정.
대표 `국제고속선기준.hwpx` rhwp **258 → 250쪽**(목표 242), 회귀 0.

## 원인 (empirical)
- 각주 90개 문서. rhwp 가 near-empty 페이지(문단 1개) 26개 양산.
- 디버그: tail 문단 pi=52 `cur_h=921.6 > avail=917.8 → fits=false → advance`.
  avail 이 1005→917.8 로 축소 = **각주 공간 예약**(각주 높이 + `footnote_safety_margin=40px` 버퍼).
  tail 이 3.8px 초과로 밀리고, 다음 문단이 새 페이지(vpos-reset)라 단독 near-empty 격리.
- 한글 LINESEG vertpos=69120 → 한글은 tail 을 본문(각주 위)에 배치. rhwp 버퍼가 밀어냄.

## 수정
`src/renderer/typeset.rs`: Task #359 `next_will_vpos_reset` 일반텍스트 분기에서 `skip_footnote_margin_once`
설정 → typeset_paragraph fit 계산에서 **각주 안전마진(40px 버퍼)만 1회 되돌림**. 실제 각주 높이는 유지.

## 검증
| 항목 | 결과 |
|------|------|
| 국제고속선기준 | 258 → **250쪽**(-8, near-empty 26→18) |
| byeolpyo1 / byeolpyo4 | 4 / 26 무회귀 |
| 승강기 [별표27](#1718) | 42 무회귀 |
| cargo test --lib | 2044 passed / 0 failed |

## 한계 / 후속
- 40px 버퍼로 잡히는 각주-tail 케이스(-8쪽)만 수정. 잔여 near-empty 18개(+8쪽)는 **다중 원인**:
  PartialParagraph/PartialTable 격리, 비각주 tail 등. 각각 별도 원인이라 본 수정 범위 밖이다. 후속 #1733에서
  분리 추적한다.
- 전체 각주 예약을 제외하는 더 공격적 버전도 시험했으나 동일 250 결과 → 겹침 위험 없는 40px 버전 채택.
- #1718(표 over-fill)과 정반대 방향(텍스트 under-fill). razor-thin 다중원인의 부분 개선.

## 산출물
- 소스: `src/renderer/typeset.rs` (state 필드 `skip_footnote_margin_once` + 가드/fit 2곳)
- 재현 HWPX: `samples/task1725/text_footnote_tail_overpagination.hwpx`
- 검증 기준 HWP: `samples/task1725/text_footnote_tail_overpagination.hwp`
- 검증 기준 PDF: `pdf/text_footnote_tail_overpagination-2024.pdf` (Hwp 2024, 242쪽)
