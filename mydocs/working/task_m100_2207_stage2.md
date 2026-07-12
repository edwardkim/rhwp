# Task M100 #2207 — 2단계 완료 보고: 원인 확정

- 이슈: #2207 / 브랜치: `local/task2207` / 작성일: 2026-07-11

## 판정: 앵커 기준점 오류 확정 — #577 가드의 오버레이 wrap 누락

[table_layout.rs 셀 비인라인 그림 분기](../../src/renderer/layout/table_layout.rs#L2856):
Task #577이 `TopAndBottom + vert=Para` 셀 내부 그림을 "compose 후 전진된 para_y"가
아닌 앵커 시점(첫 LINE_SEG vpos) 기준으로 정정했으나, 가드가 TopAndBottom 한정.
글뒤로/글앞으로(절대 오버레이) + Para 조합은 `anchor_y = para_y` 로 남아 앵커 문단
**한 줄 아래**를 기준점으로 사용한다.

## 검증 3종

1. **산술 예측 적중**: 수정 전 이미지 요소 y = 87.56px. 앵커 문단 p[0]의
   lh(1500)+ls(300) = 1800 HU = 24.0px 를 빼면 63.56px — 실제로 가드 확장 후
   렌더 y = **63.56px 정확 일치**. 한컴 잉크 top 65px(이미지 내부 여백 ~1.4px)과
   정합.
2. **Shape 경로 대칭성**: 같은 함수의 도형(Shape) 분기
   ([:3245-3252](../../src/renderer/layout/table_layout.rs#L3245-L3252))는 이미
   `vert=Para`면 wrap 무관하게 `para_y_before_compose` 사용 — 그림 분기만 누락.
3. **두-경로/타 지점 점검**: `Control::Picture` 매치 전수 확인 — 배치 지점은 수평
   셀 경로의 이 분기 하나뿐 (나머지는 높이 계상·재정렬·인라인 경로). 오버레이
   그림은 플로우를 밀지 않으므로 typeset/리플로우 측 영향 없음.

## 3단계 정정 내용 (적용·검증 중)

`overlay_para`(BehindText|InFrontOfText + Para) 조건을 신설해 #577 앵커-시점
기준을 공유. 다른 wrap(Square 등)·후속 가드(`unrestricted_take_place_cell_float`,
`top_and_bottom_para` pic_y 오버라이드)는 비접촉 — 케이스별 명시 가드 정책 준수.

픽셀 실측(수정 후): 잉크 top/bottom = 66/117 vs 한컴 65/117 (1px 이내), 하단
클리핑 해소. 표적 테스트 `tests/issue_2207_cell_overlay_picture_anchor.rs` 신설 —
수정 전 소스에서 FAILED(y=87.6) 실증.
