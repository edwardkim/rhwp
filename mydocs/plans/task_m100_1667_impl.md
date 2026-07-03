# Task M100 #1667 구현 계획서

## 개요

- 이슈: #1667 `[CI] Rust cache 전략 개선: actions/cache 유지 vs Swatinem/rust-cache 검토`
- 부모 이슈: #1668 `[Tracking/RFC] CI Build & Test 실행 시간 단계별 단축`
- 선행 문서: `mydocs/plans/task_m100_1667.md`
- 작업 브랜치: `task-1667-rust-cache-strategy`
- 1차 구현 대상: CodeQL Rust cargo cache restore/save 분리

## 결정사항

1차 구현 PR은 "PR restore-only"와 "restore/save 분리" 중 하나를 고르는 작업이 아니다.

- 정책 결정: PR run에서는 restore-only로 동작하게 한다.
- 구현 방식: `actions/cache/restore@v5`와 `actions/cache/save@v5`를 분리한다.
- trusted branch 정책: `push` on `refs/heads/devel` 또는 `refs/heads/main`에서만 save를 허용한다.
- exact hit 정책: exact hit이면 trusted branch에서도 save를 생략한다.

따라서 이번 PR의 정확한 표현은 "CodeQL Rust cache를 restore/save 분리로 바꾸어 PR restore-only /
trusted branch save-only 정책을 적용한다"이다.

## 구현 판단

이번 코드 PR의 범위는 `.github/workflows/codeql.yml`의 Rust cache step으로 한정한다.

#1667은 이름상 `Swatinem/rust-cache` 검토를 포함하지만, 선행 코멘트와 측정 결과 기준으로 바로 third-party action을
도입하지 않는다. 현재 확인된 우선 문제는 "Build & Test cache가 틀렸다"가 아니라, Build & Test 외 workflow에도
PR run에서 cache save 표면이 남아 있다는 점이다.

특히 #1702 최종 리뷰에서 CodeQL Rust analyze의 `actions/cache@v5` step은 #1664의 Build & Test 범위 밖에 남아
있다고 지적됐다. CodeQL은 `push` on `main/devel`, `pull_request`, `schedule`, `workflow_dispatch`를 모두 갖고 있어
#1664와 같은 정책을 적용하기 쉽다. 따라서 1차 구현은 CodeQL Rust cache에 PR restore-only / trusted branch save-only
정책을 적용하는 것으로 제한한다.

Render Diff cargo cache와 stale PR ref cleanup은 #1667 범위에는 포함하지만, 이번 1차 코드 PR에는 포함하지 않는다.
특히 `Linux-render-diff-cargo-*`는 PR ref 누적의 가장 큰 요인이지만, workflow가 `pull_request`/`workflow_dispatch`
중심이라 단순 PR restore-only 전환 시 warm source가 사라질 수 있다. 따라서 Render Diff 정책 변경, closed PR ref
cleanup 운영, cleanup 자동화는 CodeQL PR과 분리한다.

## 포함 범위

- `.github/workflows/codeql.yml`
  - 기존 `Cache cargo registry & build (rust)` 단일 `actions/cache@v5` step 분리
  - Rust matrix에서만 restore 수행
  - `push` on `refs/heads/devel` 또는 `refs/heads/main`에서만 save 허용
  - exact hit이면 save 생략
  - cache path와 key namespace는 현행 유지
- 문서
  - 단계 보고서 또는 measurement 문서에서 before/after 관측값 기록
  - #1667 이슈 코멘트에 PR CI 관측과 devel push 관측을 분리 보고

## 제외 범위

- `.github/workflows/ci.yml`의 Build & Test cache 변경
- `.github/workflows/render-diff.yml`의 Render Diff cargo cache 변경
- `.github/workflows/ci.yml`의 `wasm-build` cache 변경
- `Swatinem/rust-cache` 도입
- cache 삭제 또는 stale PR ref cleanup 실행
- Render Diff closed PR ref cleanup 자동화
- branch protection / required check 이름 변경
- 회귀 가드, 테스트 파일, `tests/golden_svg/**` 구조 변경

## 구현 내용

### 1. CodeQL Rust cache restore step

기존 step:

```yaml
- name: Cache cargo registry & build (rust)
  if: matrix.language == 'rust'
  uses: actions/cache@v5
```

변경 후:

```yaml
- name: Restore cargo registry & build cache (rust)
  id: codeql_rust_cache_restore
  if: matrix.language == 'rust'
  uses: actions/cache/restore@v5
```

path, key, restore-keys는 현행을 유지한다.

```yaml
path: |
  ~/.cargo/registry
  ~/.cargo/git
  target
key: ${{ runner.os }}-codeql-rust-${{ hashFiles('**/Cargo.lock') }}
restore-keys: |
  ${{ runner.os }}-codeql-rust-
```

### 2. CodeQL Rust cache save step

`Build Rust (for CodeQL)` 이후, `Perform CodeQL Analysis` 이전에 save step을 추가한다.

```yaml
- name: Save cargo registry & build cache (rust)
  if: ${{ matrix.language == 'rust' && github.event_name == 'push' && (github.ref == 'refs/heads/devel' || github.ref == 'refs/heads/main') && steps.codeql_rust_cache_restore.outputs.cache-hit != 'true' }}
  uses: actions/cache/save@v5
```

이 위치를 선택하는 이유는 다음과 같다.

- cache에 담고 싶은 주요 산출물은 `cargo build`로 생성되는 Rust build output이다.
- save 실패가 발생할 경우 원인을 `Build Rust (for CodeQL)` 직후에 확인하기 쉽다.
- CodeQL analyze check 이름과 matrix 구조를 변경하지 않는다.

### 3. 유지할 내용

- `Build Rust (for CodeQL)` 명령은 `cargo build` 그대로 둔다.
- CodeQL analyze job 이름 `Analyze (rust)`는 유지한다.
- cache key namespace `Linux-codeql-rust-*`는 유지한다.
- CodeQL JS/TS, Python matrix에는 cache step을 추가하지 않는다.

## 측정 기준

#1668 공통 기준을 따르되, CodeQL workflow 관측값은 Build & Test와 별도 표로 기록한다.

### PR run

- PR checks 완료 시간
- `Analyze (rust)` job 시간
- `Restore cargo registry & build cache (rust)` hit/miss
- `Save cargo registry & build cache (rust)` skipped 여부
- `Build Rust (for CodeQL)` 시간
- `Perform CodeQL Analysis` 시간
- PR run에서 cache reservation/read-only/save failure 경고가 사라졌는지
- required check 이름 변경 없음

### devel/main push run

- `Analyze (rust)` job 시간
- restore exact hit / fallback hit / miss
- exact hit이면 save skipped
- miss 또는 fallback이면 save 성공 여부
- saved cache 크기
- cache reservation/read-only/save failure 경고 여부

### 공통 해석

- PR에서 save가 skipped 되어도 `Build Rust (for CodeQL)` compile이 남는 것은 실패로 보지 않는다.
- 이번 PR의 목표는 CodeQL Rust cache의 PR save 표면 제거와 trusted branch save 정책 정렬이다.
- `Dirty rhwp` / local crate 재컴파일 원인 분석은 Build & Test 및 cargo fingerprint 관측과 연결되므로 별도 후속으로 둔다.

## before 기준선

이미 확보된 기준선:

- #1702 merge 후 `devel` push CodeQL 관측
  - CodeQL Rust cache fallback hit: `Linux-codeql-rust-`
  - restore size: 317,394,514 B
  - `Build Rust (for CodeQL)`: 58.97s
  - cleanup 전 post-cache save reservation 실패 위치: Analyze (rust) log line 2262-2263
- #1849 merge 후 `devel` push CodeQL 관측
  - run: `https://github.com/edwardkim/rhwp/actions/runs/28649575149`
  - `Analyze (rust)`: 약 8m27s
  - exact cache: `Linux-codeql-rust-6a1af...`
  - cache size: 529,492,545 B
- #1667 착수 시 cache inventory
  - 전체: 30개 / 11,131,139,002 B
  - `Linux-codeql-rust-*`: 5개 / 2,216,797,926 B

## 검증 계획

로컬 문법 검증:

```bash
git diff --check
ruby -e 'require "yaml"; YAML.load_file(".github/workflows/codeql.yml")'
actionlint .github/workflows/codeql.yml
```

`actionlint`가 로컬에 없으면 YAML parse와 diff whitespace 검증까지만 수행하고, GitHub Actions PR run으로 최종 검증한다.

GitHub Actions 검증:

1. 코드 PR을 draft로 생성한다.
2. PR run에서 `Analyze (rust)`를 확인한다.
3. PR run의 save step이 skipped 되었는지 확인한다.
4. cache reservation/read-only/save failure 경고가 없는지 확인한다.
5. merge 후 `devel` push run에서 exact hit 또는 save success를 확인한다.

## 위험 및 대응

| 위험 | 대응 |
|------|------|
| save condition 오타로 PR save가 다시 발생 | PR run에서 save step skipped 여부를 merge 전 확인 |
| trusted branch miss에서 save가 실행되지 않음 | merge 후 `devel` push run에서 restore/save log 확인 |
| save step이 analyze 전에 실행되어 job 시간이 늘어남 | miss/fallback run에서만 발생하며, exact hit에서는 skipped 된다 |
| CodeQL Rust compile 시간 자체가 줄지 않음 | 이번 범위의 성공 기준은 compile 제거가 아니라 save 정책 정렬 |
| Render Diff PR cache 누적이 계속됨 | 별도 PR에서 Render Diff 정책 또는 cleanup 운영으로 처리 |

## 롤백 계획

문제가 있으면 `.github/workflows/codeql.yml`의 Rust cache step을 기존 단일 `actions/cache@v5` step으로 되돌린다.
check 이름, matrix, command를 바꾸지 않기 때문에 롤백 범위는 cache step에 한정된다.

## 다음 단계

작업지시자 승인 후 다음 순서로 진행한다.

1. `.github/workflows/codeql.yml` 수정
2. 단계 보고서 작성
3. 로컬 문법 검증
4. 코드 PR 생성
5. PR CI 관측
6. merge 후 `devel` push 관측
