---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3890.md
last_verified: 2026-08-03
---

# Task #3890 Stage 2 보고 — CI 가드를 워크스페이스 전체로 확장

## 변경

`.github/workflows/ci.yml` Lint job 의 가드를 넓혔다.

```diff
- - name: Check Native FFI bindings
-   run: |
-     cargo build -p rhwp-native-ffi
-     cargo clippy -p rhwp-native-ffi --all-targets -- -D warnings
+ - name: Check workspace members (FFI bindings, tools)
+   run: |
+     cargo build --workspace
+     cargo clippy --workspace --all-targets -- -D warnings
```

#3664 는 `rhwp-subsecond` 의 상시 실패 때문에 `-p` 로 좁혀야 했다. Stage 1 이 그
결함을 고쳤으므로 원래 의도한 범위로 되돌린다. 주석에 두 이슈의 경위를 남겼다.

## 무엇이 달라지나

| | 종전(`-p`) | 현재(`--workspace`) |
|---|---|---|
| 검사 대상 | `rhwp-native-ffi` 만 | **워크스페이스 3크레이트 전부** |
| 새 멤버 추가 시 | 가드에 수동 등재 필요 | **자동 편입** |

앞으로 워크스페이스에 크레이트가 추가되면 별도 조치 없이 검사 대상이 된다. 깨진 채
들어오면 CI 가 막는데, 그것이 이 가드의 목적이다.

## red-check — 두 축 모두 검출

| 되돌린 결함 | 검출 |
|---|---|
| `rhwp-subsecond` feature 분기 제거(#3890 원결함 재현) | **오류 2건** |
| `rhwp_string_free` 의 `unsafe` 제거(#3664 축 유지 확인) | **clippy 오류 3건** |

넓힌 가드가 **새로 고친 축과 기존 축을 함께** 잡는다. 범위를 넓히면서 기존 검출력을
잃지 않았다는 증명이다.

## 검증

- 워크플로 YAML 문법 통과.
- 가드 명령 로컬 실측: `cargo build --workspace` 5.98초,
  `cargo clippy --workspace --all-targets -- -D warnings` 51.04초.
- 복원 후 재통과 확인.

## 비용 관찰

`--workspace` 로 넓히면서 clippy 대상이 늘었으나(51초), Lint job 은 `shared-key: lint`
캐시를 타므로 warm run 에서는 대부분 재사용된다. 실제 CI 시간 영향은 Stage 3 의 PR
실행에서 확인한다.
