# PR #8379 리뷰 — 연속 형광펜 모드

## 최종 판정

**승인 — 보정된 원 PR 정상 merge 완료.**

원 기여의 드래그별 칠하기·실행 취소·Esc/버튼 종료를 유지하고, F5 셀 블록과 빈 길이 선택의
잘못된 진입 판정만 보정했다. 원 head의 기존 E2E 24개 검사는 통과했으나 새 경계 검사에서
4개 assertion이 실패했다. 보정 후 34개 검사와 기존 #6635의 64개 검사가 모두 통과했다.

작업지시자의 원격 처리 승인에 따라 보정 head `e6439bd3ca7e90bd3894ac7f8524df41c6553c93`의 CI·정책 검사와
self-review를 통과한 뒤, 원 PR을 merge `ba9a3c5f222a151736a4a21f9743bf8d9ddfbd8a`로 정상 통합했다.
원 기여 commit을 보존했고 실제 merge 부모와 tree가 검증한 base/head·tree와 같음을 확인했다.
원 head의 과거 CI를 보정 head의 CI로 재사용하지 않았다.

## 접수와 후보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8379](https://github.com/edwardkim/rhwp/pull/8379), `greenoceanplanet`, 첫 외부 기여, Open·non-draft |
| 원 source | `greenoceanplanet/rhwp_mod:feat/continuous-highlighter-devel` |
| 원 head | `34b51b70098230bfc568a7c566ab73057ad6a140` |
| 검증 base | `de2295d430b588f0741021f66fae086eb4ff4009` |
| 원 공통 조상 | `1cb3a6dae1094f6ed552a8fdbffea8b89868d555` |
| 원 변경·최신 base의 로컬 병합 | `503e53e9463d242ee26b5aedf4bda251538cbe49`, 충돌 없음 |
| 로컬 보정 commit | `8eb7b8aab92551e173578e8c3c7effcef07cf725` |
| 원 source 위에 준비한 보정 head | `e6439bd3ca7e90bd3894ac7f8524df41c6553c93`, parent는 원 head |
| 검증한 병합 tree | `c93a15af1757a7568f839a57c6c92ac183d83c53` |
| branch | `review/pr-8379-continuous-highlighter` |
| reviewer | `edwardkim`, requested-reviewers API로 확인 |
| 관련 이슈 | 원 PR은 기능 제안으로 관련 이슈 없음. 기존 계약은 #4162·#4271 참조 |

기본 경로는 `maintainer_general`, 보조 경로는 `intake_and_review`, `local_validation`,
`first_time_contributor`다. 해당 문서와 템플릿, 문서·Git 및 개발 환경·GitHub 운영 절차를
읽었다. 최초 read-only 사전 조사에서 상세 diff를 먼저 읽은 순서상 차이가 있었으며,
local fetch·worktree 생성·검증은 reviewer 지정·API 재확인 뒤 수행했다.

원 PR은 8파일 +210/-3, 1commit이며 Rust·의존성·workflow를 변경하지 않는다.
메인터너 제품 보정은 입력 처리 7줄과 기존 E2E의 57줄 추가다. 원 기여 commit을 보존하고,
fork 제출용 후보에는 관련 없는 devel 이력을 추가하지 않았다. 준비한 source와 최신 base의
`merge-tree`가 실제 검증한 로컬 보정 tree와 **전체 파일에서 동일**함을 확인했다.

## 기존 해결과 재현

[#4162](https://github.com/edwardkim/rhwp/issues/4162)는 일반 텍스트 anchor가 없는 F5 셀 블록도
즉시 서식을 적용해야 한다는 계약이다. [#4271](https://github.com/edwardkim/rhwp/pull/4271)의
구현 `645c586b37`, 문서 전환 보정 `b195f323bf`, 실제 merge `d4834d7375`와
[당시 검토](pr_4271_review.md)를 확인했다. 현재 base는 이 경로를 유지한다.

`highlight-pick`이 `cursor.hasSelection()`만 사용하면서 별도 cellAnchor/cellFocus의 F5 블록을
선택 없음으로 오인했다. 반대로 본문 anchor만 남은 빈 길이 선택은 실제 글자가 선택됐다고
보아 연속 모드 대신 캐럿 대기 서식 경로로 갔다.

| 실제 브라우저 경계 | 최신 base | 원 PR + 최신 base | 보정 후보 |
| --- | --- | --- | --- |
| 표 만들기 → CELL 입력 → F5 → 초록 스워치 | 즉시 초록, undo +1 | 흰색 유지, undo +0, 연속 모드 켜짐 | 즉시 초록, undo +1, 연속 모드 꺼짐 |
| Home → Shift+Right → Shift+Left → 색 선택 | 기존 캐럿 서식 정책 | 연속 모드 진입 실패 | 연속 모드 진입, undo +0 |
| 원 연속 모드·드래그·undo·종료·일반 블록 | 해당 신규 기능 없음 | 원 E2E 24개 검사 통과 | 원 검사 포함 34개 검사 통과 |

base의 F5 대조도 같은 최적화 JS/WASM을 브라우저 응답에서 공급해 검증했다. 최초 base
하네스의 alias 구성은 기존 package를 읽거나 로딩하지 못해 근거에서 제외했고, 최종
대조에서는 초기 로딩 응답의 SHA-256이 후보와 같음을 확인했다.

## 실제 검증 결과

[정식 증적](../assets/pr8379-continuous-highlighter/README.md)에 명령·결과·JSON·화면을 보존했다.

| 검증 | 결과 |
| --- | --- |
| `tsc --noEmit` | 통과 |
| `npm test` | 2,716 PASS / 0 FAIL / 2 기존 선택적 Node WASM skip |
| `npm run build` | 통과 |
| `npm run e2e:continuous-highlighter` | 수정 전 27 PASS / 4 FAIL → 수정 후 34 PASS / 0 FAIL |
| `npm run e2e:issue-6635` | 기존 키보드·팔레트·선택·undo 64 PASS |
| E2E manifest | tracked 184개 / 행 184개, 불일치 없음 |
| 추가 실제 브라우저 진단 | F5·빈 선택·양식 보호 통과, page error 0 |
| fresh WASM | 공용 target, locked release + wasm-opt 성공, package/public 동일 |
| 브라우저 WASM 소비 | 최종 5세션의 초기 로딩 응답이 빌드 SHA-256과 일치 |

WASM SHA-256은 `b2b50b86aa1a041cfd61022a92248d545aa0910fbe58895fe5bae87553de63c7`이다.
빌드 source는 `503e53e946`이며 이후 제품 변경은 TypeScript 입력 분기와 E2E뿐이다.
새 source를 제출하는 데 다른 WASM build를 실행할 필요가 있는 Rust 변경은 없다.

최초 전체 Studio 실행은 package 준비 전에 시작해 `@wasm/rhwp.js` 로딩 등 54건이 실패했다.
빌드 완료 후 최종 전체 실행은 위와 같이 통과했다. 이를 PR 결함 또는 통과한 실행으로
기록하지 않는다. 2 skip은 #6788 혼합 모양과 pending-char-shape의 선택적 `pkg-node`
왕복이며, 이번 기능의 실제 브라우저/WASM 검사와 구분한다.

원 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/40932312119),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/40932312025), CodeQL·Adapter·
Proptest·정책 검사에는 최종 조회 시 실패·진행 중 항목이 없었다. 보정 source의 실제 새 CI 결과는 아래에 기록했다. workflow 재실행·취소를 수행하지 않았다.

## 공통 조판·입력 확인과 잔여 범위

공통 조판 원칙은 비해당이다. 이 변경은 기존 `applyCharFormat`의 호출 시점과 선택 대상을
연결하며 측정·줄 구성·페이지 분할·renderer·저장 포맷·golden·시각 래칫을 변경하지 않는다.
공통 helper로 실제 선택을 판정하고 기존 셀 블록 서식·history 경로를 소비한다.
문서별 예외, 좌표 clamp, 수치 보정이나 backend별 분기를 추가하지 않았다.

검증 입력 커밋 확인은 비해당이다. 기존 E2E의 새 문서·표 만들기·실키로 입력했으며 외부
HWP/HWPX/PDF 파일을 사용하지 않았다. 합성 입력 코드는 검토 대상 E2E에 포함돼 있다.
Rust 전체 회귀·Native Skia·Print/Native/WASM 전쪽 TSV는 이번 Studio 입력 처리 범위에
비해당으로 실행하지 않았다. 기존 CI를 그런 실행의 대체 근거로 주장하지 않는다.
화면 캡처는 팔레트 적용의 UI 증거이며 독립 한컴 출력과의 피델리티 측정이 아니다.

원 구현처럼 문서 전환 후에도 Esc/버튼 종료 전에는 모드가 유지되며, 양식 모드에서는
펜 표시가 남아도 기존 보호 경로가 실제 서식 변경을 막는다. 추가 종료 정책·각주 서식
지원·도구 접근성 변경은 이번 최소 보정에 포함하지 않았다. 성능상 새 동작은 색 선택 시
기존 선택 helper 조회이며, 드래그 paint·history 경로에 새 스캔을 넣지 않았다. 대용량 문서의
정량 성능 측정은 수행하지 않았다.

기본 작업공간의 `devel`, 사용 중인 Studio와 배포 WASM, 다른 작업의 worktree를 유지했다.
wrapper가 만든 검토 worktree의 추적 public JS는 검증 뒤 복원했으며 배포 파일을 제출하지 않는다.

## 원격 처리와 contributor PR comment 계획

[원격 처리 계획·댓글 초안](../assets/pr8379-continuous-highlighter/remote-action-plan.md)에 따라
원 source에 최소 보정 push → 새 head CI → self-review → 기록 반영·최종 head 확인 →
원 PR 정상 merge → 감사·첫 기여 환영 댓글·devel 동기화·정리를 진행하는 경로다.
승인된 code head의 push와 PR 본문 보정·API 대조를 완료했다. 첫 기여자 fork의 실행 승인이
필요한 6개 run을 정확한 head에서 확인한 뒤 승인했으며, workflow를 재실행하거나 취소하지 않았다.

운영 기록은 메인터너 일반 경로의 `post_merge.md` 7.1에 따라 archive review·asset·오늘할일을
병합 후 devel에 한 commit으로 반영한다. fork source에 오늘할일 `20261102.md`가 없어
후행 기록을 강제하면 add/add 충돌이나 관련 없는 devel 이력 반영이 필요하다.
기여 commit과 승인된 code head를 유지하는 메인터너의 직접 운영 기록 경로를 선택했다.

## 보정 head의 CI·self-review와 정상 merge

- source head: `e6439bd3ca7e90bd3894ac7f8524df41c6553c93`. 원 head 위의 별도 보정 commit이며 force-push하지 않았다.
- [CI](https://github.com/edwardkim/rhwp/actions/runs/41416091309),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41416091139),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41416091095),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/41416091123),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/41416091079),
  [최종 CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/41419949269): success.
- exact head의 check는 20 success / 1 neutral / 19 skipped, 진행 중·실패 0이다.
  Rust·Native worker는 Studio 범위의 정식 영향 정책에 따라 skip됐다.
- GitHub Frontend 실제 로그에서도 2,716 PASS / 0 FAIL / 2 skip과 undo·responsive·package
  gate 통과를 확인했다. 단순 녹색 표시만으로 실행 수치를 추정하지 않았다.
- GitHub checkout `c136f412bd01e80695202c5982a4a6a159a2f8cb`는 부모가 최신 base와 보정 source이며,
  tree는 로컬 검증 tree `c93a15af1757a7568f839a57c6c92ac183d83c53`와 같다.
- [Self-review](https://github.com/edwardkim/rhwp/pull/8379#pullrequestreview-5734367960): 새 head를 다시 검토해 추가 blocker 없음·APPROVED를 확인했다.
- `MERGEABLE/CLEAN`, Build & Test·정책 성공·최신 devel·head·merge-tree를 재확인하고
  `--match-head-commit`으로 원 PR을 정상 merge했다.
- merge `ba9a3c5f222a151736a4a21f9743bf8d9ddfbd8a`, 시각 `2026-11-02T09:23:15Z` (UTC). 실제 부모는 검증 base와 source이고,
  실제 tree도 위 검증 tree와 같다. closing issue는 없다.

[원격 수행 증적](../assets/pr8379-continuous-highlighter/remote-integration.json)에 push·fork 실행 승인·
본문/API 확인·CI·self-review·merge·종료 이슈 확인을 연결했다. 원칙대로 병합 후 제품 CI를
재실행하지 않는다. 운영 기록 반영 뒤 devel 동기화·감사 댓글·종료 정리 결과는
`/home/edward/.local/state/rhwp/pr8379-cleanup-20261102.json`에서 확인한다.

병합 후 시간 메타데이터 run `41420219752`는 success로 끝났고, 실제 collector 결과는
`ready:false`, `reason:no-verified-pr-duration-measurements`다. 이번 frontend 범위의 Rust
worker skip으로 실측 자료가 없어 갱신을 보류했다. 이를 이유로 제품 CI를 다시 실행하지 않았다.
