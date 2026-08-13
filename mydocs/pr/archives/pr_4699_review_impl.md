---
kind: pr-review-implementation
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4699 메인터너 보정 구현 기록

## 기준과 적용

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4699](https://github.com/edwardkim/rhwp/pull/4699) |
| 원 source head | `e58720af754d576bbef472093f2f72f5054a263a` |
| 보정 commit | `199caf545bc8f86048c2a4b126145eb131967fad` |
| 가시성 branch | `review/keepYaoung-4699-20260813` |
| 최신 devel merge tree | `33bd6d2ea91860e0cea33b835a7866dd3422a376` |

외부 contributor source branch에 메인터너 수정 권한이 있어, 원 commit `e58720af` 뒤에 독립 commit
`199caf545`를 추가했다. 원 contributor commit은 rebase, amend, force-push하지 않았다.

| 경로 | 보정 내용 |
| --- | --- |
| `rhwp-studio/src/command/export-html.ts` | 마지막 문단의 끝 offset을 `getParagraphLength`로 조회하고, section 변환 실패를 전체 export 실패로 전파 |
| `rhwp-studio/src/ui/chrome-mode.ts` | HTML·Word export command를 embed hidden command 집합에 추가 |
| `rhwp-studio/tests/export-html.test.ts` | 1,000,001 글자 문단의 정확한 끝 offset과 section 실패의 무다운로드 계약 회귀 추가 |
| `rhwp-studio/tests/chrome-mode.test.ts` | embed 모드에서 두 export command가 숨겨지는 회귀 추가 |

## 보정 이유

Studio embed는 host가 파일 열기·저장·다운로드 같은 문서 수명주기를 소유하는 실행 모드다. 일반 chrome의
파일 메뉴에 추가한 export command도 같은 hidden command 집합에 넣어야, embed host가 통제하지 않는
브라우저 download가 생기지 않는다.

문서 전체 내보내기의 끝 range는 문단 전체 길이를 engine 단위로 조회해야 한다. 고정 probe 문자열의 길이로
offset을 역산하면 아주 긴 마지막 문단이 잘리는 데도 예외가 발생하지 않는다. `getParagraphLength`는 이미
engine이 제공하는 해당 문단의 정확한 char 길이 API이므로 이를 재사용했다.

마지막으로 section 중 하나가 실패했을 때 일부 section만 모은 HTML을 성공으로 내보내면, 사용자에게 문서
완전성을 보장할 수 없다. 오류를 명시적으로 throw해 export를 원자적 명령으로 만들고 테스트에서 Blob과
download가 생성되지 않음을 확인했다.

## 검증 순서

1. 원 source를 최신 `upstream/devel`과 merge simulation해 충돌 없이 Studio build를 통과시켰다.
2. 보정 뒤 export와 chrome-mode 집중 회귀 22건, Studio 전체 886건, production build를 통과시켰다.
3. foreground Vite와 headless Chromium에서 정상 HTML·Word download, 강제 실패 시 무다운로드,
   embed command/menu 비노출을 실제로 확인했다.
4. `199caf545`를 contributor source branch에 push한 뒤 같은 SHA의 GitHub Actions Build & Test,
   Frontend package gates, CodeQL, Canvas visual diff 성공을 확인했다.
5. trailing review 문서를 추가하기 전 `git merge-tree --write-tree upstream/devel HEAD`와
   `git diff --check upstream/devel...HEAD`를 통과시켜 최신 기준선과의 병합 가능성을 고정했다.

다음 단계는 이 기록과 오늘할일만 담은 trailing docs-only commit을 push하고, 동일 PR·동일 source의
review-only fast-pass aggregate를 확인하는 것이다.
