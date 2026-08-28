# Task M100 #4135 Recovery R5 — F5 셀 선택 단계 표시

- **작업 기준**: `13f9de80b4c2`
- **관찰한 upstream**: `upstream/devel@b1485e0a143d`
- **브랜치**: `codex/issue-4135-contextual-shortcut`
- **계획**: [`task_m100_4135_impl.md`](../plans/task_m100_4135_impl.md)
- **선행 결과**: [`task_m100_4135_recovery_r4.md`](task_m100_4135_recovery_r4.md)
- **승인**: 작업지시자가 R4 한글 IME 재검증을 `수정이 반영되었어.`로 승인한 뒤,
  한컴의 셀 안 단계 마커와 하단 문구를 함께 쓰는 하이브리드 UX를 `그렇게 진행해줘.`로 승인
- **현재 판정**: R5 UX 계약 RED 4건 확인, 구현 전 checkpoint 대상

## 1. 사용자 결과와 범위

F5 1회와 2회가 서로 다른 셀 선택 단계인데 기존 Studio는 같은 파란 선택 배경만 보여 현재 단계와
방향키 동작을 알 수 없다. 한컴 실측 화면은 선택 셀 중앙의 회색/주황 원으로 두 단계를 구분한다.
R5는 공간적으로 가까운 마커를 주 표시로 차용하고, 색만으로 상태를 알아야 하는 문제와 학습 비용을
줄이기 위해 기존 일시 메시지와 분리된 하단 단계 이름을 보조 표시로 추가한다.

| 상태 | 셀 안 주 표시 | 하단 보조 표시 |
| --- | --- | --- |
| F5 1회 | 포커스 셀 중앙 회색 원 | `셀 선택 · 방향키로 이동` |
| F5 2회 | 포커스 셀 중앙 주황 원 | `셀 범위 선택 · 방향키로 확장` |
| F5 3회 | 마커 없음, 표 전체 파란 선택 | `표 전체 선택` |
| Escape·일반 입력·마우스 전환·undo 등 선택 해제 | 제거 | 제거 |

정렬된 선택 범위만으로는 어느 끝이 방향키로 움직이는 포커스인지 알 수 없으므로 CursorState가 focus
좌표의 복사본을 공개한다. 병합 셀에서는 focus 좌표를 포함하는 bbox 중앙에 마커를 둔다. 보호 셀 클릭으로
생기는 내부 선택은 F5 학습 상태가 아니므로 기존 하이라이트만 유지하고 단계 마커·문구는 표시하지 않는다.

하단 표시는 `#sb-message`를 재사용하지 않는다. 이 요소는 자동 저장·인쇄·파일 상태가 잠시 쓰고 이전
문구를 복원하는 채널이므로, 지속적인 F5 상태를 넣으면 서로 덮어쓰거나 낡은 상태를 되살릴 수 있다.
별도 `role=status`, `aria-live=polite` 요소를 두고 같은 단계 문자열을 다시 쓰지 않아 과도한 안내를 막는다.

## 2. RED 계약

제품 코드를 바꾸기 전에 `rhwp-studio/tests/cell-selection-phase-ux.test.ts`에 다음 계약을 고정했다.

1. Cursor focus와 phase가 renderer에 전달되고 1·2단계가 별도 클래스·테마 토큰을 쓴다.
2. 3단계는 마커를 만들지 않고 `표 전체 선택` 상태만 전달한다.
3. 전용 live status가 기존 `#sb-message`와 공존한다.
4. renderer의 모든 `clear()` 호출이 단계 상태도 `null`로 해제해 기존 여러 종료 경로를 빠뜨리지 않는다.

```text
node --test tests/cell-selection-phase-ux.test.ts
0 pass / 4 fail
```

실패 원인은 예상대로 focus 공개 API, 단계 모델·마커, 전용 status, clear 콜백이 모두 아직 없기 때문이다.
기존 제품 코드는 바뀌지 않았다.

## 3. 구현·검증 게이트

1. focus/phase 타입과 단계 이름의 단일 모델을 추가한다.
2. 셀 선택 renderer가 1·2단계 focus 셀 중앙에 마커를 그리고 clear에서 상태 콜백을 해제한다.
3. main의 EventBus를 통해 전용 하단 상태를 갱신한다.
4. focused 테스트, Studio 전체 테스트, production build를 통과시킨다.
5. 실제 브라우저의 2×2 표에서 F5 1·2·3회와 Escape를 순서대로 확인하고 light/dark 가독성을 확인한다.
6. `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`를 통과시킨다.

R5는 #4135의 기존 계산·IME 동작을 바꾸지 않는다. 원격 push, PR 생성, GitHub 코멘트는 별도 승인 전에는
수행하지 않는다.
