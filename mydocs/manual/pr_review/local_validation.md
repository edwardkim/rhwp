---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-25
---

# 로컬 사전 검증

이 가이드는 PR의 코드·sample·frontend 변경을 로컬에서 확인하는 절차다. 선택한 검증과 생략 이유를
PR별 review 문서에 남긴다. 같은 checkout·target·Cargo cache를 공유하는 Cargo 계열 명령은
**반드시 하나가 끝난 뒤 다음 명령을 실행**한다.

모든 PR review Cargo 실행에는 CARGO_INCREMENTAL=0을 사용한다. 이전 review의 debug incremental 비대화가
검증 시간과 디스크 상태를 왜곡하지 않게 하기 위함이다.

Cargo 검증을 시작하기 전에는 target 하위 directory와 실행 중인 Cargo/Rust 작업을 확인한다. 이전 review의
정확한 전용 target만 [merge 후속 처리](post_merge.md#771-검토-전용-target)의 정리 기준에 따라 처리하며,
shared target/debug, target/release, target/release-test, target/wasm32-unknown-unknown와 다른 작업의
산출물은 삭제 대상으로 가정하지 않는다.

~~~bash
find target -mindepth 1 -maxdepth 1 -type d -exec du -sh {} \;
pgrep -alf '(^|/)(cargo|rustc|wasm-pack)( |$)' || true
~~~

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
| Rust parser/model/CLI | focused test, release-test 전체, fmt, clippy |
| renderer/layout/typeset/WASM | focused test, release-test 전체, Native Skia 3종, wasm-pack build, 시각 증적 |
| rhwp-studio만 변경 | TypeScript 검사, npm test, 실제 browser 동작 |
| npm/editor public API·transport·type | 아래 package 검증 |
| CI workflow | workflow 구문·변경 조건·최신 GitHub Actions 결과 |
| 기존 golden/baseline/fixture | 관련 focused test, snapshot 결정성, 최신 PR head CI |

신규 CLI 통합 테스트는 `env!("CARGO_BIN_EXE_rhwp")`를 실행 경로로 직접 사용하지 않는다. nextest archive가
런타임에 주입하는 `CARGO_BIN_EXE_rhwp`를 먼저 읽고 컴파일타임 값을 fallback으로 쓰는 기존 `rhwp_bin()`
패턴을 따른다. 근거와 재현 조건은 [#3289 CI 보고서](../../report/task_m100_3289_report.md#멀티러너-거버넌스-운영-규칙)에
기록되어 있다.

일반 Rust 검증 예시는 다음과 같다. 명령은 같은 checkout에서 동시에 실행하지 않는다.

~~~bash
CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
CARGO_INCREMENTAL=0 cargo fmt --check
CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
~~~

renderer 영향 PR의 Native Skia 공식 회귀 범위는 다음 3종이다.

~~~bash
CARGO_INCREMENTAL=0 cargo test --profile release-test --features native-skia skia --lib
CARGO_INCREMENTAL=0 cargo test --profile release-test --features native-skia --test issue_2225_missing_picture_placeholder
CARGO_INCREMENTAL=0 cargo test --profile release-test --features native-skia --test render_p37_direct_pdf_export
CARGO_INCREMENTAL=0 wasm-pack build --target web --out-dir pkg
~~~

## 4.3.1 새 HWP/HWPX fixture의 baseline 등록 — IR sweep + overflow-cell 원장

samples 아래 HWP 또는 HWPX fixture를 새로 추가·교체·이동하면 renderer 변경 여부와 무관하게 PR 생성 또는
draft 해제 전에 **두 baseline 절차**를 수행한다: ① IR field sweep(아래), ② overflow-cell 원장(이 절 말미).

~~~bash
RHWP_IR_SWEEP_DUMP=/tmp/ir_field_sweep_current.tsv \
  CARGO_INCREMENTAL=0 cargo test --profile release-test \
  --test ir_field_sweep_baseline -- --nocapture
diff -u tests/fixtures/ir_field_sweep_baseline.tsv /tmp/ir_field_sweep_current.tsv
~~~

- baseline은 fixture 목록이 아니라 관측된 비영 왕복 발산의 래칫이다. 발산이 없으면 행을 억지로 추가하지 않는다.
- 새 발산은 먼저 RHWP_IR_SWEEP_DETAIL로 원본값·재생성값을 확인한다. 의도된 정규화임을 증명한 경우에만
  lane, 상대경로, 필드경로, 실측 건수를 사전순 TSV 행으로 추가한다.
- 원인을 모르는 증가분을 baseline으로 숨기지 않는다.
- TSV 행을 추가하면 fixture 경로·SHA-256·lane·필드·건수·상세 값 변화·판정 근거를 review 문서에 적는다.
- 마지막으로 CARGO_INCREMENTAL=0 cargo test --profile release-test --tests가 통과해야 한다.

### overflow-cell 원장 (#3668)

새 fixture 에 **쪽 밖 소실 줄**(셀 안 줄의 윗변이 쪽 하단 밖 — `LAYOUT_OVERFLOW_CELL`)이
있으면 `overflow_cell_baseline` 게이트가 "신규 발생"으로 실패한다. 절차는 IR sweep 과 같은
래칫 규약이다:

~~~bash
RHWP_OVERFLOW_CELL_DUMP=/tmp/overflow_cell_current.tsv \
  CARGO_INCREMENTAL=0 cargo test --profile release-test \
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
(cd rhwp-studio && npx tsc --noEmit --skipLibCheck ../npm/editor/index.d.ts)
npm --prefix npm/editor pack --dry-run --json
~~~

iframe RPC 완료 시점이나 기본 옵션이 바뀌면 fresh WASM build와 실행 중인 Vite 또는 새 Vite를 사용해
embed E2E를 추가한다. 기본값 변경은 옵션을 생략한 smoke에서도 loadFile 완료와 페이지 수를 기록한다.

~~~bash
CARGO_INCREMENTAL=0 wasm-pack build --target web --out-dir pkg
VITE_URL=http://127.0.0.1:7700 npm --prefix rhwp-studio run e2e:embed
~~~

대형 복합 변경 또는 승인된 전체 검증은 build, release lib, release-test, Native Skia 3종, fmt,
diff check, clippy, doc test, TypeScript, npm test, wasm-pack을 이 순서로 실행한다.

~~~bash
CARGO_INCREMENTAL=0 cargo build --release
CARGO_INCREMENTAL=0 cargo test --release --lib
CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
CARGO_INCREMENTAL=0 cargo test --profile release-test --features native-skia skia --lib
CARGO_INCREMENTAL=0 cargo test --profile release-test --features native-skia --test issue_2225_missing_picture_placeholder
CARGO_INCREMENTAL=0 cargo test --profile release-test --features native-skia --test render_p37_direct_pdf_export
CARGO_INCREMENTAL=0 cargo fmt --check
git diff --check
CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
CARGO_INCREMENTAL=0 cargo test --doc
(cd rhwp-studio && npx tsc --noEmit)
npm --prefix rhwp-studio test
CARGO_INCREMENTAL=0 wasm-pack build --target web --out-dir pkg
~~~

각 명령은 앞 명령이 끝난 뒤 실행한다. 실패하면 뒤 명령으로 건너뛰어 전체 통과처럼 기록하지 않는다.

svg_snapshot은 release-test 전체에 포함된다. 렌더 영향 PR에서 golden 실패를 좁히거나 재생성 결정성을
확인할 때만 다음을 추가한다. golden은 원 PR merge 전에 별도 commit으로 반영하고 최신 CI를 다시 확인한다.

~~~bash
CARGO_INCREMENTAL=0 cargo test --test svg_snapshot
UPDATE_GOLDEN=1 CARGO_INCREMENTAL=0 cargo test --test svg_snapshot
CARGO_INCREMENTAL=0 cargo test --test svg_snapshot
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
