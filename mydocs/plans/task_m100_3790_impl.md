# 구현계획서 — task_m100_3790

- **이슈**: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- **수행계획서**: `mydocs/plans/task_m100_3790.md`
- **브랜치**: Stage 1 `codex/issue-3790-ci-impact-shadow`, Stage 2·2.5
  `codex/issue-3790-shadow-observation`, Stage 3 `codex/issue-3790-stage3-frontend`, Stage 4
  `issue-3790-stage4-rust-native`, Stage 5A `issue-3790-stage5a-codeql-safety`
- **절차 상태**: Stage 3·4 merge·canary 완료. `upstream/devel` `e48fe86947fb`에서 Stage 5A의
  보안 check 재사용과 Rust no-build shadow를 구현하고 focused 검증을 통과했다. Draft PR #4341의
  1차 원격 canary 분석 뒤 raw blocking SARIF·동일 권한·기본 build mode의 no-prebuild shadow로
  보정하고 원격 동등성 gate를 통과했다. 수동 cache·prebuild와 측정 요소를 제거한 최종 구성도 focused
  검증과 최종 PR CI·GHAS를 통과했다. Ready 전환 뒤 self-review F1–F6을 수용해 최신
  `upstream/devel` `0664e6568e9b`을 병합하고 보정 head의 full CI·CodeQL 재검증 단계로 전환했다.

## Stage 1 — shadow classifier

1. `scripts/ci-impact-classifier.cjs`에 부작용 없는 변경 집합 판정 함수를 둔다.
2. `scripts/tests/fixtures/ci-impact-classifier-prs.json`에 #3785, #3656, #3670, #3672, #3690의 실제
   변경 파일과 기대 출력을 고정한다.
3. 단위 테스트에서 historical fixture, mode 승격, 언어 집합, review-only, fail-closed 경계를 검증한다.
4. `CI preflight`가 PR/push 파일 목록을 수집해 classifier를 호출하고 `shadow_*` output과 Job Summary를
   기록하게 한다.
5. workflow 계약 테스트에서 shadow output이 기존 worker 조건에 사용되지 않음을 확인한다.

pull request에서는 checkout된 merge ref의 classifier가 실행되므로 Stage 1 결과는 advisory다. 실제 skip을
활성화하는 PR은 base SHA의 classifier를 사용하거나 동등한 trusted execution 경계를 먼저 구현해야 한다.

## Stage 2 — shadow 실측

1. draft PR의 각 run에서 shadow summary와 실제 변경 파일을 대조한다.
2. 분류 실패, API 경계, rename, mixed 변경의 full fallback을 확인한다.
3. 실제 worker duration과 예상 절감 runner-minute를 기록한다.
4. false negative가 있으면 규칙과 fixture를 먼저 보정하고 활성화를 연기한다.

1차 실측 결과는 `mydocs/working/task_m100_3790_stage2.md`에 기록했다. 네 live run은 모두 완료됐고
#3740에서 rename full fallback과 기존 Rust fmt 차단을 확인했다. historical replay 60건, 고정 fixture와
현재까지 관측 false negative 0건을 Stage 3 활성화 근거로 사용하며 자연 발생 frontend 표본 5건은 더
기다리지 않는다.

## Stage 2.5 — trusted-base shadow

1. PR에서는 `github.event.pull_request.base.sha`의 classifier만 sparse checkout한다.
2. push·manual 실행은 해당 실행의 `github.sha`를 사용한다.
3. PR authority를 `pr-base-trusted-shadow`로 기록하고 기존 merge-ref advisory 표본과 분리한다.
4. checkout credential을 저장하지 않고 classifier node step에는 토큰을 전달하지 않는다.
5. 기존 worker 조건이 shadow output을 소비하지 않는 정적 계약을 유지한다.
6. base SHA ref, classifier 파일 존재, authority, review-only fast-pass와 fail-open shadow 동작을
   workflow 테스트로 고정한다.

이 단계의 PR CI가 통과해도 worker skip은 활성화되지 않았다. #3823 merge 뒤 base SHA classifier 경계가
devel에 반영됐으며, Stage 3부터 이 출력을 실제 frontend gate에 연결한다.

Stage 2.5가 고정하는 신뢰 경계는 `scripts/ci-impact-classifier.cjs`의 출처뿐이다. `pull_request`의
workflow YAML, 인라인 collect script, classifier 실행 명령과 Stage 3에서 추가할 worker `if`는 PR merge
ref의 제어를 받는다. 따라서 `pr-base-trusted-shadow`는 classifier-source provenance이지 실제 skip을
허용하는 trusted execution 증명이 아니다.

## Stage 2.6 — devel 활성화와 post-main enforcement 분리

1. contributor/collaborator를 구분하지 않고 모든 genuine frontend-only PR에 기존 `pull_request`
   workflow의 선택 실행을 적용한다.
2. workflow/classifier/Cargo/WASM/rename/미분류 변경과 classifier/API 오류는 full로 닫는다.
3. 약 1,500줄 local controller 프로토타입은 원격에 게시하지 않고 Stage 3~5의 최종 job 진리표를 반영할
   후속 controller의 설계·테스트 근거로 보존한다.
4. Stage 3~5와 canary는 main 릴리즈 전에 devel 대상 PR에서 실행한다. controller는 별도 main PR로
   등록하지 않고 정상 `devel → main` 릴리즈를 기다린다.
5. main 등록 뒤 controller가 PR head code/artifact를 실행하지 않고 실제 job의 expected
   `success|skipped`를 독립 감사하게 한다. repository admin이 required status를 채택해야 merge
   enforcement가 완성된다.

controller 프로토타입은 대체 controller가 main에서 live audit까지 통과하거나 maintainer가 required
policy를 미채택하기로 결정할 때까지 보존한다. 이후 재사용할 설계·테스트 근거를 계획·보고서에 옮긴 뒤
사용자 승인으로 local branch/worktree를 정리한다.

## Stage 3 — frontend unit/package/render 활성화

1. `unit`은 Studio 전체 `src`의 `tsc --noEmit`과 전체 Studio unit test를 실행한다. fresh WASM build는
   생략하며 CI 전용 tsconfig가 `@wasm/rhwp.js`만 최소 stub으로 치환한다.
2. `package`는 `unit` 계약에 Vite·extension·package build를 추가한다.
3. Render Diff의 Canvas visual diff와 CanvasKit readiness가 실제로 소비하는 경로를 각각 도출한다.
   영향축을 분리하지 않으면 두 gate 의존성의 보수적 합집합만 `render_required`에 연결한다.
4. `canvaskit` 파일명 heuristic처럼 피시험 코드와 테스트를 다르게 분류하는 규칙을 제거하고 계약
   fixture로 고정한다.
5. aggregate는 필요한 worker `success`, 불필요 worker `skipped`만 허용한다.
6. #3785/#3656은 unit, #3670과 #3672는 package, #3672는 추가로 render 경로를 실측한다.
7. label 변경은 workflow를 재시작하지 않는다. canary의 full 대조군은 같은 SHA에 대한 수동
   `workflow_dispatch`로 만들며, label 기반 강제 full은 post-main trusted controller 단계로 미룬다.
8. 작성자 association은 선택 실행 조건으로 사용하지 않는다. 외부 fork의 정상 frontend-only PR도 같은
   영향축을 사용하며, workflow 변경은 author와 무관하게 full이다.
9. WASM binding을 직접 소비하는 `src/core/**`, `src/embed/**`, `src/main.ts`, `public/**`, `src/hwpctl/**`은
   package lane으로 승격한다. CI 전용 tsconfig·stub 자체 변경도 full로 닫는다.
10. 최초 unit 소스 범위는 historical fixture로 검증한 `src/command/**`, `src/engine/command.ts`로 제한한다.
    `src/view/**`, `src/ui/**`, 그 밖의 Studio runtime은 package+render에서 시작해 canary 근거 뒤에만 넓힌다.

## Stage 4 — Rust·Native Skia 조건화

Stage 3 merge 직후 frontend-only canary PR #3951에서 unit/package/render 진리표를 확인했다. 같은 SHA의
수동 full은 기존 cold release archive timeout으로 전체 완료되지 않았지만, 성공한 frontend와 Canvas
구간에서 직접 runner time 7분 47초 절감을 확인했다. timeout은 #4029에서 분리해 추적한다.

1. Rust 비영향 PR에서 lint와 #3892의 `build-test-archive-slow`, `build-test-archive-a`,
   `build-test-archive-b` 세 builder를 생략한다.
2. 같은 영향축으로 `test-slow-shard`, `test-regular-shard-1`, `test-regular-shard-2`,
   `test-regular-shard-3` 네 worker를 조건화한다.
3. aggregate는 세 builder와 네 worker 각각에 필요한 `success` 또는 불필요한 `skipped`만 허용한다.
4. Rust 변경 중 render 비영향 경로는 Native Skia를 생략한다.
5. Rust formatter·passthrough invalidation·IR baseline 회귀가 필요한 경로는 기존 전체 검증을 유지한다.
6. #3684를 완료한 #3810의 정리 직후 cache 기준선 4.73GB와 조건화 이후 다음 sweep 직후 총량을
   같은 시점 조건으로 대조한다. **수행 결과**: 스윕 직후 8.84GB(50개)로 기준선 대비 +87% 회귀했으나
   Stage 4가 원인이 아니다. 추세가 Stage 4 merge 이전에 이미 완성됐고, 원인은 (그룹, ref) 쌍 수 증가와
   삭제된 브랜치의 고아 캐시다. 대응은 [#4080](https://github.com/edwardkim/rhwp/issues/4080)으로 분리했다.
7. Native Skia가 직접 실행하는 `tests/issue_2225_missing_picture_placeholder.rs`와
   `tests/render_p37_direct_pdf_export.rs`는 일반 Rust 비렌더 경로와 달리 `native_skia_required=true`로
   고정한다.
8. Native Skia는 Rust renderer뿐 아니라 frontend font asset·render 생성 도구 같은 비-Rust 입력에서도
   필요할 수 있으므로 `rust_required=false`, `native_skia_required=true` 조합을 지원한다.
   다만 default-feature 테스트가 소비하는 `ttfs/**`·`tests/fixtures/fonts/**`의 글꼴 파일과
   `samples/render-p35-font-native-bitmap.hwpx`는 `rust_required=true`를 함께 설정한다.
9. aggregate는 Rust false일 때 lint·세 builder·네 worker가 모두 `skipped`, Native false일 때 Native
   job이 `skipped`인지 독립 검증하고 알 수 없는 축 값은 실패시킨다.
10. `tests/issue_2293_chart_png_text.rs`가 어떤 CI job에서도 실행되지 않던 기존 누락은
    #4040으로 분리하고 Stage 4 영향축 활성화의 blocker로 취급하지 않는다.

## Stage 5A — 보안 check 재사용과 Rust no-build shadow

1. `codeqlResult`가 고른 PR CodeQL workflow run은 기존처럼 event, base branch, head repository,
   head branch와 candidate SHA를 모두 검증한다.
2. 그 candidate SHA에 대해 check-runs를 조회하고 app slug `github-advanced-security`, name `CodeQL`,
   동일 `head_sha`인 단일 policy check를 식별한다. `started_at`이 없거나 현재 workflow run attempt보다
   이르면 현재 check가 없는 것으로 취급해 이전 attempt 결과를 재사용하지 않는다.
3. 세 `Analyze (...)` job은 언어별로 모두 성공해야 한다. 별도 GHAS check가 없으면 `missing`, 완료 전이면
   `pending`, conclusion이 `success`가 아니면 `failed`로 닫는다. 이 check는 실측상 첫 언어 분석에서
   종결되므로 뒤에 도착한 언어의 policy 결과까지 보증한다고 해석하지 않는다.
4. 기존 세 언어 blocking matrix와 Rust stable toolchain·cache·`cargo build`는 비교 기준선으로 유지한다.
5. PR non-fast-pass 전용 `Rust no-build shadow`를 추가한다. 같은 Rust toolchain을 설치하되 cache와
   `cargo build`는 생략하고 CodeQL init에 `languages: rust`, `build-mode: none`을 지정한다.
6. shadow analyze는 `upload: never`, 고유 output directory를 쓰며 SARIF는 pinned
   `actions/upload-artifact`로만 보존한다. 따라서 code scanning 결과와 required check identity를
   오염시키지 않는다.
7. 정적 workflow 계약 테스트로 보안 check가 실패한 경우와 blocking Rust lane 보존, shadow 격리를
   고정한 뒤 YAML·classifier 인접 회귀를 focused 검증한다.

### Stage 5A 1차 canary 판정

- PR #4341의 CodeQL run `31311707469`에서 blocking Rust 704초, no-build shadow 658초로 shadow가
  46초(6.5%) 빨랐다. 그러나 실제 analyze 단계는 576초와 585초로 shadow가 9초 느렸고, 차이는 주로
  blocking의 cargo cache 복원 13초와 사전 build 49초에서 나왔다.
- 두 lane의 성공 추출 파일은 1,097개로 같았지만 오류 파일은 blocking 7개, shadow 3개로 달랐다.
  동일 CLI 2.26.2와 raw diagnostic 2건을 사용했어도 database가 완전히 동등하다고 판정할 수 없다.
- shadow raw SARIF artifact에는 32개 결과와 fingerprint가 있었지만 blocking raw SARIF는 artifact로
  남기지 않았다. Code Scanning analysis API의 server-processed 결과는 PR baseline 반영 뒤 0건이라
  raw result·fingerprint 비교를 대신할 수 없다.
- shadow job은 `security-events` 권한이 없어 CodeQL Action feature API를 읽지 못했다는 annotation을
  남겼다. 따라서 blocking과 동일 권한이라는 A/B 전제가 충족되지 않았다.
- 결론은 `build-mode: none` 활성화 보류다. 다음 canary는 blocking raw Rust SARIF를 artifact로 남기고,
  shadow 권한을 blocking과 같게 맞춘 뒤, 가능하면 기본 build mode에서 cargo prebuild만 제거해 변수를
  하나로 제한한다.

### Stage 5A 보정 canary

- blocking matrix의 기본 build mode, Rust cache·수동 `cargo build`, Code Scanning upload는 그대로 두고
  CodeQL CLI SARIF 출력 `rust-blocking-results`를 7일 artifact로 추가한다.
- shadow도 기본 build mode와 `security-events: write`, `contents: read`를 사용한다. cache·수동
  `cargo build`만 생략하고 `upload: never`를 유지해 Code Scanning 결과를 만들지 않는다.
- shadow check·artifact 이름을 `Rust no-prebuild shadow`와 `rust-no-prebuild-sarif-*`로 바꿔 첫
  `build-mode: none` 측정과 구별한다.
- 원격에서는 두 raw SARIF의 result fingerprint·artifact URI와 추출 성공·오류 수가 같고 feature API
  권한 annotation이 사라지는지 확인한 뒤 활성화 여부를 판정한다.

### Stage 5A 보정 canary 판정

- candidate `484f6a3286dfd71b61809b95374a0fce31f8d8e9`, CodeQL run `31313096097`의 모든
  workflow job과 GHAS `CodeQL` check가 성공했다. blocking·shadow check annotation은 모두 0건이다.
- blocking Rust는 701초, no-prebuild shadow는 642초로 59초(8.4%) 단축됐다. analyze는 각각
  582초와 579초여서 차이 3초이고, 절감분은 cache 복원 10초와 수동 `cargo build` 50초에 대응한다.
- 두 raw SARIF의 CodeQL CLI 2.26.2, tool metadata, config, 32개 전체 result object와 partial
  fingerprint가 완전히 같다. 규칙별 결과도 hard-coded cryptographic value 31건, weak cryptographic
  algorithm 1건으로 같다.
- 성공 추출은 1,097파일, unresolved macro는 63건으로 같다. blocking에만
  `target/debug/build/serde*` 생성 파일 4개가 추가됐고 네 파일 모두 semantic analyzer unavailable
  warning이라 유효한 소스 coverage나 alert를 늘리지 않았다.
- 따라서 기본 build mode와 내부 `autobuild.sh`를 유지한 채 수동 cargo cache·prebuild를 제거하는 gate는
  통과다. `build-mode: none`은 1차 canary가 동등성을 증명하지 못했으므로 활성화하지 않는다.
- PR #4341의 최종 형태에서는 blocking `Analyze (rust)` check identity를 유지하면서 cache restore/save와
  수동 `cargo build`, 측정용 shadow·raw artifact를 제거하고 최종 CI를 다시 확인한다.

### Stage 5A 최종 전환

- 기존 `analyze` matrix와 `Analyze (rust)` check identity, `security-events: write`, 기본 build mode,
  stable Rust toolchain, 정상 Code Scanning upload를 유지한다.
- Rust 전용 cargo cache restore/save와 수동 `cargo build`를 제거한다. CodeQL 내부 `autobuild.sh`는
  기본 analyze 동작으로 유지된다.
- canary 측정용 `rust-no-prebuild-shadow` job과 blocking·shadow raw SARIF output·artifact를 제거한다.
- 최종 계약은 수동 cache·build와 측정 요소가 다시 들어오면 실패하도록 고정했다. TDD RED 2건을 확인한
  뒤 Stage 5A 6/6, 연관 Python workflow 계약 74/74, classifier 28/28, `actionlint`,
  `git diff --check`가 통과했다.

### Stage 5A 최종 CI 판정

- candidate `c2674bd336a26448d1673f7f70389cb8fc2a0ce8`의 CodeQL run `31314188222`와 CI run
  `31314188326`이 모두 성공했다. PR은 `MERGEABLE / CLEAN`이고 실패 check는 0건이다.
- 최종 `Analyze (rust)` job은 542초, analyze 단계는 480초다. 보정 blocking 701초보다 159초,
  같은 기본 build mode의 보정 shadow 642초보다 100초 짧지만 analyze 자체도 99초 변동했으므로 전부를
  구현 효과로 귀속하지 않는다. 같은 run A/B에서 확인한 보수적 절감은 59초(8.4%)다.
- CodeQL CLI 2.26.2, `database trace-command --index-traceless-dbs`, Rust 내부 `autobuild.sh`가 유지됐다.
  `Analyze (rust)`와 GHAS `CodeQL` annotation은 0건이고 임시 Actions artifact도 0개다.
- Rust Code Scanning analysis `1591906480`은 25개 규칙, 결과 0건으로 최종 처리됐다. 세 Analyze job과
  별도 GHAS `CodeQL` check가 모두 성공했으므로 Stage 5A 코드·CI gate는 통과다.
- PR #4341 제목·본문을 최종 no-prebuild 동작과 canary 근거로 보정했다. 2026-08-09 당시에는 Draft
  상태를 유지하고 이후 review 요청을 작업지시자 gate로 남겼다.

### PR #4341 self-review 보정

- 최신 `upstream/devel` `0664e6568e9b`을 병합하고 `.github/workflows/ci.yml`의 단일 충돌에서
  CodeQL 계약 테스트와 devel의 Docker·release·setup 계약 테스트를 모두 보존했다.
- GHAS `CodeQL` check는 실측상 첫 언어 분석에서 종결되고 뒤에 도착한 언어 분석으로 갱신되지 않는다는
  범위를 코드 주석·계획·작업 기록에 명시했다. 세 Analyze job 성공은 계속 독립적으로 요구한다.
- check-run에 없는 `created_at` fallback과 유한값처럼 보이는 `Date.parse(0)`, 앞선 filter 때문에 도달할
  수 없던 identity mismatch 분기를 제거했다. `started_at` 누락·이전 attempt는 현재 check 부재로 처리해
  기존 fail-closed 결과를 유지한다.
- 작업 단계명이던 `test_codeql_stage5a_workflow.py`는 장기 계약 이름인 `test_codeql_workflow.py`로
  바꾸고 CI·wiring 참조를 함께 갱신했다. 실제 GHAS check가 Python 뒤, JavaScript/TypeScript·Rust보다
  먼저 끝난 순서를 mock에 반영했다.
- `python3 -m unittest` 연관 workflow 계약 10개 파일은 86/86, classifier는 28/28 통과했다.
  `actionlint .github/workflows/ci.yml .github/workflows/codeql.yml`과 `git diff --check`도 통과했다.
- workflow 실행 경로와 최신 devel merge가 포함되므로 보정 head는 fast-pass하지 않고 full CI·CodeQL을
  새로 통과해야 한다.

## Stage 5B 이후

- Stage 5A 원격 canary에서 blocking/shadow SARIF와 duration을 대조하고 required status check 구성을
  repository admin에게 확인한 뒤 CodeQL 언어별 matrix를 활성화한다.
- Stage 3 merge 직후 첫 canary와 Stage 5 canary의 selective/full 결과를 비교한다.
- default-branch controller는 Stage 3~5 진리표가 확정된 뒤 축소 구현하고 정상 릴리즈로 main에 등록한다.
- artifact 재시도는 #3892의 논리 label `slow/1/2/3`별 test archive, archive expected count와 worker run
  count를 함께 다루고, draft 경량화와 별도 PR로 진행한다.
- #3789가 완료되기 전에는 `src/main.rs`의 Render Diff trigger를 좁히지 않는다.

## 집중 검증

```bash
node --test scripts/tests/ci-impact-classifier.test.cjs
python3 -m unittest scripts/tests/test_ci_impact_workflow.py
python3 -m unittest scripts/tests/test_render_diff_workflow.py
npm --prefix rhwp-studio run e2e:renderer-contract
git diff --check
```

Stage 1 검증 결과는 `mydocs/working/task_m100_3790_stage1.md`, Stage 2·2.5 결과는
`mydocs/working/task_m100_3790_stage2.md`, Stage 3 결과는
`mydocs/working/task_m100_3790_stage3.md`, Stage 4 결과는
`mydocs/working/task_m100_3790_stage4.md`에 명령과 종료 상태를 기록한다.
