# #3820 Stage 195: HWPX local reset stream 범위

## 관찰

Stage 194에서 direct RowBreak reset 전체를 mid-page absorb로 보냈더니
`issue1949` giant-cell은 115쪽으로 회복했지만, #3930 HWPX는 383쪽에서 381쪽으로
줄었다. direct reset 전체를 같은 의미로 처리할 수 없다.

## 원인

HWPX direct RowBreak cell의 단일 저장 reset은 physical fragment boundary다. 반면
한 셀의 continuation에 저장 reset이 여러 개 있으면 writer가 local cursor range를
나눈 stream이며, 각 reset이 쪽 경계를 뜻하지 않는다. 기존 구현은 이 local stream을
특정 table height와 문단 수로 예외 처리해 일반화되지 못했다.

## 변경

HWPX non-inline 1×1 RowBreak continuation에서 stored frame reset이 둘 이상이면
local reset stream으로 판정해 mid-page absorb 경로를 사용한다. 그 밖의 HWPX direct
reset과 재귀 nested frame은 엄격한 물리 조각 경계를 유지한다. native HWP direct
cell은 기존처럼 generic absorb 판단을 사용한다.

## 검증

- giant-cell HWP/HWPX 115쪽 bbox 회귀 가드
- #3930 HWPX/HWP 383쪽 저장 레이아웃 가드
