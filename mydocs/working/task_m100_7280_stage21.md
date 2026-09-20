# Task #7280 Stage 21 — R2s 형제 표 지연 후보 조회 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2r](task_m100_7280_stage20.md), 시작 head `3ef227511`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현 완료, 고정 제품 SHA 집중 검증 예정. R2 전체 완료가 아니다.

## 1. 분리한 책임과 보존 계약

`controls/deferred.rs`의 읽기 전용 `CoanchoredTableQuery`로 같은 문단에 매달린
RowBreak 표의 지연 배치 판별과 후행 후보 선택을 이동한다. Paragraph·FormattedParagraph와
공유 TacFlowQuery만 보유한다. 판별에 필요한 현재 쪽 수와 항목 슬라이스를 받지만
TypesetEngine/TypesetState 전체나 가변 IR은 받지 않는다.

이월 트리거는 기존 표 조건 → 물리 페이지 증가 → 마지막 항목의 continuation 순서로 판별한다.
후행 후보에만 적용하는 **양수 세로 오프셋** 조건을 트리거와 합치거나 완화하지 않는다.
가시 텍스트와 줄 점유를 새로 등치시키지 않으며 기존 조건의 정당성을 승인하는 작업도 아니다.
profile은 기존 TAC 판별의 단락 평가 지점에서 읽는다. 새 조회 객체 생성 시에는 참조만 보유한다.
쪽 수와 항목 슬라이스는 부수효과 없는 관측값이며 판별 전에 상태를 바꾸지 않는다.

### 실제 소비 경로

기존 block 표 배치 → 이월 여부 조회 → 기존 정렬 순서의 `skip(order_pos + 1)`에서 후보 선택 →
기존 큐 extend → 각주 등록 → 기존 loop break 순서를 유지한다. 결과에는 원래 문단/컨트롤 인덱스,
첫/마지막 표 플래그와 **원 배치 시점의 para_start_height**가 그대로 담긴다.

flush에서는 현재 열 너비로 문단을 다시 format한 뒤 같은 후보 판별을 재사용한다.
큐 drain/유지 시점, Before/AfterTableParagraph·SectionEnd 선택, 실제 블록 배치,
각주와 vpos 상태 반영은 부모에 그대로 남는다. 지연 배치의 렌더 원점은 현재 흐름 높이를,
분할 예산 원점은 저장한 최초 문단 높이를 사용하는 기존 #1860 계약도 변경하지 않는다.
컷·요구 높이·예약 높이·paint 산식은 이번 절편의 변경 대상이 아니다.

IR/API, 테스트 source/assertion/ID, baseline/golden/ignore, CI 정책은 변경하지 않는다.
지연 큐와 결과 타입의 상태 캡슐화 및 실제 표 배치 분리는 후속 책임으로 남긴다.

## 2. 검증 계획

`output/7280/stage21/verify-deferred.mjs`로 세 판별 본문과 후보 선택을 원래 표현으로 복원해
대조하고, 부모의 큐/flush/각주/반복 종료와 기존 테스트·state·공유 TACquery 불변을 확인한다.
정적 비교는 실행 검증의 대체물이 아니다.

별도 review worktree에서 파생 suite 준비, 고정 baseline 대비 manifest/unit-tier,
fmt, native Clippy, 집중 nextest를 순차 실행한다. R2r의 218건에 기존 #1686 4건,
#6795 5건, #5906 1건을 추가한다. #1686은 HWP/HWPX 이월 표·후속 제목 순서와 쪽 경계,
#6795는 0-offset 형제 표 비이월·겹침·순서와 통짜 배치 대조군,
#5906은 지연된 형제 표의 실제 행 경계·하단을 보호한다.
기존 한컴 근거를 인용한 계약이며 이번에 직접 시각 판독을 완료했다는 의미는 아니다.
각주가 있는 이월 후보의 모든 조합이나 큐 flush 지점별 실행 추적은 새로 추가하지 않는다.

전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM과 직접 시각 대조는
통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위에 포함하지 않는다.
