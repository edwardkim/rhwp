# PR #7465 검토 기록

## 최종 판정

**승인.** 검토 head `f1f67db89937f37804b291f00415fd9f2292280e`는 검토 범위에서 병합 차단 결함이 없었습니다. 사용자 승인 후 2026-09-28 23:38 KST에 일반 merge로 병합했습니다.

- 원 PR: [#7465](https://github.com/edwardkim/rhwp/pull/7465)
- 이슈: [#7463](https://github.com/edwardkim/rhwp/issues/7463)
- 기여자: [@lidge-jun](https://github.com/lidge-jun), source `lidge-ai/rhwp:fix/select-all-hancom`
- reviewer: @postmelee, [승인 리뷰](https://github.com/edwardkim/rhwp/pull/7465#pullrequestreview-5339589526)
- merge SHA: `4f9d33afdeafc5cc9df525eaf15bcf67ecd7e956`
- 병합 직전: 정확한 승인 head 유지, `APPROVED`, `MERGEABLE`, `CLEAN`, 미해결 review thread 0, 필수 Build & Test 성공.

## 역할과 처리 경로

collaborator가 외부 기여 PR을 검토했습니다. 조직 fork에 reviewer 문서를 push할 수 없는 상황에서 사용자는 원 PR merge 후 별도 기록 PR 등록·merge 및 후속 코멘트를 명시적으로 승인했습니다. 이 지시에 따라 contributor branch 변경 없이 원 PR을 일반 merge하고, 이 문서·검증 로그·오늘할일만 후속 review-only PR로 보존합니다. 원 기여 코드 보정과 커밋 rewrite는 없습니다. 원 contributor fork branch는 유지합니다.

적용 문서: `pr_review_workflow.md`, `intake_and_review.md`, `local_validation.md`, `edit_command_review_checklist.md`, `collaborator_external_pr.md`, `review_only_fast_pass.md` B, `post_merge.md`. 후속 기록 PR은 collaborator self 경로로 처리하며 보호 규칙 우회 없이 최신 head의 CI와 mergeability를 확인합니다.

## 문제와 변경 검토

⌘/Ctrl+A가 셀·중첩 셀·글상자의 편집 문맥을 따르도록 선택 범위를 제한하고, 본문 전체 선택에는 표의 선택 오버레이를 포함합니다. 전체 선택으로 화면이 문서 끝으로 이동하는 현상을 방지하고 툴바와 input/textarea/select/contentEditable/textbox·모달의 키 소유 경계를 분리합니다.

확인한 경로는 셀 선택의 첫·끝 문단 계산, 기존 anchor 초기화, 중첩 cell path, 글상자 범위, 본문 표 앵커 수집과 문서 변경 시 캐시 무효화, caret 이동과 스크롤 유지, 입력 요소·모달에서 문서 명령을 가로채지 않는 동작입니다. 선택 자체는 문서를 변경하거나 undo history를 추가하지 않으며 선택 후 삭제/undo는 브라우저 검사로 확인했습니다.

## 직접 실행한 검증

검증 당시 source는 위 head와 같았으며 이후 source 변경은 없습니다. 이전에 실행한 결과를 보존하며 후속 문서 작업을 이유로 같은 검사를 재실행하지 않았습니다.

| 명령/범위 | 실제 결과 | 증적 |
| --- | --- | --- |
| Node v24.15.0: Studio `node --test tests/*.test.ts ../npm/editor/tests/*.test.mjs` | 1,811 PASS / 0 FAIL / 0 SKIP | [unit 로그](../assets/pr7465/7465-unit.txt) |
| select-all 및 #3414 focused Node 검사 | 11/11 PASS | [focused 로그](../assets/pr7465/7465-focused.txt) |
| Studio `npx tsc --noEmit` | exit 0 | [tsc 로그](../assets/pr7465/7465-tsc.txt) |
| Chrome `select-all-cmd-a.test.mjs --mode=headless` | 63개 PASS, pageerror 0 | [E2E 로그](../assets/pr7465/7465-e2e.txt) |
| merge simulation / whitespace | 검토 당시 devel `d6cf1605327ced1c276f17a529192a743a766209`와 충돌 없음, diff check 성공 | 병합 직전 GitHub CLEAN 재확인 |

브라우저 검증은 기존 로컬 WASM을 사용했습니다. reviewer가 #7465에 fresh WASM 빌드를 수행한 것으로 보고하지 않습니다. 작성자의 별도 fresh WASM 실행 설명과 reviewer 직접 실행을 구분합니다.

## CI 및 적용하지 않은 게이트

정확한 head의 [CI run](https://github.com/edwardkim/rhwp/actions/runs/36387655805)에서 [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/36387655805/job/108818298589), Frontend package gates가 성공했고 CodeQL, Canvas visual diff, adapter inter-diff, prop roundtrip 및 CI Impact Policy 성공도 확인했습니다. 변경 범위에 따라 Rust lint/builders/Native Skia/WASM Build와 일부 frontend gate는 skipped였습니다. 모든 job이 실행됐다고 해석하지 않습니다.

조판 원칙·조판 Visual Sweep은 **비해당**입니다. 변경은 Studio 선택 상태와 UI 오버레이이며 문서 모델·조판·본문 paint·페이지 배치를 바꾸지 않습니다. Rust lint·Rust 통합 회귀 재실행은 이 frontend-only 변경에 비해당입니다. 검증 입력은 E2E 코드에서 생성한 문서이며 별도 외부 HWP/PDF를 수용 근거로 사용하지 않았습니다.

원 CI의 artifact 목록에는 trusted merge-tree 증명만 있고 B/C/D 실행 시간 측정 artifact는 없습니다. 따라서 duration 정책 갱신은 증거 부족으로 보류 대상입니다. 병합 후 새 검증 CI를 실행하지 않았으며, 이 보류를 검증 실패로 해석하지 않습니다.

## 제한과 잔여 위험

실제 여러 `document.sections`가 있는 문서의 표 강조 및 한컴 실기기 동작은 reviewer가 직접 대조하지 않았습니다. 작성자의 한컴 Mac 셀 선택 관측과 구분합니다. 성능 정량 측정도 별도 미실행입니다. 이 제한은 원 PR 승인 리뷰에 공개했으며 현재 검토 범위의 blocker로 판정하지 않았습니다.

## Merge 후 contributor PR comment 계획

후속 기록 PR을 devel에 병합한 뒤 원 PR에 기여 감사, 원 merge SHA, 실제 Node/Chrome 검증, CI와 비해당 범위, 영구 리뷰 문서 링크를 한국어 존댓말로 게시합니다. 조판 시각 검증으로 수용한 PR이 아니므로 다른 PR의 Visual Sweep 이미지·수치를 가져오지 않습니다.

관련 #7463은 devel merge 후 자동 close되지 않은 상태를 확인했습니다. 기록 통합 및 devel sync 뒤 재조회하고 여전히 OPEN이면 승인된 후속 처리 범위에서 수동 close 후 병합/검증 근거 코멘트를 남깁니다. 이미 같은 작업의 코멘트가 있으면 중복 게시하지 않습니다. 종료 이후 원 contributor fork와 #7464/#7468의 진행 중인 검토 자료는 보존하며 #7465 전용 임시 branch/worktree만 정리합니다.
