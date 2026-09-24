# Issue #3512 Chrome E2E 구현·검증 보고서

## 2026-09-24 CI 정책 보완

- 통합한 devel: `505661360e9a2d596f55300d0cb0c5222f0e14b4`, merge `b9e7d6498`.
  9월 20일 오늘할일 add/add 충돌은 #7283·#7285 양쪽 기록을 보존해 해소했다.
- 정책 source: `bc6c1bf00`. `devel`은 확인된 CLI/Native source와 전용 도구를 제외하고 공용
  WASM 입력은 유지한다. `main` 대상 CI는 전체 suite를 실행하고 review-only 재사용을 하지 않는다.
- 기존 정책에 새 검사 적용: 9개 중 4개 실패. 보정 후 CI·harness Node **182/182**, 관련 workflow
  Python **137/137**, 변경 workflow 4개의 `actionlint -shellcheck=`, `git diff --check` 통과.
- Node 검사는 실제 CI inline script를 실행해 `main` fast-pass 거부, `devel` 문서 fast-pass 유지,
  baseRef·파일 목록 전달을 확인한다. CLI/Native 단독·공용 코드 혼합·양방향 rename·목록 누락·
  브랜치 누락과 policy의 success/skip/failure 집계도 확인했다.
- 이번 수정은 CI 분류·배선·문서만이다. 제품 Rust/Studio/확장 runtime와 browser 시나리오는
  수정하지 않았다. 이 PR의 diff에 Rust source/test 변경이 없어 로컬 전체 Cargo/Visual Sweep은
  실행하지 않았다. 아래 과거 10회/3회 결과는 당시 source·browser의 증거이며 새 head 결과로 세지 않는다.
- 새 head의 원격 CI·Chrome 결과와 최종 merge-tree 검증은 [PR #7283](https://github.com/edwardkim/rhwp/pull/7283)
  본문에서 정확한 SHA·run URL과 연결한다. main 정책의 로컬 계약 통과와 실제 릴리즈 승격 실행은 구분한다.

최신 운영 정책·정확한 제외 경로와 비용 설명은
[확장 매뉴얼 3.9](../manual/chrome_edge_extension_build_deploy.md#39-ci-선택-실행브라우저-cache실패-진단-3515)에 있다.

## 2026-09-20 최초 구현·반복 검증 이력

- Epic: [#3512](https://github.com/edwardkim/rhwp/issues/3512), 구현: #3513 → #3515
- 기존 기반: #3514 / 통합 PR #5912, 다운로드 경합 대조군: #7279
- 최종 기준 devel: `517df04ba110dc108fbbb203e72229c51819b591`
- 최종 browser/CI source SHA: `8d1eb08a4f87cb54bccb6897c84f9d877bfe8fa5`
- 이전 browser source `d6b4e704c`의 10회 결과도 보존했다. console 출처 구분과 CodeQL fixture 보완 이후 최종 source에서 전체 10회를 다시 검증했다.
- 브랜치: `codex/issue-3513-extension-lifecycle`
- 환경: macOS arm64, Node 24.15.0, Puppeteer 25.10.0, Chrome for Testing 152.0.7977.75, extension 0.8.6 / MV3
- PR: [#7283](https://github.com/edwardkim/rhwp/pull/7283), 최종 code candidate `8d1eb08a4f87cb54bccb6897c84f9d877bfe8fa5`
- 상태: 구현·로컬 10회·Linux 3회 연속 검증 완료. 문서 trailing head의 필수 CI 및 병합 승인 확인 후 통합한다. merge·이슈 close 미실행.

## 구현과 관측 계약

실제 options UI에서 OFF를 저장한 뒤 재진입·worker 종료·같은 격리 profile 재실행을 각각 검사한다.
HWP/HWPX에 대해 OFF는 viewer 0개, ON 및 worker wakeup은 viewer 정확히 1개를 요구한다.
확장 없이 실제 다운로드를 완료한 뒤 같은 profile로 확장을 시작해 과거 기록의 보존과 viewer 0개를
확인한다. 완료 이벤트와 fixture 바이트도 대조하며, 잠깐 열렸다 닫힌 탭을 포함한 생성 이력으로
중복을 검출한다. 기대값은 구현 내부 storage가 아니라 #3513의 사용자 관찰 계약에서 정했다.

Chrome CI는 이 세부 검증과 기존 smoke/download 검사를 한 번씩 실행한다. 같은 run의 Frontend
package gates가 fresh WASM으로 만든 dist를 artifact ID로 전달한다. 영향 분류는 Chrome/shared
worker, 실제 Studio viewer 입력과 WASM을 포함한다. 전용 Firefox/Safari/VSCode/npm editor,
문서, 빌드에서 제외되는 Studio public/test 자료만 바뀌면 skip한다. 불명확한 경로·rename 누락·
잘린 목록·tag/manual·분류 실패는 실행한다.

CI Impact Policy와 Build & Test가 같은 frontend package 승격 및 Chrome success/skip을 검사한다.
정책 v7 발행자와 세 소비자의 연결, 기존 trusted sparse checkout 배선도 회귀 검사한다.
Chrome cache는 PR에서 복원만 한다. 기본 브랜치의 별도 수동 workflow에서만 저장할 수 있고,
그 workflow의 최초 승격에는 기존 CI를 contracts-only adapter로 쓴다. 설치·계약 검증과 실제
shared cache 저장 성공은 별개다.

제품 소스·권한·CSP·조판 baseline은 변경하지 않았다. Rust source/test 변경은 없으므로 Rust lint
전체·조판 Visual Sweep은 비해당이다. 실제 runtime 검증용 WASM은 최신 devel에서 다시 빌드했다.

## 검증

| 검사 | 실행과 결과 |
| --- | --- |
| fresh WASM + production Chrome dist | `scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt` 후 `npm --prefix rhwp-chrome run build`, 통과. WASM release 컴파일 2분 42초 |
| 기존 shared/Chrome adapter/options | `node --test rhwp-chrome/sw/*.test.mjs rhwp-shared/sw/*.test.js rhwp-chrome/options.test.mjs`: 174 통과 |
| CI classifier/policy/controller/evidence/report + Chrome/tab/runner | 174 통과, 명령은 아래 참조 |
| workflow·promotion 계약 | 91 통과, required 집계의 run/skip/failure/cancelled·trusted checkout·artifact·cache 경계 포함 |
| YAML/Actions 구문 | 변경 workflow 4개의 `actionlint -shellcheck=` 통과. 아래 기존 ShellCheck 주의사항 참조 |
| freshness mutation | 정상 Node 상태 계약 14 통과 → freshness 방어 제거 시 3 실패, 검출 확인 |
| 중복 mutation | 정상 임시 dist 대조군 통과 → 중복 방어 제거 시 실제 Chrome의 `Tab budget exceeded: expected 1, created 2` 검출 |
| 실패 증적 | print 제목을 임시 변경한 실제 smoke가 exit 1, `surface=print` JSON과 PNG 생성. aggregate는 실패 뒤 중단. 원본 복원 확인 |
| 브라우저 전체 10회 | 최종 source 10/10 통과, retry 0. 회당 46.126~48.315초, 평균 46.874초. lifecycle 100개 검사 |
| GitHub Actions | 최종 code candidate에서 Chrome 3회 연속 통과. 아래 실행 ID·시간·캐시 상태 참조 |

```bash
node --test scripts/tests/ci-impact-classifier.test.cjs \
  scripts/tests/ci-impact-policy.test.cjs scripts/tests/ci-impact-controller-contract.test.cjs \
  scripts/tests/ci-workflow-evidence.test.cjs scripts/tests/ci-impact-report.test.cjs \
  scripts/tests/chrome-extension-impact.test.cjs rhwp-chrome/e2e/page-budget.test.mjs \
  rhwp-chrome/e2e/tab-monitor.test.mjs rhwp-chrome/e2e/run-ci.test.mjs \
  rhwp-chrome/e2e/failure-diagnostics.test.mjs
python3 -m unittest scripts/tests/test_ci_impact_workflow.py \
  scripts/tests/test_chrome_extension_workflow.py scripts/tests/test_ci_impact_policy_workflow.py \
  scripts/tests/test_workflow_promotion_preflight.py scripts/tests/test_workflow_promotion_evidence.py
node rhwp-chrome/e2e/lifecycle-mutations.mjs
node rhwp-chrome/e2e/run-ci.mjs
```

브라우저 전체 명령은 smoke, 다운로드 11건(초기 상태 저장 경합·own Blob 포함), lifecycle 10개를
순차로 실행한다. 자동 retry는 없고 실패한 회차에서 반복을 중단한다. GitHub job 90초 warm 목표와
5분 hard timeout은 선행 WASM/build 시간과 구분한다. 로컬 숫자를 GitHub warm-cache 시간으로
보고하지 않는다.

`actionlint`의 ShellCheck 포함 실행은 기존 CI summary의 Markdown backtick format 문자열에서
SC2016 info 1건을 보고했다. 최초 기준 `722fb38a`의 동일 명령에서도 같은 경고를 재현했다. 이번
수정에서 새 경고를 추가하지 않았으며 Actions/YAML 자체의 오류와 구분한다.

같은 CI run을 자체 검증과 cache adapter가 공유하면 기존 promotion collector가 `evidencePath` 키로
한 항목을 덮어썼다. 회귀 검사는 수정 전 `1 != 2`로 실패했고 수정 후 통과했다. 이제 각 대상 파일의
hash와 실행 mode를 따로 보존한다. 항목 순서를 바꾸어도 결과가 같으며, hash 누락·Chrome job skip은
실제 offline verifier가 거부한다. 후보의 workflow inventory에 정책 누락은 0건이다.

회차별 시간·버전·source SHA·산출물 hash는 [로컬 검증 기록](3512-extension-e2e/validation.json)에 보존했다.

## 실패 기록과 검증 한계

초기 기준에서는 lifecycle 100회가 통과했다. 전체 runner 반복에서는 8회 통과 후 9번째
`wake-download-hwp`가 Chrome `canceled` 상태가 되어 중단됐다. viewer는 1개였으나 다운로드 완료
조건은 실패했으므로 이 실행을 통과로 세지 않는다. 직후 호스트 디스크 여유는 약 122MB였다.
직접적인 Chrome 오류 코드는 당시 증적에 없으며 디스크 부족을 확정 원인으로 단정하지 않는다.

작업 전용 checkout에서 사용하지 않는 PDF 사본을 sparse checkout으로 제외했다. 사용자 원본과
기존 `samples/exam_eng.pdf` 변경은 보존했다. 취소 상태를 timeout까지 기다리지 않고 실패시키며,
실패한 테스트 소유 loopback URL의 download ID/state/error와 디스크 여유만 기록하도록 진단을
보강했다. 최종 Linux 3회에서는 다운로드 취소가 재현되지 않았다. 초기 실패의 직접 원인은 여전히 미확정이다.

과거 다운로드 E2E는 기록 존재와 탭 수를 검증한다. 완료된 기록이 `onCreated`를 다시 발생시키는
것은 아니므로 freshness mutation 검출은 Node 상태 계약의 증거이며 Chrome history E2E의
검출력으로 승격하지 않는다. 워커 종료는 테스트가 실제 target 종료를 확인하고 다운로드로 다시
깨우는 경로이며, 자연 idle suspend의 발생 빈도나 Web Store 업데이트를 보증하지 않는다.

## 최초 CI에서 확인한 보완

[첫 실행](https://github.com/edwardkim/rhwp/actions/runs/35490767626)의 lint job은 새 Chrome workflow 계약
검사가 중앙 lint 목록에 없고 Node 발견 목록에도 등록되지 않은 점을 검출했다. `970e22d79`에서
두 연결을 보완했다. 관련 계약 94개와 전체 workflow 계약 248개가 통과했고 Actions 구문도 통과했다.
같은 실행에서 전체 checkout에 98초가 걸려, Chrome 소비 job은 harness와 tracked 입력 3개만,
cache 준비 job은 두 package 파일만 checkout하도록 줄였다. 검사 범위나 5분 timeout은 완화하지 않았다.
첫 run은 수정 push 뒤 stale 취소됐으며 3회 성공 검증에 포함하지 않는다.

두 번째 [CI](https://github.com/edwardkim/rhwp/actions/runs/35491118060)는 frontend·Rust 검증을 통과했지만
Chrome job의 runner 단위 테스트가 stdout/stderr 도착 순서를 가정해 실패했다. 브라우저 시나리오는
시작 전이었다. `3e948d8de`에서 큰 payload와 정리 확인 marker를 같은 stdout pipe로 출력하도록
테스트를 고쳤고, 8개 guard가 통과했다. 테스트 완화나 자동 retry는 추가하지 않았다.
Chrome sparse checkout은 이 실행에서 1초였고 cache miss 뒤 Chrome 설치는 약 5초였다.

이 CI가 생성한 artifact `10599237830`의 API digest를 확인한 뒤 실제 dist를 macOS Chrome에서
추가 실행했다. smoke/download/lifecycle 전체가 49.407초에 통과했고 기존 로컬 dist를 복원했다.
이는 Linux에서 만든 배포본의 호환성 증거이며 Linux browser 3회 결과와 구분한다. 최초 로컬 호출은
sandbox의 loopback listen EPERM으로 실행 전 실패했고, 허용된 환경에서 다시 실행한 결과다.

세 번째 [CI](https://github.com/edwardkim/rhwp/actions/runs/35493391043)는 Ubuntu 24.04의 AppArmor 제한으로
Chrome 시작 시 `No usable sandbox`를 보고했다. `17886c336`에서 러너 이미지에 이미 설치된
`/opt/google/chrome/chrome-sandbox`를 `CHROME_DEVEL_SANDBOX`로 지정하고 root 소유·4755 권한을
실행 전에 확인했다. [Chromium 공식 문서의 helper 재사용 절차](https://chromium.googlesource.com/chromium/src/+/main/docs/security/apparmor-userns-restrictions.md)를 적용했다.
관련 계약 78개와 Actions 구문이 통과했다. 제품·harness 동작은 변경하지 않았다.
실제 실패 artifact `10600081753`은 2,322바이트이며 `extension-smoke.log`, `result.json`,
`smoke.json`만 들어 있음을 API digest와 ZIP 목록으로 확인했다. 전체 profile·원문 문서는 없다.

네 번째 [CI](https://github.com/edwardkim/rhwp/actions/runs/35493967034)는 기존 helper의 사전 조건 검사에서
실패했다. 어떤 사전 조건이 충족되지 않았는지는 해당 shell 로그만으로 구별되지 않는다.
최종 후보는 기존 helper 가정을 제거하고 설치된 CfT 실행 파일의 경로·문자 범위를 확인한 뒤,
그 경로 하나에만 user namespace를 허용하는 AppArmor profile을 적용한다. 종료 시 profile을 제거한다.
로컬에서 실제 경로 조회 명령을 실행해 비동기 `executablePath()`의 await도 확인했다.
GitHub token 권한과 제품 소스·브라우저 시나리오는 동일하다.

[Linux 실행](https://github.com/edwardkim/rhwp/actions/runs/35494638744)에서는 AppArmor 설정·해제와
smoke/download, lifecycle 9개가 통과했다. 마지막 과거 다운로드 검사는 기록 표시·설정 ON·viewer 0개를
확인했지만, Chrome 내부 다운로드 화면에서 발생한 resource console error 2건으로 최종 실패했다.
당시 listener에는 source URL이 없어 정확한 resource는 해당 실패 로그만으로 확정할 수 없다.

`cb2b4eea0`은 console의 실제 source URL과 page URL을 보존하고, 명시적인 `chrome://` 출처의 내부 UI
오류만 별도 진단으로 남긴다. 확장·fixture 오류와 출처가 불명확한 오류는 계속 실패한다. 실제 options
코드에 오류를 잠시 주입한 음성 대조는 exit 1, 원복한 대조군은 exit 0이었다. 기록의 화면 표시·다운로드
완료·viewer 수 assertion과 page error 검사는 유지했다. 이 harness 변경 후 전체 로컬 10회를 다시 통과했다.
이전 실행 결과는 별도로 보존했으며 새 source의 증거로 재사용하지 않았다. Node 계약 174개와 workflow 계약 248개가 통과했다.


이전 후보의 [첫 Chrome 성공 job](https://github.com/edwardkim/rhwp/actions/runs/35495528465/job/106038714425)은
실제 source가 `chrome://fileicon/`인 두 오류를 기록하면서 전체 suite를 51.192초에 통과했다.
해당 URL은 격리된 테스트 다운로드 파일의 Chrome 내부 아이콘 요청이다. 실제 제품 오류를 무시한
결과가 아니며, 원래 오류 출처가 없던 실패 로그의 원인을 직접 확정한 증거와도 구분한다.

CodeQL의 [경고 #206](https://github.com/edwardkim/rhwp/security/code-scanning/206)은 fixture 페이지가
등록 여부를 검사한 query 값을 HTML에 직접 넣는 경로를 지적했다. `8d1eb08a4`에서 서버가 소유한
등록 키를 찾아 URI encoding 후 출력하고, 미등록 요청은 404로 반환한다. 제품 source는 동일하다.
실제 과거 다운로드 대조 검사를 통과했으며, 이 최종 후보에서 로컬 10회와 Linux 3회 연속 검증을 다시 통과했다. CodeQL 재검사도 통과했다.
이전 `cb2b4eea0`의 전체 CI/Chrome 성공은 참고 기록이며 최종 후보의 통과 횟수에 포함하지 않는다.

## 최종 Linux 연속 검증

최종 code candidate `8d1eb08a4f87cb54bccb6897c84f9d877bfe8fa5`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/35496249526)와
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35496249622)이 성공했다. 첫 full CI 이후 이미
성공한 Chrome job을 의도적으로 두 번 반복했다. 이는 workflow 3개가 아니라 **동일 run의 attempt 1~3**이며,
각 Chrome job ID는 다르다. 실패 재시도는 없고 각 회차의 smoke/download/lifecycle이 모두 통과했다.

| 회차 | Chrome job | browser 검사 | job 전체 | browser cache |
| --- | --- | --- | --- | --- |
| 1 | [job 106040698962](https://github.com/edwardkim/rhwp/actions/runs/35496249526/job/106040698962) | 49.008초 | 68초 | miss |
| 2 | [job 106041526394](https://github.com/edwardkim/rhwp/actions/runs/35496249526/job/106041526394) | 54.848초 | 77초 | miss |
| 3 | [job 106041813670](https://github.com/edwardkim/rhwp/actions/runs/35496249526/job/106041813670) | 50.999초 | 67초 | miss |

세 회차는 동일한 immutable dist artifact `10601400592`를 소비했다. Linux 실행은 PR merge
checkout이며, API의 PR head와 실제 checkout SHA를 [회차별 기록](3512-extension-e2e/validation.json)에
따로 남겼다. Ubuntu 24.04, Node 22, Puppeteer 25.10.0, Chrome 152.0.7977.75, extension 0.8.6/MV3였다.
AppArmor profile 설정·정리도 각 회차 성공했다. Chrome 내부 file icon 오류는 source URL과 함께 남기며
확장·fixture·불명확한 출처 오류는 계속 실패한다.

세 회차 모두 browser cache miss였다. 측정값은 **cold job 시간**이며 warm-cache hit/90초 실측으로
표현하지 않는다. 기본 브랜치에 신규 workflow가 등록된 뒤 shared cache seed/hit을 별도 확인해야 한다.
현재 측정의 job 시간은 앞선 fresh WASM 및 package build 시간을 제외한다. hard timeout은 5분이다.

## 남은 통합과 후속

#3512 본문을 현행화하고 #3514/#3513/#3515를 native sub-issue로 연결했다. #3514만 closed이며
#3513/#3515는 구현·검증·통합의 실제 진행 상태에 맞춰 open으로 유지한다. 원격 게시와 Actions
검증은 사용자 승인에 따라 진행했다. GitHub Actions 3회와 Firefox 보류 근거는 충족했다. 최신 PR head의
필수 CI 및 작업지시자 승인 후 통합하고 이슈 종료 여부를 확인한다.

Firefox Phase 2는 MV3 worker와 설치 방식이 Chrome과 달라 별도 구현이 필요하다. Chrome의 Linux
재현성과 실행 비용은 확인했으며, Phase 1 통합과 기본 브랜치 운영 적용을 먼저 마친 뒤 별도 범위를 정한다. Chrome Phase 1 완료 후에만 `확장 E2E 증적
보고서 연동` 후속 이슈를 만든다. 별도 증적 저장소는 runner 중립 PNG/JSON 인터페이스와 같은
source SHA·시나리오·실행 ID의 연결을 다룬다. 공개 갤러리나 게시 권한은 이번 Epic에 포함하지 않는다.

운영 등급은 O3이며 기존 required context와 branch push CI 정책은 유지한다. rollback은 이 작업의
CI·harness 커밋을 되돌리는 devel 대상 PR이다. 개발·실행 절차는
[브라우저 빌드 매뉴얼](../manual/chrome_edge_extension_build_deploy.md#38-설정-수명주기와-다운로드-탭-불변식-3513),
운영·승격 경계는 [GitHub 운영 매뉴얼](../manual/github_operations.md)을 따른다.
