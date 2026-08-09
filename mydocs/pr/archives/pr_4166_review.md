---
kind: pr_review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4166 검토 - Studio 문단 상태·표 경계·서브모드 서식 통합

## 대상과 변경 경계

| 항목 | 값 |
| --- | --- |
| 통합 PR / 작성자 | [#4166](https://github.com/edwardkim/rhwp/pull/4166) / @jangster77 |
| 기준 `devel` | `98acdd9a1a12578e1da0ceeffcdc54a60c750ab2` |
| 통합 code head | `d2a0f2e6f2ff0b1f444cadbfb376fc2288d900ee` |
| 가시성 검토 브랜치 | `review/jeong-sik-20260807-integration` |
| 원 PR | [#4120](https://github.com/edwardkim/rhwp/pull/4120), [#4133](https://github.com/edwardkim/rhwp/pull/4133), [#4134](https://github.com/edwardkim/rhwp/pull/4134) / @jeong-sik |
| 원 contributor head | #4120 `93ee8fcf9e84a7b3316e28d292f27b9ca9e31df6`, #4133 `f44e506c2f841c8d2131ed3b857fcd1bce60f72b`, #4134 `088af5756a2be0d500f5a845623b8aa0b609cd81` |
| 원 PR 적용 순서 | #4120 → #4133 → #4134 |
| 연동 이슈 | `Closes #4109` |

세 원 PR의 최신 원격 head가 위 SHA와 같은 것을 통합 직전 다시 확인했다. contributor 원 commit은
rebase, amend, reset, force-push하지 않았고, 최신 `devel` 위 검토 브랜치에 cherry-pick으로만 누적했다.

- #4120은 머리말/꼬리말 문단 상태를 툴바·눈금자와 문단 모양 대화상자의 공통 reader로 통일하고,
  Percent가 아닌 줄 간격에서 툴바 선택값을 비운다.
- #4133은 반올림 경계에서 갈라진 같은 표 선 좌표를 1.0px 이내 대표 좌표로 병합하고, 병합 전 좌표도
  대표 인덱스를 찾을 수 있게 map을 보존한다.
- #4134는 이미 존재하던 머리말/꼬리말·각주 문단 서식 WASM API를 Studio 입력 경로에 연결한다.

Canvas paint, HWP/HWPX 파서, pagination, 저장 포맷 또는 기준 PDF를 바꾸지 않는다. 표 경계의
DOM resize overlay와 입력·history 경로만 바꾸므로 PDF/SVG visual sweep 대상은 아니며, 아래의
headless Chromium 동작 검증으로 실제 상호작용을 확인했다.

## 충돌과 메인터너 보정

#4134의 `mutation-routing-guard` 기준선은 앞서 `devel`에 들어간 #4119의 input-handler 변경과
같은 행을 수정해 충돌했다. #4119의 한 호출과 #4134의 두 호출을 모두 반영해 기준선 `30`으로
해소했다. 기능 코드나 contributor 원 commit은 생략하지 않았다.

초기 통합 상태에서 #4134를 실제 브라우저로 검증하자, 머리말과 각주 모두 줄 간격 변경 자체와
값 Undo는 성공했지만 Undo 뒤 편집 모드가 `true → false`로 이탈했다. 원인은 일반
`SnapshotCommand`가 `EditContext`를 노출하지 않아 history 복원기가 이를 본문 명령으로 처리한 것이다.

메인터너 보정 `d2a0f2e6f`은 일반 `SnapshotCommand`의 기존 본문 복귀 정책을 변경하지 않았다.
대신 `applyParaFormatInHf`와 `applyParaFormatInFootnote`의 descriptor에 현재 문맥을 붙이고,
그 두 경우에만 `SubmodeSnapshotCommand`를 사용한다. 이 전용 클래스만 `EditContext`를 노출하므로
머리말 구조 삽입처럼 Undo 뒤 본문으로 돌아가야 하는 기존 snapshot 작업의 의미는 유지된다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| 적용 source SHA 재확인 | #4120, #4133, #4134의 원격 head가 누적 검토에 쓴 SHA와 모두 일치 |
| diff 정합 | `git diff --check upstream/devel...HEAD` 통과 |
| focused Studio 검증 | 문단 서식 배선·서브모드 history·mutation guard 35 passed |
| TypeScript | `npx tsc --noEmit` 통과 |
| Studio 전체 단위 테스트 | `npm test` 802 passed, 0 failed |
| Studio production build | `npm run build` 통과. Vite chunk 크기 경고만 있고 exit 0 |
| 표 경계 좌표 | `table-border-lines.test.ts`가 반올림 분리 병합, 대표 인덱스 map, 최소 셀 폭, 사슬 병합 차단을 검증 |
| Chromium 툴바 | 본문 300%·머리말 100%에서 머리말 편집 툴바가 100%를 표시했고, Fixed 30 줄 간격에서 선택 인덱스가 `-1`이 됨 |
| Chromium 머리말 | 160% → 300% 적용, Undo 160%, Redo 300%, 세 단계 모두 머리말 편집 모드 유지 |
| Chromium 각주 | `footnote-01.hwp`에서 130% → 300% 적용, Undo 130%, Redo 300%, 세 단계 모두 각주 편집 모드 유지 |
| GitHub code candidate | code head `d2a0f2e6f`의 CI, CodeQL, Render Diff 모두 성공 |

`mutation-routing-guard`는 이번 변경과 무관한 기존 이관 후보
`src/engine/input-handler-table.ts: 9 → 7`을 안내로 출력했다. 해당 파일을 이 통합 PR에서
수정하지 않았고, guard도 통과했으므로 기준선 변경 범위에 포함하지 않았다.

## 수용 판단과 merge 조건

**메인터너 보정 포함 수용 권고.** 원 PR 세 건의 입력·툴바·표 경계 계약과 #4134의 Undo/Redo
편집 문맥을 실제 브라우저까지 확인했다.

이 문서와 오늘할일은 통과한 code head 뒤에만 붙이는 trailing documentation commit이다. 문서 push 뒤에는
최신 PR head의 preflight·branch protection aggregate가 성공했고, `mergeable=MERGEABLE`,
`mergeStateStatus=CLEAN`인지 다시 확인해야 한다. 그 조건과 작업지시자의 merge 승인이 충족되면 #4166을
병합하고 #4109의 자동 종료, 원 PR #4120·#4133·#4134의 후속 comment·close, 원격 작업 브랜치 정리를
순서대로 확인한다.
