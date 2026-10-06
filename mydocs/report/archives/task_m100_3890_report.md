---
kind: report
status: active
canonical: mydocs/plans/archives/task_m100_3890.md
last_verified: 2026-08-03
---

# Task #3890 최종 보고 — rhwp-subsecond feature 게이트 정합과 가드 확장

- Issue: [#3890](https://github.com/edwardkim/rhwp/issues/3890) (M100) — [#3664](https://github.com/edwardkim/rhwp/issues/3664) 분리
- 브랜치 `local/task3890` / 2026-08-03 당일 완결
- 단계 기록: `mydocs/working/task_m100_3890_stage{1,2,3}.md`

## 결과

| 검사 | 착수 전 | 완료 후 |
|---|---|---|
| `cargo check -p rhwp-subsecond` (기본 feature) | **실패** `E0433` | **성공** |
| `cargo build --workspace` | **실패** | **성공** |
| `cargo clippy --workspace -- -D warnings` | **실패** | **통과** |
| CI 가드 범위 | FFI 1크레이트(`-p`) | **워크스페이스 전체** |

## 근인

`tools/rhwp-subsecond/src/main.rs`(전문 3줄)가 feature 와 무관하게
`rhwp::subsecond_dev` 를 불렀다. 그 모듈은 루트에서 `#[cfg(feature = "subsecond-dev")]`
뒤에 있으므로 **기본 feature 로는 컴파일 자체가 불가능**했고, `--workspace` 를 쓰는
모든 명령이 상시 실패했다.

## 해법 선택 — 이슈의 3안 중 2안을 기각

도입 PR([#3255](https://github.com/edwardkim/rhwp/pull/3255) 검토 기록)을 확인하니
**feature 미포함은 실수가 아니라 설계**였다:

> "Subsecond hotpatch 프로덕션 격리 — 3중 확인 완료: ① Rust `subsecond-dev` feature 가
> default 미포함 … 신규 dep `subsecond` optional·dev 전용 → WASM 번들 무영향"

- `default = ["subsecond-dev"]` — **그 격리를 정면으로 깬다. 기각.**
- 워크스페이스 제외 — #3664 가 만든 "가드 밖에서 썩는" 상황을 재생산한다. **기각.**
- **`main.rs` feature 분기 — 채택.** 격리를 지키면서 빌드를 고치는 유일한 길.

feature 없이 실행하면 안내 후 **종료 코드 2**(`cli_commands.md` 사용법 오류 계약).

## 부가 성과 — 가드를 원래 의도 범위로 되돌렸다

#3664 는 이 결함 때문에 CI 가드를 `-p rhwp-native-ffi` 로 좁혀야 했다. 이제 넓혔다:

```yaml
- name: Check workspace members (FFI bindings, tools)
  run: |
    cargo build --workspace
    cargo clippy --workspace --all-targets -- -D warnings
```

| | 종전 | 현재 |
|---|---|---|
| 검사 대상 | FFI 1개 | **3크레이트 전부** |
| 새 워크스페이스 멤버 | 가드에 수동 등재 필요 | **자동 편입** |

## red-check — 두 축 검출

| 되돌린 결함 | 검출 |
|---|---|
| `rhwp-subsecond` feature 분기 제거(#3890 원결함) | 오류 **2건** |
| `rhwp_string_free` 의 `unsafe` 제거(#3664 축) | clippy 오류 **3건** |

**범위를 넓히면서 기존 검출력을 잃지 않았다**는 증명이다.

## 격리 무회귀 — 3층 확인

| 대상 | `subsecond v0.7` |
|---|---|
| `rhwp-subsecond` 기본 feature | **0** |
| 루트 `rhwp` (네이티브) | **0** |
| 루트 `rhwp` (**wasm32 타깃**) | **0** |
| `--features subsecond-dev` | 1 (의도된 경로) |

## 검증

release-test 전체 exit 0 · `--workspace` build/clippy 통과 · fmt · wasm 타깃 체크 ·
워크플로 YAML 통과.

## 교훈

**"고칠 수 없는 제약"으로 보이던 것이 사실은 별도 결함이었다.** #3664 에서 가드를
`-p` 로 좁힌 것은 당시로선 옳은 판단이었지만, 그 제약의 출처를 이슈로 분리해 두었기에
하루 만에 풀 수 있었다. 우회하며 남긴 기록이 다음 작업의 출발점이 된다.

## 남긴 것

`--workspace` 확장의 CI 시간 영향은 PR 실행에서 확인한다 — 로컬 cold 51초를 CI
예측으로 쓰지 않는다.
