---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3890.md
last_verified: 2026-08-03
---

# Task #3890 Stage 1 보고 — feature 게이트 정합

## 결과 — `--workspace` 가 통과한다

| 조합 | 착수 전 | 완료 후 |
|---|---|---|
| `cargo check -p rhwp-subsecond` (기본 feature) | **실패** `E0433` | **성공** |
| `-p ... --features subsecond-dev` (네이티브) | 성공 | 성공 |
| `-p ... --target wasm32 --features subsecond-dev` | 성공 | 성공 |
| **`cargo clippy --workspace -- -D warnings`** | **실패** | **통과** — 3크레이트 전부 검사 |

## 구현

`tools/rhwp-subsecond/src/main.rs` 를 feature 로 분기했다.

- `#[cfg(feature = "subsecond-dev")]` — 종전 동작(`link_wasm_exports`) 그대로.
- `#[cfg(not(...))]` — 안내 후 `exit(2)`. 실제 종료 코드 **2** 확인
  (`cli_commands.md` 의 "사용법 오류" 계약).

모듈 주석에 이 크레이트가 왜 feature 뒤에 있는지(#3255 프로덕션 격리)와 무엇이
깨져 있었는지를 남겼다.

## 최대 위험 — 격리 회귀: 발생하지 않음

계획서 §6 이 지목한 위험을 의존 그래프로 확인했다.

| 대상 | `subsecond v0.7` 포함 |
|---|---|
| `cargo tree -p rhwp-subsecond` (기본) | **0** |
| `cargo tree -p rhwp-subsecond --features subsecond-dev` | 1 |
| `cargo tree -p rhwp` (루트 기본) | **0** |

#3255 가 3중 확인한 "dev 전용" 성질이 그대로다. **기본 빌드에는 개발 도구가 섞이지
않는다.**

## 왜 2안·3안을 쓰지 않았나 (재확인)

- `default = ["subsecond-dev"]` — 위 격리를 정면으로 깬다.
- 워크스페이스 제외 — #3664 가 만든 "가드 밖에서 썩는" 상황을 재생산한다.

## 검증

`cargo fmt --check` 통과. 4조합 빌드 전부 확인.

## 다음

Stage 2 — #3664 가 `-p` 로 좁혔던 CI 가드를 `--workspace` 로 넓힐 수 있게 됐다.
red-check 로 검출력을 증명한다.
