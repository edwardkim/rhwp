# PR #7124 검토

- 검토일: 2026-09-14
- PR: https://github.com/edwardkim/rhwp/pull/7124
- 작성자: dependabot[bot]; 대상 branch: devel; 별도 관련 issue 없음(의존성 갱신).
- 검토 head: `eeba403b869ea00407f3cbfa46800710049ba084`
- 최신 devel 검사 기준: `93ffc3dd59c120bd54df4c2ac6d1ddbe630f8a2d`
- 규모: 1 commit, 2파일, +5/-5. 작성 시점 `MERGEABLE / CLEAN`.
- base route: maintainer_general
- modifiers: intake_and_review, local_validation, rework_and_exceptions
- loaded documents: pr_review_workflow.md, pr_review/README.md, 위 기본·보조 문서, codex/docs_and_git_workflow.md.
- 검토 branch/worktree: `review/pr7124`, `/home/edward/mygithub/rhwp-review-7124`.
- 접수: reviewer edwardkim 추가 및 API 재확인. 기존 jangster77 유지.

## 변경과 위험 분석

`rhwp-studio/package.json`과 lockfile의 `@types/chrome`을 0.2.8→0.2.9로 갱신한다.
[0.2.8 registry metadata](https://registry.npmjs.org/@types/chrome/0.2.8)와
[0.2.9 registry metadata](https://registry.npmjs.org/@types/chrome/0.2.9)의 실제 tarball을 메모리에서 비교했다.

- 기능상 차이는 `chrome.devtools.inspectedWindow.eval`의 Promise 반환형 및 callback 인자,
  `EvaluationExceptionInfo`의 판별 가능한 union 타입이다. 단순 버전 숫자만 보고 호환성을 판정하지 않았다.
- Studio source에서 해당 API/타입의 사용은 없었다. 사용 중인 runtime.sendMessage, tabs.create,
  extension 계열 타입에는 해당 패키지 diff가 없다.
- 배포 패키지는 선언 파일·README·LICENSE·metadata로 구성되며 설치 script와 실행 JS가 없다.
  전이 의존성은 변경되지 않았다. README와 contributor metadata 차이도 확인했다.
- source, Rust, workflow, renderer, fixture, baseline 변경은 없다. 별도 코드 보정은 불필요하다.

## 실제 검증

검토 head에서 Node v24.15.0 / npm 11.12.1로 실행했다.

| 검사 | 결과 |
| --- | --- |
| `git merge-tree --write-tree upstream/devel upstream/pr7124-head` | 충돌 없음; tree `95d11570607a1238ad7a865ac330e0e1e1c8ca90` |
| 위 base와 merge tree의 `git diff --check` | 통과 |
| Studio `npm ci --ignore-scripts --no-audit --no-fund` | 성공; lockfile 변경 없음 |
| Studio `./node_modules/.bin/tsc --noEmit` | 통과 |
| Studio `npm test` | 1,694건 중 1,692 통과, 2 skipped, 실패 0 |
| Studio `npm run e2e:document-title` | 실제 headless Chrome 통과: 문서 열기, 실패 시 보존, HWPX 저장, 새 문서 |
| exact head CI | [34800357352](https://github.com/edwardkim/rhwp/actions/runs/34800357352) success; required Build & Test pass |

브라우저 테스트는 별도 7701 포트를 사용했고 종료 후 테스트 서버도 종료했다. 기존 7700 서버는 변경하지 않았다.
WASM은 기본 작업트리의 기존 `pkg/`를 review worktree로 복사해 재사용했다. 이번 PR로 Rust/WASM을
재빌드했다는 의미는 아니다. 사용한 WASM SHA-256:
`61465d69c436028b5f5c8db93297a92fb7283cb58450ce0525b5b342943c1b9a`.
브라우저 확인은 기본 사용 경로의 smoke 검증이며 확장 프로그램 전체 API를 직접 검증한 것은 아니다.
Rust 변경이 없어 Cargo 전체 회귀·새 WASM 빌드·시각 비교는 실행하지 않았다.
설치 중 기존 source-map/glob deprecation 경고가 있었으며, 이 PR에서 해당 패키지를 갱신하거나
보안 audit를 수행한 것으로 기록하지 않는다.

## 공통 조판 원칙 및 입력 증거

- 조판 원칙 준수 검토: 비해당. 개발용 선언 패키지 변경으로 조판 규칙·측정·배치·출력 및 baseline을 바꾸지 않는다.
- 검증 입력 커밋 확인: 충족. E2E가 사용하는 기존 `samples/para-001.hwp`의 검토 head blob과
  실제 파일 SHA-256 모두 `bab4561ceb02cdfa184a1689be9619c08e18d6021cdbc423486b848bc14d267e`로 일치한다.
  PDF/신규 fixture는 사용하지 않았다. HWPX는 테스트 중 메모리에서 생성·저장하는 산출물이다.
- 제품 상수·정책·호출 경로 변경: 비해당. API 실행 구현이 아니라 미사용 DevTools 선언 타입 갱신이다.
- 시각 판정은 요구·수행하지 않았다. E2E 통과를 한컴 출력과의 시각 일치로 해석하지 않는다.

## 최종 판정

**승인**. 검토 범위에서 병합을 막을 결함은 발견하지 않았다.

병합 직전 최신 PR head, required checks, 최신 base와의 충돌 여부를 재확인하고 작업지시자의
명시적 병합 승인을 받아야 한다. 이 판정은 GitHub approve/comment/push/merge를 수행한 것이 아니다.
후속 #7123도 같은 Studio lockfile을 수정하므로 #7124 병합 뒤 별도로 충돌과 최신 CI를 확인한다.

## 병합 후 기록

- 2026-09-14 작업지시자가 병합과 후속 처리를 승인했다.
- exact head와 required Build & Test, 최신 devel merge tree를 재확인한 뒤 병합했다.
- merge: `9d341790f5e14dd686d951062059a71c1abb2f90` (2026-09-14 15:49:13 KST).
- 운영 기록은 maintainer 직접 반영 경로로 archive review와 오늘할일만 devel에 보존한다.
- 관련 issue 없음. Dependabot 원격 branch는 이번 작업에서 만든 branch가 아니므로 임의 삭제하지 않는다.
