# Stage 1 완료보고서 — Task #1355 근본원인 계측

## 계측 방법
`layout.rs` 미주 제목 배치부에 임시 로깅(`RHWP_DBG_1355`)을 삽입, p18 문30 및 전 페이지
미주 제목의 gap 구성요소를 캡처(완료 후 제거).

## 핵심 결과 (96dpi px)

문30(p18):
```
y_before=326.1  y_off=352.5  prev_bottom=299.6  gap=26.5
```
- `y_before_vpos`(흐름 위치) = prev_bottom(299.6) + 흐름전진 26.5 = 326.1 → **흐름이 이미 gap 포함**
- `y_offset`(최종) = 352.5 = y_before(326.1) + **또 26.5** → **gap 이중계상(약 2배)**
- +26.5px 는 상류 `hcursor.vpos_adjust`(layout.rs:3133)가 saved-vpos 기준으로 추가

대조 — 정상 케이스(문24/문28/문29 일부): `prev_bottom == y_before`(흐름이 gap 미생성),
`vpos_adjust`가 gap 1회 추가 → 정상.

## 판별 지표
`flow_advance = y_before_vpos - prev_content_bottom_y`
- `flow_advance ≈ gap` (예: 문30) → 흐름이 이미 gap 생성 → vpos_adjust 추가분이 **이중계상**
- `flow_advance == 0` → 흐름 gap 없음 → vpos_adjust 추가가 정상

## 원인 확정
미주 제목 직전 콘텐츠의 trailing line-spacing 이 "미주 사이" gap 을 이미 만든 경우에도
`vpos_adjust` 가 saved-vpos 기준 gap 을 한 번 더 더해 제목 앞 여백이 2배가 된다.
기존 `compact_endnote_title_gap_after_single_equation_tail` 게이트는 `prev_gap<50px`,
`question<29` 등 조건으로 본 케이스(gap=26.5px)를 못 잡는다.

## 비고 — p21→p22 오버플로
별건 계측 결과 p21 의 `(ⅰ)~(ⅲ)에서` 다음 쪽 밀림은 **doubling 제목과 무관**
(해당 페이지에 doubling 제목 없음, 수정 전후 동일). 이는 `issue_1082` 테스트가 추적 중인
**#1336 기지 잔여(미주 다단 fit 캡, ~50px)** 로 본 타스크 범위와 다른 원인 → 별도 추적.
