# #6916 Stage 3 — Gym 없는 제품 빌드·배포 검증

- Issue: [#6916](https://github.com/edwardkim/rhwp/issues/6916)
- 날짜: 2026-09-09
- 승인: Stage 2 결과 후 메인테이너의 Stage 3 진행 승인
- 상태: 승인된 로컬 검증 완료. 결과 승인·remote push·PR 생성 대기.
- 계획: [구현계획](../plans/task_m100_6916_impl.md)의 Stage 3

## 1. 실행 신원과 격리

- 제품 source: `882c0dfc9a9a73d3bf5b679436697a85b000cc5f`의 Git archive.
- 코드 검증 후보: `369e789eea1839462674e8a96f09c453ff2ba111`.
  위 source와의 차이는 계획·Stage 2 보고 문서 3개뿐이다.
- Gym 없는 source: `/tmp/rhwp-6916-stage3.GhIEHx/source`.
  `git archive HEAD | tar -x --exclude='gym' -C <source>`로 생성하고 Gym 경로 부재를 확인했다.
  주 checkout에서 Gym을 삭제하거나 sparse 설정을 바꾸지 않았다.
- 독립 native target: `/tmp/rhwp-6916-stage3.GhIEHx/native-target` (처음부터 빈 경로).
- 독립 Docker WASM target: `/tmp/rhwp-6916-stage3.GhIEHx/wasm-target`.
- 전체 검증·선택적 Gym 공간: `/home/edward/mygithub/rhwp-6916-review`, detached `369e789ee`.
- 전체 검증 캐시: `/home/edward/mygithub/rhwp-shared-review-target`.
- 로그·실행 보조 스크립트: `output/6916/stage3/` (로컬 증적, PR 제출물 아님).

Linux native는 Rust/Cargo 1.93.1, Node v24.15.0, npm 11.12.1이다.
nextest는 0.9.137이며 권장 0.9.140 대비 설정 경고 여부를 결과에 구분한다.
Docker는 기존 `rhwp-wasm:latest` 이미지 ID `4824b312625e`를 사용하되 저장소의
`rust-toolchain.toml`이 요구하는 toolchain으로 빌드한다. 실제 버전은 Docker 로그에 남긴다.
같은 checkout/캐시의 Cargo 명령은 병렬 실행하지 않는다.

## 2. 검증 결과

| 항목 | 명령·결과 |
| --- | --- |
| Gym 없는 Linux 제품 | `cargo build --locked --release --bin rhwp --manifest-path <source>/Cargo.toml --target-dir <native-target>`: exit 0, 7m 59s |
| 릴리스 포장 | `release-binary.yml`과 같은 `rhwp/rhwp`, LICENSE, README.md, README_EN.md를 tar.gz로 포장·별도 install에 해제 |
| 저장소 밖 설치본 | `--version` = `rhwp v0.8.6`, `--help` = 15,886 bytes, MCP initialize/list/read 성공 |
| MCP 전체 리소스 | 15개 모두 읽기 성공. 정적 11개의 광고된 size와 본문 UTF-8 바이트 수 일치. 생성형 4개는 기존 계약대로 size 미광고 |
| 선택적 Gym canary | 별도 review worktree에서 아래 외부 바이너리로 core-cli 54/54 passed, failed/skipped/missingArtifact/failedScore/buildError 모두 0, exit 0 |
| Gym 없는 Docker WASM | Rust/Cargo 1.93.1, wasm-pack 0.15.0, `scripts/wasm-pack-locked.sh --target web`: 성공, 7m 06s (Rust compile 4m 24s 포함) |
| npm 포함물 | `scripts/prepare-npm.sh` 후 `npm pack --dry-run --json`: exit 0, 아래 7개 파일만 포함, unpackedSize 11,067,732 bytes |

설치본에는 Gym 경로를 주지 않았고 환경변수는 PATH/LANG만 전달했다.
실제 JSON-RPC 응답은 `install-mcp-responses.json`, 요약은 `install-check.json`이다.
첫 로컬 설치 검사 스크립트가 생성형 리소스에도 size를 강제해 assertion 실패했다.
`src/mcp_serve.rs`의 기존 size 생략 계약을 확인하고 검사만 정정해 재실행했다.
제품 코드는 바꾸지 않았으며, Gym 정적 리소스의 size 검증은 그대로 유지했다.

제품 바이너리 SHA-256은
`7186832ca29aaf4289e5b6b3caa1094098be59e91829b734a1eaeedc894fa68e`다.
포장은 `output/6916/stage3/rhwp-6916-linux-x64.tar.gz`에 있다.

선택적 Gym은 메인테이너 감사 모드로 다음 명령을 실행했다.

```bash
python3 gym/tools/build_baseline.py \
  --agent maintainer-6916-canary --pack core-cli \
  --bin /tmp/rhwp-6916-stage3.GhIEHx/install/rhwp/rhwp --json
```

Gym runner HEAD는 `369e789ee`, tree는 `cf99a26a26de71b1cbdb9183332df85a8513525c`다.
Python 3.12.3 / Linux x86_64이며, stdout `gym-canary.json`과 stderr를 분리했다.
`ok`만 보지 않고 built=taskCount=54, 모든 result.ok 및 오류·누락·skip 0을 재검산했다.
Docker 컴파일 대기 중 이미 완성된 바이너리로 canary를 실행했으며 Cargo 병렬 실행은 없었다.

npm은 `@rhwp/core@0.8.6`이며 포함물은 LICENSE, README.md, package.json,
rhwp.d.ts, rhwp.js, rhwp_bg.wasm, rhwp_bg.wasm.d.ts다. Gym 파일은 없다.
실제 npm registry 게시가 아닌 dry-run 포함물 검사다. WASM 본체는 10,378,214 bytes다.

Docker 빌드 자체는 성공했으나 후속 npm 준비가 첫 실행에서 권한 오류로 종료했다.
로컬 보조 스크립트에서 HOST UID/GID를 1000으로 잘못 가정한 원인이었다.
실제 `id`의 1002:1002를 확인하고 **이번에 생성한 격리 pkg만** 소유자를 정정했다.
스크립트도 `id -u`/`id -g`를 전달하도록 고친 뒤 npm 준비·dry-run을 성공시켰다.
WASM 재빌드나 제품 source 변경 없이 실행 환경만 정정했다.

### PR 전 Rust lint

`output/6916/stage3/review-checks.sh`에서 아래 명령을 review worktree·공유 review target에
순차 실행했고 모두 exit 0이었다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --locked --target-dir <review-target> -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown \
  --target-dir <review-target> -- -D warnings
cargo build --locked --workspace --target-dir <review-target>
cargo clippy --locked --workspace --all-targets --target-dir <review-target> -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
```

native Clippy 52.04s, WASM Clippy 48.27s, workspace build 1m 16s,
all-target Clippy 1m 12s였다. 이는 해당 실행 시간이며 성능 개선 계측이 아니다.
manifest는 1,218 sources / 28 suites + 20 exceptions = 48 integration targets를 확인했다.
파생 suite·manifest는 review 전용이며 source PR에 포함하지 않는다.

### 전체 integration 회귀

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp-shared-review-target --tests --no-fail-fast
```

exit 0. **9,302 passed, 0 failed, 46 skipped**, 78 binaries.
test build 3m 29s, 실행 303.119s, run ID `efa4eeb9-4e61-4b9c-9272-f641ad81a6e8`.
전체 검증 드라이버는 11:36:24~11:49:30 KST에 실행했다.
종료 후 동일 명령의 `nextest list --message-format json`으로 9,348건의 inventory를 확인했고,
제외된 46건은 모두 `ignored=true`, filter mismatch reason `ignored`였다.
임의 필터나 실패 제외로 통과시킨 것이 아니다. 목록과 재계산은 `nextest-list.json`, `skip-summary.json`에 있다.

대형 표 #2063 테스트는 232.871s 후 통과했다. 이 테스트가 실행되는 동안 뒤의 #2007 단독 실행
예약 때문에 진행 숫자가 잠시 멈췄다. 기존 `threads-required = num-test-threads` 설정을 바꾸지 않았다.
slow 표시는 실패나 timeout이 아니며 최종 summary의 4 slow 모두 통과했다.
nextest 0.9.137은 권장 버전과 `ci-duration-observation.junit.report-skipped` 미지원 경고를 냈다.
이번 실행은 default profile이고 모든 테스트 종료 결과를 확인했다. 해당 CI profile의 JUnit 출력은
이번 검증 대상이 아니며, 경고를 없애려고 저장소 설정을 변경하지 않았다.

## 3. 보호 범위와 남은 절차

기존 Studio `pkg/rhwp.js`, `pkg/rhwp_bg.wasm`은 빌드 전 해시를 남겼고
독립 WASM 빌드 후 `sha256sum -c`로 두 파일 모두 일치함을 확인했다.
WASM과 npm 준비는 격리 source의 pkg에서만 수행한다. 기존 개발 서버, 공유 캐시,
Gym 제출물은 삭제·덮어쓰기 하지 않는다.

Linux native·Docker WASM·npm 포함물이 이번 실행 범위다. Windows/macOS 실행 및
frontend·VSIX·브라우저 확장 전체 재빌드는 수행하지 않는다. 승인된 bin-only 영향 분석에 따른다.
Gym canary는 외부 제품 바이너리 지정 경로만 확인하며 전수 벤치마크·제품 정확성 판정이 아니다.

로컬 결과 승인 뒤 원격 push·PR 생성과 exact head의 CI/Oracle 실행 확인이 남는다.
main 적용, 배포, 이슈 close는 이번 로컬 검증 승인의 범위가 아니다.

검증 중 `gh api repos/edwardkim/rhwp/git/ref/heads/devel`로 읽은 원격 SHA는
`c72ad805cc60e4a5cf5689c18b44e7214cec68fe`로 기존 기준선과 같았다. push 직전에는 다시 fetch·확인한다.
최종 정산은 [결과보고서](../report/task_m100_6916_report.md)를 따른다.

PR 본문 초안은 `output/6916/stage3/pr-body.md`에 준비했다. 다음 명령은 **승인 후 실행할 초안**이며
이번 단계에서는 실행하지 않았다. 먼저 최신 base와 충돌·변경 입력을 확인하고 필요한 재검증을 마친다.

```bash
git push -u origin task_m100_6916
gh pr create --repo edwardkim/rhwp --base devel --head task_m100_6916 \
  --title 'fix(mcp): 제품에서 Gym 필수 의존 제거 (#6916)' \
  --body-file output/6916/stage3/pr-body.md \
  --assignee edwardkim --milestone v1.0.0 \
  --label enhancement --label packaging --label mcp
```
