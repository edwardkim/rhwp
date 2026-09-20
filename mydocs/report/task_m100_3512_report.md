# Issue #3512 Chrome E2E 구현·검증 보고서

- Epic: [#3512](https://github.com/edwardkim/rhwp/issues/3512), 구현: #3513 → #3515
- 기존 기반: #3514 / 통합 PR #5912, 다운로드 경합 대조군: #7279
- 최종 기준 devel: `517df04ba110dc108fbbb203e72229c51819b591`
- 실행 source SHA: `d6b4e704c6281f892b15ba78bb35b03224136963`
- 최종 CI collector source: `1bba134e8` — 브라우저 실행 이후 promotion collector와 그 테스트만 추가 수정, browser/WASM 입력은 동일
- 브랜치: `codex/issue-3513-extension-lifecycle`
- 환경: macOS arm64, Node 24.15.0, Puppeteer 25.10.0, Chrome for Testing 152.0.7977.75, extension 0.8.6 / MV3
- 상태: 구현·최종 로컬 검증 완료. 원격 push·PR 생성·Actions 검증·merge·이슈 close 미실행.

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
| CI classifier/policy/controller/evidence/report + Chrome/tab/runner | 172 통과, 명령은 아래 참조 |
| workflow·promotion 계약 | 91 통과, required 집계의 run/skip/failure/cancelled·trusted checkout·artifact·cache 경계 포함 |
| YAML/Actions 구문 | 변경 workflow 4개의 `actionlint -shellcheck=` 통과. 아래 기존 ShellCheck 주의사항 참조 |
| freshness mutation | 정상 Node 상태 계약 14 통과 → freshness 방어 제거 시 3 실패, 검출 확인 |
| 중복 mutation | 정상 임시 dist 대조군 통과 → 중복 방어 제거 시 실제 Chrome의 `Tab budget exceeded: expected 1, created 2` 검출 |
| 실패 증적 | print 제목을 임시 변경한 실제 smoke가 exit 1, `surface=print` JSON과 PNG 생성. aggregate는 실패 뒤 중단. 원본 복원 확인 |
| 브라우저 전체 10회 | 10/10 통과, retry 0. 회당 46.905~48.232초, 평균 47.399초. lifecycle 100개 검사 |
| GitHub Actions | 미실행. 같은 검증 범위의 retry 없는 3회와 Linux runner/cache hit/시간은 원격 확인 필요 |

```bash
node --test scripts/tests/ci-impact-classifier.test.cjs \
  scripts/tests/ci-impact-policy.test.cjs scripts/tests/ci-impact-controller-contract.test.cjs \
  scripts/tests/ci-workflow-evidence.test.cjs scripts/tests/ci-impact-report.test.cjs \
  scripts/tests/chrome-extension-impact.test.cjs rhwp-chrome/e2e/page-budget.test.mjs \
  rhwp-chrome/e2e/tab-monitor.test.mjs rhwp-chrome/e2e/run-ci.test.mjs
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
보강했다. 원격 Linux 재현 여부는 아직 미검증이다.

과거 다운로드 E2E는 기록 존재와 탭 수를 검증한다. 완료된 기록이 `onCreated`를 다시 발생시키는
것은 아니므로 freshness mutation 검출은 Node 상태 계약의 증거이며 Chrome history E2E의
검출력으로 승격하지 않는다. 워커 종료는 테스트가 실제 target 종료를 확인하고 다운로드로 다시
깨우는 경로이며, 자연 idle suspend의 발생 빈도나 Web Store 업데이트를 보증하지 않는다.

## 남은 통합과 후속

#3512 본문을 현행화하고 #3514/#3513/#3515를 native sub-issue로 연결했다. #3514만 closed이며
#3513/#3515는 구현·검증·통합의 실제 진행 상태에 맞춰 open으로 유지한다. 원격 게시와 Actions
검증은 별도 승인 대상이다. GitHub Actions 3회, PR 통합, Firefox 처리 근거가 확인되어야 Epic을
완료한다.

Firefox Phase 2는 MV3 worker와 설치 방식이 Chrome과 달라 별도 구현이 필요하므로, 먼저 Chrome의
Linux CI 재현성과 비용을 확인할 때까지 보류한다. Chrome Phase 1 완료 후에만 `확장 E2E 증적
보고서 연동` 후속 이슈를 만든다. 별도 증적 저장소는 runner 중립 PNG/JSON 인터페이스와 같은
source SHA·시나리오·실행 ID의 연결을 다룬다. 공개 갤러리나 게시 권한은 이번 Epic에 포함하지 않는다.

운영 등급은 O3이며 기존 required context와 branch push CI 정책은 유지한다. rollback은 이 작업의
CI·harness 커밋을 되돌리는 devel 대상 PR이다. 개발·실행 절차는
[브라우저 빌드 매뉴얼](../manual/chrome_edge_extension_build_deploy.md#38-설정-수명주기와-다운로드-탭-불변식-3513),
운영·승격 경계는 [GitHub 운영 매뉴얼](../manual/github_operations.md)을 따른다.
