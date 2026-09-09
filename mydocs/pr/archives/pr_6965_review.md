# PR #6965 — 작성자 self-review

- PR: https://github.com/edwardkim/rhwp/pull/6965
- Issue: #6961 — https://github.com/edwardkim/rhwp/issues/6961
- base route: collaborator_self_merge.md
- modifiers: intake_and_review.md, local_validation.md, review_only_fast_pass.md
- loaded documents: pr_review_workflow.md, pr_review/README.md, collaborator_self_merge.md,
  intake_and_review.md, local_validation.md, review_only_fast_pass.md, docs_and_git_workflow.md,
  github_operations.md
- 승인 범위: 사용자 “진행해줘”는 두 branch push와 draft PR 생성을 승인했다.
- 작성자 self-review이며 reviewer assign 또는 GitHub approve를 수행하지 않는다.

## PR metadata — 작성 시점 참고값

| 항목 | 값 |
| --- | --- |
| 작성자 | postmelee |
| base | devel |
| head branch | codex/issue-6961-download-filename |
| 최초 제출 head | `6e957ebfed0fa3f527026244c72fc6ddca0bc943` |
| 검증한 code commit | `65e4b32e0bde1fdb0bc99300e37d85d5f57af5e7` |
| 최초 제출 규모 | 13 files, +238 / -6 |
| draft / mergeable | true / MERGEABLE |
| GitHub checks | 실행 중; 최신 head 재확인 필요 |

## 범위와 검토

공통 basename 함수와 Chrome/Firefox viewer URL 경계만 변경했다. POSIX/Windows/UNC 경로를 제거하고 한글·퍼센트 문자열·구분 문자 인코딩 왕복을 보존한다. 다운로드 상태 머신과 Studio 저장 구현은 변경하지 않았다.

Rust/parser/renderer/layout/WASM/Studio source 및 sample 변경은 없다. 렌더 출력 변경 주장이
없으므로 visual sweep과 Rust lint·전체 회귀는 대상 밖이다. 기존 WASM 재사용 사실은
[결과보고](../../report/task_m100_6961_report.md)에 기록했다.

## 완료한 검증

- `node --test --test-reporter=spec rhwp-shared/sw/*.test.js rhwp-chrome/sw/*.test.mjs rhwp-firefox/sw/*.test.mjs`: 171/171 통과.
- 두 확장 production build 및 `node --test scripts/frontend-extension-dist.test.mjs`: 통과(계약 3/3).
- 실제 Chrome/Firefox E2E와 저장본 HWP 재파싱 결과는 결과보고에 기록했다.
- 사용자가 두 수정의 결합 dist를 직접 등록해 다운로드 탭 1개, 정상 저장/다른 이름 저장
  파일명, 저장 시 추가 탭 없음, 편집 내용 보존을 모두 확인했다.
- 최신 `upstream/devel` `0d36da409`와 merge-tree 충돌 없음. 새 base 변경은 머리말 정렬이며
  이번 확장 경로와 겹치지 않는다. 제출한 code 검증 이후의 변경은 `mydocs/`뿐이다.
- `git diff --check`와 추가·변경 문서의 내부 링크를 확인했다.

## 관계와 남은 조건

관련 [PR #6966](https://github.com/edwardkim/rhwp/pull/6966)와 독립된 수정이다.
사용자가 확인한 결과는 두 수정이 포함된 패키지 기준이므로 배포 시 둘 다 포함해야 한다.
초기 Firefox 큐의 첫 호출 시점 보정과 자동화의 한계는 #6964 결과보고를 따른다.

현재 source에는 오늘할일 `20260910.md`가 없고 최신 base에는 다른 PR의 기록만 있다.
무관한 오늘할일·archive 링크를 source에 복사하지 않고 이번 PR의 진행 상태는 이 review와
결과보고에 남긴다. 개별 구현 계획은 기존 이슈 계획/단계 보고가 있어 별도 review_impl은 생략한다.

## 최종 판정

- 판정: 승인
- 근거: 위 code 범위의 로컬 검증, 실제 브라우저 결합 검증과 사용자 직접 검증을 통과했다.
- merge 전 조건: review 기록을 포함한 최신 head의 GitHub required checks 통과,
  mergeability 재확인, 사용자의 ready 전환 및 merge 승인.
- 이 판정은 GitHub approve 또는 merge가 아니다. 현재 승인된 원격 조치는 push와 draft 생성이다.
