# PR #4805 자체검토 — 한글 IME Ctrl+A 모두 선택

## 절차 라우팅

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_self_merge.md, intake_and_review.md, local_validation.md
current code head: b6143525614b890ef9f760d7aca2c1f88af47c81 (작성 시점 참고값)
```

## PR 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4805](https://github.com/edwardkim/rhwp/pull/4805) |
| 작성자 | `jangster77` (collaborator self PR) |
| base / head | `devel` ← `task_m100_3950` |
| code head | `b6143525614b890ef9f760d7aca2c1f88af47c81` |
| 규모 | 9 files, +190/-1 |
| 관련 이슈 | `Closes #3950` |
| 작성 시점 상태 | `MERGEABLE`, checks 진행 중으로 `BLOCKED` |

collaborator 자신의 PR이므로 외부 reviewer는 요청하지 않고 자체검토로 처리한다. 위 상태값은 작성 시점
참고값이며, merge 전에는 trailing head, GitHub Actions 및 mergeability를 다시 확인해야 한다.

## 변경 검토

1. `edit:select-all`에만 `KeyA` 물리 키 보정을 추가했다. `Ctrl+Shift+A`의 표 평균 단축키는 shift
   조건이 달라 기존 매핑과 충돌하지 않는다.
2. IME 분기는 기존 Ctrl+M chord 처리 뒤에 기본 Ctrl/Meta 단축키 dispatch를 둔다. 따라서 Ctrl+M 우선순위와
   매칭되지 않은 조합·탐색 키의 기존 반환 경로를 유지한다.
3. `ㅁ`과 `Process`의 `KeyA` 매핑 및 IME 조기 반환 전 dispatcher 전달을 회귀 계약으로 추가했다.
   키보드 모듈의 확장자 없는 내부 import는 Node 단독 실행과 맞지 않아, 분기 전달은 소스 계약 테스트로
   고정하고 실제 Windows Chrome의 한글 문서 전체 선택으로 동작을 보완 확인했다.
4. renderer, layout, HWP/HWPX fixture, golden 또는 workflow 변경은 없다. 시각·fixture 증적 보조 경로는
   적용하지 않았다.

## 실행한 검증

- `node --test tests/shortcut-map.test.ts tests/ime-shortcut-routing.test.ts` — 8 passed
- `npx.cmd tsc --noEmit` — 통과
- `npm.cmd test` — 931 passed, 0 failed, 1 skipped
- Windows Chrome에서 `가나다라마바사` 문서의 `Ctrl+KeyA` 전체 선택 — 통과
- `git diff --check` — 통과

## 판정

코드 범위와 회귀 계약이 이슈 원인에 맞고, 발견한 보정 필요 사항은 없다. 최신 trailing head의 required
checks가 모두 통과하고 작업지시자가 merge를 승인하면 squash merge를 권고한다.
