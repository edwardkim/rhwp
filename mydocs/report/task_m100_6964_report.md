# Task #6964 — Firefox 중복 탭 및 자체 저장 재오픈 수정 결과

- Issue: https://github.com/edwardkim/rhwp/issues/6964
- 기준: upstream/devel `a3cd825c23d550e0c5f46b4e2eb3735a537f23f2`
- 구현: `c4c757a18` + 첫 이벤트 보정 `5de285adb`, branch `codex/issue-6964-download-duplicates`
- 관련: [계획](../plans/task_m100_6964.md), [구현 기록](../working/task_m100_6964_stage2.md)
- #6961은 별도 브랜치/커밋이며 이 구현에 포함하지 않는다.
- 상태: 로컬 구현·검증 완료, 원격 push/PR/배포 전.

## 결과

Firefox adapter는 같은 다운로드 ID의 생성·변경·완료 이벤트를 순서대로 처리한다.
이벤트 수신 시간을 보존하고 실패 후에도 큐를 진행한다. 다른 ID는 독립적으로 실행된다.
Chrome/Firefox는 자신의 extension origin에 속한 Blob 저장을 자동 열기에서 제외한다.
공통 상태 머신과 과거 항목·미추적 변경 이벤트 보호는 그대로 유지한다.

#6961 검증에서 원본 기준 패키지도 다운로드 1건에 서로 다른 탭 ID 2개를 만들었다.
수정 패키지는 다운로드→편집→저장→다른 이름으로 저장 전체에서 실제 뷰어 탭 1개를 유지했다.
세 다운로드(입력 1, 저장 출력 2)는 모두 complete이고 출력 HWP 재파싱에서 입력한 글자가 보존됐다.

## 검증 증거

| 검증 | 결과 |
| --- | --- |
| 수정 전 새 adapter 테스트 | 49개 중 6개 실패 / 43개 통과 |
| 최종 shared/sw + Chrome/Firefox adapter | 148/148 통과 |
| 두 확장 production build | 통과, 기존 pkg WASM 재사용 |
| frontend-extension-dist 계약 | 3/3 통과 |
| 실제 Chrome E2E | 기존 4종 모두 통과; 자체 Blob 저장 바이트 동일, 추가 탭 0 |
| 실제 Firefox 155.0.1, #6964 단독 | 입력/저장/다른 이름 저장, 탭 1개, 편집 보존 |
| 실제 Firefox, #6961 결합 | 정상 basename, 저장 출력 2개, 탭 1개, 편집 보존 |
| 두 브랜치 merge-tree | conflict 없음 |

실행 명령:

```sh
node --test --test-reporter=spec rhwp-shared/sw/*.test.js rhwp-chrome/sw/*.test.mjs rhwp-firefox/sw/*.test.mjs
node rhwp-firefox/build.mjs
node rhwp-chrome/build.mjs
node --test scripts/frontend-extension-dist.test.mjs
PUPPETEER_EXECUTABLE_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node rhwp-chrome/e2e/download-interceptor.test.mjs
FIREFOX_EXECUTABLE_PATH=/Applications/Firefox.app/Contents/MacOS/firefox node rhwp-firefox/e2e/download-save.test.mjs
```

결합 검증은 #6964 production dist의 unbundled `sw/viewer-launcher.js`와
`sw/document-filename.js`만 #6961 production dist에서 복사한 뒤 아래 명령으로 실행했다.
소스 브랜치는 독립적으로 유지했으며, 최종 dist에는 임시 진단 계측이 없다.

```sh
FIREFOX_EXECUTABLE_PATH=/Applications/Firefox.app/Contents/MacOS/firefox RHWP_EXPECT_BASENAME=1 node rhwp-firefox/e2e/download-save.test.mjs
```

최종 결과:

```json
{"viewerTabs":1,"completedDownloads":3,"savedNames":["신청서(SW) (개인)_1(1).hwp","신청서(SW) (개인)_1(2).hwp"],"editPreserved":true}
```

괄호 번호는 같은 디렉터리의 기존 파일과 충돌할 때 Firefox가 붙이는 정상 중복 방지 번호다.
로컬 로그: `/private/tmp/issue6964-tests.log`, `issue6964-dist.log`,
`issue6964-chrome-e2e.log`, `issue6964-firefox-e2e.log`, `issue6964-firefox-combined-e2e.log`.

## 검증 한계와 배포 전 확인

초기 큐 구현은 첫 이벤트까지 Promise.resolve().then으로 미뤘다. 일부 headless 실행에서
다운로드는 완료됐지만 자동 열기가 누락됐다. 준비 확인만으로 해결되지 않아 첫 browser API
요청을 기존처럼 이벤트 전달 중 즉시 시작하도록 보정했다. 같은 ID로 이미 실행 중인 작업이
있을 때만 후속 이벤트를 기다린다. 첫 API 호출 시점 계약 테스트는 초기 큐에서 실패하고
최종 보정에서 통과한다. 보정 후 UUID를 고정하지 않은 새 프로필의 결합 E2E 3회가 연속 통과했다.
Firefox 내부 수명 관리의 정확한 원인을 단정하지 않으며, 종전 이벤트 전달 시점을 보존한
구현과 실제 브라우저 결과를 검증 근거로 삼는다. 초기 누락 로그와 최종 연속 실행 로그를 보존했다.

- 첫 호출 계약 red: `/private/tmp/issue6964-first-event-red.log`
- 최종 보정 결합 연속 실행: `/private/tmp/issue6964-firefox-immediate-1.log`부터 `-3.log`
- 최초 기준본 추가 대조에서는 뷰어가 생성됐으나 10초 fixture 로딩 제한에 걸려 저장 검증까지
  진행하지 못한 실행도 있었다. 이를 자동 열기 누락의 동일 원인 증거로 사용하지 않는다.

브라우저는 격리된 임시 프로필을 사용했다. production textarea 입력 이벤트와 파일 메뉴를
실행했지만 OS 네이티브 저장 대화상자는 자동 다운로드 디렉터리로 대체했다.
Rust/WASM/Studio source 무변경이므로 Rust 전체 회귀·lint와 레이아웃 시각 검증은 적용하지 않았다.
“모든 환경에서 부작용 없음”을 보증하는 결과는 아니다. 원격 작업과 배포는 수행하지 않았다.


## 사용자 직접 검증

사용자가 제공한 #6961 + #6964 결합 Firefox dist를 직접 테스트하고 다음 세 항목이 모두
문제없다고 확인했다. 에이전트 자동 검증과 구분한 사용자 보고다.

1. HWP 다운로드 시 뷰어가 1개만 열린다.
2. 편집 후 저장/다른 이름으로 저장에서 파일명에 `_Users_…` 경로 접두사가 붙지 않는다.
3. 저장 시 추가 뷰어가 열리지 않으며 저장본에 편집 내용이 남는다.

## 기존 탭 수에 대한 해석

사용자는 기존 배포본에서 처음 다운로드할 때는 뷰어 1개, 저장할 때는 2개가 열렸던 것으로
기억한다고 덧붙였다. 기존 코드에서 가능한 관측이며 “모든 다운로드가 항상 2개”라는 뜻은 아니다.
수정 전 기준본 추가 실행에서도 `issue6964-baseline-trial-1.log`의 뷰어는 1개,
`issue6964-baseline-trial-3.log`의 뷰어는 2개로 관측됐다. 해당 실행들은 fixture 로딩 대기에서
종료됐으므로 전체 저장 여정 통과 증거가 아니라 관측된 탭 수의 근거로만 사용한다.

자체 저장 Blob을 외부 입력처럼 자동 여는 문제와 같은 다운로드 이벤트가 겹쳐 중복 처리되는
문제는 구분한다. 전자는 기존 뷰어 외에 저장본 뷰어를 추가하고, 후자가 겹치면 추가 생성도
중복될 수 있다. 사용자의 과거 “2개”가 전체 탭 수인지 추가 탭 수인지는 단정하지 않는다.


## Draft PR 제출

사용자의 명시 승인 후 upstream 작업 branch에 push하고 [PR #6966](https://github.com/edwardkim/rhwp/pull/6966)를 devel 대상으로 draft 생성했다.
[작성자 검토 기록](../pr/archives/pr_6966_review.md)을 같은 PR에 포함한다. 최초 head는
`53e3e701b0656468198f7d102c0077032332163e`이며, 이후 기록 commit도 같은 branch에 push한다.
생성 후 API에서 base/head/draft와 한글 본문을 재확인했다. CI는 작성 시점 실행 중이며,
ready 전환·merge·issue close·배포는 수행하지 않았다.
