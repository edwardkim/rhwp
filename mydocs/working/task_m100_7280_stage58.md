# Task #7280 Stage 58 — devel 통합본 최종 검증

- Issue: #7280. 이전: [Stage57](task_m100_7280_stage57.md).
- 승인: 작업지시자의 “진행을 승인합니다.”에 따라 전체 회귀·남은 lint/build·Native/fresh WASM 대조 진행.
- 검증 head: `29130d5397da3dc3cd3d1d9416b153584e1a4f41`.
- 비교 base: fetch로 재확인한 `1966af77fa8046c844d654b157b5168baad8a30e`.
- 제품 내용은 merge `6003fe35689ee9be02d2b05737430e648b07361e`와 동일하다.
- 상태: **workspace 빌드 중 디스크 부족으로 중단, 공간 확보 승인 대기**.
  제품 변경·baseline 완화·push·PR·댓글은 수행하지 않았다.

## 환경과 증적

clean review worktree `rhwp-review-7280-r3k`를 위 head로 고정했다. 최초 baseline worktree와
기존 증적은 보존한다. 고정 Cargo target은 주 저장소의 `target/pr-review`이며 Cargo 작업은
순차 실행한다. 시작 시 16 CPU/31GiB RAM, 디스크 여유 약 6.6GiB를 확인했다.
빌드 동시성 4, 테스트 동시성 8을 사용한다. 산출물이나 다른 worktree를 삭제하지 않는다.

증적 접두사: `output/7280/stage58-integration/`. `lint.sh`는 prepare → fmt → Native Clippy →
WASM Clippy → workspace build → workspace/all-target Clippy → 고정 base 정책 검사 순서다.
전체 회귀·Native Skia·fresh Docker WASM·시각 대조의 실제 실행 결과와 한계는 아래에 보완한다.

## 실행 결과와 중단 지점

| 검사 | 결과 |
| --- | --- |
| prepare / fmt check | PASS |
| Native Clippy `--locked … -- -D warnings` | PASS |
| WASM Clippy `--locked -p rhwp --lib --target wasm32-unknown-unknown … -- -D warnings` | PASS (52.89초) |
| workspace build `--locked --workspace` | **환경 실패**: 디스크 부족, exit 101 |
| manifest / unit-tier `--check --base-ref 1966af77…` | PASS. 중단 후 경량 검사만 별도 실행 |
| workspace all-target Clippy | 선행 build 실패로 미실행 |
| 전체 release-test / Native Skia 3종 | 미실행 |
| 새 Native/fresh Docker WASM 및 최신 base와 출력 대조 | 미실행 |

`build-workspace.log`에서 `rhwp-q-kit` 컴파일의 `No space left on device (os error 28)`와
`rhwp-q-font-layout-evidence` 링크의 Bus error를 확인했다. 조판 계약 실패나 Rust 소스 오류로
분류하지 않는다. 공간 부족 상태의 연속 빌드나 출력 생성을 중단했으며 Cargo/Rust 잔여 프로세스는 없다.

중단 후 `/dev/sdd` 가용 공간은 약 **220MiB**다. 읽기 전용 계측 결과
`target/pr-review/debug/incremental` 약 **63GiB**, `debug/deps` 약 **122GiB**,
`debug/build` 약 **483MiB**다. 이 경로는 공유 검증 캐시이므로 승인 없이 삭제하거나
고정 target 경로를 이동하지 않았다. 다른 worktree·원본·이전 증적도 보존했다.

`manifest.log`는 1,399 sources / 28 suites + 20 exceptions,
`unit-tier.log`는 4,205 tests / 298 modules / cfg support 28을 기록한다.
Stage57의 집중 403건 통과는 같은 제품 내용의 선행 증거이지만, 이번 전체 회귀를 수행했다는
의미가 아니다. 새 시각 산출물이나 시각 통과 판정도 없다.

## 재개 조건

작업지시자가 공간을 확보하거나, 구체적인 재생성 가능 캐시 정리 범위를 승인해야 한다.
정리 후보는 `/home/edward/mygithub/rhwp/target/pr-review/debug/incremental`의 증분 캐시다.
해당 캐시를 정리하면 다음 컴파일 시간이 늘지만 소스·샘플·검증 보고서는 제거 대상이 아니다.
실제 삭제는 별도 승인 후 대상 경로와 다른 빌드의 사용 여부를 재확인해 수행한다.

공간 확보 후 동일 base/head를 확인하고 workspace build부터 다시 실행한다. 이후 all-target Clippy,
Native/fresh WASM 영향 출력 대조 및 전체 회귀·Native Skia gate를 마쳐야 한다.
현재 상태는 PR 생성/제출 준비 완료가 아니다.
