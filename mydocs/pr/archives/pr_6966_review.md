# PR #6966 — 작성자 self-review

- PR: https://github.com/edwardkim/rhwp/pull/6966
- Issue: #6964 — https://github.com/edwardkim/rhwp/issues/6964
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
| head branch | codex/issue-6964-download-duplicates |
| 최초 제출 head | `53e3e701b0656468198f7d102c0077032332163e` |
| 검증한 code commit | `5de285adbfcf9e52b5e03c0ca6da750ec931714d` |
| 최초 제출 규모 | 11 files, +612 / -8 |
| draft / mergeable | true / MERGEABLE |
| GitHub checks | 실행 중; 최신 head 재확인 필요 |

## 범위와 검토

Firefox의 같은 ID 이벤트를 직렬화하되 첫 browser API 호출은 이벤트 수신 중 즉시 시작한다. 공통 상태 머신의 과거 항목 보호·TTL·미추적 변경 제외를 유지한다. Chrome/Firefox의 자체 extension Blob 저장만 자동 열기에서 제외한다.

Rust/parser/renderer/layout/WASM/Studio source 및 sample 변경은 없다. 렌더 출력 변경 주장이
없으므로 visual sweep과 Rust lint·전체 회귀는 대상 밖이다. 기존 WASM 재사용 사실은
[결과보고](../../report/task_m100_6964_report.md)에 기록했다.

## 완료한 검증

- `node --test --test-reporter=spec rhwp-shared/sw/*.test.js rhwp-chrome/sw/*.test.mjs rhwp-firefox/sw/*.test.mjs`: 148/148 통과.
- 두 확장 production build 및 `node --test scripts/frontend-extension-dist.test.mjs`: 통과(계약 3/3).
- 실제 Chrome/Firefox E2E와 저장본 HWP 재파싱 결과는 결과보고에 기록했다.
- 사용자가 두 수정의 결합 dist를 직접 등록해 다운로드 탭 1개, 정상 저장/다른 이름 저장
  파일명, 저장 시 추가 탭 없음, 편집 내용 보존을 모두 확인했다.
- 최신 `upstream/devel` `0d36da409`와 merge-tree 충돌 없음. 새 base 변경은 머리말 정렬이며
  이번 확장 경로와 겹치지 않는다. 제출한 code 검증 이후의 변경은 `mydocs/`뿐이다.
- `git diff --check`와 추가·변경 문서의 내부 링크를 확인했다.

## 관계와 남은 조건

관련 [PR #6965](https://github.com/edwardkim/rhwp/pull/6965)와 독립된 수정이다.
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


## 추가 Chrome packaged smoke 검증

- 검증 head: `9bbdc46dbc55dc3dd3e3dedc4ecb04cb6071815e` (기존 PR source 변경 없이 실행)
- 실행: `PUPPETEER_EXECUTABLE_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' npm --prefix rhwp-chrome run test:e2e:smoke`
- 결과: production build 성공, page-budget/proxy 계약 4/4 통과,
  `PASS: viewer/options/print/service worker/content script` 확인, exit 0.
- 로컬 Chrome을 headless·격리 프로필로 실행했으며 기존 pkg WASM을 재사용했다.
- MV3 background 시작과 메시지 정책, HWP3 문서 canvas, 다크 아이콘 자산,
  settings hydration, print.html 로드, content script 배지를 확인했다.
  console/page/worker 오류와 예상 밖 탭은 관측되지 않았다.
- smoke는 autoOpen=false이므로 다운로드 감지·편집 후 저장 회귀 테스트를 대체하지 않는다.
  기존 다운로드 E2E와 Firefox/사용자 저장 검증을 보완하는 결과다.
- 로그: `/private/tmp/pr6966-chrome-smoke.log`.


## 게시 리뷰 검토 후 보정 (2026-09-10)

- 보정 코드: `c58dedc0b` — Firefox mock에 cancel/erase 호출 기록을 추가하고 두 브라우저에서 직접 단언한다. Firefox의 `test:e2e:download` 진입점과 README 준비 절차를 추가했다.
- 다운로드 어댑터 및 공통 상태 머신의 실행 로직은 동일하다. Firefox 첫 API 호출 시점의 내부 원인은 가설로 남기며 suspend를 확정 원인으로 주석에 추가하지 않았다.
- `node --test rhwp-shared/sw/*.test.js rhwp-chrome/sw/*.test.mjs rhwp-firefox/sw/*.test.mjs`: 148/148 통과. 로그 `/private/tmp/pr6966-correction-tests.log`.
- 새 명령 `FIREFOX_EXECUTABLE_PATH=/Applications/Firefox.app/Contents/MacOS/firefox RHWP_EXPECT_BASENAME=1 npm --prefix rhwp-firefox run test:e2e:download`: Firefox 155.0.1에서 viewer 1, 다운로드 3, 저장본 파일명 정상, 편집 내용 보존 통과. 로그 `/private/tmp/pr6966-correction-firefox-e2e.log`.
- 위 E2E는 사용자가 검증했던 #6965 결합 dist와 기존 pkg WASM을 재사용했다. 이번 source 변경은 mock/실행 진입점/문서뿐이므로 dist를 재빌드하지 않았다. 실행 스크립트는 지정된 패키지를 그대로 검사하고 CI 자동 편입은 하지 않는다.
- Chrome 완료 이벤트의 복구 보장은 반례가 있다. 실제 Chrome 152.0.7977.83에서 자연 다운로드 이벤트와 최초 session 상태 쓰기 지연을 조합했을 때 현재 코드·수정 전 base 모두 대조군 3/3 탭 1, 지연군 3/3 탭 0이었다. 파일 바이트는 보존됐다. 일반 환경의 자연 발생 빈도는 미확인이다.
- 재현 코드·근거·검증 한계를 [별도 이슈 #6988](https://github.com/edwardkim/rhwp/issues/6988)에 등록했다. 이번 PR에 Chrome 동시성 구조 변경을 추가하지 않았다.
- 사용자 요청으로 보정 push 및 PR 보정 코멘트 게시를 진행한다. ready 전환·merge는 이번 요청 범위에 포함하지 않는다.


## 재검토 및 병합 준비 (2026-09-12)

- 사용자가 두 PR의 기록 갱신·push·CI 복구·Ready 전환·순차 병합과 병합 후 이슈/comment/로컬 검토 정리를 승인했다.
- 재검토 head: `526ce2331f01174d687cd36d3a398762b6858389`. 새로운 제품 코드 결함은 발견하지 못했다.
- 해당 head에서 `node --test rhwp-shared/sw/*.test.js rhwp-chrome/sw/*.test.mjs rhwp-firefox/sw/*.test.mjs`를 실행해 **148/148 통과**했다.
- 두 PR head 결합 tree `ba5c093a2a97f193e941203f621bcbf7c0a81422`에서 같은 테스트 **188/188**, Chrome·Firefox production build, `node --test scripts/frontend-extension-dist.test.mjs` **3/3**을 통과했다. 기존 pkg WASM과 설치된 Node 의존성을 재사용했다.
- 최신 fetch `upstream/devel` `f537df5ea`와 각 PR의 merge-tree 및 diff whitespace 검사는 통과했다. 결합 tree의 실행 검증을 최신 devel까지 합친 실행 검증으로 주장하지 않는다.
- 이번 재검토에서 실제 브라우저 E2E·OS 네이티브 저장 대화상자는 재실행하지 않았다. 앞선 실제 브라우저·사용자 검증 기록은 참고 증거로 확인했다. Rust/WASM/Studio 실행 소스·렌더 출력 변경이 없어 Rust 전체 회귀와 visual sweep은 대상 밖이다.
- 앞선 CI 관찰 시 두 PR은 Draft였고 `CI Impact Policy`가 `workflow-not-success:CI:cancelled`로 실패했다. 이는 CI 취소의 집계 결과이며 제품 테스트 실패로 판단하지 않았다. 이 실패 상태에서는 병합을 보류한다.
- 이 후행 기록 commit을 포함한 **최신 head의 CI·CodeQL·CI Impact Policy 성공 확인 후에만** 기존 코드 수용 판정에 따라 병합한다. 이후 첫 PR 병합으로 base가 전진하면 두 번째 PR의 mergeability와 CI를 재확인한다.
- 두 수정은 함께 배포해야 한다. Chrome의 기존 상태 저장 경합은 별도 [#6988](https://github.com/edwardkim/rhwp/issues/6988) 범위로 남는다.
- review/report는 이미 archive review 및 이슈별 보고서로 PR에 포함되어 있다. 오늘할일·결합 review_impl의 생략 판단은 앞선 기록을 유지한다. 병합 확정 SHA와 issue 종료·CI 최종 근거는 병합 후 각 PR/issue comment에 남기며 별도 문서 PR을 만들지 않는다.
- 원격 source branch 삭제는 이번 정리 범위에 포함하지 않는다. 기본 작업공간의 `samples/exam_eng.pdf` 변경과 무관한 작업공간·공유 cache를 보존한다.
