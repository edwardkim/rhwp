---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3664.md
last_verified: 2026-08-03
---

# Task #3664 Stage 2 보고 — CI 가드

## 배치 결정 — Lint job, `-p` 지목

`.github/workflows/ci.yml` 의 Lint job(fmt·clippy·WASM check) 끝에 추가했다.

```yaml
- name: Check Native FFI bindings
  run: |
    cargo build -p rhwp-native-ffi
    cargo clippy -p rhwp-native-ffi --all-targets -- -D warnings
```

### 왜 `--workspace` 가 아니라 `-p` 인가 (실측 근거)

워크스페이스 편입(Stage 1)만으로는 부족하다는 것이 실측으로 드러났다.

| 명령 | FFI 검사 여부 |
|---|---|
| `cargo clippy` (현행 CI) | **안 봄** — 루트 크레이트만 |
| `cargo clippy --workspace` | 봄. 그러나 **`rhwp-subsecond` 에서 별도 실패**(`E0433: could not find subsecond_dev` — feature 조건부라 기본 빌드에서 깨진다. 이 축과 무관한 기존 성질) |
| **`cargo clippy -p rhwp-native-ffi`** | **봄, 부작용 없음** ← 채택 |

`--workspace` 로 넓히면 무관한 크레이트의 기존 실패가 이 가드를 상시 red 로 만든다.
`-p` 지목이 정확하다.

### 왜 Lint job 인가

- 무거운 test job 에 넣으면 피드백이 늦다.
- 별도 job 은 러너를 하나 더 쓴다(오늘 #3684 캐시 축 작업과 상충).
- Lint job 은 이미 fmt·clippy 를 도는 곳이라 성격이 같고, FFI 의존이 `rhwp_core`
  하나뿐이라 캐시(`shared-key: lint`)를 그대로 탄다.

## red-check — 두 축 모두 검출력 증명

가드가 "통과하는 것"이 아니라 "결함을 잡는 것"인지 확인했다.

| 되돌린 결함 | 결과 |
|---|---|
| `cell_path` 인자 제거(원래 결함 재현) | **오류 3건 검출** → 복원 후 빌드 성공 |
| `unsafe` 표시 제거 | **clippy 오류 3건 검출** → 복원 후 통과 |

컴파일 축과 위생 축이 각각 독립적으로 잡힌다. 이 이슈가 생긴 원인(가드 부재)이
같은 형태로 재발하면 CI 가 실패한다.

## 검증

- 워크플로 YAML 문법 파싱 통과.
- 로컬에서 가드 명령 2개 모두 통과(build 1.84s, clippy 0.23s).
