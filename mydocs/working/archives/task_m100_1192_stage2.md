# Task #1192 Stage 2 완료 보고서 — codeql.yml (B + F)

- **이슈**: #1192
- **브랜치**: `local/task1192`
- **단계**: Stage 2 / 3
- **대상**: `.github/workflows/codeql.yml`

## 적용 내용

### B — CodeQL rust cargo 캐시 추가
- `analyze` job, `Install Rust toolchain`(rust) 다음 / `Build Rust (for CodeQL)` 앞에
  `Cache cargo registry & build (rust)` step 삽입.
- `if: matrix.language == 'rust'` 로 rust matrix 에만 적용 (js-ts/python 은 영향 없음).
- path 는 ci.yml 과 동일(`~/.cargo/registry`, `~/.cargo/git`, `target`),
  key 는 별도 namespace `${{ runner.os }}-codeql-rust-${{ hashFiles('**/Cargo.lock') }}`
  + restore-keys 폴백. ci.yml 캐시(`-cargo-`)와 분리되어 상호 오염 없음.
- 효과: CodeQL rust 매 PR cold build → warm build (~4~5분 절감).

### F — concurrency 취소 그룹
- codeql.yml 최상위(`workflow_dispatch:` 다음)에 추가:
  ```yaml
  concurrency:
    group: codeql-${{ github.workflow }}-${{ github.head_ref || github.ref }}
    cancel-in-progress: ${{ github.event_name == 'pull_request' }}
  ```
- PR 이벤트에서만 cancel → push/schedule(주간 cron) run 보존.

## 검증

| 확인 | 결과 |
|------|------|
| YAML 유효성 (`yaml.safe_load`) | ✅ VALID (name/on/concurrency/jobs, analyze 6 steps) |
| B: cache step + rust-gated | ✅ cache step 1, `codeql-rust-` 2회(key+restore-keys) |
| B: rust `if` guards | ✅ 3 (toolchain + cache + build) |
| F: concurrency | ✅ 1 (추가) |

## 다음 단계

Stage 3 — render-diff.yml: F(concurrency) + 전체 검증(devel push CI 관찰) + 최종 보고서.
