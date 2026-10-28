---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3890.md
last_verified: 2026-08-03
---

# Task #3890 Stage 3 보고 — 전체 검증

| 게이트 | 결과 |
|---|---|
| `cargo test --profile release-test --tests` 전체 | **exit 0** |
| `cargo build --workspace` | 성공 (5.98초) |
| `cargo clippy --workspace --all-targets -- -D warnings` | **통과** ← 이 이슈의 목표 |
| `cargo fmt --check` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 (51.9초) |
| 워크플로 YAML 문법 | 통과 |

## 격리 3층 확인

계획서 §6 의 최대 위험(프로덕션 격리 회귀)을 세 지점에서 확인했다.

| 대상 | `subsecond v0.7` 포함 |
|---|---|
| `cargo tree -p rhwp-subsecond` (기본 feature) | **0** |
| `cargo tree -p rhwp` (루트, 네이티브) | **0** |
| `cargo tree --target wasm32 -p rhwp` | **0** |
| `cargo tree -p rhwp-subsecond --features subsecond-dev` | 1 (의도된 경로) |

#3255 가 3중 확인한 "dev 전용" 성질이 그대로다. **개발 도구가 프로덕션 번들에 섞이지
않는다.**

## 비용 관찰

`--workspace` 확장으로 clippy 대상이 늘었다(로컬 cold 51초). Lint job 은
`shared-key: lint` 캐시를 타므로 warm run 영향은 작을 것으로 보이나, **실제 CI 시간은
PR 실행에서 확인한다** — 로컬 수치를 CI 예측으로 쓰지 않는다.
