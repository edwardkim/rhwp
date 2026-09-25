# PR #7283 self-review — Chrome 확장 E2E와 영향 기반 CI

## 최종 판정

**승인 — 코드 검토 판정.** 아래 P2는 `3aaa49cf9`에서 수정했고 회귀 검증을 통과했다.
merge 전 조건은 수정 후 최신 PR head의 GitHub Actions 통과와 작업지시자 병합 승인이다.
이 기록은 GitHub approve 또는 merge를 수행하지 않는다.

## 2026-09-25 리뷰 후 보완

- 코드 후보: `3aaa49cf9` (`fix(ci): accept successful Chrome fallback in impact audit`).
- Chrome이 선택 사항일 때 독립 감사는 `skipped`와 `success`를 허용한다. CI가 파일 목록 조회나
  분류 실패 때문에 추가 실행한 정상 결과를 받아들이도록 기존 `requireSafeJobConclusion`을 재사용했다.
- 필수 Chrome의 skip, missing job, failure/cancelled/timed_out/neutral은 거부한다.
  review-only fast-pass의 Chrome skip 계약과 CI 자체 Build & Test 집계 조건은 유지했다.
- 정식 `scripts/tests/ci-impact-policy.test.cjs`에서 Firefox·CLI 전용 변경에 대해 controller 정상 조회와
  CI `collection-error`를 분리하고 실제 fallback 분류 결과로 job 증적을 만들었다.
  정상 skip 대조군과 성공한 전체 실행은 통과하고 추가 실행의 실패·취소·시간 초과는 실패했다.
- 새/보강 검사 **수정 전 3개 실패 → 수정 후 통과**. 기존 safe-full 감사 테스트에도 Chrome을 포함했다.
- 로컬 Node **177/177**, Python **137/137** 통과. 명령은 아래 리뷰와 같은 묶음이며
  새 회귀 검사 1개가 추가됐다. 로그: `/private/tmp/rhwp-7283-fix-before.log`,
  `/private/tmp/rhwp-7283-fix-node.log`, `/private/tmp/rhwp-7283-fix-python.log`.
- 실제 원격 API 오류를 유발하지 않고 policy 함수에 해당 입력을 전달해 검증했다.
  제품·browser harness·workflow YAML·Rust 변경은 없어 브라우저/전체 Rust/시각 검증은 로컬에서 반복하지 않았다.
- 아래 원격 33 success/4 skipped와 브라우저 증적은 수정 전 `cde215c6`에 해당한다.
  수정 후 원격 결과는 PR의 최신 head에서 확인해야 한다.

## 2026-09-25 현재 head 코드 리뷰

- 검토 head: `cde215c6e3ae94a4095cd7c0fe5ec53322ea6183`, base `devel`.
  PR diff 기준 공통 조상 `505661360e9a2d596f55300d0cb0c5222f0e14b4`.
- collaborator self 경로. intake_and_review, local_validation, rework_and_exceptions,
  review_template을 적용했다. reviewer 지정이나 GitHub review·comment·push·merge는 수행하지 않았다.
- lifecycle, 탭 감시, 실행기/실패 진단, 영향 분류, job 집계/감사, 캐시 및 promotion 증적 처리를 검토했다.
  38파일, +2,665/-100. 조판·제품 runtime·fixture 변경은 없어 조판 원칙과 Visual Sweep은 **비해당**이다.

### P2 — 보수적으로 추가 실행한 Chrome 성공을 감사에서 거부 (발견 당시, 위 보완으로 해결)

위치: `scripts/ci-impact-policy.cjs:782–783`.

`rhwp-firefox/background.js`만 수정한 devel PR에서 CI의 파일 목록 API 조회가 실패하면
`.github/workflows/ci.yml`의 수집 catch가 `collection-error`를 기록한다. 분류기는 full 실행을 선택하고
Chrome E2E와 package job을 실행한다. 별도 controller의 조회가 성공하면 Chrome은 필요 없다고
판정한다. 이때 CI 전체와 Chrome job이 모두 성공해도 `requireJobConclusion(..., 'skipped')`가
`CI:job-not-skipped:Chrome extension E2E:completed:success`를 반환한다.
정상적으로 복구한 실행이 독립 감사에서 실패하므로 #3515의 판정 실패 시 실행 정책과 모순된다.

실제 policy 함수와 기존 테스트 fixture helper로 정상 skip 대조군은 success, collection-error로
full 실행한 성공 증적은 failure임을 재현했다. 브라우저 오류가 아니라 감사 로직의 재현 가능한 결함이다.
필요하지 않은 lane에는 기존 Rust/Native/frontend 감사처럼 `skipped` 또는 `success`를 허용하고,
필수 Chrome의 skip과 실행된 Chrome의 failure/cancelled는 계속 거부해야 한다. 기존 safe-full 테스트도
Chrome을 full 실행에 포함해야 한다. CI 자체의 로컬 output과 job 결과 일치 검사는 그대로 유지할 수 있다.

### 실행한 검증과 한계

- 현재 head에서 Node **176/176**, Python **137/137** 통과.
- Node: Chrome impact, CI classifier/policy/controller/evidence/report, tab-monitor,
  failure-diagnostics 계약. Python: CI/Chrome/controller/CodeQL/review-only/wiring/promotion 계약 8모듈.
- 재현 스크립트: `/private/tmp/rhwp-7283-audit-repro.cjs`.
  결과: `/private/tmp/rhwp-7283-audit-repro.json`.
  일반 테스트 로그: `/private/tmp/rhwp-7283-review-node.log`, `/private/tmp/rhwp-7283-review-python.log`.
- 원격 재조회: 현재 head **33 success / 4 skipped**.
  [CI 35981603616](https://github.com/edwardkim/rhwp/actions/runs/35981603616)의 Chrome E2E도 success.
  위 장애 복구 반례는 해당 정상 실행에서 발생하지 않아 CI 녹색과 모순되지 않는다.
- 이번 리뷰에서 실제 브라우저/전체 Rust 빌드는 재실행하지 않았다. 현재 head의 원격 실행과 기존
  source별 증적을 참고했다. 원격 API 장애는 유발하지 않고 해당 입력과 성공 job 증적으로 감사 함수를 실행했다.
- 기존 E2E 입력 3개는 검토 commit에 포함되어 있고 checkout의 SHA-256은 아래와 같다. 새로운
  HWP/HWPX/PDF 입력은 생성하지 않았다.

| 입력 | SHA-256 |
| --- | --- |
| `samples/hwp3-pagedef-1915.hwp` | `b272fdd218b4e91355167e63438a1605ef6902d75970c4ff5b8bae67087122d0` |
| `samples/hwpx_sample2.hwpx` | `188bdfe21f89e117d8897f4102aa6f741962b23a3019ad2aaf7bdc222d90fdb2` |
| `samples/re-font-dotum-empty-hancom.hwp` | `1ee8871f37bec2e97d0928709dc411c0eacad656f35aa3d92c5cf89f61c5761b` |

이 절은 수정 전 head의 리뷰 기록이다. 결함의 해결과 수정 후 검증은 위 보완 절을 따른다.

## 2026-09-24 재검토

- 기본 경로: collaborator self. 보조: intake_and_review, local_validation,
  review_only_fast_pass, rework_and_exceptions. 기존 자체 PR의 보정이며 reviewer 추가는 없다.
- 기존 head `2c195048b`에 devel `505661360`을 통합한 `b9e7d6498` 위의 정책 후보 `bc6c1bf00`을 검토했다.
  작업 중 base가 `8619e6d4f5e10f4ab6478798bdf3e5660a8ba100`으로 전진했지만 관련 CI 코드 변경은 없고
  merge-tree 충돌도 없었다. 기록 때문에 source를 반복 동기화하지 않았다.
- `src/main.rs`, `src/cli/**`, `src/bin/**`, font generator, Native source는 확장 WASM의
  root library 입력이 아닌 binary/소비자 경로다. 전용 Swift 패키징·VS Code 검사도 제외했다.
  공유 library·Cargo/lock/build 입력과 미분류 경로는 계속 실행한다.
- baseRef는 CI 수집 → Chrome 분류기, trusted policy의 pullRequest → 같은 분류기로 전달된다.
  `main`에서는 full Chrome + package를 요구하고 preflight의 review-only 재사용을 차단한다.
  `devel`에서는 제외 경로가 기존 frontend `none`을 package로 올리지 않는지 확인했다.
- 수정 전 새 계약 4개 실패 → 수정 후 Node **182개**·Python **137개** 통과, Actions 구문 4개 통과.
  actual workflow 실행으로 main/devel 분기와 입력 배선을 확인했고 제품 함수를 복제하지 않았다.
- 조판 원칙·Visual Sweep: **비해당**. 새 변경은 CI routing이며 조판·runtime·fixture 바이트를 바꾸지 않는다.
  기존 HWP/HWPX 3개 입력의 이전 검증과 새 CI 실행은 source SHA로 구분한다.
- rollback은 정책 보정 commit을 되돌리는 PR이며 required check 이름·권한·캐시 저장 범위는 유지했다.

[보고서](../../report/task_m100_3512_report.md)의 이번 보완 기록과 아래 최초 검증 이력을 구분한다.

## 검토 경로와 metadata

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`, `rework_and_exceptions.md` (1,000줄 초과)
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서, `github_operations.md`, `docs_and_git_workflow.md`, `dev_environment_guide.md`
- 작성자 self-review이며 별도 reviewer는 지정하지 않았다. 코드·테스트·운영 정책을 로컬 검증과 원격 실행으로 나누어 검토했다.

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#7283](https://github.com/edwardkim/rhwp/pull/7283), Open, postmelee |
| base | `devel`, `517df04ba110dc108fbbb203e72229c51819b591` |
| 원격 code candidate | `8d1eb08a4f87cb54bccb6897c84f9d877bfe8fa5` (중앙 배선·Linux 실행 환경 보완) |
| 규모 | 최초 게시 32파일, +1,660 / -94. source/test/workflow와 설명·검증 기록 포함 |
| mergeability | `MERGEABLE`, `BLOCKED`. 최신 required check와 merge 조건은 병합 직전 재조회 필요 |
| 이슈 | Epic #3512, #3513 → #3515. 기존 #3514/#5912와 #7279 회귀 검사 재사용 |

## 변경 범위와 호출 경로 검토

- 실제 production dist에서 options UI 저장·재진입, worker 종료·재기동, 같은 격리 profile 재시작,
  HWP/HWPX 다운로드 완료·바이트와 viewer 0/1개, 과거 다운로드 기록 보존을 검사한다.
  `monitorTabs`는 생성 이력을 보존하여 잠깐 열렸다 닫힌 두 번째 탭도 검출한다.
- CI는 trusted base의 Chrome 영향 분류 → frontend package 승격 → fresh WASM/dist artifact ID →
  Chrome job → `Build & Test` 집계로 연결한다. trusted policy도 같은 영향 함수와 예상 job 결과를
  소비한다. 분류 누락·잘린 목록·불확실성은 실행으로 처리하고 fast-pass는 Chrome skip을 요구한다.
- `run-ci.mjs`는 세 suite 전체를 한 번씩 실행하며 case/repeat 환경변수로 검사 축소를 허용하지 않는다.
  220초 전체 예산, 단계별 timeout, 실패 전파와 소유 profile 정리를 확인했다.
- PR browser cache는 exact lock key 복원 전용이다. 신규 기본 브랜치 수동 workflow의 cache 저장은
  별도 실행 경계이며 최초 승격은 CI 설치 실행과 cache 분기 계약의 contracts-only adapter를 쓴다.
- promotion collector가 하나의 CI run을 두 workflow의 증거로 연결할 때 기존 단일 키 덮어쓰기를
  수정했다. 각 subject의 hash/mode를 보존하고 offline verifier의 실패 조건은 유지했다.

제품 source, 확장 권한·CSP, Rust source/test, 조판 baseline은 바뀌지 않았다. 조판 원칙·Visual Sweep은
**비해당**이다. 이번 주장은 설정 유지·탭 개수와 CI 실행 계약이며 PDF 조판 일치가 아니다.

## 검증 입력 커밋 확인

판정 **충족**. 기존 tracked fixture를 재사용했고, 아래 실제 실행 파일과 candidate `git show` 내용의
바이트·SHA-256이 같음을 확인했다. 앞 두 입력은 실제 HWP/HWPX 다운로드 및 viewer smoke, 마지막은
기존 #7279 초기 다운로드 상태 경합 검사에 사용했다. 새 fixture·기준 PDF는 추가하지 않았다.

| 저장소 경로 | SHA-256 |
| --- | --- |
| `samples/hwp3-pagedef-1915.hwp` | `b272fdd218b4e91355167e63438a1605ef6902d75970c4ff5b8bae67087122d0` |
| `samples/hwpx_sample2.hwpx` | `188bdfe21f89e117d8897f4102aa6f741962b23a3019ad2aaf7bdc222d90fdb2` |
| `samples/re-font-dotum-empty-hancom.hwp` | `1ee8871f37bec2e97d0928709dc411c0eacad656f35aa3d92c5cf89f61c5761b` |

확인 commit은 `8d1eb08a4f87cb54bccb6897c84f9d877bfe8fa5`이다. 저장소에 있던 입력을 다운로드 fixture로
재사용한 것이며 한컴 기준 출력과의 시각 일치를 주장하지 않는다.

## 완료한 로컬 검증

[정식 보고서](../../report/task_m100_3512_report.md)와 [회차별 검증 기록](../../report/3512-extension-e2e/validation.json)에
명령·source SHA·시간·mutation·최초 실패 기록을 연결했다.

- 최신 기준의 fresh WASM 및 production Chrome dist 빌드가 통과했다.
- 전체 smoke/download/lifecycle 10회 연속 통과, retry 0. 평균 46.874초, 범위 46.126~48.315초였다.
- 기존 확장 Node 174개, CI·harness Node 174개, Python workflow/promotion 계약 91개가 통과했다.
- freshness 제거는 Node 상태 계약에서 3건 실패, 중복 방어 제거는 실제 Chrome에서 viewer 2개를 검출했다.
  과거 다운로드 E2E 자체가 freshness 제거를 검출한다고 해석하지 않았다.
- 실패를 유도한 smoke에서 exit 1, 최소 JSON/PNG 생성과 원본 복원을 확인했다.
- 변경 workflow의 Actions 구문을 통과했다. ShellCheck SC2016 info 1건은 기존 base에서도 재현됐으며
  새 오류와 구분했다. 새 Rust 변경이 없어 전체 Rust lint를 반복하지 않았다.
- 최종 browser source는 `8d1eb08a4f87cb54bccb6897c84f9d877bfe8fa5`이다. console 출처 구분과 fixture HTML 보완 이후 전체 로컬 10회와 Linux 3회를 다시 검증했다. production dist와 tracked fixture 바이트는 동일하다.

동작 기반 회귀 검증은 **충족**이다. 실제 options·download 진입점과 worker 재기동 경계, 불확실한
분류 fallback, 0/1/초과 탭 결과, 정상 대조군 및 방어 제거 음성 대조를 위 증거에서 확인했다.

## 원격 검증과 남은 조건

[첫 CI](https://github.com/edwardkim/rhwp/actions/runs/35490767626)의 중앙 배선 검사에서 새 Chrome 계약 검사 등록 누락을 검출했다. lint job 연결과 검사 목록을 보완했고 관련 94개 및 전체 workflow 계약 248개가 통과했다. 전체 checkout의 98초 준비 시간을 확인해 Chrome 소비 job과 cache 준비 job에 필요한 파일만 checkout하도록 좁혔다. 첫 run은 수정 push 뒤 stale 취소됐으며 성공 회차에 포함하지 않는다.

두 번째 CI에서는 runner 단위 테스트의 stdout/stderr 도착 순서 가정을 검출했다. 같은 pipe에 payload와
정리 marker를 기록하도록 `3e948d8de`에서 보완했고 guard 8개가 통과했다. 해당 실행의 배포 artifact를
macOS Chrome에서 추가 실행해 전체 suite 49.407초 통과 및 원래 dist 복원을 확인했다.

세 번째 CI의 Chrome launch는 Ubuntu sandbox 제한으로 실패했다. `17886c336`에서 기존 root 소유
sandbox helper를 연결했고 관련 계약 78개가 통과했다. 실제 실패 artifact의 JSON/LOG 3개·2,322바이트와
API digest를 확인했다. 상세 원인과 Chromium 근거는 정식 보고서에 연결했다.

기존 sandbox helper의 사전 조건이 충족되지 않아, 최종 후보는 확인된 CfT 실행 파일 하나에만
AppArmor user namespace 권한을 적용하고 종료 시 profile을 제거한다. 실행 경로 조회는 비동기 API를
기다리며 로컬 명령 실행으로 확인했다. GitHub token 권한·제품 코드·browser assertion은 유지했다.

`fcb630d5e`의 Linux 실행은 sandbox 설정·해제, smoke/download와 lifecycle 9개가 통과했다.
과거 다운로드는 UI·탭 검사를 마친 뒤 Chrome 내부 화면의 console error로 실패했다. `cb2b4eea0`에서
source URL이 명시적인 `chrome://` 오류만 로그에 보존하고 확장·fixture·불명확한 출처 오류는 실패로
유지했다. 실제 확장 오류 주입 시 exit 1, 원복 시 exit 0을 확인했다. 화면 표시와 0/1 탭 assertion은 동일하다.

[최종 후보 CI](https://github.com/edwardkim/rhwp/actions/runs/35496249526)에서 Chrome job을 3회 연속 통과했다.
첫 full CI 성공 뒤 동일 artifact를 소비하는 성공 job을 두 번 반복했으며 실패 재시도는 없다. 각각의
job ID·원격 시간·실제 checkout SHA는 [정식 보고서](../../report/task_m100_3512_report.md#최종-linux-연속-검증)에
기록했다. cold job 실측과 미검증인 shared cache seed/hit을 구분한다.
초기 전체 로컬 반복의 9회차 다운로드 취소는 실패로 보존했고 원인은 미확정이다.

운영 등급 O3. rollback은 CI/harness 변경을 되돌리는 devel 대상 PR이다. required context 이름과
branch push 실행 정책은 유지했다. Firefox는 별도 설치·worker 구현이 필요해 Chrome Phase 1 통합과 기본 브랜치 운영 적용 뒤로
보류하며 부모 이슈에 근거를 남겼다. `확장 E2E 증적 보고서 연동`은 Chrome Phase 1 통합 완료 이후의 후속 이슈다.


이전 후보의 [첫 Chrome 성공 job](https://github.com/edwardkim/rhwp/actions/runs/35495528465/job/106038714425)은
실제 source가 `chrome://fileicon/`인 두 오류를 기록하면서 전체 suite를 51.192초에 통과했다.
해당 URL은 격리된 테스트 다운로드 파일의 Chrome 내부 아이콘 요청이다. 실제 제품 오류를 무시한
결과가 아니며, 원래 오류 출처가 없던 실패 로그의 원인을 직접 확정한 증거와도 구분한다.

CodeQL의 [경고 #206](https://github.com/edwardkim/rhwp/security/code-scanning/206)은 fixture 페이지가
등록 여부를 검사한 query 값을 HTML에 직접 넣는 경로를 지적했다. `8d1eb08a4`에서 서버가 소유한
등록 키를 찾아 URI encoding 후 출력하고, 미등록 요청은 404로 반환한다. 제품 source는 동일하다.
실제 과거 다운로드 대조 검사를 통과했으며, 이 최종 후보에서 로컬 10회와 Linux 3회 연속 검증을 다시 통과했다. CodeQL 재검사도 통과했다.
이전 `cb2b4eea0`의 전체 CI/Chrome 성공은 참고 기록이며 최종 후보의 통과 횟수에 포함하지 않는다.

## 2026-09-20 판정 이력

기능·회귀·Linux 반복 실행 증거는 **충족**이다. 제품 source·조판 변경은 **비해당**이며,
shared cache seed/hit은 기본 브랜치 workflow 등록 이후의 운영 확인으로 **미검증**이다.

**머지 보류** — code candidate의 구현·필수 원격 검증은 완료했다. 이 기록을 포함한 trailing head의
merge tree·링크·오늘할일 보존과 최신 required checks를 확인하고 작업지시자 승인을 받아야 한다.
최종 trailing head의 SHA·merge tree 검증·CI 결과는 PR 본문에 연결한다. 실제 merge·이슈 close 및
Chrome Phase 1 이후 증적 보고서 후속 이슈는 승인된 통합 절차에서 수행한다.
