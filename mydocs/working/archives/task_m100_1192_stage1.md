# Task #1192 Stage 1 완료 보고서 — ci.yml (A + C + F)

- **이슈**: #1192
- **브랜치**: `local/task1192`
- **단계**: Stage 1 / 3
- **대상**: `.github/workflows/ci.yml`

## 적용 내용

### A — 테스트 중복 제거
- `Canvas layer parity tests` step(`cargo test canvas_layer_tree_matches_legacy --lib --verbose`) 제거.
- 주석으로 근거 명시: 해당 테스트는 feature gate 없는 lib 테스트 → `Run tests`(cargo test)에 포함.
- **Native Skia tests step 은 유지** (native-skia 가 default feature 아님 → 유일 실행 경로).

### C — Free disk space 축소 (build-and-test + wasm-build 2개 job)
- 유지: `sudo rm -rf /usr/local/lib/android`, `sudo rm -rf /usr/share/dotnet`, `df -h`.
- 제거: `sudo apt-get clean`, `sudo rm -rf /opt/ghc`,
  `sudo rm -rf /opt/hostedtoolcache/CodeQL`, `sudo docker image prune --all --force`.

### F — concurrency 취소 그룹
- ci.yml 최상위(`env:` 다음)에 추가:
  ```yaml
  concurrency:
    group: ci-${{ github.workflow }}-${{ github.head_ref || github.ref }}
    cancel-in-progress: ${{ github.event_name == 'pull_request' }}
  ```
- PR 이벤트에서만 cancel → devel/main push 머지 검증 run 보존.

## 검증

| 확인 | 결과 |
|------|------|
| YAML 유효성 (`yaml.safe_load`) | ✅ VALID (top keys: name/on/jobs/concurrency) |
| A: Canvas parity step | ✅ 제거 (grep 0) |
| native-skia step | ✅ 유지 (grep 1) |
| C: apt clean / ghc / CodeQL / docker prune | ✅ 제거 (grep 0) |
| C: android / dotnet / df -h | ✅ 유지 (각 2, 양 job) |
| F: concurrency | ✅ 추가 (grep 1) |

## 다음 단계

Stage 2 — codeql.yml: B(rust cargo 캐시) + F(concurrency).
