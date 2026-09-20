# Task #7280 Stage 40 — R2ak 지연 그림 소유 후보 조회 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2aj](task_m100_7280_stage39.md), 시작 head `57ec65b1d`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구조 이동 구현. 고정 head 검증 대기.

## 책임과 보존 계약

`native_hwp5_square_picture_next_page_owner`는 façade로 유지하고 후보 조회를
`controls/deferred_picture::next_page_owner`로 이동한다. 이 후보에만 쓰는 연속 저장 밴드
문단 조회 helper도 같은 모듈로 옮기며 기존 unit test는 test-only import로 같은 구현을 호출한다.
테스트 본문·이름·assertion은 변경하지 않는다.

Query는 `TypesetState`를 받지 않고 필요한 profile/단 수/현재 항목 차용/각주 높이/흐름 높이만
`DeferredPicturePage`로 관측한다. state가 이를 만든다. 단순 scalar/참조 읽기는 진입 시 수행하지만
profile 판정·항목 존재 판정·각주 guard는 원래 순서에 남긴다. 가용 높이는 closure로 전달해
reset_idx==0의 저장 전진량 비교 및 최종 frame 비교 시점에만 조회한다. 진단 호출을 선행하지 않는다.

실제 경로: 구역의 비표 그림 처리 → 후보 Query → `DeferredSquarePictureControl` 큐 등록 후
continue → 기존 `push_new_page`의 anchor 등록/보류 목록 반영 → 그림 발행이다.
Query의 저장 밴드 대상·WrapAnchorRef는 원래 호출자가 그대로 큐에 전달한다. 현재 본문을 다음
쪽으로 옮기거나 현재 쪽에 그림을 즉시 발행하지 않는다. 큐·페이지 수명 코드는 R5에 그대로 남긴다.

높이 경로는 저장 줄 높이/간격 → 필요 시 현재 각주 예산과 비교 → 밴드 대상 제한 →
그림 frame + 캡션 측정 + 캡션 간격 → 최종 가용 높이 비교다. `LayoutEngine` 캡션 측정 의존은
기존 위치/횟수를 보존한 잔여 의존이며 새 공통 측정 정책으로 바꾸지 않는다.
표 컷/이어받기·각주 예약 알고리즘과 본문/미주 인덱스는 비변경이다.

기존 호환 조건·상수·IR/API·테스트 기준값·ignore·CI를 변경하지 않는다.
기존 예외를 유지하는 구조 이동이며 조판 규칙의 타당성을 새로 승인한 것은 아니다.

## 검증 계획과 한계

`output/7280/stage40/verify-deferred-picture.mjs`로 Query와 밴드 helper를 원본과 대조하고,
snapshot 필드·façade 인자·나머지 parent/state/controls(호출자·큐·테스트 포함) 불변을 확인한다.
이는 분기 커버리지나 직접 시각 판정이 아니다.

집중 선택은 Stage39의 325건에 기존 layout 계약 2건을 더한 327건이다.

- 기존 `native_hwp5_square_picture_uses_the_next_page_wrap_owner`: p155 본문 유지,
  p156 그림 64 소유/좌표/중복 부재와 좁은 본문 비겹침.
- 기존 `native_hwp5_square_picture_figure_56_uses_the_same_next_page_owner_contract`:
  다음 문단이 좁은 밴드로 시작하는 그림 56의 p126/p127 소유 계약.
- 기존 typeset band unit: 빈 guide를 포함하는 연속 범위 및 첫 범위 밖 문단에서 종료.
- 추가 선택 `issue_3821_page_tail_square_picture_wrap_reaches_visible_text_after_guides`:
  실물 p156의 guide 뒤 visible 문단과 그림 사이 간격.
- 추가 선택 `issue_3738_picture_caption_float_clears_caption_before_next_body_text`:
  캡션과 후속 본문 간격 대조군. 모든 후보 guard를 실행한다는 뜻은 아니다.

위 원본 fixture를 review worktree에서 확인하고 동일 baseline PASS 이름과 대조한다.
비Picture/다단/각주 없음 등 모든 거절 조합과 closure 호출 횟수의 동적 계측은 미검증이다.
새 테스트·기준값 없이 suite 준비 → manifest/unit-tier baseline 비교 → fmt → native Clippy →
집중 nextest를 순차 실행한다. 주 checkout의 파생 suite는 건드리지 않는다.
전체 회귀·WASM/workspace lint/build·Native Skia·fresh Docker WASM·직접 출력 비교는
구현계획 §7의 책임 묶음/제출 전 게이트에 남긴다. 원격 push·PR·댓글은 범위 밖이다.

## 후속

R2 진입/소유 경계의 잔여 목록을 재점검하고 책임 묶음 통합 검증으로 이어간다.
R3 표 분할/이어받기, R4 각주/미주 본체, R5 구역 수명/상태 접근 제어는 별도 책임으로 유지한다.
