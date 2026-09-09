# Task #1733 (부분) — 국제고속선기준 잔여 over-pagination tail 완화 (250→245)

## 배경
#1730(#1725 각주 안전마진 tail, 258→250) 머지 후 잔여 +8쪽을 #1733 로 분리. 본 PR 은 그중
tail-before-vpos-reset 계열 2가지를 추가 완화한다.

## 원인 (empirical, #1730 debug 연장)
잔여 near-empty(문단 1개) 페이지의 tail 문단을 디버그 추적:
- pi=537/2378: 각주 있음(fn_h=35.4). 그런데 tail 과 새 페이지 사이 **빈 문단 1개**가 껴
  immediate-next reset 을 놓쳐 #1725 가드 미발동 → 각주 버퍼가 tail 을 밀어냄(short 7.8px).
- pi=718/995/1789/2128: **각주 없음**(fn_h=0). 페이지가 그림/텍스트로 꽉 차 tail 이 수 px
  over-fill(short 4.7~13px)로 밀림.
공통: 다음 문단이 새 페이지를 시작(vpos-reset)하는데 tail 이 밀려 단독 near-empty 격리.
한글 LINESEG 는 tail 을 본문 하단(각주 위/여백 침범)에 배치.

## 수정 (tail-before-vpos-reset 한정 2가지)
`src/renderer/typeset.rs`:
1. **빈 문단 건너뛰기 가드**: tail 과 새 페이지 사이 빈 문단(컨트롤 없음) 1개가 껴도 tail 로 인식
   → #1725 각주/안전마진 완화 대상 포함.
2. **소량 오버플로 허용** `TAIL_BREAK_OVERFLOW_TOLERANCE_PX=20px`: 각주 없이 page-full over-fill
   로 밀리는 tail 에 한해 20px 오버플로 1회 허용(한글 정합, 여백 침범 무시).

## 검증
| 항목 | 결과 |
|------|------|
| 국제고속선기준 | 250 → **245쪽**(누적 258→245, near-empty 18→13; 목표 242에 +3) |
| byeolpyo1 / byeolpyo4 | 4 / 26 무회귀 |
| 승강기 [별표27](#1718) | 42 무회귀 |
| cargo test --lib | 2044 passed / 0 failed |

## 잔여 (#1733 계속)
- pi=3725(오버플로 30.9 > 20 허용치), pi=3173(각주 90.9 대형, 40px 초과), PartialParagraph/
  PartialTable 격리. 허용치 상향/전체각주제외로 더 잡히나 겹침 위험 증가 → 보수적으로 보류.
- 각 완화는 "다음 문단이 새 페이지를 시작하는 tail" 에만 발동 → 한글 정합·저위험.
