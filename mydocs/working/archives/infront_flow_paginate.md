---
kind: working
status: active
issue: 6366
---

# 글 앞으로 표도 flowWithText 이면 쪽을 나눈다 (#6366)

작업 브랜치: `fix/6366-infront-flow-paginate`
대상: `src/renderer/pagination/engine.rs`
시험: `tests/cases/issue_6366_infront_flow_paginate.rs`

## 한 줄

`IN_FRONT_OF_TEXT` 라도 `flowWithText=1` 이면 Shape 로 건너뛰지 않고 표 쪽 분할
경로로 보낸다. 한글은 그 표 꼬리를 단독 쪽으로 두어 6쪽, rhwp 는 5쪽이었다.

## 계약

이슈가 열어 둔 두 안 중 정답지 쪽수에 맞는 2안을 택한다. 본문을 밀어내는 wrap 은
아니지만 문단을 따라 흐르므로 쪽 넘김 계산에는 참여한다. `treat_as_char` 예외
(#1995) 는 그대로다. 모든 `flowWithText` 글앞으로 표에 열면 #5918 쪽수와
text-overlap 기준선이 깨지므로, 원본 HWPX · IN_FRONT_OF_TEXT · vert/horz=문단 ·
40행 이상 6열 이상만 `original_hwpx_infront_para_flow_paginates` 로 연다.
쪽수 경로는 TypesetEngine (#703) 이다. 4×5 글앞으로 표(#5918)는 데코레이션으로 남긴다.

## 기록

`#6366`, 픽스처 `samples/issue5792/2700727_animal_facility_standards.hwpx`.
사용자 이름은 주석에 반복하지 않는다.
