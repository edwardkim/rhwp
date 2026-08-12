---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# 로컬 사전 검증

이 가이드는 PR의 코드·sample·frontend 변경을 로컬에서 확인하는 절차다. 선택한 검증과 생략 이유를
PR별 review 문서에 남긴다. 같은 checkout·target·Cargo cache를 공유하는 Cargo 계열 명령은
**반드시 하나가 끝난 뒤 다음 명령을 실행**한다.

모든 PR review Cargo 실행은 기본 증분 빌드를 사용한다. 전체 회귀는 host마다 고정한
`target/pr-review`를 재사용해 이전 review의 debug/release 산출물과 분리한다. Cargo가 소스·feature·compiler
변경을 판별해 필요한 unit만 다시 빌드한다.

`target/pr-review`는 **이동하거나 이름을 바꾸지 않는다**. 일부 통합 테스트의 `CARGO_BIN_EXE_*` fallback은
컴파일 당시 절대 target 경로를 가질 수 있어, 빌드 뒤 directory를 옮기면 실행 파일을 찾지 못한다. 최초
생성부터 최종 경로를 지정하고, 다음 review도 같은 경로를 사용한다.

Cargo 검증을 시작하기 전에는 target 하위 directory와 실행 중인 Cargo/Rust 작업을 확인한다.
`target/pr-review`와 shared target/debug, target/release, target/release-test,
target/wasm32-unknown-unknown, 다른 작업의 산출물은 삭제 대상으로 가정하지 않는다.

~~~bash
find target -mindepth 1 -maxdepth 1 -type d -exec du -sh {} \;
pgrep -alf '(^|/)(cargo|rustc|wasm-pack)( |$)' || true
~~~

### 고정 review target과 실행 환경

전체 Rust 회귀의 기본 명령은 다음과 같다. 같은 `target/pr-review`를 사용하는 Cargo 계열 명령은 반드시
앞 명령의 종료를 확인한 뒤 실행한다.

~~~bash
cargo nextest run \
  --cargo-profile release-test \
  --target-dir target/pr-review \
  --tests --test-threads 12 --no-fail-fast
~~~

### 시각 대조용 최신 바이너리 준비는 별도다

renderer/layout 변경을 한컴 기준 PDF와 비교할 때는 비교 하네스가 수정 후 바이너리를 실행하도록 다음
명령을 먼저 쓸 수 있다.

~~~bash
cargo build --profile release-test --target-dir target/pr-review
RHWP_BIN=target/pr-review/release-test/rhwp \
  venv/bin/python tools/fidelity_compare/fidelity_compare.py <키> <시작쪽> <끝쪽> \
  --out-dir /tmp/rhwp-fidelity-<키>
~~~

`cargo build`는 **컴파일 전용 준비 단계**이며 테스트를 실행하지 않는다. 시각 보정 중에는 이 명령으로
빠르게 최신 SVG를 확인하되, code head가 바뀐 뒤 PR 검증을 완료하려면 위의 전체 `cargo nextest run
... --tests --no-fail-fast`를 다시 성공시켜야 한다. 하네스 옵션과 기준 PDF provenance는
[`tools/fidelity_compare/README.md`](../../../tools/fidelity_compare/README.md)를 따른다.

2026-08-09 Linux 검증 호스트(`ubuntu-ted`, Intel Xeon E5640 16 vCPU, RAM 15 GiB)에서 이 명령의 fixed
target cold run은 build 포함 17분 42초였다. 같은 target을 그대로 쓴 warm run은 compile 0.96초·전체 8분
27초였고, 최신 priority 설정을 포함한 재검증은 compile 0.75초·5,470/5,470 통과·전체 7분 56초였다.
test 자체의 실행 시간은 host 부하에 따라 달라지므로, 이 수치는 재컴파일 제거 효과와 해당 시점의 측정값을
분리해 읽는다.

macOS도 POSIX 명령은 동일하다. 다만 `--test-threads 12`는 12 이상 논리 CPU와 충분한 RAM이 있는 host의
측정값이다. CPU·RAM이 더 작은 host에서는 `sysctl -n hw.ncpu`와 `sysctl -n hw.memsize`를 확인하고 thread
수를 논리 CPU 이하로 낮춘다.

Windows는 cargo-nextest가 설치되어 있고 PowerShell 또는 cmd에서 Windows 경로만 일관되게 사용하면 같은
방식으로 가능하다. 2026-08-09 `win10-ted`의 cmd 환경(4 logical CPU, RAM 8 GiB)에서
`cargo-nextest 0.9.140`과 `target\\pr-review`를 실제로 검증했다. 긴
`overflow_cell_baseline` 선택 실행은 cold build 포함 18분 55초(build 12분 27초, test 363.036초), 같은
target을 재사용한 warm 실행은 6분 11초(build 2.74초, test 359.563초)로 통과했다. Windows 전체 `--tests`
실행 명령도 아래와 같지만, 이 확인에서는 target 재사용을 직접 검증할 수 있는 장시간 baseline만 실행했다.
이 host에서는 4 thread를 상한으로 쓴다.

~~~powershell
Set-Location 'C:\\Users\\admin\\Desktop\\rhwp\\rhwp'
$env:CARGO_INCREMENTAL = '0'
cargo nextest run `
  --cargo-profile release-test `
  --target-dir target/pr-review `
  --tests --test-threads 4 --no-fail-fast
~~~

PowerShell의 `target/pr-review`는 Windows에서 정상 경로로 해석된다. WSL 경로와 Windows 경로, 또는 서로
다른 shell의 환경 변수 문법을 같은 명령에 섞지 않는다.

### 장시간 baseline의 조기 실행

`overflow_cell_baseline::overflow_cell_lines_do_not_grow`는 samples 전수를 자체 worker로 검사해 60초를
넘길 수 있다. `.config/nextest.toml`은 이 binary에 `priority = 100`을 적용한다. nextest는 개별 테스트를
별도 프로세스로 스케줄하므로, 동일한 nextest run의 시작 시점에 이 long-running baseline을 먼저 배치하면서도
별도 Cargo 프로세스·별도 target을 병렬로 열지 않는다. `SLOW` 행은 시작 로그가 아니라 60초 경과 알림이므로
출력 순서만으로 시작 순서를 판단하지 않는다. 2026-08-09 전체 run에서 이 baseline은 시작 뒤 다른 3,923개
테스트 완료와 겹쳐 실행됐고, 최종 472.343초에 통과했다.

이 설정은 OS thread를 추가하는 방식이 아니라 nextest의 test-process 우선순위 lane이다. 해당 baseline은
내부 worker를 이미 사용하므로, host마다 고정 `threads-required`를 강제하면 작은 Windows host나 CI runner의
전체 병렬도를 오히려 낮출 수 있다. 새로 60초 이상으로 반복 확인된 baseline은 실행 시간 근거와 함께 같은
override에 추가한다. 임의의 모든 느린 테스트를 정적 목록으로 넣지 않는다.

## 4.1 PR branch fetch

~~~bash
git fetch upstream pull/N/head:local/prN
~~~

## 4.1.1 devel 정합 가시성 검토 branch

사용자가 터미널·VS Code에서 진행을 보거나 외부 PR을 실제 merge 대신 누적 체리픽으로 검토하면,
PR head를 기본 작업트리에 바로 checkout하지 않는다. 먼저 clean한 기본 작업트리에서 현재 PR head를
포함한 가시성 branch를 만든다. 시작 전에 `upstream/devel`이 PR head의 조상인지 확인해, VS Code 그래프에서
`devel` 위의 contributor 변경과 이후 메인터너 보정이 함께 보이게 한다.

~~~bash
git status --short
git fetch upstream devel pull/N/head:refs/remotes/upstream/prN-head
git merge-base --is-ancestor upstream/devel upstream/prN-head
git switch -c review/<contributor>-<yyyymmdd> upstream/prN-head
git status --short --branch
~~~

- 사용자 또는 다른 도구의 변경이 있으면 중단하고 보고한다.
- 시작·적용·conflict·검증 상태 보고에 검토 branch명, 기준 devel SHA, 원 PR 번호와 적용 SHA를 적는다.
- 검토 산출물과 메인터너 보정은 이 branch에만 연속해서 쌓고, devel에는 직접 commit하지 않는다. 보정 단계가
  시작됐다고 `review/prN-maintainer` 같은 두 번째 local branch를 만들거나 checkout하지 않는다.
- PR head가 최신 `upstream/devel`의 조상이 아니면 여기서 억지 merge나 rebase를 하지 않는다. 오래된 base
  처리로 전환해 source head를 다시 고정한 뒤, 같은 가시성 branch에서 후속 단계를 이어간다.
- CI 또는 승인 대기 중에는 branch를 유지한다. 종료 뒤 cleanup은 [merge 후속 처리](post_merge.md)의
  branch/worktree/target 정리 게이트를 따른다.

## 4.2 merge simulation

~~~bash
git switch -c prN-merge-test upstream/devel
git merge local/prN --no-commit --no-ff
git status
~~~

devel 위에 PR head를 합친 결과 tree와 conflict를 확인한다. conflict 해결 방침은 작업지시자 결정이
필요하다. 여러 PR의 누적 검토는 [다수 PR과 update branch](multi_pr_update_branch.md)를 따른다.

## 4.3 변경 범위별 기본 검증

| 변경 범위 | 기본 검증 |
| --- | --- |
| mydocs만 변경 | git diff --check, 문서 경로·링크·변경 범위 확인. Cargo 생략 |
| Rust parser/model/CLI | focused test, release-test 전체, fmt, clippy. 단, 4.3.0의 검토 재사용 조건이면 focused test와 GitHub 전체 CI 근거 |
| renderer/layout/typeset/WASM | focused test, release-test 전체, Native Skia 3종, wasm-pack build, 시각 증적. 단, 4.3.0의 검토 재사용 조건이면 focused test, WASM·시각 증적과 GitHub 전체 CI 근거 |
| rhwp-studio만 변경 | TypeScript 검사, npm test, 실제 browser 동작 |
| npm/editor public API·transport·type | 아래 package 검증 |
| CI workflow | workflow 구문·변경 조건·최신 GitHub Actions 결과 |
| 기존 golden/baseline/fixture | 관련 focused test, snapshot 결정성, 최신 PR head CI |

### 4.3.0 PR 검토의 GitHub Full CI 재사용

이 표의 기본 검증은 새 code head를 만들거나 아직 GitHub 전체 검증이 없는 PR에 적용한다. 이미
GitHub Full CI가 완료된 code head를 maintainer가 재검토할 때는
[통합 워크플로우의 3.2.2절](../pr_review_workflow.md#322-녹색-github-code-head의-중복-로컬-전체-회귀-생략)의
네 조건을 모두 만족하면, 해당 code head에서 이미 성공한 `release-test` 전체와 Native Skia 광범위
회귀를 로컬에서 반복하지 않는다.

- 이 예외는 source·test·fixture·workflow·baseline·asset 보정이 전혀 없고, current-base merge가
  clean 또는 `mydocs/` 한정 bridge인 경우에만 사용한다.
- focused test, `git diff --check`, 변경 문서 링크 검사, renderer의 실제 WASM/브라우저 시각 검증은
  생략하지 않는다.
- Docker 등 표준 실행 환경이 없어서 host fallback을 썼다면, 표준 경로를 통과했다고 쓰지 않고
  대체 명령과 환경 부재를 review 문서에 함께 기록한다.
- candidate SHA와 녹색 run, 생략한 명령과 사유를 PR review 문서에 적고, 최신 trailing 문서 head의
  aggregate 상태는 merge 직전에 다시 확인한다.

### 4.3.0.1 컨트리뷰터 PR의 성능 검증 책임

공개 제출 계약은 [CONTRIBUTING의 성능 검증 책임](../../../CONTRIBUTING.md#성능-검증-책임)을 따른다.
컨트리뷰터에게 특정 로컬 장비의 절대 시간, 비공개 코퍼스 또는 메인테이너 전용 브라우저 계측의 통과를
PR 제출 조건으로 요구하지 않는다. 성능 영향 PR에는 가능한 범위의 재현 절차와 동일 환경 전후 관측값을
요청할 수 있지만, 측정 환경이 없다는 이유만으로 접수 자체를 막지 않는다.

merge 판단에서는 다음 경계를 적용한다.

- 공개된 결정적 성능 회귀 테스트와 GitHub required checks는 일반 정확성 gate와 같이 유지한다.
- CI job timeout은 runner hang을 실패로 드러내는 상한이며, 별도 계약이 없는 한 제품 성능 목표나
  컨트리뷰터의 로컬 통과 수치로 해석하지 않는다.
- 환경 의존 성능은 메인테이너가 통제된 환경에서 재검증한다. 절대 시간보다 같은 환경·같은 입력의
  변경 전후 비교와 호출 횟수·repaint·long task 같은 결정적 관측을 우선한다.
- 성능 회귀로 merge를 보류하려면 공개 가능한 sample·명령·환경·관측 결과를 review에 남긴다. 가능하면
  최소 공개 fixture와 자동 래칫을 마련한다. 비공개 자료에서만 발견한 경우 자료 자체나 식별 파일 목록을
  공개하지 않고, 재현 가능한 최소 사례 또는 비식별 집계 근거로 전환한다.
- 심각한 회귀가 확인되더라도 특정 메인테이너 장비나 비공개 자료의 수치 재현을 컨트리뷰터의 단독 수정
  의무로 돌리지 않는다. 메인테이너 보정, 공개 재현 제공 또는 후속 issue 분리 중 처리 경로를 명시한다.

신규 CLI 통합 테스트는 `env!("CARGO_BIN_EXE_rhwp")`를 실행 경로로 직접 사용하지 않는다. nextest archive가
런타임에 주입하는 `CARGO_BIN_EXE_rhwp`를 먼저 읽고 컴파일타임 값을 fallback으로 쓰는 기존 `rhwp_bin()`
패턴을 따른다. 근거와 재현 조건은 [#3289 CI 보고서](../../report/task_m100_3289_report.md#멀티러너-거버넌스-운영-규칙)에
기록되어 있다.

일반 Rust 검증 예시는 다음과 같다. 명령은 같은 checkout에서 동시에 실행하지 않는다.

~~~bash
cargo nextest run \
  --cargo-profile release-test \
  --target-dir target/pr-review \
  --tests --test-threads 12 --no-fail-fast
cargo fmt --check
cargo clippy --all-targets -- -D warnings
~~~

renderer 영향 PR의 Native Skia 공식 회귀 범위는 다음 3종이다.

~~~bash
cargo test --profile release-test --features native-skia skia --lib
cargo test --profile release-test --features native-skia --test issue_2225_missing_picture_placeholder
cargo test --profile release-test --features native-skia --test render_p37_direct_pdf_export
wasm-pack build --target web --out-dir pkg
~~~

## 4.3.1 새 HWP/HWPX fixture의 baseline 등록 — IR sweep + overflow-cell 원장

samples 아래 HWP 또는 HWPX fixture를 새로 추가·교체·이동하면 renderer 변경 여부와 무관하게 PR 생성 또는
draft 해제 전에 **두 baseline 절차**를 수행한다: ① IR field sweep(아래), ② overflow-cell 원장(이 절 말미).

~~~bash
RHWP_IR_SWEEP_DUMP=/tmp/ir_field_sweep_current.tsv \
  cargo test --profile release-test \
  --test ir_field_sweep_baseline -- --nocapture
diff -u tests/fixtures/ir_field_sweep_baseline.tsv /tmp/ir_field_sweep_current.tsv
~~~

- baseline은 fixture 목록이 아니라 관측된 비영 왕복 발산의 래칫이다. 발산이 없으면 행을 억지로 추가하지 않는다.
- 새 발산은 먼저 RHWP_IR_SWEEP_DETAIL로 원본값·재생성값을 확인한다. 의도된 정규화임을 증명한 경우에만
  lane, 상대경로, 필드경로, 실측 건수를 사전순 TSV 행으로 추가한다.
- 원인을 모르는 증가분을 baseline으로 숨기지 않는다.
- TSV 행을 추가하면 fixture 경로·SHA-256·lane·필드·건수·상세 값 변화·판정 근거를 review 문서에 적는다.
- 마지막으로 이 문서 상단의 `release-test` 전체 `cargo nextest run`이 통과해야 한다.

### overflow-cell 원장 (#3668)

새 fixture 에 **쪽 밖 소실 줄**(셀 안 줄의 윗변이 쪽 하단 밖 — `LAYOUT_OVERFLOW_CELL`)이
있으면 `overflow_cell_baseline` 게이트가 "신규 발생"으로 실패한다. 절차는 IR sweep 과 같은
래칫 규약이다:

~~~bash
RHWP_OVERFLOW_CELL_DUMP=/tmp/overflow_cell_current.tsv \
  cargo test --profile release-test \
  --test overflow_cell_baseline -- --nocapture
diff -u tests/fixtures/overflow_cell_baseline.tsv /tmp/overflow_cell_current.tsv
~~~

- 원장은 0 이 아닌 문서만 `상대경로\t줄수` 사전순으로 기록한다. 0 인 문서는 행을 만들지 않는다.
- **원인 정정이 원칙이다** — 소실 줄은 사용자에게 보이지 않는 콘텐츠(#3236 계열)이므로,
  fixture 가 의도적으로 그 결함을 재현하는 경우(회귀 fixture)에만 행을 추가한다.
  페이지별 발생 위치는 `rhwp export-svg <파일> -o <dir> --json` 의 `overflowCellLines` 로 좁힌다.
- 기존 문서의 수치 **증가**는 렌더 회귀다 — baseline 으로 숨기지 않는다. 감소·해소는
  diff 로 확인한 뒤 래칫을 조인다(행 갱신·삭제).
- 행을 추가·갱신하면 문서 경로·줄수·판정 근거를 review 문서에 적는다.

## 4.3.2 @rhwp/editor package 검증

npm/editor의 public API, transport, index.d.ts, README 또는 package manifest 변경은 Studio test만으로 끝내지 않는다.

~~~bash
npm --prefix npm/editor test
node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs
(cd rhwp-studio && npx tsc --ignoreConfig --noEmit --skipLibCheck ../npm/editor/index.d.ts)
(cd npm/editor && npm pack --dry-run --json)
~~~

iframe RPC 완료 시점이나 기본 옵션이 바뀌면 fresh WASM build와 실행 중인 Vite 또는 새 Vite를 사용해
embed E2E를 추가한다. 기본값 변경은 옵션을 생략한 smoke에서도 loadFile 완료와 페이지 수를 기록한다.

~~~bash
wasm-pack build --target web --out-dir pkg
VITE_URL=http://127.0.0.1:7700 npm --prefix rhwp-studio run e2e:embed
~~~

대형 복합 변경 또는 승인된 전체 검증은 build, release lib, release-test, Native Skia 3종, fmt,
diff check, clippy, doc test, TypeScript, npm test, wasm-pack을 이 순서로 실행한다.

~~~bash
cargo build --release
cargo test --release --lib
cargo nextest run \
  --cargo-profile release-test \
  --target-dir target/pr-review \
  --tests --test-threads 12 --no-fail-fast
cargo test --profile release-test --features native-skia skia --lib
cargo test --profile release-test --features native-skia --test issue_2225_missing_picture_placeholder
cargo test --profile release-test --features native-skia --test render_p37_direct_pdf_export
cargo fmt --check
git diff --check
cargo clippy --all-targets -- -D warnings
cargo test --doc
(cd rhwp-studio && npx tsc --noEmit)
npm --prefix rhwp-studio test
wasm-pack build --target web --out-dir pkg
~~~

각 명령은 앞 명령이 끝난 뒤 실행한다. 실패하면 뒤 명령으로 건너뛰어 전체 통과처럼 기록하지 않는다.

svg_snapshot은 release-test 전체에 포함된다. 렌더 영향 PR에서 golden 실패를 좁히거나 재생성 결정성을
확인할 때만 다음을 추가한다. golden은 원 PR merge 전에 별도 commit으로 반영하고 최신 CI를 다시 확인한다.

~~~bash
cargo test --test svg_snapshot
UPDATE_GOLDEN=1 cargo test --test svg_snapshot
cargo test --test svg_snapshot
git add tests/golden_svg/
git commit -m "test(svg_snapshot): regenerate golden after #N (...)"
# 작업지시자 push 승인 뒤
git push <PR-head-remote> HEAD:<PR-head-branch>
~~~

시각 검증 판정을 받은 PR은 Cargo 성공 뒤에도 [시각·fixture 증적](visual_fixture_evidence.md)의
대표 page·asset 기록을 완료해야 merge 판단으로 넘어갈 수 있다.

## 4.4 merge simulation 정리

~~~bash
git merge --abort
git fetch upstream devel
git switch devel
git merge --ff-only upstream/devel
git branch -D prN-merge-test
~~~

merge가 시작되지 않았거나 Already up to date면 abort는 생략한다. 이 절은 simulation branch만 정리한다.
fetch branch, review branch, docs-only branch, worktree, 검토 전용 target은 review 종료 뒤
[merge 후속 처리](post_merge.md)의 최종 종료 게이트에서 정리한다.
