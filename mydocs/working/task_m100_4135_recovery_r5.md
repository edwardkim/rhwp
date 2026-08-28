# Task M100 #4135 Recovery R5 — F5 셀 선택 단계 표시

- **작업 기준**: `13f9de80b4c2`
- **관찰한 upstream**: `upstream/devel@b1485e0a143d`
- **브랜치**: `codex/issue-4135-contextual-shortcut`
- **계획**: [`task_m100_4135_impl.md`](../plans/task_m100_4135_impl.md)
- **선행 결과**: [`task_m100_4135_recovery_r4.md`](task_m100_4135_recovery_r4.md)
- **승인**: 작업지시자가 R4 한글 IME 재검증을 `수정이 반영되었어.`로 승인한 뒤,
  한컴의 셀 안 단계 마커와 하단 문구를 함께 쓰는 하이브리드 UX를 `그렇게 진행해줘.`로 승인
- **현재 판정**: R5 구현·자동·실브라우저 검증과 작업지시자 수동 확인 GREEN, PR 준비 진입

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

## 4. 구현 결과

- `cell-selection-phase.ts`가 1·2·3단계 타입과 하단 사용자 문구를 단일 소유한다.
- CursorState는 복사된 focus 좌표만 공개하고 anchor·내부 가변 상태는 노출하지 않는다.
- renderer는 1·2단계에서 focus를 포함하는 bbox 중앙에 10px 원 마커를 추가한다. 3단계는 전체 셀
  하이라이트가 충분한 주 표시라 마커를 만들지 않는다.
- renderer의 public `clear()`가 마커·하이라이트 제거와 `onPhaseChange(null)`을 함께 소유한다. 기존
  Escape, 일반 입력, 마우스 전환, 개체 선택, undo 경로의 clear 호출을 각각 고쳐 복제하지 않았다.
- main은 callback을 `cell-selection-phase-changed` EventBus 이벤트로 연결한다. 전용
  `#sb-cell-selection`은 같은 label을 다시 쓰지 않아 zoom·방향키 재렌더 때 live region이 반복 발화하지
  않는다.
- 라이트·다크 테마에 각각 회색/주황 마커 토큰을 두고 기존 파란 셀 선택 배경은 보존했다.
- 보호 셀 클릭으로 생기는 내부 선택은 F5 단계가 아니므로 기존 파란 하이라이트만 유지한다.

## 5. 자동 검증

```text
node --test \
  tests/cell-selection-phase-ux.test.ts \
  tests/cell-selection-caret-sync.test.ts \
  tests/issue-4135-contextual-shortcut.test.ts \
  tests/issue-4135-block-calculation-plan.test.ts
29 pass / 0 fail

npm test
1,251 tests / 1,250 pass / 1 skip / 0 fail

npm run build
PASS — TypeScript + Vite, 241 modules transformed
```

R5는 Studio UI 오버레이 변경이며 문서 renderer/layout·저장 출력은 바꾸지 않는다. 시각 검증 거버넌스의
`studio/확장 UI (렌더 엔진 무관)` 경로에 따라 PDF/SVG visual sweep 대신 실제 기능 스모크와 라이트·다크
브라우저 화면 확인을 적용했다.

## 6. 실브라우저 검증

기존 미저장 문서가 있는 `7715` 탭은 건드리지 않고 `http://127.0.0.1:7716/?r5-phase-ux=1`의 새 탭에서
빈 문서에 2×2 표를 만들었다.

| 조작 | DOM 계측 | 시각 판정 |
| --- | --- | --- |
| F5 1회 | single marker 1, range marker 0, highlight 1 | focus 셀 중앙 회색 원, `셀 선택 · 방향키로 이동` |
| F5 2회 | single marker 0, range marker 1, highlight 1 | focus 셀 중앙 주황 원, `셀 범위 선택 · 방향키로 확장` |
| 2단계에서 오른쪽 방향키 | range marker 1, highlight 2 | 주황 원이 오른쪽 focus 셀로 이동 |
| F5 3회 | marker 0, highlight 4 | 2×2 전체 파란 선택, `표 전체 선택` |
| Escape | marker 0, highlight 0, status hidden·빈 문자열 | 셀 선택 표시 완전 제거 |

위 전이를 다크 테마에서 순서대로 확인하고, 라이트 테마에서도 1단계 회색·2단계 주황 마커와 하단 문구의
가독성을 다시 확인했다. 한컴 실측의 주 표시를 차용하면서도 하단 텍스트로 색상 외 상태 정보를 함께
제공한다.

## 7. 작업지시자 최종 확인

작업지시자가 같은 로컬 빌드에서 위 F5 단계 표시를 직접 확인하고 `확인되었어.`라고 수동 게이트를
승인했다. 이에 따라 R5에는 남은 제품 구현이나 UX 판정 항목이 없다. 후속은 새 Recovery 단계를 만들지
않고 최신 `upstream/devel` 통합, 변경 범위별 전체 로컬 게이트, 최종 보고서·PR 본문 초안 작성의 PR 준비
절차로 전환한다. 원격 push와 PR 생성은 계속 별도 승인 대상이다.
