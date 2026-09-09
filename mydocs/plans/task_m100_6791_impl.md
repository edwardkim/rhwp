# 구현계획 — Task M100 #6791 공개 기여 검증 범위와 worktree 준비 순서

- Issue: [#6791](https://github.com/edwardkim/rhwp/issues/6791)
- 수행계획: [task_m100_6791.md](task_m100_6791.md)
- 기준 devel: `ff1ce007b428547da74e0d6b7e9a196592c60ff6`
- 작업 브랜치: `codex/6791-contributing-validation`
- 수행계획 승인: 2026-09-06 사용자 “진행해줘”, 기록 commit `a2c00f16d`
- 구현계획 승인: 2026-09-06 사용자 “진행해줘”. 공개 문서 Stage 1 착수 승인.
- Stage 2 승인: 2026-09-06 사용자 “진행해줘”.
- Stage 3 승인: 2026-09-06 사용자 “진행해줘”. clean 실검증·최종 보고·로컬 PR 준비 진행.
- 원격 승인: 2026-09-06 사용자 “진행해줘”. upstream push·Open PR #6810 생성 완료. merge·이슈 close는 별도 승인.
- 현재 상태: Stage 3 및 PR 생성 완료. [Stage 3 보고서](../working/task_m100_6791_stage3.md), [최종 결과](../report/task_m100_6791_report.md)

## 1. 구현 계약

문제의 출처는 baba9811의 [PR #6786](https://github.com/edwardkim/rhwp/pull/6786) 본문
「외부 기여자 검증 절차 확인」이다. source 제출 checkout과 검증용 worktree의 구분을 공개 안내에서
완결하고, 생성 가능 여부를 사람의 maintainer 권한과 혼동하지 않도록 한다.

공개 문서 변경은 `CONTRIBUTING.md`와 `.github/pull_request_template.md` 두 파일로 한정한다.
생성기·Cargo·CI 정책·제품 코드·원 PR #6786은 변경하지 않는다. 단계를 바꿀 때 현재 단계 변경과
보고서를 commit하고 다음 단계로 넘어간다. 구현계획 승인 뒤 Stage 1을 시작하며 단계별 결과 승인을 유지한다.

## 2. Stage 1 — 변경 범위와 frontend 검증 정렬

### CONTRIBUTING.md

상단의 무조건 Rust 명령 목록을 제거하고 `PR 전 체크리스트`의 범위 표로 연결한다. 상단 문구의 요지는
“변경 범위에 해당하는 검증이 하나라도 실패하면 push·PR 생성 전에 수정하고 다시 통과시킨다”로 한다.
Rust 관련 변경에서는 전체 fmt, native·WASM32·workspace Clippy를 유지하며, Studio 단독은 frontend
절차를 따른다고 명시한다.

체크리스트에는 다음 구분을 제시한다.

| 변경 | 공개 안내에 명시할 검증 |
| --- | --- |
| Rust parser/model/CLI source | Rust lint 묶음, 관련 focused 회귀, release-test 전체 integration |
| Rust test/baseline helper | Rust lint 묶음, 관련 focused 회귀·snapshot 결정성, 최신 CI. 테스트만 바뀌어도 lint 생략 불가 |
| Rust renderer/layout/typeset/WASM source | Rust lint와 전체 회귀, Native Skia 3종, fresh WASM·시각 검증을 추가 |
| Studio 단독 | TypeScript, 단위 테스트, production build, 변경 기능의 E2E·실제 브라우저 확인. build 전에 fresh WASM 준비 |
| 혼합 | 각 변경 범위 검증의 합집합. Studio 변경이 Rust 검증을 대체하지 않음 |
| 기존 fixture/golden/baseline data만 변경 | 관련 focused 회귀·snapshot 결정성. Rust helper도 수정하면 Rust lint 추가 |
| 문서만 변경 | diff·링크·내용 정합. 문서에서 안내하는 실행 절차를 바꾸면 해당 절차도 직접 확인 |

Cargo.toml·Cargo.lock·Rust toolchain·빌드 설정을 함께 바꾸면 Studio 단독으로 분류하지 않도록 적는다.
표는 대표 변경 범위의 로컬 검증 계약이며 CI job skip을 보장하는 표가 아니다. 복합적인 설정·workflow
변경을 문서/Studio 단독으로 축소하지 않는다.

frontend 절의 첫 문장에서 Studio 단독과 Rust/WASM source가 섞인 변경을 구분한다. 기존 명령을 유지하면서
실행 순서를 다음과 같이 연결한다.

1. Node/npm·Rust toolchain·wasm-pack 준비와 `npm --prefix rhwp-studio ci`.
2. 기존 `scripts/wasm-pack-locked.sh` 또는 Windows wrapper로 해당 commit의 `pkg/`를 새로 준비.
3. TypeScript 검사 → Studio 단위 테스트 → production build.
4. 변경 기능의 E2E와 실제 브라우저 동작 확인. 새 E2E는 package script·MANIFEST도 확인.

`npm/editor`의 기존 package·embed 검사, Docker 표준 경로와 native 진단 경로 구분은 유지한다.
WASM package를 만드는 단계와 Rust fmt·Clippy·전체 회귀를 같은 검증으로 표현하지 않는다.

### Stage 1 종료 조건

- 상단·범위 표·frontend 절에서 Studio 단독을 Rust 전체 lint 대상으로 잘못 안내하지 않는다.
- Rust source 및 test/baseline helper의 세 Clippy와 fmt 의무가 남는다.
- 내부 review 문서를 읽어야만 기본 검증 범위를 알 수 있는 문장을 공개 절차로 대체한다.
- 변경 파일 상대 링크와 `git diff --check`, 문구 정합 확인 뒤
  `mydocs/working/task_m100_6791_stage1.md`와 함께 commit한다.

## 3. Stage 2 — 원본 commit·검증 worktree·push 연결

### CONTRIBUTING.md 공개 Rust 절차

PR 전 체크리스트 아래에 `Rust 검증 worktree 준비와 실행` 절을 만든다. 기여자 본인이 생성하는 검증용
worktree임을 명시한다. PR이 이미 열렸거나 maintainer 권한이 있어야만 만들 수 있는 작업공간이 아니다.

먼저 변경 파일을 source branch에서 명시적으로 stage·commit한 뒤 clean 상태를 확인하도록 안내한다.
그 다음 다음 형태의 명령을 제공한다. 아래는 구현 시 사용할 명령 구조이며 실제 공개 문구는 Stage 2에서
작성한다. 디렉터리와 조건부 검사 구분을 각 코드 블록 앞에서 설명한다.

```bash
# 원본을 commit한 source checkout에서 실행. 같은 경로가 이미 있으면 새 경로를 사용한다.
git status --short
git rev-parse HEAD
git worktree add --detach ../rhwp-rust-review HEAD
cd ../rhwp-rust-review

# 검증 worktree에서 한 명령이 끝난 뒤 다음 명령을 실행한다.
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown \
  --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
```

실제 공개 예제는 중간 명령 실패 시 뒤 명령을 성공 결과처럼 이어서 집계하지 않도록 중단 동작을
분명하게 만든다. POSIX 셸 예제임을 밝히고, toolchain·rustfmt·clippy·wasm32 target 및 nextest 설치
조건을 먼저 설명한다. 고정 thread 수나 특정 사람의 절대 경로는 사용하지 않는다.

조건부 후속 명령은 범위별로 분리한다.

```bash
# Rust source 내부 #[cfg(test)]를 변경한 경우(root src와 crates/*/src 모두 포함)
node scripts/rust-unit-test-tiers.mjs --check

# 해당 회귀 source를 선택할 때: 아래 이름을 실제 source basename으로 바꾼다.
node scripts/run-rust-test.mjs <확장자를_뺀_test_source_이름> -- \
  --cargo-profile release-test --target-dir target/pr-review

# 범위 표에서 전체 integration 회귀가 필요한 경우
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --tests --no-fail-fast

# 적용 검증을 완료한 검증 worktree에서 마지막으로 확인
node scripts/rust-test-suite-manifest.mjs --check
git diff --check
git status --short
```

회귀 test source를 추가·변경할 때 필요한 기존 생성기 계약 테스트의 적용 범위도 표시한다. generated suite
번호를 고정한 `cargo test --test regression_suite_001` 형태를 공개 예제로 사용하지 않는다.

검사 완료 후 원본 checkout으로 돌아가 HEAD가 검증 SHA와 같은지 확인하고 push하도록 Fork 흐름에
commit → 별도 worktree 검증 → 원본 SHA 재확인 → push 순서를 넣는다. 검증이 끝나지 않은 로컬
commit은 만들 수 있지만, 필수 검증 실패 상태의 commit을 제출할 수 있다는 뜻은 아니다.

### 실패·원본 보정·생성물 처리

- `does not exist`와 generated suite 경로가 나오면 준비되지 않아 실패한 검사로 기록한다. 이 메시지를
  실제 포맷 diff나 포맷 통과로 바꾸지 않는다. 검증 worktree에서 prepare 후 같은 check를 재실행한다.
- 실제 fmt diff가 있으면 변경 파일의 포맷을 고쳐 source branch에 보정 commit을 만든다. review worktree에서
  `cargo fmt --all`로 진단·수정한 경우 tracked diff를 확인하고 필요한 원본 변경만 source branch에 반영한다.
  검증 공간의 dirty 파일만 통과시킨 결과를 기존 source HEAD의 통과로 기록하지 않는다.
- 범위 밖 대량 포맷 정규화는 별도 이슈·브랜치로 분리한다. “전체 fmt 실행 금지”와 “전체 포맷 변경을
  기능 PR에 포함하지 말라”를 구분하고 기존 포맷 정책의 목적을 유지한다.
- 새 source commit에서 준비·검증을 다시 시작한다. 기존 worktree를 강제 reset하거나 다른 작업의 경로를
  덮어쓰는 명령을 예제로 넣지 않는다.
- `tests/generated/`, `tests/suites/manifest.json`은 ignored 검증 산출물이며 stage하지 않는다. 독립 PR 간
  공통 harness 충돌 방지 이유를 유지한다. 일반 `git restore`로 ignored 파일이 지워진다고 설명하지 않는다.
- 기본 prepare는 Cargo.toml을 바꾸지 않는다. `--sync-cargo-targets`·rebalance·정책 변경은 기존 maintainer
  전용 작업과 구분한다. 원본 checkout에서 파생 파일을 생성·등록하는 제출 절차는 만들지 않는다.

### 연관 문단과 PR 템플릿

- `처음 참여하시나요`의 clean clone 직후 전체 cargo test를 새 공개 준비 절차로 연결한다.
- 회귀 가이드의 신규 파일 관례를 실제 `tests/cases/issue_...rs`와 일치시킨다. 기여자 자신의 별도
  worktree가 허용된다는 설명을 재사용한다.
- `포맷 정책`과 `코드 스타일`의 축약 lint 명령은 새 Rust 절차를 가리키게 한다. 한컴 PDF 비교 절의
  검증은 렌더링 영향 범위에 적용한다고 연결하고 일반 Studio 문구와 혼동하지 않게 한다.
- PR 템플릿은 변경 범위, 검증한 SHA, 실제 명령·결과, 해당 없음 사유를 적도록 한다. 무조건
  `cargo test`·native Clippy 하나만 체크하는 항목을 Rust 검증 절 링크와 범위별 항목으로 바꾼다.
- 실패·미실행·해당 없음은 PASS와 구분한다. Studio/혼합·관련 E2E·generated 미제출 항목을 유지한다.
- 원 PR #6786의 문제 제기 출처는 이슈·계획·단계/최종 기록에 보존한다.

### Stage 2 종료 조건

두 공개 문서의 scope·실행 장소·원본 SHA·실패 처리·생성물 미제출 설명이 일치해야 한다. 내부 문서의
해석을 외부 기여자의 의무로 남기지 않는다. 상대 링크·새 anchor·`git diff --check` 확인 후
`mydocs/working/task_m100_6791_stage2.md`와 함께 commit한다.

## 4. Stage 3 — 최종 공개 절차 검증과 로컬 PR 준비

Stage 2 commit의 새 clean detached worktree에서 공개한 준비·fmt·manifest 명령을 그대로 실행한다.
이전 기준선 실험 worktree나 #6786 review worktree를 재사용하지 않는다.

| 확인 | 판정 기준 |
| --- | --- |
| 검증 SHA·초기 상태 | Stage 2 HEAD와 일치, tracked/untracked 변경 없음, generated suite 없음 |
| 공개 prepare → fmt check → manifest check | 순차 실행 exit 0, 검사별 명령·SHA·결과 기록 |
| tracked Rust·Cargo 불변 | 모든 tracked `.rs`와 root Cargo.toml·Cargo.lock의 전후 SHA-256 동일 |
| 생성물 | ignored이며 staged 목록과 PR diff에 포함되지 않음 |
| 링크 | 변경 Markdown을 지정한 `check_markdown_links.py` exit 0, anchor는 실제 제목과 별도 대조 |
| 범위 | 공개 2파일 + #6791 계획·단계·보고 문서만 변경. 코드·CI·Cargo 변경 없음 |
| 최종 diff | `git diff --check` 통과, 커밋 후 clean |

이 문서 작업의 실제 실행은 위 준비·포맷·manifest 및 문서 검증이다. 전체 Rust build·Clippy·nextest와
Studio build는 반복하지 않는다. 공개 문서에 안내하는 그 명령들은 기존 CI·권위 문서와 대조하며
실행하지 않은 항목을 PASS로 쓰지 않는다. renderer 추가 검증을 간략히 옮기는 경우에도 기존 3종·순서를
누락하거나 별도 검증을 새로 면제하지 않는다.

2026-09-06 기존 CI classifier v7에 변경 경로를 넣어 확인했다.

- #6786의 Studio 5개 경로: `rust_required=false`, `frontend_mode=package`.
- 이 작업의 공개 2파일 경로: `.github/pull_request_template.md` 때문에
  `classification_status=full`, `reason=fail-closed:workflow-contract`, `rust_required=true`.

따라서 이 문서 PR의 GitHub CI가 Rust를 skip한다고 주장하지 않는다. 이 결과는 현재 분류 정책의 관측이며
CI 변경 요청이 아니다. 로컬 문서 검증의 완료와 PR 생성 이후 GitHub CI 완료를 분리한다.

단계 결과는 `mydocs/working/task_m100_6791_stage3.md`, 최종 결과·PR 본문 초안·실제 push/생성 명령은
`mydocs/report/task_m100_6791_report.md`에 둔다. 로컬 commit을 마친 다음 원격 조치 승인을 요청한다.
PR 번호 기반 review·오늘할일은 번호가 확정된 뒤 collaborator self 경로로 작성한다.

## 5. 승인 대상

수행계획의 범위를 두 공개 파일 안에서 구체화했다. 2026-09-06 사용자가 위 파일별 변경과 Stage 1 착수를
승인했다. Stage 1 완료 보고·commit 뒤 다음 단계 승인 경계를 유지한다. remote push·PR 생성·merge·이슈
close는 포함하지 않는다. 구현계획 승인 전에는 공개 가이드·템플릿을 수정하지 않았다.
