# Contributing to rhwp

> **PR·push 전에는 [변경 범위별 체크리스트](#pr-전-체크리스트)를 확인하세요.**
> 해당하는 필수 검증이 하나라도 실패하면 수정·재검증 전에는 `git push` / `gh pr create` 하지 마세요.
> Rust source·test/baseline helper 변경에는 전체 fmt와 native·WASM32·workspace Clippy가 필요합니다.
> Studio 단독 변경은 [프런트엔드 검증](#프런트엔드-변경-검증)을 따르고, 혼합 변경은 두 범위를 모두
> 검증합니다. 파생 suite·manifest는 검증 산출물이며 PR에 포함하지 않습니다.

rhwp에 관심을 가져주셔서 감사합니다!

"모두의 한글"은 이름 그대로 모두의 참여로 완성됩니다. 코드 기여, 버그 리포트, 문서 개선, HWP 샘플 파일 제공 — 어떤 형태든 환영합니다.

## 처음 참여하시나요?

### 1. 프로젝트 체험하기

코드를 보기 전에 먼저 사용해보세요:

- **[온라인 데모](https://edwardkim.github.io/rhwp/)** — 브라우저에서 바로 HWP 파일 열기
- **[VS Code 확장](https://marketplace.visualstudio.com/items?itemName=edwardkim.rhwp-vscode)** — VS Code에서 HWP 미리보기
- **[npm 패키지](https://www.npmjs.com/package/@rhwp/editor)** — 3줄로 HWP 에디터 임베드

### 2. 개발 환경 설정

```bash
# 클론
git clone https://github.com/edwardkim/rhwp.git
cd rhwp
cargo build --locked
```

위 빌드는 컴파일만 수행하며 파생 integration suite가 없어도 실행할 수 있습니다.
테스트·전체 포맷 검사는 [Rust 검증 worktree 준비와 실행](#rust-검증-worktree-준비와-실행)을 따라 별도
작업공간에서 시작하세요. 깨끗한 clone에는 전체 테스트·포맷 검사에 필요한 파생 integration suite가 없습니다.
Studio를 실행하려면 [프런트엔드 변경 검증](#프런트엔드-변경-검증)의 의존성·WASM 준비를 먼저 마친 뒤
그 절의 개발 서버 명령을 사용하세요. 실제 기여는 아래 Fork 흐름을 따릅니다.

### 3. 첫 기여 찾기

- [`good first issue`](https://github.com/edwardkim/rhwp/labels/good%20first%20issue) 라벨이 붙은 이슈
- 렌더링 불일치 제보 (한컴과 비교하여 스크린샷 첨부)
- 문서 오타/개선
- [Discussions](https://github.com/edwardkim/rhwp/discussions)에서 질문/아이디어 제안

### 4. 업스트림에 기여할지 먼저 판단하기

rhwp는 모든 파생 제품을 한 저장소에서 직접 만들지 않습니다. 공통 엔진, 공용 Web/WASM API,
CLI·MCP 계약과 현재 공식 배포 대상의 개선은 rhwp 업스트림에 기여합니다. 특정 운영체제의 데스크톱·
모바일 앱, 사내 뷰어, Google Docs 연계, 조직별 인증·배포·업무 화면은 별도 다운스트림 프로젝트에서
구현하는 것을 기본으로 합니다.

두 범위가 섞여 있다면 제품 구현은 다운스트림에 두고, 여러 프로젝트가 재사용할 결함 수정이나 공용
확장점만 작은 이슈와 PR로 분리해 주세요. 자세한 판단 기준과 공식 배포 범위는
[프로젝트 로드맵의 업스트림과 다운스트림 경계](ROADMAP.md#업스트림과-다운스트림의-경계)를
참고하세요.

## 기여 방법

### 버그 리포트

HWP 파일이 한컴과 다르게 렌더링되면 알려주세요:

1. [이슈 생성](https://github.com/edwardkim/rhwp/issues/new?template=bug_report.md)
2. **한컴 스크린샷** + **rhwp 스크린샷** 비교 첨부
3. 가능하면 HWP 파일 첨부 (개인정보 제거 후)

디버깅 정보를 함께 제공하면 수정이 빨라집니다 (아래 "디버깅 가이드" 참고).

### 코드 기여 — Fork & PR 워크플로우

컨트리뷰터는 **Fork 기반**으로 작업합니다. 저장소에 직접 push할 수 없으며, PR을 통해 코드를 제출합니다.

```
[본인 Fork]                              [edwardkim/rhwp]

1. Fork (GitHub UI)
   edwardkim/rhwp → myid/rhwp

2. Clone + upstream 등록 (최초 1회)
   git clone https://github.com/myid/rhwp.git
   cd rhwp
   git remote add upstream https://github.com/edwardkim/rhwp.git

3. 브랜치 생성 + 작업 — 반드시 최신 upstream/devel 기준
   git fetch upstream
   git switch -c fix/issue-123 upstream/devel
   (코드 수정 + 관련 focused 테스트)

4. 제출할 원본 파일만 stage + commit
   git add <수정한 원본 파일들>
   git commit -m "fix: 변경 설명"
   (생성 harness·manifest·빌드 산출물은 포함하지 않음)

5. 해당 commit 검증
   (PR 전 체크리스트의 변경 범위 적용)
   (Rust 검증은 별도 review worktree에서 suite 준비 후 실행)

6. 검증 SHA와 source branch HEAD가 같은지 확인한 뒤 Push (본인 Fork에)
   git push origin fix/issue-123

7. PR 생성 (GitHub UI)                   ──→ devel 브랜치로 PR
                                              CI 자동 실행 (빌드+테스트+Clippy)
                                              메인테이너 코드 리뷰
                                              승인 후 merge
```

**중요:**
- PR 대상 브랜치는 **`devel`** 입니다 (`main` 아님)
- PR을 생성하면 CI가 자동으로 빌드 + 테스트 + Clippy를 실행합니다
- CI가 통과하지 않으면 merge할 수 없습니다
- 메인테이너의 코드 리뷰 승인 후 merge됩니다
- Issue와 PR은 같은 번호 공간을 쓰며, PR 번호는 **PR 생성이 성공한 시점**에 채번됩니다.
  생성 전에 다음 번호를 예측하거나, 번호만 확보하기 위해 Draft PR을 만들지 마세요.
- 구현과 PR 전 검증을 마친 기여는 일반 Open PR로 제출합니다. Draft는 아직 완료되지 않은 WIP에
  대해 조기 피드백을 요청할 때만 사용합니다.
- **하나의 PR에 여러 fix를 담을 때는 이슈별로 커밋을 분리**해주세요. 여러 수정이 한 커밋에
  섞이면 회귀 추적·선별 반영·리뷰가 어려워져 머지가 지연됩니다.
- **외부 기여자는 `mydocs/orders/YYYYMMDD.md`를 PR에 포함하지 마세요.** 이 파일은 저장소 운영 기록입니다.
  승인된 collaborator self PR의 번호 기반 기록은 아래 역할별 예외를 따릅니다.

### 메인터너 검토 기록과의 구분

외부 기여자의 제출 절차는 이 문서의 **코드 기여**, **PR 전 체크리스트**, **회귀 테스트 가이드**가
전부입니다. 저장소에 함께 있는 다음 문서는 메인터너가 접수·보정·병합 후속 처리를 할 때 쓰는 내부
운영 기록이므로, 외부 기여 PR에 해석하거나 첨부하지 마세요.

- `AGENTS.md` 및 AI 도구별 부트스트랩 파일
- `mydocs/manual/pr_review_workflow.md`와 `mydocs/manual/pr_review/` 하위 문서
- `mydocs/pr/`, `mydocs/pr/archives/`, `mydocs/pr/assets/`, `mydocs/orders/` 하위 파일

특히 `pr_N_review.md`, `pr_N_review_impl.md`, 오늘할일, 메인터너 검토용 비교 이미지와 병합·후속처리
기록은 **메인터너 또는 승인된 collaborator**가 역할별 절차에 따라 작성합니다. 기여자는 재현 명령, 테스트 결과, 공개 가능한 fixture와 필요한
스크린샷을 PR 본문에 적거나 첨부하면 충분합니다. 메인터너가 특정 기록 파일의 추가를 명시적으로
요청한 경우에만 그 요청 범위에서 예외로 합니다.

collaborator 자신의 PR은 작업지시자의 push·PR 생성 승인 후 번호가 확정되면,
[collaborator self 절차](mydocs/manual/pr_review/collaborator_self_merge.md#821-pr-채번과-오늘할일-생성갱신-시점)에
따라 review·오늘할일을 같은 PR의 후속 commit에 포함합니다. 이 운영 예외는 외부 기여자의 제출 의무가 아닙니다.

### Claude·Codex capability 기여

재사용할 Claude 에이전트·Claude Skill·Codex Skill을 추가하거나 변경하기 전에는
[에이전트 capability 카탈로그](mydocs/manual/agent_capability_registry.md)를 읽어 기존 기능과 중복되지
않는지 확인해주세요.

- 같은 사용자 산출물·권위 문서·비범위면 새 기능을 만들지 않고 기존 capability에 어댑터만 추가합니다.
- 새 capability면 전용 Issue를 먼저 만들고 `CAP-<Issue 번호>`를 사용합니다. 로컬 순번을 임의로
  정하지 않습니다.
- 진입점·권위 문서·상태 변경은 같은 PR에서 카탈로그에 반영하고, Codex Skill은 카탈로그의 검증 절에
  따라 `quick_validate.py`를 실행합니다.

### PR 전 체크리스트

변경 파일과 동작을 기준으로 아래에서 해당하는 검증을 모두 선택하세요. Rust 검증을 위한 파생 suite는
원본을 커밋한 뒤 [별도 review worktree에서 준비](#rust-검증-worktree-준비와-실행)합니다.
이 worktree는 기여자 본인이 제출 전 검증을 위해 만들 수 있습니다. source 제출 checkout과 분리하는
절차이지, maintainer만 생성기를 실행할 수 있다는 뜻이 아닙니다.

| 변경 범위 | 제출 전 필수 검증 |
| --- | --- |
| Rust parser/model/CLI source | 아래 Rust lint 묶음, 관련 focused 회귀, `release-test` 전체 integration |
| Rust test/baseline helper | Rust lint 묶음, 관련 focused 회귀와 해당 snapshot 결정성 확인. 테스트만 바꿔도 fmt·세 Clippy를 생략하지 않음 |
| Rust renderer/layout/typeset/WASM source | Rust lint 묶음, 관련 focused 회귀, `release-test` 전체 integration, Native Skia 3종(lib·누락 이미지·직접 PDF 회귀), fresh WASM build와 [시각 검증](#렌더링-pr-자가-검증-도구-한컴-없이-가능) |
| Studio 테스트 전용 (`rhwp-studio/tests/**`만 변경) | [프런트엔드 검증](#프런트엔드-변경-검증)의 unit 경로: CI unit 설정 TypeScript 검사·단위 테스트. WASM·production build 생략 |
| 그 외 Studio 단독 (Rust 검증 입력 변경 없음) | [프런트엔드 검증](#프런트엔드-변경-검증)의 package 경로: fresh dev WASM, TypeScript·단위 테스트·production build, 브라우저 동작 변경 시 관련 E2E·실제 브라우저 확인 |
| `npm/editor` public API·transport·type·package manifest | [프런트엔드 검증](#프런트엔드-변경-검증)의 package 계약 검사, iframe RPC·기본 옵션·WASM 초기화 변경 시 embed E2E |
| Rust와 Studio 등 혼합 변경 | 해당하는 모든 행의 검증. 프런트엔드 검증이 Rust 검증을 대체하지 않음 |
| 기존 fixture/golden/baseline data만 변경 | 관련 focused 회귀와 해당 snapshot 결정성 확인. Rust helper도 바꾸면 Rust lint 묶음 추가 |
| CI workflow·archive label·trusted post-merge reuse | [정본 범위 표와 추가 계약 검사](mydocs/manual/pr_review/local_validation.md#43-변경-범위별-기본-검증), [운영 변경 등급별 검증](mydocs/manual/github_operations.md) |
| 문서만 변경 | 아래 commit 범위 공백 검사, 변경 문서의 링크·내용 정합성. 실행 절차를 바꿨다면 해당 절차의 실제 동작 확인 |

`Cargo.toml`, `Cargo.lock`, Rust toolchain·빌드 설정도 Rust 검증 입력입니다. 이런 파일을 함께 바꾸면
Studio 단독으로 분류하지 않고 Rust lint·관련 회귀와 영향을 받는 프런트엔드 검증을 수행하세요.
이 공개 표는 [로컬 검증 정본 §4.3](mydocs/manual/pr_review/local_validation.md#43-변경-범위별-기본-검증)의
기여자용 안내입니다. 새 sample·fixture 등 표 밖의 변경은 정본에서 해당 범위를 확인하세요. 위 표는 로컬 검증의
범위이며 CI job의 skip을 보장하지 않습니다. 모든 PR은 최신 head의 GitHub required checks를 충족해야 합니다.

검증한 commit SHA와 실제 명령·결과를 PR 본문에 기록하세요. 범위상 해당하지 않아 실행하지 않은 검사는
그 사유를 적고, 실행 실패나 미완료를 PASS로 표시하지 않습니다.

#### 모든 범위의 commit 공백 검사

원본 commit 후 source checkout에서 다음을 실행합니다. `upstream`은 앞서 등록한 원본 저장소 remote입니다.
기준 devel SHA와 제출 HEAD를 결과에 함께 기록하세요. 깨끗한 트리의 `git diff --check`만으로는 이미
commit한 공백 오류·충돌 마커를 검사할 수 없으므로 PR 범위 검사도 수행합니다.

```bash
(
  set -eu
  git fetch upstream devel
  git rev-parse upstream/devel HEAD
  git diff --check upstream/devel...HEAD
  git diff --check
  test -z "$(git status --porcelain)"
)
```

### Rust 검증 worktree 준비와 실행

검증이 필요한 원본 파일을 source branch에서 먼저 commit하세요. **로컬 commit → 그 commit의 별도
worktree 검증 → 같은 commit push** 순서입니다. 필수 검증 실패 상태의 로컬 commit을 만드는 것은 가능하지만
push·PR 생성은 검증을 통과한 뒤에 합니다. 생성 harness·manifest는 commit하지 않습니다.

다음 예제는 macOS/Linux의 Bash·Zsh 또는 Windows Git Bash용입니다. Git, Node.js, Rust/rustup을 설치하고,
저장소 루트에서 `rustup show`로 `rust-toolchain.toml`의 toolchain·rustfmt·Clippy·WASM target을 준비하세요.
Rust 회귀를 실행할 환경에는 nextest도 필요합니다(`cargo nextest --version`으로 확인하고, 없으면
`cargo install cargo-nextest --locked`). PowerShell/cmd에서는 POSIX 변수·줄 연장 문법을 그대로 쓰지 말고
Git Bash에서 아래 준비·검증을 실행하거나 같은 SHA·worktree 순서를 해당 셸 문법으로 적용하세요.

#### 1. 검증 worktree 준비와 포맷 검사

source checkout의 저장소 루트에서 실행합니다. 먼저 위 commit 공백 검사로 최신 devel을 가져오세요.
작업 중 변경이 남아 있으면 원본 commit을 마치세요.
아래 worktree 경로가 이미 있으면 `-rust-review` 접미사를 새 이름으로 바꾸고, 기존 작업공간을 덮어쓰지
마세요. 이후 블록은 **같은 셸에서**, 이 블록이 성공한 뒤 실행합니다. 괄호 안의 `set -eu`는 실패 시 해당
블록을 즉시 중단하며, 어느 블록이든 실패하면 다음 블록·push로 넘어가지 않습니다.

```bash
rhwp_source_dir="$(git rev-parse --show-toplevel)" &&
rhwp_review_sha="$(git rev-parse HEAD)" &&
rhwp_review_base_sha="$(git rev-parse upstream/devel)" &&
rhwp_review_dir="${rhwp_source_dir}-rust-review" &&
rhwp_review_target_dir="${rhwp_source_dir}-review-target" &&
(
  set -eu
  git -C "$rhwp_source_dir" status --short
  test -z "$(git -C "$rhwp_source_dir" status --porcelain)"
  git -C "$rhwp_source_dir" diff --check "$rhwp_review_base_sha...$rhwp_review_sha"
  git -C "$rhwp_source_dir" worktree add --detach "$rhwp_review_dir" "$rhwp_review_sha"
  cd "${rhwp_review_dir:?먼저 1번 준비 블록을 실행하세요}"
  test "$(git rev-parse HEAD)" = "${rhwp_review_sha:?}"
  git rev-parse HEAD
  node scripts/rust-test-suite-manifest.mjs --prepare
  cargo fmt --all -- --check
)
```

기여자 본인도 이 worktree에서 `--prepare`를 실행할 수 있습니다. 기본 명령은 generated suite·manifest만
만들고 root `Cargo.toml`을 바꾸지 않습니다. `tests/generated/regression_suite_*.rs`가 없다는 오류는
검사 준비 부족으로 인한 실패이며, 실제 포맷 diff와 구분해 기록합니다. source checkout의 이 실패를
PASS로 바꾸지 말고 준비된 worktree에서 같은 검사를 다시 실행하세요. 실제 포맷 diff가 나오면 아래
[포맷 정책](#포맷-정책)에 따라 원본을 보정하고 새 commit을 다시 검증합니다.

target은 source checkout 옆의 이 작업 전용 절대 경로에 둡니다. worktree 이름을 바꾸는 재검증에서도
`rhwp_review_target_dir`는 유지해 Cargo 캐시를 재사용합니다. 모든 Cargo `--target-dir`과 host WASM의
`CARGO_TARGET_DIR`가 같은 경로를 가리켜야 합니다. 환경변수만 바꿔도 명시된 `--target-dir`은 바뀌지 않습니다.
캐시는 이전 검증 결과를 재사용하는 근거가 아니며 새 SHA의 필수 검사는 다시 실행합니다.

#### 2. Rust lint

위 표에서 Rust lint가 필요한 변경은 세 Clippy를 모두 통과해야 합니다. 1번의 전체 fmt check와 함께
CI의 Format check·native·WASM cfg·workspace lint에 대응합니다. native Clippy 하나만으로는 충분하지
않습니다. 같은 worktree·target의 Cargo 명령을 동시에 실행하지 않습니다.

```bash
(
  set -eu
  cd "${rhwp_review_dir:?먼저 1번 준비 블록을 실행하세요}"
  test "$(git rev-parse HEAD)" = "${rhwp_review_sha:?}"
  cargo clippy --locked --target-dir "${rhwp_review_target_dir:?}" -- -D warnings
  cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown \
    --target-dir "${rhwp_review_target_dir:?}" -- -D warnings
  cargo build --locked --workspace --target-dir "${rhwp_review_target_dir:?}"
  cargo clippy --locked --workspace --all-targets --target-dir "${rhwp_review_target_dir:?}" -- -D warnings
)
```

#### 3. 해당 정책 검사와 회귀

Rust integration test source를 변경한 경우에는 배정 규칙의 계약 검사도 실행합니다.

```bash
(
  set -eu
  cd "${rhwp_review_dir:?먼저 1번 준비 블록을 실행하세요}"
  test "$(git rev-parse HEAD)" = "${rhwp_review_sha:?}"
  node --test scripts/tests/rust-test-suite-manifest.test.mjs
)
```

`src/**` 또는 `crates/*/src/**`의 `#[cfg(test)]`를 변경한 경우에만 다음 무생성 정책 검사를 추가합니다.

```bash
(
  set -eu
  cd "${rhwp_review_dir:?먼저 1번 준비 블록을 실행하세요}"
  test "$(git rev-parse HEAD)" = "${rhwp_review_sha:?}"
  node scripts/rust-unit-test-tiers.mjs --check
)
```

관련 focused 회귀는 실제 test source의 확장자를 뺀 이름으로 선택합니다. 아래 `TEST_SOURCE_NAME`을 바꿔
실행하세요. wrapper가 현재 suite와 filter를 찾고 `--locked`를 적용하므로 suite 번호를 고정하지 않습니다.

```bash
(
  set -eu
  cd "${rhwp_review_dir:?먼저 1번 준비 블록을 실행하세요}"
  test "$(git rev-parse HEAD)" = "${rhwp_review_sha:?}"
  node scripts/run-rust-test.mjs TEST_SOURCE_NAME -- \
    --cargo-profile release-test --target-dir "${rhwp_review_target_dir:?}"
)
```

범위 표에서 전체 integration 회귀가 필요한 변경은 다음도 실행합니다.

```bash
(
  set -eu
  cd "${rhwp_review_dir:?먼저 1번 준비 블록을 실행하세요}"
  test "$(git rev-parse HEAD)" = "${rhwp_review_sha:?}"
  cargo nextest run --locked \
    --cargo-profile release-test --target-dir "${rhwp_review_target_dir:?}" --tests --no-fail-fast
)
```

- `release-test` 프로필은 PR CI와 같은 기준이며 debug 대비 수 배 빠릅니다.
- nextest는 현재 host에 맞는 기본 동시성을 사용합니다. 기본값을 먼저 쓰고, CPU·메모리·동시 작업을
  확인해 조정할 때만 `--test-threads <현재 환경에 맞는 값>`을 추가하세요. 문서의 고정 수치를 복사하지
  마세요.
- `cargo test --lib` 만으로는 통합 테스트 회귀를 잡지 못합니다 — `--tests` 를 포함해주세요.

Rust renderer/layout/typeset/WASM 변경은 같은 worktree에서 Native Skia 3종을 추가합니다.

```bash
(
  set -eu
  cd "${rhwp_review_dir:?먼저 1번 준비 블록을 실행하세요}"
  test "$(git rev-parse HEAD)" = "${rhwp_review_sha:?}"
  cargo test --locked --profile release-test --target-dir "${rhwp_review_target_dir:?}" --features native-skia --lib
  node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- \
      --cargo-profile release-test --target-dir "${rhwp_review_target_dir:?}" --features native-skia
  node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- \
      --cargo-profile release-test --target-dir "${rhwp_review_target_dir:?}" --features native-skia
)
```

그 뒤 WASM을 준비하고 [시각 검증](#렌더링-pr-자가-검증-도구-한컴-없이-가능)을 진행하세요.
렌더링 변경의 표준 WASM 빌드는 Docker `wasm` 서비스입니다. 검증 worktree에서 아래처럼
최초 환경 파일을 준비하되 기존 파일을 덮어쓰지 않습니다.

```bash
(
  set -eu
  cd "${rhwp_review_dir:?먼저 1번 준비 블록을 실행하세요}"
  test "$(git rev-parse HEAD)" = "${rhwp_review_sha:?}"
  if [ ! -f .env.docker ]; then cp .env.docker.example .env.docker; fi
  docker compose --env-file .env.docker run --rm wasm
)
```

Docker를 사용할 수 없을 때의 진단 경로는 같은 worktree에서
`CARGO_TARGET_DIR="${rhwp_review_target_dir:?}" scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`입니다.
이 경우 Docker 부재·대체 명령을 기록하고, 최적화된 표준 빌드를 통과했다고 쓰지 마세요. Windows native
wrapper는 아래 프런트엔드 절에 있습니다. 같은 commit으로 이미 WASM을 준비했다면 이어지는 frontend
검사·시각 대조를 위해 다시 빌드할 필요는 없습니다.

비교 도구가 최신 실행 파일을 보도록 `cargo build --locked --profile release-test --target-dir "${rhwp_review_target_dir:?}"`로
검증 worktree의 `rhwp` 바이너리를 만들 수 있습니다. 이 빌드는 테스트를 실행하지 않으므로 위 회귀를
대체하지 않습니다. 혼합 변경의 frontend 검사도 마친 뒤 다음 최종 확인으로 넘어갑니다.

#### 4. manifest 확인과 검증한 원본 제출

적용되는 검증을 모두 통과한 뒤 실행합니다. 이 블록의 성공만으로 앞서 실패하거나 실행하지 않은 검사가
통과한 것은 아닙니다. `git status` 확인은 검증 중 tracked 원본에 변경이 생겼는지도 검사합니다.

```bash
(
  set -eu
  cd "${rhwp_review_dir:?먼저 1번 준비 블록을 실행하세요}"
  test "$(git rev-parse HEAD)" = "${rhwp_review_sha:?}"
  node scripts/rust-test-suite-manifest.mjs --check
  git diff --check "${rhwp_review_base_sha:?}...${rhwp_review_sha:?}"
  git diff --check
  git status --short
  test -z "$(git status --porcelain)"
)
```

검증 SHA와 결과를 PR 본문용으로 기록하세요. `tests/suites/suite-policy.json`과
`tests/suites/unit-test-tier-policy.json`은 추적 정책이고, `tests/generated/`·`tests/suites/manifest.json`은
검증 worktree의 ignored 산출물입니다. 생성해 검사할 수 있지만 source checkout에 복사하거나 stage하지
않습니다. 일반 `git restore`는 ignored 파일을 지우지 않습니다. 증적을 보존한 뒤 검증 worktree를
정리할 때도 해당 작업공간만 대상으로 하며, source checkout이나 다른 작업의 산출물을 지우지 마세요.

일반 기여는 새 integration 원본을 `tests/cases/`에만 둡니다. 배정 정책 변경·`--rebalance`와
`--sync-cargo-targets`는 별도 maintainer 작업입니다. 후자는 통합 불가 예외 target registry가 바뀌는
경우의 Cargo marker 블록 동기화에만 사용하며 일반 검증 준비에는 필요하지 않습니다.

**모든 해당 검증이 통과했을 때만** source branch에서 아래를 실행합니다. HEAD가 검증 SHA와 다르거나
미커밋 변경이 있으면 중단됩니다. 수정·추가 commit이 생겼다면 그 commit으로 준비와 검증을 다시 수행하세요.
앞서 저장한 `rhwp_review_sha`를 새 HEAD로 바꾸는 것만으로 검증 결과를 갱신할 수는 없습니다.

```bash
(
  set -eu
  cd "${rhwp_source_dir:?}"
  test -n "$(git branch --show-current)"
  test "$(git rev-parse HEAD)" = "${rhwp_review_sha:?}"
  test -z "$(git status --porcelain)"
  git push -u origin HEAD
)
```

위 명령은 본인 Fork의 `origin`에 현재 작업 branch를 push합니다. 이후 `devel` 대상 PR을 만들고 최신
head의 required checks를 확인하세요.

#### 5. 검증 worktree 정리

증적을 worktree 밖에 보존하고 실행 중인 검사·개발 서버가 없을 때만 정리합니다. 이 명령은 검증 실패 후
보정 파일을 source에 반영한 뒤에도 사용할 수 있습니다. ignored 산출물도 함께 삭제되므로 아래 목록을
먼저 확인하세요. 기록할 원본 변경이 남았거나 Git이 제거를 거부하면 원인을 확인하고 강제 삭제하지 않습니다.

```bash
(
  set -eu
  cd "${rhwp_source_dir:?}"
  test "${rhwp_review_dir:?}" != "$rhwp_source_dir"
  git -C "$rhwp_review_dir" status --short --ignored
  test -z "$(git -C "$rhwp_review_dir" status --porcelain)"
  test "$(git -C "$rhwp_review_dir" rev-parse HEAD)" = "${rhwp_review_sha:?}"
  git worktree remove "$rhwp_review_dir"
)
```

source checkout과 외부 `rhwp_review_target_dir`는 보존됩니다. 보정 commit은 같은 셸에서도 **1번 준비부터**
다시 시작하세요. 정상 제거 뒤에는 `git worktree prune`을 추가로 실행할 필요가 없습니다.

### 프런트엔드 변경 검증

Studio 단독 변경은 아래 프런트엔드 검증을 수행합니다. Rust source·test/baseline helper·Cargo 설정 등
Rust 검증 입력도 바꿨다면 [변경 범위별 체크리스트](#pr-전-체크리스트)의 Rust 검증을 함께 수행하세요.
Studio 단독 변경에 Rust 전체 fmt·Clippy·integration 회귀를 추가로 요구하지는 않습니다.
`npm/editor/`나 WASM과 Studio의 연결 코드 변경은 아래의 해당 package·embed 검사도 확인하세요.

메인터너용 PR review 문서를 읽거나 저장소에 검토 기록을 추가할 필요는 없습니다. PR 본문에 검증한
commit SHA, 실제 명령·결과, 수동 확인한 동작과 사용한 공개 sample을 적어주세요.

모든 frontend 블록의 실행 위치는 **Studio 단독이면 원본 commit의 clean source checkout 루트**,
**혼합 변경이면 Rust를 검사한 동일 SHA의 review worktree 루트**입니다. 혼합 변경은 위 셸에서
`cd "${rhwp_review_dir:?}"`로 이동하고 `git rev-parse HEAD`가 `rhwp_review_sha`와 같은지 확인하세요.
Windows native shell도 동일한 절대 경로로 이동하고 SHA를 확인합니다. 다른 checkout의 `pkg/`를 복사하지 않습니다.
아래부터 끝까지 선택한 트리를 유지하고, 새 셸에서는 실행 경로·SHA 확인을 다시 수행하세요.

먼저 Node.js/npm을 준비하고 선택한 루트에서 원본 commit·clean 상태를 확인한 뒤 의존성을 설치합니다.

```bash
(
  set -eu
  git rev-parse --show-toplevel HEAD
  test -z "$(git status --porcelain)"
  npm --prefix rhwp-studio ci
)
```

#### Studio 테스트 전용

변경 전체가 `rhwp-studio/tests/**` 안에만 있는 경우 CI `frontend-unit-gates`와 같은 검사를 실행합니다.
Rust·production source·package·설정·E2E를 함께 바꿨다면 이 분기를 단독으로 적용하지 않습니다.
이 경로에는 Rust toolchain·WASM package·production build가 필요하지 않습니다.

```bash
(
  set -eu
  (cd rhwp-studio && npx tsc --project tsconfig.ci-unit.json --noEmit)
  npm --prefix rhwp-studio test
)
```

검사 후 아래 최종 SHA·clean 확인과 commit 공백 검사를 마치면 package 경로를 생략할 수 있습니다.

#### Studio package·브라우저 검증

그 외 Studio source·package·브라우저 변경은 같은 선택 루트에서 진행합니다. 저장소의
`rust-toolchain.toml`에 지정된 Rust toolchain·WASM target과 `wasm-pack`을 추가로 준비하세요.

**타입 검사·build·브라우저 검증 전에 해당 commit의 WASM package를 새로 만드세요.** 깨끗한 checkout에는
`pkg/`가 없으며, 다른 commit의 WASM을 복사하면 현재 변경을 검증한 것이 아닙니다. CI의 frontend package
gate와 같이 `--dev`로 fresh WASM을 먼저 만듭니다. Studio production bundle 검사는 아래에서 별도로
실행합니다. 이 준비는 Studio가 사용할 WASM 산출물을 만드는 단계로,
Studio 단독 변경에 Rust 전체 lint·회귀를 요구하는 것과는 구분합니다.

렌더링 출력·성능이나 release WASM을 검증할 때는 Rust 절의 표준 Docker 빌드·시각 검증을 적용합니다.
`--dev` 성공을 최적화된 release WASM의 검증으로 기록하지 않습니다. 동일 SHA로 이미 요구되는 WASM을
준비했다면 package 검사를 위해 재빌드하지 않습니다.

macOS/Linux에서는 raw `wasm-pack build` 대신 아래 wrapper를 사용합니다. `wasm-pack`의 사전 metadata
호출까지 `--locked`로 고정하므로, 검증 과정에서 루트 `Cargo.lock`이 갱신되는 것을 막습니다.

```bash
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --dev
```

혼합 변경에서 host WASM 캐시를 재사용하려면 위 명령의 `target/pr-review` 대신
`"${rhwp_review_target_dir:?}"`를 지정합니다. 반복 실행용 alias도 같은 선택 루트에서만 사용하세요.

```bash
alias rhwp-wasm-build='CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --dev'
rhwp-wasm-build
```

Windows에서는 native wrapper를 사용합니다.

```powershell
$env:CARGO_TARGET_DIR = 'target\pr-review'
.\scripts\wasm-pack-locked.ps1 --target web --out-dir pkg --dev
Remove-Item Env:CARGO_TARGET_DIR
```

`cmd.exe`에서는 아래처럼 `doskey` macro를 현재 세션에 등록할 수 있습니다. macro는 세션 종료 시 사라집니다.

```bat
doskey rhwp-wasm-build=scripts\wasm-pack-locked.cmd --target web --out-dir pkg --dev $*
set "CARGO_TARGET_DIR=target\pr-review"
rhwp-wasm-build
set "CARGO_TARGET_DIR="
```

WASM 준비가 성공한 뒤 Studio의 타입·단위·번들을 순차 확인합니다. 각 명령이 실패하면 원인을 수정하고
다시 실행한 결과를 기록하세요. 생성된 `pkg/`·빌드 산출물은 PR에 포함하지 않습니다.

```bash
(
  set -eu
  (cd rhwp-studio && npx tsc --noEmit)
  npm --prefix rhwp-studio test
  npm --prefix rhwp-studio run build
)
```

사용자 상호작용, Canvas, 선택·입력·저장, bridge, plugin 등 브라우저 동작을 바꿨다면
[`rhwp-studio/e2e/MANIFEST.md`](rhwp-studio/e2e/MANIFEST.md)에서 변경 기능에 대응하는 E2E를 골라 함께
실행합니다. 예를 들어 `e2e/`에 새 회귀를 추가했다면 manifest와 package script도 함께 갱신하고,
해당 script를 PR 본문에 기록합니다. 편집 command·Undo/Redo 변경은
[편집 Command/Undo 체크리스트](mydocs/manual/edit_command_review_checklist.md)도 사용하세요.
headless Chrome을 실행할 수 없다면 미실행 사유와 체크리스트의 수동 확인 결과를 기록하고 검토자에게
대체 증적을 제시합니다. 체크리스트만으로 필수 E2E·required check를 통과했다고 표시하지 않습니다.

```bash
# 예: 수정한 기능에 맞는 한 가지 이상의 E2E를 선택한다.
npm --prefix rhwp-studio run e2e:undo

# 실제 브라우저 수동 확인이 필요한 UI 변경은 개발 서버를 외부 인터페이스에도 열어 실행한다.
npm --prefix rhwp-studio run dev -- --host 0.0.0.0 --port 7700
# 브라우저에서 http://localhost:7700 을 열어 수정한 흐름을 확인한 뒤 서버를 종료한다.
```

`npm/editor`의 public API, transport, 선언 파일 또는 package manifest를 바꿨다면 Studio test만으로 끝내지
말고 package 계약도 확인합니다. iframe RPC·기본 옵션·WASM 초기화가 바뀌면 fresh WASM build 뒤 관련 embed
E2E까지 실행합니다.

```bash
npm --prefix npm/editor test
node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs
(cd rhwp-studio && npx tsc --ignoreConfig --noEmit --skipLibCheck ../npm/editor/index.d.ts)
(cd npm/editor && npm pack --dry-run --json)
# 최초 한 번의 .env.docker 준비와 Docker 미설치 host의 진단 경로는 개발 환경 안내를 따른다.
docker compose --env-file .env.docker run --rm wasm
VITE_URL=http://127.0.0.1:7700 npm --prefix rhwp-studio run e2e:embed
```

브라우저 화면·영상·개인정보가 포함된 sample은 저장소에 커밋하지 않고 PR 본문에 공개 가능한 범위로 첨부합니다.
렌더링 또는 페이지네이션을 바꿨다면 이 절차에 더해 아래의 시각 검증 안내를 따릅니다.

#### frontend 최종 확인

선택한 source 또는 review 루트에서 다시 `git rev-parse HEAD`와 `git status --short`를 확인하고 시작 SHA와
같은지 대조합니다. [commit 공백 검사](#모든-범위의-commit-공백-검사)도 수행하세요. 혼합 변경은 같은 셸에서
Rust 절의 manifest·원본 제출 확인으로 돌아갑니다. Studio 단독도 미커밋 변경이나 새 commit이 생기면
새 SHA로 해당 검증을 다시 수행하고, 검증한 source HEAD를 제출합니다.

### 성능 검증 책임

PR을 제출하기 위해 컨트리뷰터가 특정 로컬 환경의 **절대 성능 수치**, 비공개 코퍼스 또는
메인테이너 전용 벤치마크를 통과할 필요는 없습니다. 하드웨어·OS·폰트·브라우저 상태에 따라 달라지는
수치는 공통 제출 기준으로 사용할 수 없으며, 통제된 환경의 최종 성능 판정은 메인테이너가 수행합니다.

성능에 영향을 줄 수 있는 PR은 가능한 범위에서 다음을 적어주세요. 측정 환경이 없으면 `미측정`이라고
명시해도 PR을 제출할 수 있습니다.

- 예상 영향: 개선, 회귀 가능성, 영향 없음 또는 미확인
- 재현 절차와 사용한 공개 sample
- 측정했다면 환경과 변경 전후 관측값 — 단일 실행의 절대 시간보다 같은 환경의 상대 비교를 권장

이 정책은 성능 회귀를 면제하지 않습니다. 저장소에 공개된 결정적 성능 회귀 테스트와 GitHub required
checks는 기존과 같이 merge gate입니다. 추가 환경 검증에서 심각한 회귀를 발견해 merge를 보류할 때는
메인테이너가 공개 가능한 재현 절차·fixture 또는 자동 래칫을 제공하고 보정 범위를 함께 설명합니다.
비공개 자료나 특정 메인테이너 장비에서만 재현되는 수치 자체를 컨트리뷰터의 수정 의무로 돌리지 않습니다.

### 회귀 테스트 가이드

버그 수정 PR 에서 리뷰가 가장 먼저 확인하는 항목입니다. 아래 관례를 따르면 검토와 merge 가
크게 빨라집니다.

1. **red→green 회귀 테스트 동봉** — 수정 전 결함을 재현·고정하는 테스트를 함께 제출합니다.
   Rust 파일명 관례: `tests/cases/issue_{이슈번호}_{짧은_설명}.rs`. Studio 회귀는 변경에 맞는 단위 테스트나
   `rhwp-studio/e2e/`에 둡니다. 수정을 되돌리면 실패하고, 수정을 적용하면 통과해야 합니다.

   새 Rust integration 파일은 `tests/cases/`에만 만듭니다. PR에는 원본 `.rs`만 포함하고 suite를 직접
   선택하지 않습니다. 기여자 또는 검토자가 별도 review worktree에서, 또는 CI가 `--prepare`를 실행하면
   생성기가 source weight를 계산해 기존 integration suite 중 가장 가벼운 곳에 자동 배정합니다.

   `tests/generated/*.rs`, `tests/suites/manifest.json`은 **PR에 포함하지 않습니다.** 이 파일들은 검토·CI
   checkout에서만 만드는 파생 산출물입니다. 이를
   커밋하면 독립 PR끼리 같은 harness·manifest를 수정해 불필요한 충돌을 만들므로 CI가 거부합니다.
   `Cargo.toml`의 generated test target 블록도 일반 PR에는 포함하지 않습니다. 단, 통합 불가 예외 target이
   바뀐 메인터너 전용 registry 동기화 PR은 `--sync-cargo-targets`로 해당 marker 블록만 갱신할 수 있습니다.
   `#[path]`·root `mod`·feature-gated test처럼 module harness와의 호환성을 판단해야 하는 원본은 일반 기여자가 registry를
   늘리지 않습니다. 기존 source의 module 호환성이 확인된 경우에만 메인터너가 `suite-policy.json`의 좁은
   `moduleIntegrationOverrides`에 blocker 이름을 명시해 suite 배정을 허용합니다.
   원본을 이름 변경·삭제해도 PR에는 `tests/cases/` 변경만 제출합니다. `--rebalance`는 일반 기여
   절차에 포함하지 않으며, 배정 정책 자체를 바꾸는 메인터너 전용 별도 작업에서만 검토합니다.

   원본 commit 뒤 [Rust 검증 worktree 준비와 실행](#rust-검증-worktree-준비와-실행)을 따릅니다.
   prepare → fmt·lint·해당 회귀 → manifest check 순서이며, source 제출 checkout에서는 파생 파일을
   준비하지 않습니다. 생성 파일은 review worktree의 검증 증적일 뿐 stage하지 않습니다.

   제품 소스의 `#[cfg(test)]`에는 새 테스트 모듈이나 test support 항목을 추가하지 않습니다. 공개 API로
   재현할 수 있는 테스트는 `tests/cases/`에 작성하고, 기존 소스 테스트의 차등 이동 상태는 다음 명령으로
   확인합니다. root `src/`와 내부 `crates/*/src/`가 모두 검사 대상입니다. private 구현 불변식을 검증해야
   하는 예외나 새 내부 crate 경계는 별도 단계에서 근거와 기준선 변경을 함께 검토합니다.

   ```bash
   node scripts/rust-unit-test-tiers.mjs --check
   ```

   CI는 PR base와 현재 source를 다시 비교하고, unit-tier inventory도 source에서 메모리로 재계산한다.
   커밋된 generated harness·manifest는 거부한다. Cargo generated block은 명시적 registry 동기화에서 marker
   블록만 바꾼 경우에만 허용한다. 새 integration source는 `tests/cases/`만 허용하며, source-side 테스트는 Git
   rename으로 확인되는 순수 crate 이동처럼 개수가 늘지 않는 경로 변경만 허용한다.
2. **수정 전 실패 증명 (권장)** — "수정 커밋만 원복한 상태에서 신규 테스트가 실제로 FAIL"
   함을 PR 본문에 기록해주세요. 테스트가 결함을 판별한다는 증명이 되어 리뷰 신뢰도가
   높아집니다.
3. **기존 기대값(잠정 핀) 변경 시** — 페이지 수 등 잠정 핀 수치를 바꾸는 PR 은 다음을
   지켜주세요. 임의 갱신은 받지 않습니다.
   - 정답지 방향 근거 명시 (예: "PDF 정답 315 방향 +3, 잔여 −3")
   - 테스트 주석에 갱신 이력을 누적 (어떤 이슈의 어떤 정정으로 값이 왜 변했는지 —
     `tests/issue_2070_rowbreak_density.rs` 의 3단 이력 주석이 모범 사례)
   - "핀 미만/초과 시 의심 지점" 안내 메시지 유지
4. **인접 핀 무회귀 확인** — 수정 영역 주변의 알려진 핀 테스트(예: 페이지네이션이면
   byeolpyo 계열)가 유지되는지 `--no-fail-fast` 로 전체 실행하여 확인해주세요.

### 포맷 정책

rhwp는 저장소 루트의 `rust-toolchain.toml`과 `rustfmt.toml`을 기준으로 Rust 포맷을 관리합니다.
Rust 검증이 필요한 변경은 [준비된 review worktree](#rust-검증-worktree-준비와-실행)에서 확인합니다.

```bash
cargo fmt --all -- --check       # 준비된 review worktree에서 CI와 같은 전체 포맷 검사
```

generated suite의 `does not exist` 오류는 준비 부족에 의한 검사 실패입니다. 포맷 위반 여부를 확인한
것이 아니며, prepare 후 같은 check를 통과해야 합니다. 실제 포맷 diff가 나오면 다음 순서로 보정하세요.

1. 검증 worktree에서 `cargo fmt --all`로 포맷을 적용한 뒤 `git diff`로 원본 변경을 확인합니다.
2. 이 PR에 필요한 원본 파일의 포맷 보정만 source branch에 반영하고 새 commit을 만듭니다. generated
   harness·manifest를 복사하거나 stage하지 않습니다. 범위 밖 대량 포맷 diff가 있으면 최신 devel과의
   차이를 확인하고, 전체 정규화 변경은 별도 포맷 이슈·브랜치로 분리합니다.
3. 새 원본 commit에서 별도의 깨끗한 review worktree를 만들어 prepare부터 해당 검증을 다시 실행합니다.
   포맷을 적용한 dirty worktree에서 check가 통과해도, 보정 전 source commit을 검증한 결과가 아닙니다.

전체 포맷 **검사**와 포맷을 적용하는 명령의 실행은 허용하지만, 기능 PR에 무관한 전체 포맷 **변경**을
섞지 않습니다. 이 구분에 따라 다음 원칙을 유지합니다.

- 기능 변경과 전체 포맷 정규화는 같은 커밋에 섞지 않습니다.
- rustfmt 옵션이나 Rust toolchain 버전을 바꾸는 작업은 별도 이슈로 분리합니다.

### 한컴 PDF 와의 일치 검증에 대해

> ⚠️ **한컴 PDF 출력은 정답지가 아닙니다.**
>
> 동일 HWP 파일도 한컴 환경 (버전 / 폰트 설치 / OS / 출력 방법) 에 따라 PDF 결과가 다릅니다. 페이지 분할까지 환경별로 달라지는 사례가 발견되었습니다 (PR #360 정황). 따라서 **"한컴 PDF 와 일치"** 만을 PR 검증 기준으로 제출하셔도 머지가 보장되지 않습니다.

렌더링 영향 변경의 검증 기준 (우선순위, 실제 실행 범위는 [체크리스트](#pr-전-체크리스트) 적용):

1. **결정적 자동 검증** (필수):
   - Rust 렌더링 source 변경은 위 Rust lint·전체 `cargo nextest run`·Native Skia 회귀
   - SVG snapshot은 전체 integration 회귀에 포함되며, 실패 원인 확인·결정성 확인 시 준비된 worktree에서
     `node scripts/run-rust-test.mjs svg_snapshot -- --cargo-profile release-test --target-dir "${rhwp_review_target_dir:?}"`로 선택 실행
   - Studio 렌더링 UI 변경은 해당 frontend·E2E 회귀와 fresh WASM 검증

2. **시각 검증** (참고):
   - 한컴 PDF / 한컴 화면 캡처 + rhwp SVG 비교 — **본인 환경 명시 필수** (한컴 버전, OS, 폰트 등)
   - 페이지 분할 영향 PR 의 경우 메인테이너 환경 재검증 후 머지 결정

3. **다른 렌더링 결과** (참고):
   - HTML / Canvas / VS Code 확장 등 다른 출력 경로와의 일관성

### 페이지 분할 / 페이지네이션 영향 PR 의 경우

페이지 분할은 한컴 환경 의존성이 가장 큰 영역입니다. 이 영역의 PR 은 다음 절차 권장:

1. PR 본문에 검증 환경 명시 (한컴 버전, OS, 폰트, 출력 방법)
2. 메인테이너 환경 재검증 후 머지 결정 (작업지시자가 직접 확인)
3. 회귀 테스트 동봉 — 위 "회귀 테스트 가이드" 절의 관례를 따라주세요

### 렌더링 PR 자가 검증 도구 (한컴 없이 가능)

렌더링·레이아웃을 수정하는 PR 은 제출 전 아래 도구로 자가 검증하면 리뷰 왕복이 크게
줄어듭니다. 모두 **한컴 설치 없이** (macOS/Linux 포함) 실행할 수 있습니다.
재사용 가능한 Rust 도구 source는 `tools/diagnostics/`, fixture 생성기는
`tools/fixture_generators/`에 둡니다. 루트 `Cargo.toml`에 명시적으로 등록된 도구만
`cargo run --example <이름>`으로 실행하며, 이슈 전용 probe는
`mydocs/tech/investigations/issue-####/probes/`에서 해당 이슈의 근거로 관리합니다.

```bash
# 개체(표·그림) geometry 무회귀 — 원커맨드: devel 을 worktree 빌드해 baseline 자동 생성 후
# 현 트리와 대조, PR 본문용 markdown 요약(output/ovr/ovr_diff.md)까지 출력
python tools/object_visual_regression.py --preset ovr5 -o output/ovr --diff-against devel

# (수동 3단계 흐름도 그대로 동작 — 수정 전 baseline 저장, 수정 후 비교)
python tools/object_visual_regression.py <샘플.hwp> -o output/ovr --no-hwp --save-baseline
python tools/object_visual_regression.py <샘플.hwp> -o output/ovr2 --no-hwp --baseline output/ovr/baseline.json

# 편집-스윕 — 편집 경로 PR(vpos·pagination·undo)의 가짜 페이지 변동 검출
# devel 과 브랜치에서 각각 스윕 → 공통/해소/신규 분류 리포트 (신규 존재 시 exit 1)
cargo run --release --example edit_sweep -- samples -o output/sweep/branch.tsv
cargo run --release --example edit_sweep -- --compare output/sweep/devel.tsv output/sweep/branch.tsv -o output/sweep/report.md

# 라운드트립 시각 기하 회귀
cargo run --release --bin rhwp -- render-diff <샘플.hwp>

# HWPX→HWP 변환 페이지네이션 정합
python tools/roundtrip_fidelity_harness.py --files <샘플.hwpx> --workdir output/rtf -o output/rtf/result.tsv
```

- OVR(개체 시각 회귀)로 "변경 범위 밖 문서의 개체가 움직이지 않았음"을 결과와 함께
  PR 본문에 적어주시면 리뷰가 빨라집니다 — `--diff-against devel` 이 출력하는
  `ovr_diff.md` 표를 그대로 붙여넣으면 됩니다 (git 상태 전환·baseline 관리 불필요).
- 어떤 PR 에 어떤 시각 증거가 필요한지는
  [시각 검증 거버넌스](mydocs/manual/verification/visual_verification_governance.md)를 참고하세요 —
  시각 검증은 전수 절차가 아니라 **PR 의 수정 목적과 사용자에게 보이는 동작 기준으로 선택**합니다.
- 전체 CLI 도구는 [cli_commands.md](mydocs/manual/cli_commands.md) 참조.
- 자가 검증 통과는 회귀 없음의 증명이며, 한컴 정합의 최종 판정은 메인테이너 환경에서
  이루어집니다.

### HWP 샘플 파일 제공

다양한 HWP 파일로 테스트할수록 렌더링 품질이 올라갑니다. 개인정보가 없는 공공 문서나 테스트용 파일을 제공해주시면 큰 도움이 됩니다.

- **스크린샷·비교 이미지는 저장소에 커밋하지 말고 PR 본문에 첨부**해주세요 (필요 시
  메인테이너가 판정 자료를 `mydocs/pr/assets/` 에 반영합니다).
- **한컴 편집기 PDF 를 오라클로 제공하실 때**: `pdf/{원본 stem}-{한컴버전}.pdf` 명명
  (예: `pdf/issue1835_tac_stale_height-2022.pdf`), PR 본문에 생성 환경(한컴 버전)을
  명시해주세요. 한컴 정본 오라클은 버전·크기와 무관하게 `pdf/**`에 일반 Git blob으로만
  커밋하며 `pdf-2020/`, `pdf-large/` 같은 최상위 분리 폴더와 Git LFS pointer를 사용하지 않습니다.
  파일 하나는 50 MiB(52,428,800 bytes) 미만이어야 합니다. 상한을 넘으면 그대로 제출하지 말고
  축소 fixture·페이지 발췌·외부 증적 방식을 이슈에서 먼저 합의해주세요. 재현 fixture는 가능하면
  1~2페이지로 축소해 `samples/`에 포함합니다.

  ```bash
  python3 scripts/check_pdf_repository_policy.py
  ```

  `samples/**` 입력 fixture, `mydocs/**/assets` 검토 증적, 도구의 tiny test fixture처럼 역할이 다른
  PDF는 각 소유 경로를 유지합니다. 이 파일을 한컴 정본 오라클로 사용할 때만 `pdf/**` 규칙을
  적용합니다.

## 브랜치 규칙

| 브랜치 | 용도 | 보호 규칙 |
|--------|------|----------|
| `main` | 릴리즈 (안정 버전) | PR 필수 + CI 통과 + 리뷰 1명 |
| `devel` | 개발 통합 (PR 대상) | CI 통과 필수 |

- 컨트리뷰터 PR → `devel`
- 릴리즈 시 `devel` → `main` + 태그

## 디버깅 가이드

렌더링 버그를 조사할 때 코드 수정 없이 사용할 수 있는 3종 도구:

```bash
# 1. 문단/표 식별 (디버그 오버레이)
cargo run --bin rhwp -- export-svg sample.hwp --debug-overlay

# 2. 페이지 배치 목록
cargo run --bin rhwp -- dump-pages sample.hwp -p 3

# 3. 특정 문단 상세 (ParaShape, LINE_SEG, 표 속성)
cargo run --bin rhwp -- dump sample.hwp -s 0 -p 45
```

디버그 오버레이는 문단/표에 라벨을 표시합니다:
- 문단: `s{섹션}:pi={인덱스} y={좌표}`
- 표: `s{섹션}:pi={인덱스} ci={컨트롤} {행}x{열} y={좌표}`

이 정보를 이슈에 첨부하면 버그 수정이 빨라집니다.

## 프로젝트 구조

```
src/
├── model/          ← 순수 데이터 구조 (의존성 없음)
├── parser/         ← HWP/HWPX 파일 → 모델 변환
├── document_core/  ← 편집 명령 + 조회 (CQRS)
├── renderer/       ← 레이아웃, 페이지네이션, SVG/Canvas
├── serializer/     ← 모델 → HWP 파일 저장
└── wasm_api.rs     ← WASM 바인딩

rhwp-studio/        ← 웹 에디터 (TypeScript + Vite)
```

의존성 방향: `model` ← `parser` ← `document_core` ← `renderer` ← `wasm_api`

## 코드 스타일

- Rust 변경은 [Rust 검증 절차](#rust-검증-worktree-준비와-실행)의 fmt·native/WASM32/workspace Clippy 통과 (경고 0건)
- `unwrap()` 최소화
- 모든 문서는 한국어로 작성
- **소스 포맷 분기**: HWP3/HWPX 등 원본 포맷에 따른 레이아웃 분기가 필요하면
  boolean 전달이나 포맷 이름 비교 대신 `Document::layout_profile()` 질의를
  사용합니다 (`mydocs/tech/parser_architecture.md` 의 "소스 출처와 레이아웃
  호환 정책" 참조). 새 판별이 필요하면 profile 질의를 추가하는 방식으로
  엽니다.

## 문서 작성 규칙

rhwp는 코드뿐 아니라 **작업 과정의 기록**도 프로젝트의 일부입니다(Hyper-Waterfall 방법론). PR에 문서를 포함하시는 경우 아래 규칙을 지켜주세요.

> **문서 거버넌스**: 절차의 권위는 canonical 문서에 단일 기록됩니다 — 진입점은
> [`mydocs/README.md`](mydocs/README.md)(문서 지도·manifest)이고, 이 문서의 표는 요약입니다.
> 충돌 시 canonical 문서가 우선합니다.
>
> **AI 도구를 쓰신다면**: 일부 도구는 저장소 루트의 [`AGENTS.md`](AGENTS.md)를 자동으로
> 읽을 수 있습니다. 그러나 외부 기여 PR의 제출 범위와 절차는 이 `CONTRIBUTING.md`가 우선합니다.
> `AGENTS.md`에 있는 메인터너 운영 절차를 따라 review 문서·오늘할일·병합 기록을 PR에 추가하지
> 마세요. 위 **메인터너 검토 기록과의 구분**을 따릅니다.

### 폴더 구조 (`mydocs/` 하위)

> 폴더 역할의 canonical 은 [`docs_and_git_workflow.md`](mydocs/manual/codex/docs_and_git_workflow.md) 의 Folder Roles 입니다. 아래 표는 기여자 관점 요약입니다.

| 폴더 | 용도 |
|------|------|
| `orders/` | 메인터너 전용 일일 운영 기록 (`yyyymmdd.md`만 허용, 외부 기여자 PR에서는 수정하지 않음) |
| `plans/` | 수행 계획서, 구현 계획서 |
| `working/` | 단계별 완료 보고서 (`_stage{N}.md`) |
| `report/` | 최종 결과보고서 (`_report.md`) **— 최종 보고서는 반드시 여기** |
| `feedback/` | 피드백, 코드 리뷰 의견 |
| `tech/` | 기술 조사·분석 (스펙 정오표, 라이브러리 발견 등) |
| `manual/` | 사용자/개발자 매뉴얼 |
| `troubleshootings/` | 트러블슈팅 (재발 방지용 해결 기록) |
| `pr/` | **PR 검토 기록** (메인테이너·collaborator가 관리, 외부 기여자는 작성 불필요) |

### 문서 메타데이터 (front matter)

`mydocs/manual/`, `mydocs/tech/`, `mydocs/troubleshootings/` 에 문서를 추가·수정할 때는
**front matter 4필드가 필수**입니다:

```markdown
---
kind: investigation        # canonical | guide | reference | investigation | decision | snapshot | memory
status: active             # active | historical | superseded
canonical: mydocs/manual/codex/docs_and_git_workflow.md   # 이 문서가 따르는 권위 문서 경로
last_verified: 2026-07-17  # 역할·canonical 관계를 마지막으로 확인한 날짜
---
```

로컬 검사 (CI 미실행 — 필요 시 실행):

```bash
python3 scripts/check_document_metadata.py   # front matter 4필드 검사
python3 scripts/check_markdown_links.py      # 상대 링크 검사
```

`plans/`, `working/`, `report/`, `orders/` 의 타스크 문서에는 front matter가 필요 없습니다.

### 파일명 규칙

타스크 관련 문서는 다음 형식을 따릅니다:

- 수행 계획서: `task_{milestone}_{이슈번호}.md` (예: `task_m100_235.md`)
- 구현 계획서: `task_{milestone}_{이슈번호}_impl.md`
- 단계별 보고서: `task_{milestone}_{이슈번호}_stage{N}.md` (`working/`)
- 최종 보고서: `task_{milestone}_{이슈번호}_report.md` (`report/`)

**주의 사항:**

- `task_` 접두어 고정 (`task_bug_`, `task_feat_` 등은 사용하지 않음)
- 마일스톤은 `m{숫자}` 형식 (예: `m100`). 생략·약식 금지
- 후속 수정: `_v2`, `_v3` 버전 접미어 사용 (`_fix`, `_hotfix` 금지)
- `orders/` 에는 `yyyymmdd.md` 외의 파일을 두지 않습니다. 이슈 상세 조사는 `troubleshootings/` 또는 `tech/` 로
- 최종 보고서(`_report.md`)는 반드시 `report/` 폴더에 위치 (`working/` 아님)

### 기여자가 작성해야 하는 문서 범위

기여자는 본인 작업 범위(내부 타스크 문서: `plans/`, `working/`, `report/`, `tech/`, `troubleshootings/` 등)만 작성합니다.
`orders/`는 병합 뒤 상태를 기록하는 메인터너 전용 운영 문서이므로, 외부 기여자 PR에서 만들거나 갱신하지
않습니다.

**`pr/` 폴더는 메인테이너와 collaborator가 PR을 검토한 기록을 남기는 전용 공간**이므로,
외부 기여자는 직접 작성할 필요가 없습니다. PR 생성으로 번호가 확정된 뒤 메인테이너나 collaborator가
`pr_{번호}_review.md`, `pr_{번호}_report.md` 등을 해당 PR branch의 후속 commit으로 생성합니다. 이 파일들은
나중에 **PR 처리 이력으로 공개**되므로, 본인 PR이 어떻게 검토되었는지 추적 가능합니다.

### 이 규칙이 애매하다면

애매한 상황이 있다면 PR 코멘트로 질문해주세요. 메인테이너가 안내드리고, 필요하면 이 문서를 보완합니다. (이 규칙 자체가 PR 리뷰 과정에서 지속적으로 다듬어지고 있습니다.)

## HWP 단위 참고

- 1 inch = 7,200 HWPUNIT
- 1 mm ≈ 283.465 HWPUNIT

## 소통

- **[Discussions](https://github.com/edwardkim/rhwp/discussions)** — 질문, 아이디어, 기술 토론
- **[Issues](https://github.com/edwardkim/rhwp/issues)** — 버그 리포트, 기능 요청

## Notice

본 제품은 한글과컴퓨터의 한글 문서 파일(.hwp) 공개 문서를 참고하여 개발하였습니다.

## License

이 프로젝트는 [MIT License](LICENSE)로 배포됩니다. 기여하신 코드도 동일한 라이선스가 적용됩니다.

## LLM/에이전트 보조 기여

이 저장소는 AI 에이전트도 기여 도구로 사용할 수 있다. Claude Code·Copilot·Cursor·Codex·
Gemini CLI·Windsurf·Cline은 도구별 지침 파일을 자동으로 읽을 수 있다. 다만 외부 기여자가
PR에 포함할 파일과 검증 범위는 이 문서가 정하며, 자동 로딩된 내부 지침은 메인터너의 접수·검토·
병합 운영을 외부 PR에 복제하라는 의미가 아니다.

| 도구 | 자동 로딩 파일 |
|---|---|
| Claude Code | `CLAUDE.md` → `AGENTS.md` (+ `.claude/skills/` 자동 발견 — 기여 절차는 `rhwp-contributor`) |
| Codex | `AGENTS.md` |
| GitHub Copilot | `.github/copilot-instructions.md` |
| Cursor | `.cursor/rules/rhwp.mdc` |
| Gemini CLI | `GEMINI.md` |
| Windsurf / Cline | `.windsurfrules` / `.clinerules` |
| AGENTS.md 표준 진영 — Codex·OpenCode·Jules·Amp·Zed·Devin·Antigravity·Grok Build·Kimi CLI·Pi 등 | `AGENTS.md` (도구 공통 표준) |
| 오케스트레이터(ADE) — Orca 등 워크트리 병렬 진영 | 자체 파일 없음 — 부리는 각 에이전트(Claude Code·Codex·OpenCode 등)의 파일이 그대로 적용 |
| AWS Kiro (Amazon Q 후계) | `.kiro/steering/` |
| Qwen Code | `QWEN.md` |
| Aider 계열(컨벤션 파일) | `CONVENTIONS.md` |
| Zed 계열(.rules) | `.rules` |
| Goose | `.goosehints` |
| Replit Agent | `replit.md` |
| RooCode / Kilo Code | `.roo/rules/` / `.kilocode/rules/` |
| JetBrains Junie | `.junie/guidelines.md` |
| Trae / Amazon Q(일몰 예정·Kiro 승계) / Augment / Continue | `.trae/rules/` · `.amazonq/rules/` · `.augment/rules/` · `.continue/rules/` |
| llms.txt 소비 도구 | `llms.txt` |

**모델이 무엇이든 같은 길** — DeepSeek·GLM·Llama·Qwen·MiMo·MiniMax 등 어떤 모델(무료 모델 포함)을
쓰든, 그 모델을 부리는 위 CLI/IDE 가 이 파일들을 자동으로 읽으므로 결국 같은 규약에 도착한다.
저장소 파일을 읽지 않는 도구(영상·미디어 생성형 등)는 이 표의 범위 밖이다.

에이전트 보조로 작업했다면 사용한 도구, 재현 명령, 테스트 결과를 PR 본문에 간단히 적어주세요.
메인터너 운영용 capsule·review archive·오늘할일 파일은 첨부 대상이 아닙니다.
