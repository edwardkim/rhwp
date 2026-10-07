# PR #7584 통합 검토 — Dependabot #7579–#7583

## 최종 판정

**승인 — 로컬 통합 코드 후보 `24748e4b1c658643f5ada26519488b9a534166e9`.**
다섯 원 PR의 변경을 번호 순으로 체리픽했고, 통합 상태의 설치·빌드·타입 검사·단위 테스트·브라우저 검사와 Actions 계약 검사가 통과했다. 원 PR별 판정도 아래 표와 같이 모두 `승인`이다.

이 판정은 GitHub review event나 원격 병합이 아니다. 통합 PR의 원격 head가 생기면 그 정확한 head의 required checks와 mergeability를 확인하고, 이미 받은 작업지시자 승인에 따라 병합한다. 원 PR은 현재 OPEN이며 닫거나 직접 병합하지 않았다.

## 접수와 범위

- 검토일: 2026-10-05 KST. 작성자: `dependabot[bot]`. 다섯 PR 모두 `devel` 대상, 단일 커밋, 초안 아님, 관련 closing issue 없음.
- 기본 경로: `maintainer_general`. 보조 경로: `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `rework_and_exceptions`(Dependabot). `pr_review_workflow.md`, 자식 선택표·선택 문서, `github_operations.md`, `docs_and_git_workflow.md`를 확인했다.
- reviewer `jangster77`을 각 원 PR에 요청하고 REST API로 재확인했다. `gh pr edit`는 Projects classic GraphQL 오류로 실패해 REST `requested_reviewers`를 사용했다.
- 고정 base: `upstream/devel` = `e0f6b0942775b863e3bef292f133508b14a12e9e`. 로컬 브랜치: `review/dependabot-20261005`.
- 원 head를 `refs/pull/N/head`에서 가져와 `git cherry-pick -x`로 아래 순서대로 적용했다. Dependabot 저자와 원본 SHA를 보존했다. #7581의 Studio `package.json`·lockfile은 자동 병합됐고 수동 충돌 해결은 없었다.

| 원 PR | 변경 | 원 head | 체리픽 커밋 | 원 head CI 참고값 | 판정 |
| --- | --- | --- | --- | --- | --- |
| [#7579](https://github.com/edwardkim/rhwp/pull/7579) | Studio Vite 8.3.1→8.3.2 | `c0efe769760714f04438633e502548ce3653c956` | `6b840d440` | 19 성공, 실패·대기 0 | 승인 |
| [#7580](https://github.com/edwardkim/rhwp/pull/7580) | Firefox Vite 8.3.1→8.3.2 | `80bb587f0e202aa403f9b7cff0ba470faf3bc06a` | `3ad41e224` | 17 성공, 실패·대기 0 | 승인 |
| [#7581](https://github.com/edwardkim/rhwp/pull/7581) | Studio `@types/chrome` 0.3.0→0.3.4 | `046a8e67c6d4c2331911b8891100fad97ef23a4e` | `b07ff0ae5` | 19 성공, 실패·대기 0 | 승인 |
| [#7582](https://github.com/edwardkim/rhwp/pull/7582) | Chrome Vite 8.3.1→8.3.2 | `6da5d7089552207cbb2599b23fd67644579be122` | `d3e1ebe5c` | 17 성공, 실패·대기 0 | 승인 |
| [#7583](https://github.com/edwardkim/rhwp/pull/7583) | `taiki-e/install-action` 2.87.20→2.87.22 | `3e11c2a19289fe686aa972b1554c11a88ffc64bd` | `24748e4b1` | 29 성공, 실패·대기 0 | 승인 |

작성 시점 다섯 원 PR 모두 `MERGEABLE/CLEAN`이며 `Build & Test`가 성공했다. #7583의 archive 빌드 A–D·테스트 A–D도 성공했다. 상태는 변할 수 있으므로 원격 작업 직전 다시 조회한다.

## 변경 검토

- 세 프런트엔드 패키지의 Vite manifest와 lockfile은 동일한 8.3.2 tarball 무결성 값 및 Rolldown `~1.2.11` 요구를 기록한다. `npm ci` 뒤 세 패키지에서 설치된 Vite 8.3.2를 확인했다.
- Studio `@types/chrome` manifest와 lockfile만 0.3.4로 갱신됐다. 설치된 타입 패키지 0.3.4를 확인하고 Studio 전체 TypeScript 검사와 브라우저 경로를 실행했다. 런타임 코드나 공개 API는 수정하지 않았다.
- 두 nextest archive workflow의 `taiki-e/install-action` 고정 SHA만 `83ac0ad63c0167e6f06796fab0fce28db1bf3db0`으로 바뀌었다. 공식 GitHub tag API의 `v2.87.22` ref가 같은 commit을 가리켰다. trigger, permission, job 명령, `tool: nextest`는 바뀌지 않았다.
- Rust source·test, renderer·layout·paint, WASM source, 기준 PDF·golden·fixture, 허용치 변경은 없다. 조판 원칙 준수 검토와 Visual Sweep은 **비해당**이다. package 갱신으로 프런트엔드 빌드 도구와 타입 해석은 영향받으므로 아래 제품 경로 검증을 수행했다.

## 통합 코드 head 검증

실행 환경: Ubuntu `ubuntu-ted`, Node 24.15.0, npm 11.12.1. 검증 당시 코드 head `24748e4b1c658643f5ada26519488b9a534166e9`; 로그는 ignored `output/pr-review/dependabot-20261005/logs/`에 있다. 표의 PASS는 통합 head에서 직접 실행한 결과이며 원 PR CI 결과와 구분한다.

| 검사 | 결과 |
| --- | --- |
| `git diff --check upstream/devel...HEAD`; `git merge-tree --write-tree upstream/devel HEAD` | PASS, 충돌 없음 |
| `npm --prefix {rhwp-studio,rhwp-chrome,rhwp-firefox} ci --no-audit --no-fund --prefer-offline` | 3개 PASS; 추적 파일 변경 없음 |
| `cd rhwp-studio && ./node_modules/.bin/tsc --noEmit` | PASS |
| `npm --prefix rhwp-studio test` | 1,815 pass, 2 skipped, 0 fail |
| Studio, Chrome, Firefox의 `npm run build` | 3개 PASS |
| `node --test scripts/frontend-extension-dist.test.mjs` | 3 pass, 0 fail |
| `VITE_PORT=7701 npm --prefix rhwp-studio run e2e:document-title` | headless Chrome에서 문서 열기·실패 보존·HWPX 저장·창 제목 PASS |
| `npm --prefix rhwp-chrome run test:e2e:smoke` | 4개 page-budget 검사와 viewer/options/print/service worker/content script 스모크 PASS |
| `FIREFOX_EXECUTABLE_PATH=/usr/bin/firefox TMPDIR=... npm --prefix rhwp-firefox run test:e2e:download` | Firefox 157에서 다운로드 3건·편집 내용 보존 PASS |
| `actionlint` 대상 archive workflow 2개 | PASS |
| `python3 -m unittest discover -s scripts/tests -p 'test_nextest_archive_workflow.py'` | 15 tests PASS |
| `python3 -m unittest discover -s scripts/tests -p 'test_workflow_contract_wiring.py'` | 3 tests PASS |

검증 명령의 첫 Studio TypeScript 실행에서 `npm --prefix ... exec -- tsc`가 프로젝트 설정을 읽지 못해 도움말만 출력했다. 저장소 문서의 작업 디렉터리 방식으로 다시 실행해 PASS를 확인했다. Firefox 첫 실행은 `FIREFOX_EXECUTABLE_PATH`가 없어 테스트 시작 전 중단됐고, 설치된 `/usr/bin/firefox`와 별도 `TMPDIR`을 지정해 다시 실행해 PASS를 확인했다. 두 초기 명령을 제품 실패로 세지 않는다.

### 검증 입력 커밋 확인

브라우저 검사의 세 HWP 입력은 통합 코드 head에 이미 추적되며, 실행 파일과 `git cat-file -p HEAD:<path>`의 SHA-256이 일치한다. 새 기준 PDF·시각 비교 입력은 사용하지 않았다. 브라우저 검사는 기존 `pkg/` WASM을 재사용했으며 이번 변경에 Rust/WASM 수정이 없으므로 fresh WASM 시각 일치 주장으로 해석하지 않는다.

| 입력 | 역할 | SHA-256 |
| --- | --- | --- |
| `samples/para-001.hwp` | Studio 문서 제목 E2E | `bab4561ceb02cdfa184a1689be9619c08e18d6021cdbc423486b848bc14d267e` |
| `samples/hwp3-pagedef-1915.hwp` | Chrome 확장 스모크 | `b272fdd218b4e91355167e63438a1605ef6902d75970c4ff5b8bae67087122d0` |
| `samples/re-font-dotum-empty-hancom.hwp` | Firefox 다운로드·편집 E2E | `1ee8871f37bec2e97d0928709dc411c0eacad656f35aa3d92c5cf89f61c5761b` |

## 통합과 후속 조건

한 통합 PR에 위 다섯 체리픽과 이 검토 기록을 포함한다. 코드 후보 뒤 review 기록만 후행하므로 GitHub CI에서 최신 통합 head의 frontend package gate, Actions/nextest archive jobs, required checks를 확인한다. 원 PR의 녹색 CI를 통합 head의 CI 완료로 간주하지 않는다. 통합 [PR #7584](https://github.com/edwardkim/rhwp/pull/7584)를 Open으로 생성했다. 작업지시자가 오늘할일 포함·CI 모니터링·성공 뒤 merge와 후속 처리를 승인했다. 검증 후보의 Full CI는 아래와 같이 완료됐으며 원 PR close는 통합 병합 이후 수행한다.

제안 PR 제목: `chore: Dependabot 프런트엔드와 nextest Action 갱신 통합`.
원 PR은 통합 PR 병합 및 `devel` 포함을 확인한 뒤 각 원본 SHA·체리픽 SHA·통합 merge SHA와 검증 결과를 남기고 superseded로 정리한다. 관련 closing issue는 없다. Dependabot 원격 branch는 이번 작업에서 만든 브랜치가 아니므로 삭제하지 않는다.

## 통합 PR 접수와 승인 범위

- 통합 PR 작성자 `jangster77`의 self-review이며 통합 PR에는 reviewer를 지정하지 않는다. 기본 경로 `collaborator_self_merge`, 보조 `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `rework_and_exceptions`, `review_only_fast_pass`, `post_merge`를 적용한다. 원 Dependabot PR의 접수 기록은 위에 보존했다.
- 원격 head는 `integration/dependabot-7579-7583-20261005`이며 이 archive 기록과 오늘할일을 동일 PR에 포함한다. 검증 후보의 CI 완료를 아래에 기록했다. 이 문서 후행 commit의 최신 required check와 mergeability를 확인하면 승인된 일반 merge를 수행한다.
- merge 뒤 문서·오늘할일·5개 갱신의 devel 포함과 duration 갱신 결과를 확인한다. 통합 PR 및 원 PR에 merge SHA·실제 CI·로컬 검증을 게시하고 원 5건을 superseded로 닫는다. 이번 검토의 임시 원격/로컬 branch·fetch refs·ignored 로그만 정리하며 공유 target은 보존한다.

## 통합 후보 Full CI 완료

- exact 후보 `5d0e8cdb32d395309fe7a76e3cef80b09d897c5b`의 [CI 37265915618](https://github.com/edwardkim/rhwp/actions/runs/37265915618), [CodeQL 37265915563](https://github.com/edwardkim/rhwp/actions/runs/37265915563), [Render Diff 37265915335](https://github.com/edwardkim/rhwp/actions/runs/37265915335), [Adapter 37265915480](https://github.com/edwardkim/rhwp/actions/runs/37265915480), [Proptest 37265915519](https://github.com/edwardkim/rhwp/actions/runs/37265915519)가 모두 success로 완료됐다. 32 SUCCESS, 실패·대기 0, `MERGEABLE/CLEAN`을 확인했다.
- Lint, Frontend package gates, Native Skia, archive 빌드 A–D와 테스트 A–D, Build & Test가 모두 성공했다. 이번 통합 후보에서 새 install-action의 실제 설치·빌드·테스트 경로를 확인했다.
- 이 후행 commit은 archive review와 오늘할일만 갱신한다. 원 PR head·제품 코드·package lock·workflow는 변경하지 않았다. 최신 후행 head의 trusted 정책·preflight·required checks를 확인한 뒤 정확한 head SHA를 `--match-head-commit`으로 지정해 병합한다.
