---
kind: guide
status: active
canonical: mydocs/manual/edit_command_review_checklist.md
last_verified: 2026-07-17
---

# 편집 Command/Undo 검토 체크리스트

새 rhwp-studio 편집 기능 또는 PR을 검토할 때 다음을 확인한다.

## 1. 문서 mutation 여부

- 이 액션이 문서 저장 결과를 바꾸는가?
- 문서 저장 결과를 바꾼다면 Undo/Redo 기대치가 있는 사용자 액션인가?
- 단순 조회, export, render, 보기 옵션이라면 history 밖에 두었는가?

## 2. 라우터 통과 여부

- 새 document mutation이 `executeOperation()` 또는 후속 편집 라우터를 통과하는가?
- 직접 `services.wasm.*` 또는 `this.wasm.*` mutation을 호출한다면 허용 예외인가?
- 예외라면 command 내부 저수준 호출인지, drag preview처럼 `recordApplied`가 필요한 경로인지 명확한가?

## 3. Undo/Redo payload

- redo가 `execute()` 재호출로 같은 결과를 만들 수 있는가?
- undo에 필요한 before 값이 command에 저장되는가?
- UI 표시용 JSON 전체가 아니라 core 복원에 안정적인 ID/delta/snapshot을 사용했는가?

## 4. snapshot 사용 기준

snapshot을 사용한다면 다음 중 하나에 해당해야 한다.

- paste/cut/delete처럼 여러 control/resource가 동시에 바뀐다.
- 정확한 delta command가 아직 과도하게 복잡하다.
- 실패 시 문서 복구가 어렵다.

snapshot 사용 시 operation type과 resource discard 경로도 확인한다.

## 5. refresh와 dirty scope

- mutation 후 렌더링 갱신 정책이 명확한가?
- text edit처럼 page-local refresh가 가능한가?
- full refresh가 필요한 구조 변경인가?
- selection/caret 복원 정책이 기존 UX와 일치하는가?

## 6. page-local 캐시 정합성

- dirty scope가 화면 repaint 범위뿐 아니라 mutation으로 무효화되는 레이아웃 캐시 계층까지 포함하는가?
- 문단, 셀, 바깥 표처럼 상위 레이아웃이 하위 내용에 의존한다면 가장 가까운 유효한 상위 scope까지 무효화하는가?
- 모델은 최신인데 layout tree, page tree 또는 cursor 조회용 캐시만 이전 상태로 남을 수 있는 경로가 없는가?
- 캐시 key 하나만 제거하는 경우 같은 문단·셀을 가리키는 다른 key와 파생 캐시가 남지 않는가?
- page-local 무효화로 처리할 수 없는 구조 경계 변화는 명시적인 effect로 상위 계층에 전달되는가?

## 7. pagination flush와 cursor 순서

- 줄·셀 흐름이 유지되는 안정 입력은 동기 pre-cursor pagination flush 0회로 진행하고, 남은
  pending은 기존 idle/manual/full-edit 정책으로 마감하는가?
- page-local 범위를 벗어나는 경계 변화는 논리적 편집 1회당 동기 pagination flush를 정확히
  한 번 시도하는가?
- mutation 결과의 pending effect를 flush 판단보다 먼저 등록하는가?
- boundary/flush-required effect가 있다면 cursor/selection 위치 계산보다 먼저 flush를 시도하고,
  실패 시 pending 보존·caret 정확성·retry 정책이 명시되어 있는가?
- cursor 계산 뒤 flush하여 방금 계산한 위치를 다시 stale 상태로 만들거나, refresh 뒤 중복 flush하지 않는가?
- page-local repaint와 전체 pagination flush를 동일한 조건으로 묶지 않고 각각 필요한 범위로 판정하는가?

## 8. effect의 one-shot 전달

- command가 mutation effect를 구조화된 결과로 전달하고 전역 flag나 호출 순서에 의존하지 않는가?
- 일반 execute에서 effect를 한 번만 소비하고 history에 이전 effect가 남지 않는가?
- redo는 최초 execute의 effect를 재사용하지 않고 실제 재실행 결과로 effect를 다시 계산하는가?
- IME composition과 iOS raw input처럼 여러 mutation이 하나의 논리적 입력을 이루면 effect를 누적하되 flush는 경계에서 한 번만 수행하는가?
- 안정 입력과 경계 입력이 섞여도 누적 effect를 잃거나 같은 effect를 두 번 소비하지 않는가?

## 9. pending 상태 초기화

- 문서 load/교체, 입력 handler 활성화·비활성화, composition 취소·종료에서 pending effect를 초기화하는가?
- 새 문서나 다음 입력 세션이 이전 문서·세션의 pending effect를 소비하지 않는가?
- delete, fallback 등 즉시 처리하는 mutation은 이전 deferred pending을 먼저 정리하고 자체 refresh/flush 정책만 적용하는가?
- 예외나 조기 return 경로의 pending/caret 복구 정책이 명시되어 있고 다음 입력이나 문서로 상태가 누출되지 않는가?

## 10. recordApplied 경로

IME, drag, resize처럼 이미 mutation을 적용한 뒤 history에 기록하는 경우:

- command가 before/after payload를 모두 갖고 있는가?
- `history.recordWithoutExecute()`를 직접 호출하지 않고 router를 통과하는가?
- refresh를 호출부가 이미 처리한다면 `refresh: 'none'`을 명시했는가?

## 11. 수동 판정

최소 확인:

- Undo 1회
- Redo 1회
- 같은 액션 반복 후 Undo/Redo
- selection/caret 위치
- 저장 전 dirty 상태
- 안정 입력의 동기 pre-cursor 경로에서 pagination flush가 발생하지 않는지
- 줄·셀 흐름 경계 입력에서 cursor 계산 전 동기 flush를 한 번만 시도하는지
- IME/iOS 입력 완료, 취소, Undo/Redo 뒤 raw effect accumulator가 남지 않는지

도메인별 추가 확인:

- 표: 셀/표 selection 복원
- 그림/도형: object selection 복원, 이동/크기/회전 유지
- 쪽/머리말/꼬리말: 페이지네이션 및 바탕쪽/필드 영향
- 필드/form: 저장 후 roundtrip 영향
