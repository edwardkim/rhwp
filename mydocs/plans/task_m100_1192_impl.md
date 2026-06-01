# 구현 계획서 — Task #1192: CI 시간 단축

- **이슈**: #1192
- **브랜치**: `local/task1192`
- **작성일**: 2026-05-31
- **수행계획서**: `task_m100_1192.md` (승인 완료)
- **대상**: `.github/workflows/{ci.yml, codeql.yml, render-diff.yml}` 만 (소스 무변경)

## 단계 구성 (3단계)

### Stage 1 — ci.yml: A(중복 제거) + C(디스크 축소) + F(concurrency)

1. **A**: `build-and-test` job 의 `Canvas layer parity tests` step(89~90행) 제거.
   - `cargo test canvas_layer_tree_matches_legacy --lib --verbose` 한 step 삭제.
   - 근거: 해당 테스트는 feature gate 없는 lib 테스트로 `Run tests`(cargo test)에 포함.
2. **C**: `Free disk space` step 2곳(build-and-test 55~63행, wasm-build 124~132행) 축소.
   - 유지: `sudo rm -rf /usr/local/lib/android`, `sudo rm -rf /usr/share/dotnet` (대용량·빠름).
   - 제거: `sudo apt-get clean`, `sudo rm -rf /opt/ghc`,
     `sudo rm -rf /opt/hostedtoolcache/CodeQL`, `sudo docker image prune --all --force`(느림).
   - `df -h` 는 유지(검증 로그).
3. **F**: `ci.yml` 최상위(`name:` 다음, `on:` 앞 또는 `env:` 부근)에 concurrency 추가:
   ```yaml
   concurrency:
     group: ci-${{ github.workflow }}-${{ github.head_ref || github.ref }}
     cancel-in-progress: ${{ github.event_name == 'pull_request' }}
   ```
4. 검증: `actionlint` (있으면) 또는 YAML 파싱 확인. 커밋 + stage1 보고서.

### Stage 2 — codeql.yml: B(rust 캐시) + F(concurrency)

1. **B**: `analyze` job 에 rust 전용 cargo 캐시 step 추가.
   - 위치: `Install Rust toolchain`(61행, `if: matrix.language == 'rust'`) 다음,
     `Build Rust (for CodeQL)`(65행) 앞.
   - 내용:
     ```yaml
     - name: Cache cargo registry & build (rust)
       if: matrix.language == 'rust'
       uses: actions/cache@v5
       with:
         path: |
           ~/.cargo/registry
           ~/.cargo/git
           target
         key: ${{ runner.os }}-codeql-rust-${{ hashFiles('**/Cargo.lock') }}
         restore-keys: |
           ${{ runner.os }}-codeql-rust-
     ```
2. **F**: codeql.yml 최상위에 concurrency 추가 (group `codeql-...`, 동일 패턴).
   - 단, codeql 은 schedule(cron) 트리거도 있음 → cancel-in-progress 는 PR 한정 유지.
3. 검증 + 커밋 + stage2 보고서.

### Stage 3 — render-diff.yml: F(concurrency) + 전체 검증 + 보고서

1. **F**: render-diff.yml 최상위에 concurrency 추가 (group `render-diff-...`).
   - render-diff 는 pull_request 트리거 위주 → cancel-in-progress PR 한정.
2. **전체 검증**:
   - 로컬 YAML 유효성 확인.
   - 변경을 devel 머지 → push → 실제 CI 1회 실행 관찰:
     - A: `Run tests` 로그에 canvas_layer_tree_matches_legacy + native-skia 각 1회 확인.
     - B: CodeQL rust 로그 cache restore + 시간 단축 확인.
     - C: `df -h` 디스크 여유 확인.
     - F: 같은 PR 연속 push 시 이전 run cancelled 확인(후속 PR 에서 관찰).
   - **롤백 기준**: 디스크 초과로 빌드 실패 시 C 즉시 복원. 다른 항목 실패 시 해당 stage revert.
3. 최종 보고서 `task_m100_1192_report.md` + orders 갱신 + 커밋.

## 적용 순서 / 머지 전략

- 3 stage 모두 `local/task1192` 에서 커밋.
- 완료 후 `local/task1192` → devel 머지 + push. push 직후 devel CI 가 **변경된 워크플로우로**
  실행되므로, 그 run 이 곧 Stage 3 의 실제 검증이 됨.
- 워크플로우 변경은 PR CI 에 즉시 반영되지 않을 수 있어(기존 정의로 실행되는 경우 있음)
  devel push run 으로 최종 확인.

## 위험 및 가드 (재확인)

| 항목 | 위험 | 가드 |
|------|------|------|
| A | 테스트 커버리지 손실 | canvas parity 만 제거, native-skia 유지 — 검증 완료 |
| C | 디스크 초과 빌드 실패 | 보수적 축소 + `df -h` 확인 + 즉시 롤백 |
| F | push 머지 검증 run 취소 | `cancel-in-progress` PR 이벤트 한정 |
| B | 캐시 오염 | 별도 key namespace(`codeql-rust`), ci.yml 캐시와 분리 |

## 검증 기준 (수행계획서 5절과 동일)

- 변경 후 devel CI 전체 green + Build & Test / CodeQL rust 시간 단축.
- 테스트 1회씩 정상 실행 로그.
- 디스크 여유 유지.
