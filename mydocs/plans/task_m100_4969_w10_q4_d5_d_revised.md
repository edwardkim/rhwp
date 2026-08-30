# 수정 수행계획 — Task M100 #4969 W10-Q4-D5-D clean-room causal WASM 복구

- **상위 계획**: [`task_m100_4969_w10_q4_d5.md`](task_m100_4969_w10_q4_d5.md)
- **선행 결과**: [`task_m100_4969_w10_q4_d5_c.md`](../working/task_m100_4969_w10_q4_d5_c.md)
- **선행 분류**: `blocked` — correctness·성능 qualified, causal WASM size unavailable
- **기계 판독 계획**:
  [`w10_q4_d5_d_execution_plan.json`](../tech/investigations/issue-4969/w10_q4_d5_d_execution_plan.json)
- **작성일**: 2026-08-31 KST
- **상태**: 다음 절편 구현 자동 승인
- **제품 변경**: 없음 — disposable A축과 계측·보고만 수행

## 1. 목적

D5-B의 첫 causal A 시도는 B 전체 commit을 `git revert`해 문서와 Studio E2E 충돌까지 함께 끌어들였다.
WASM byte delta가 묻는 것은 Rust 제품 source의 Q4-D 포함 여부이지 계획·보고서·test-only 하네스의 역적용
가능성이 아니다. D5-D는 hard gate를 완화하지 않고, 동일 최신 B tree에서 **WASM에 들어가는 Q4-D Rust source
delta만** 제거한 clean-room A를 만든다.

이 절편은 과거 절대 WASM 값을 재사용하거나 충돌을 손으로 맞추지 않는다. source-only inverse가 기계적으로
적용되고 허용된 파일만 달라지며 A가 정상 빌드될 때만 causal A/B로 인정한다.

## 2. 정확한 A/B 정의

- **B**: 계획 checkpoint 뒤의 clean task branch exact head. 최신 `upstream/devel` merge와 Q4-D/D5 제품
  정정을 모두 포함한다.
- **A**: B exact head의 detached disposable worktree에서 아래 Rust 제품 commit의 `src/**` diff만 최신
  것부터 역적용한 tree.

역적용 대상은 다음 일곱 commit이다.

1. `513068f8e` — malformed vertical rejection 보존 정정
2. `e7f814330` — certified exact-source 반복 준비 비용 정정
3. `c9b551909` — Rust CanvasKit selector
4. `d49400aad` — vertical GlyphRun publication
5. `bb334756b` — source/leaf shadow mapping
6. `992fe4acaf92` — bounded vertical HWP5 table layout
7. `d0948166a` — exact-source-bound dormant owner

`b1a33b6fc`는 Studio TypeScript/E2E만 바꾸며 `rhwp_bg.wasm`에 들어가지 않으므로 WASM A축 역적용 대상에서
제외한다. test-only `e31c5c15d`, 문서 commit, upstream merge commit도 제외한다.

## 3. source-only inverse 보호 규칙

각 commit의 parent→commit diff를 해당 commit이 실제 변경한 `src/**` 경로로 한정해 reverse 적용한다.
순서는 최신→과거로 고정한다.

hard gate는 다음과 같다.

1. reverse patch fuzz·수동 conflict resolution·`--reject`를 허용하지 않는다.
2. A의 tracked diff는 아래 allowlist의 `src/**` 파일만 포함한다.
3. A와 B의 `Cargo.lock`, Cargo feature, build script, toolchain, Dockerfile, sample/font bytes는 동일하다.
4. A는 `cargo check --locked -p rhwp --lib --target wasm32-unknown-unknown`을 통과해야 한다.
5. A diff에 conflict marker, unmerged index, untracked source가 없어야 한다.
6. 역적용 뒤 Q4-D mode 문자열과 vertical GlyphRun publication 진입점이 제품 source에서 사라졌는지 확인한다.

allowlist는 일곱 commit의 Rust 제품 경로 합집합으로 고정한다.

- `src/renderer/mod.rs`
- `src/renderer/shaping_vertical.rs`
- `src/renderer/kerning.rs`
- `src/renderer/layout.rs`
- `src/renderer/layout/table_cell_content.rs`
- `src/renderer/layout/table_layout.rs`
- `src/renderer/layout/table_partial.rs`
- `src/renderer/render_tree.rs`
- `src/renderer/layer_renderer.rs`
- `src/paint/mod.rs`
- `src/paint/builder.rs`
- `src/paint/resources.rs`
- `src/paint/shaping_glyph_vertical.rs`

## 4. Docker WASM 대조

A와 B는 같은 Dockerfile, UID/GID, wasm-pack 0.15.0 정책을 사용한다. compose project를
`rhwp4969d5a`와 `rhwp4969d5b`로 분리해 cargo/wasm-pack/target volume을 공유하지 않는다. 두 project의
wasm image ID가 같지 않으면 비교를 중단한다.

각 축에서 표준 `scripts/wasm-pack-locked.sh --target web` 빌드를 한 번 실행하고 다음을 기록한다.

- optimized `pkg/rhwp_bg.wasm` bytes와 SHA-256
- `B bytes - A bytes`와 A 대비 percent
- build 성공 여부와 wall time
- Docker image ID, Rust/wasm-pack 정책

build wall time은 cache·열 상태가 달라 성능 결론으로 사용하지 않는다. bytes와 digest만 causal size 증적이다.
고유 project volume은 결과 확보 뒤 제거하고 disposable A worktree도 제거한다. B의 ignored `pkg`는 stage하지
않는다.

## 5. 판정

- source-only A가 모든 보호 규칙을 통과하고 A/B build가 성공하면 causal WASM hard gate는 `available`이다.
- delta는 부호와 크기에 관계없이 그대로 공개한다. 사후 크기 SLA를 만들지 않는다.
- correctness·성능 선행 gate와 causal size 영수증이 모두 완전하면 Q4-D를
  `qualified-bounded-subset`으로 갱신한다.
- reverse/compile/image/build 어느 한 gate라도 실패하면 기존 `blocked`를 유지하고 실패 이유를 고정한다.

## 6. 제출 경계

- current branch 제품 source를 변경하지 않는다.
- generated integration suite·manifest, `pkg`, `dist`, Docker volume은 제출하지 않는다.
- 결과 문서와 기계 판독 JSON만 별도 checkpoint로 커밋한다.
- push·PR·GitHub comment는 결과 checkpoint와 분리한다.

## 다음 경계

메인테이너가 다음 절편 구현을 승인했고, 야간 자동 승인 지시에 따라 이 수정 수행계획과 plan checkpoint를
자동 승인한다. checkpoint 뒤에만 disposable A 구성과 Docker A/B 계측을 시작한다.
