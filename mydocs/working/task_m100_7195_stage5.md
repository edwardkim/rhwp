---
kind: investigation
status: active
---

# #7195 5단계 — 최신 devel 통합 검증

## 승인과 입력

2026-09-17 작업지시자가 기록 커밋 → 충돌 통합 → 검증 순서를 승인했다.
4단계 기록은 `6533d50e6`으로 커밋했다. 통합 대상은 fetch한
`upstream/devel fcbd00e0fa`이며 PR #7200까지 포함한다.
원격 push·PR·이슈 종료는 수행하지 않는다. 별건 실패의 추가 수정·ignore 확대도 하지 않는다.

## 충돌 해소 근거

유일한 텍스트 충돌은 `typeset.rs`의 행 컷 재시도다.

- devel: 최초 컷에 `advance_row_cut_with_mixed_nested_reserve`를 적용하여
  실제 선택한 중첩 뷰포트의 추가 공간을 예약한다.
- #7195: 넘치는 조각을 재시도할 때 `advance_row_cut_within_capacity`를 사용하여
  줄 소유는 유지하되 저장 프레임 끝까지 예산 밖으로 재확장하지 않는다.
- 통합: 최초 예약 경로는 devel대로 유지한다. 재시도는 측정한
  `painted_tail = split_total - consumed_height - padding`을 예약한 엄격한 컷을 사용한다.
  이 꼬리는 중첩 예약 공간도 포함하므로 이미 예약을 뺀 `budget` 대신
  원래 `content_budget`에서 빼서 같은 공간을 두 번 차감하지 않는다.
  재시도 결과는 `row_cut_content_height`로 다시 측정하여 후보의 물리 점유를 검사한다.
  일반 텍스트 행은 중첩 예약이 없으므로 #7195의 동작을 유지한다.

같은 helper라는 명칭만으로 동등성을 주장하지 않는다. 최초 컷과 재시도의 예약 계층을
구분하고 #7195 저장 프레임/본문 경계/소유 보존과 #7140 중첩 예약 계약을 함께 실행한다.
양쪽에 기존부터 있던 강제 전진 및 후속 이슈 대상의 예외는 이번 충돌 해소에서 확대하지 않는다.

## 검증 계획과 상태

1. 별도 review worktree에서 suite 준비 후 집중 계약 실행.
2. 영향 페이지 출력 및 기존 승인 출력과의 비교.
3. 통합본 전체 회귀와 lint 3종/workspace build/policy/Native Skia/Docker WASM.
4. 4단계 28건 실패와 통합 후 결과를 이름·원인별로 대조.

현재는 통합 코드 준비 단계다. 테스트·시각 판정은 아직 미검증이며 통합 성공을 조판 성공으로
간주하지 않는다. 기준값 및 기대값은 변경하지 않았다.
