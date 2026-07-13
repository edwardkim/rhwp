# Task M100 #2183 구현 계획서 — 프론트 패키지 CI gate

- 이슈: #2183
- 상위 추적: #2022
- 선행 기준선: #2124 / PR #2174
- 연계 PR: #2187
- 후속 Phase: #2125
- 마일스톤: M100 / v1.0.0
- 브랜치: `task2183-frontend-ci-gate`
- 기준 브랜치: `upstream/devel`
- 기준 커밋: `3cf6d949a7617f066850065e245f4902a98e3c9e`
- 작성일: 2026-07-11
- 단계: 구현 계획서
- 선행 수행 계획서: `mydocs/plans/archives/task_m100_2183.md`

## 1. 구현 목표

프론트 영향 경로가 변경된 CI run에서 clean checkout을 기준으로 fresh WASM binding을 생성하고,
Studio·Chrome·Firefox·VS Code·`@rhwp/editor`의 package gate를 실행한다. 새 worker 결과는 별도 required
check를 만들지 않고 기존 `CI / Build & Test`에 집계한다.

구현은 다음 계약을 만족해야 한다.

1. 프론트 변경을 실제로 검증하지 않은 green을 만들지 않는다.
2. 파일 목록 판정 실패나 잘림은 gate 생략이 아니라 frontend 실행으로 귀결한다.
3. ignored generated output인 `pkg/` 또는 tracked legacy WASM을 source of truth로 사용하지 않는다.
4. 기존 Rust worker, review-only fast-pass, release WASM job의 의미를 바꾸지 않는다.
5. package test 개수나 audit 경고 수를 하드코딩하지 않고 명령 종료 상태만 gate로 사용한다.

## 2. 이슈 제안과 fresh WASM 결정의 차이

#2183 원문은 fresh WASM 비용 때문에 binding gate를 advisory 또는 후속 판단으로 두었다. 그러나 #2124에서
다음을 확인했다.

- clean checkout에는 ignored output인 `pkg/`가 없다.
- Studio의 `@wasm` alias, Chrome/Firefox build, VS Code CopyPlugin은 모두 `pkg/`를 소비한다.
- tracked `web/rhwp.*`나 Studio public 산출물을 `pkg/` 대용으로 복사하면 stale declaration/binary를 이용한
  false green이 된다.

따라서 실제 package build gate를 추가하면서 fresh WASM만 제외하는 조합은 성립하지 않는다. #2183에서는
release artifact를 만들지 않고 frontend worker의 검증 입력으로만
`wasm-pack build --target web --dev`를 실행한다.

이 결정은 원 이슈보다 gate 범위를 넓히므로 draft PR에서 maintainer에게 명시적으로 리뷰를 요청한다. 비용이
수용되지 않으면 stale output으로 대체하지 않고, frontend worker 전체 또는 fresh build 최적화를 별도 설계로
되돌려야 한다.

## 3. 변경 파일과 비변경 계약

### 3.1 변경 파일

| 파일 | 변경 |
|------|------|
| `.github/workflows/ci.yml` | frontend 영향 판정 output, frontend package worker, aggregate 판정 추가 |
| `mydocs/working/task_m100_2183_stage{N}.md` | 단계별 구현·검증 결과 기록 |
| `mydocs/report/task_m100_2183_report.md` | 최종 결과와 CI 비용·후속 판단 기록 |
| `mydocs/orders/20260711.md` | 단계 상태 갱신 |

### 3.2 변경하지 않는 표면

- frontend package source와 lockfile
- `npm/editor/package.json`과 무의존 iframe/message public contract
- `wasm-build` release job의 trigger, profile, artifact
- Render Diff, CodeQL, deploy, npm publish workflow
- Rust test profile과 기존 Rust cargo cache key
- branch protection 설정과 `Build & Test` check 이름
- `assets/fonts` 이전, `web/fonts` 삭제, legacy `/web` 정리
- browser install E2E, VS Code Extension Host, Safari/Xcode build

## 4. preflight 영향 판정 구현

### 4.1 output

`preflight.outputs`에 다음 값을 추가한다.

| output | 기본값 | 의미 |
|--------|--------|------|
| `frontend_required` | `'true'` | frontend worker 실행 필요 여부 |
| `frontend_reason` | `'frontend-detection-unavailable'` | 판정 근거와 fallback 원인 |

output은 새 `Detect frontend impact` step에서 설정한다. action 자체가 시작되지 못하거나 step output이 비어도
expression 기본값이 `true`가 되게 한다. 기존 `fast_pass`, `reason`, `candidate_sha`는 변경하지 않는다.

### 4.2 경로 predicate

다음 경로를 frontend 영향으로 판정한다.

```text
rhwp-studio/**
rhwp-chrome/**
rhwp-firefox/**
rhwp-safari/**
rhwp-vscode/**
rhwp-shared/**
npm/editor/**
scripts/frontend-*.mjs
web/**
Cargo.lock
src/wasm_api.rs
.github/workflows/ci.yml
```

directory는 `startsWith`, 단일 파일은 exact match, frontend script는 `scripts/frontend-` prefix와 `.mjs`
suffix를 함께 검사한다. 경로를 정규식 한 줄로 축약하지 않아 리뷰 시 누락 여부를 확인할 수 있게 한다.

현재 workflow-level `paths-ignore`의 `assets/**`는 유지한다. `assets/fonts/**`는 #2125에서 실제 canonical
경로가 생길 때 trigger와 함께 추가한다.

### 4.3 이벤트별 파일 수집

`actions/github-script@v8`의 read-only API를 사용한다.

| 이벤트 | 수집 방식 | fallback |
|--------|-----------|----------|
| `pull_request` | `github.paginate(github.rest.pulls.listFiles)` | 0건 또는 오류면 실행 |
| `push` | `before...after` compare의 `data.files` | zero SHA, 0건, 300건 이상, 오류면 실행 |
| `workflow_dispatch` | 파일 목록 없음 | 실행 |
| tag push | 변경 범위보다 release 안전 우선 | 실행 |
| 그 외 이벤트 | 판정 계약 없음 | 실행 |

PR file API는 paginate한 전체 목록을 사용한다. push compare의 file 목록은 API 한계로 잘릴 수 있으므로
300건 이상이면 `frontend_required=true`로 처리한다. 판정 step은 `continue-on-error: true`를 유지할 수 있지만,
예외 catch와 output 기본값 모두 실행 쪽으로 닫는다.

`frontend_reason`은 최소한 `frontend-path:<path>`, `no-frontend-path`, `pr-file-list-empty`,
`push-zero-before`, `push-file-list-unusable`, `manual-or-tag`, `unsupported-event`, `detection-error`를 구분한다.
aggregate 로그에서 이 값을 출력해 실행·skip 판단을 run 하나만으로 추적할 수 있게 한다.

### 4.4 fast-pass와의 우선순위

frontend 판정은 기존 review-only detector와 별도 step으로 둔다. worker 조건에서는 다음 순서를 적용한다.

1. preflight가 성공해야 한다.
2. `fast_pass != 'true'`여야 한다.
3. `frontend_required == 'true'`여야 한다.

review-only fast-pass가 성립하면 frontend 판정값과 무관하게 모든 worker를 skip하고 기존 candidate
`Build & Test` 결과를 재사용한다. preflight job 자체가 실패하면 aggregate가 실패한다.

fork PR에서는 candidate head SHA가 Actions run/job metadata에는 노출되지만 Check Runs API가 같은 SHA에 0건을
반환할 수 있다. 따라서 기존 `checks: read` 조회를 우선 유지하고, `Build & Test` check-run이 없을 때만
read-only `actions: read` fallback을 사용한다.

fallback은 현재 PR head branch의 `ci.yml` pull-request runs를 조회한 뒤 응답의 `head_sha`를 candidate SHA와
로컬에서 exact match하고, 해당 run의 `Build & Test` job이 `completed/success`인지 확인한다. Actions API의
`head_sha` server filter도 fork run을 누락할 수 있으므로 이를 authority로 사용하지 않는다. run/job 누락,
진행 중, 실패, API 오류는 모두 fast-pass 거부와 full CI 실행으로 처리한다.

## 5. frontend worker 구현

### 5.1 job 조건과 권한

- job id: `frontend-package-gates`
- 표시 이름: `Frontend package gates`
- runner: `ubuntu-latest`
- `needs: preflight`
- 권한: `contents: read`
- 조건: preflight 성공, fast-pass 아님, frontend 영향 있음

Rust workers와 병렬로 실행하며 다른 worker의 artifact나 workspace를 소비하지 않는다.

### 5.2 toolchain과 fresh binding

순서는 다음으로 고정한다.

1. `actions/checkout@v5`
2. `dtolnay/rust-toolchain@stable`, toolchain `1.93.1`, target `wasm32-unknown-unknown`
3. 기존 `wasm-build`/Render Diff와 같은 wasm-pack installer
4. frontend 전용 cargo cache restore
5. `wasm-pack build --target web --dev`
6. Node.js 22 setup

fresh output은 CI 검증용이며 upload, commit, release publish를 하지 않는다. `wasm-opt`와 release profile도
요구하지 않는다.

### 5.3 cache

frontend WASM cargo cache는 기존 Rust·release·Render Diff key와 분리한다.

```text
path:
  ~/.cargo/registry
  ~/.cargo/git
  target
key: ${{ runner.os }}-frontend-wasm-cargo-${{ hashFiles('**/Cargo.lock') }}
restore-prefix: ${{ runner.os }}-frontend-wasm-cargo-
```

- pull request: restore-only
- trusted `devel`/`main` push: exact miss일 때만 save
- tag/manual: save하지 않음

Node는 `actions/setup-node@v4`의 npm download cache를 사용한다. `cache-dependency-path`에는 다음 네 lockfile을
명시하고 `node_modules`는 cache하지 않는다.

- `rhwp-studio/package-lock.json`
- `rhwp-chrome/package-lock.json`
- `rhwp-firefox/package-lock.json`
- `rhwp-vscode/package-lock.json`

### 5.4 install과 gate 순서

package install은 각 lockfile의 재현성을 보존하기 위해 다음을 실행한다.

```bash
npm --prefix rhwp-studio ci
npm --prefix rhwp-chrome ci
npm --prefix rhwp-firefox ci
npm --prefix rhwp-vscode ci
```

`npm/editor`에는 현재 lockfile과 dependency가 없으므로 `npm ci`를 만들지 않는다. gate는 다음 순서로
실행한다.

```bash
node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs
npm --prefix npm/editor test --if-present
node --test rhwp-shared/sw/*.test.js rhwp-chrome/sw/*.test.mjs rhwp-firefox/sw/*.test.mjs
npm --prefix rhwp-studio run test
npm --prefix rhwp-studio run build
npm --prefix rhwp-chrome run build
npm --prefix rhwp-firefox run build
node --test scripts/frontend-extension-dist.test.mjs
npm --prefix rhwp-vscode run compile
```

Studio install은 Chrome/Firefox build가 Studio cwd에서 Vite를 실행하기 전에 완료한다. extension build가
각각 자체 `dist/`를 생성한 뒤에 dist contract를 실행한다. VS Code compile도 같은 fresh `pkg/rhwp_bg.wasm`을
소비한다.

#2187이 `npm/editor`에 `scripts.test`를 추가하면 `--if-present`가 transport tests를 자동 실행한다. 현재
branch에서는 root editor contract가 항상 실행되므로 test script 부재가 검증 공백이 되지 않는다.

## 6. `Build & Test` aggregate 구현

`build-and-test.needs`에 `frontend-package-gates`를 추가하고 로그에 다음을 출력한다.

- preflight result
- fast-pass
- frontend-required와 reason
- Rust default worker result
- native Skia worker result
- frontend worker result

판정 진리표는 다음과 같다.

| preflight | fast-pass | frontend required | Rust 2종 | frontend result | aggregate |
|-----------|-----------|-------------------|----------|-----------------|-----------|
| 실패/cancel | 임의 | 임의 | 임의 | 임의 | 실패 |
| 성공 | true | 임의 | skipped | skipped | 성공 |
| 성공 | false | true | 모두 success | success | 성공 |
| 성공 | false | true | 모두 success | skipped/실패/cancel | 실패 |
| 성공 | false | false | 모두 success | skipped | 성공 |
| 성공 | false | false | 모두 success | success/실패/cancel | 실패 |
| 성공 | false | 임의 | 하나라도 비-success | 임의 | 실패 |

`frontend_required`가 `true`/`false` 이외 값이면 required로 간주한다. job 조건 때문에 worker가 실행되지
않더라도 aggregate가 실패하므로 알 수 없는 output이 silent skip이 되지 않는다.

## 7. 단계별 구현과 커밋

### Stage 1 — workflow 구현과 정적 검증

작업:

- preflight frontend output과 detector 추가
- `frontend-package-gates` worker 추가
- `Build & Test` needs와 진리표 판정 추가
- 기존 worker와 release WASM diff가 의도한 영역 밖으로 번지지 않았는지 확인

검증:

```bash
actionlint .github/workflows/ci.yml
git diff --check
git diff -- .github/workflows/ci.yml
rg -n "frontend_required|frontend-package-gates|Frontend package gates" .github/workflows/ci.yml
```

산출물:

- `mydocs/working/task_m100_2183_stage1.md`
- workflow와 Stage 1 보고서를 한 구현 커밋으로 묶음

### Stage 2 — local consumer gate 재현

저장소 규칙상 로컬 WASM은 Docker로 생성한다. CI의 `--dev` profile 자체는 GitHub runner에서 검증하고,
로컬에서는 release Docker output으로 같은 consumer 명령이 모두 통과하는지 확인한다.

```bash
docker compose --env-file .env.docker run --rm wasm
npm --prefix rhwp-studio ci
npm --prefix rhwp-chrome ci
npm --prefix rhwp-firefox ci
npm --prefix rhwp-vscode ci
node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs
npm --prefix npm/editor test --if-present
node --test rhwp-shared/sw/*.test.js rhwp-chrome/sw/*.test.mjs rhwp-firefox/sw/*.test.mjs
npm --prefix rhwp-studio run test
npm --prefix rhwp-studio run build
npm --prefix rhwp-chrome run build
npm --prefix rhwp-firefox run build
node --test scripts/frontend-extension-dist.test.mjs
npm --prefix rhwp-vscode run compile
```

검증 결과에는 test 개수 대신 각 command의 PASS/FAIL과 Node/npm/Rust/Docker 버전을 기록한다. `npm audit`
경고는 별도 관찰로 남기고 lockfile 수정이나 `npm audit fix`를 수행하지 않는다.

산출물:

- `mydocs/working/task_m100_2183_stage2.md`

### Stage 3 — draft PR GitHub Actions 실측

push와 draft PR 생성 전 PR 본문·리뷰 요청 초안을 작업지시자에게 먼저 제시한다. 승인 후에만 등록한다.

확인 항목:

1. `.github/workflows/ci.yml` 변경으로 frontend worker가 실제 실행된다.
2. fresh dev WASM, package install, 모든 consumer gate가 clean runner에서 통과한다.
3. frontend worker 실패/cancel이 `Build & Test`를 차단한다.
4. 기존 Rust workers와 병렬 실행되며 required check 이름이 유지된다.
5. trusted push cache save 조건과 PR restore-only 조건이 로그와 일치한다.
6. 총 duration, frontend job duration, cache hit/miss를 기록한다.

path skip과 review-only fast-pass는 PR 본문에 truth table·정적 근거를 제시하고, 필요하면 maintainer가 허용한
별도 trailing docs commit 또는 후속 실측으로 확인한다. 검증만을 위해 제품 파일을 임의 변경하는 commit은
추가하지 않는다.

산출물:

- `mydocs/working/task_m100_2183_stage3.md`

### Stage 4 — 결산과 후속 순서 확정

- #2183 현재 본문에는 체크리스트가 없으므로, 원문의 제안 항목과 산출물 근거를 최종 보고서에서 1:1로
  연결한다.
- merge 준비 시 완료 조건 체크리스트를 본문에 추가할지, 최종 코멘트의 근거표로 결산할지 초안을 제시해
  maintainer와 작업지시자 승인을 받는다.
- `mydocs/report/task_m100_2183_report.md`에 CI 비용과 fresh WASM 결정의 리뷰 결과를 기록한다.
- #2187 rebase 후 `npm/editor` package test가 실제 실행되는 조건을 확인한다.
- #2183 merge 뒤 #2187 리뷰, 그 뒤 #2125 착수 순서를 #2022 상태와 대조한다.
- issue 체크리스트 수정, 코멘트, close는 각각 초안을 제시하고 작업지시자 승인 후 수행한다.

산출물:

- `mydocs/working/task_m100_2183_stage4.md`
- `mydocs/report/task_m100_2183_report.md`

## 8. 실패 처리와 롤백 기준

| 실패 | 처리 |
|------|------|
| path API 오류/목록 잘림 | frontend 실행; skip으로 완화하지 않음 |
| fresh WASM build 실패 | package gate 전체 실패; tracked output 복사 금지 |
| 특정 `npm ci` 실패 | 해당 lockfile 문제로 기록; npm install fallback 금지 |
| package build/test 실패 | 실제 gate 실패로 유지; `continue-on-error` 금지 |
| audit warning | 로그에 기록; #2183 실패 조건으로 승격하지 않음 |
| frontend runner 비용 과다 | 실측을 maintainer에게 제시하고 cache/profile/trigger 후속 판단 |
| aggregate expression 이상 | `Build & Test` green 전 merge 금지 |

workflow rollback이 필요하면 새 frontend detector·worker·aggregate 연결을 같은 단위로 되돌린다. worker만
제거하고 aggregate 참조를 남기거나, fresh build만 제거해 stale output을 소비하는 중간 상태는 허용하지 않는다.

## 9. 완료 조건

1. `actionlint`와 `git diff --check`가 통과한다.
2. local Docker fresh output에서 모든 consumer gate가 통과한다.
3. draft PR clean runner에서 `wasm-pack --dev`와 frontend package gate가 통과한다.
4. frontend 영향 변경은 frontend 성공 없이는 `Build & Test`가 green이 되지 않는다.
5. 비-frontend 변경과 review-only fast-pass의 기존 의미가 유지된다.
6. branch protection, release WASM, package public/security contract를 변경하지 않는다.
7. CI 시간·cache 결과와 원 이슈 대비 fresh WASM 범위 확장을 maintainer가 리뷰할 수 있게 기록한다.
8. 작업지시자 승인 전 push·PR·GitHub 코멘트·issue close를 수행하지 않는다.

## 10. 구현 승인 후 첫 작업

구현 승인을 받으면 `.github/workflows/ci.yml` 한 파일에 preflight detector, frontend worker, aggregate 연결을
한 번에 추가한다. 부분 상태를 commit하지 않고 `actionlint` 통과 후 Stage 1 보고서와 함께 첫 구현 커밋을
만든다.
