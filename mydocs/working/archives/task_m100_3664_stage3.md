---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3664.md
last_verified: 2026-08-03
---

# Task #3664 Stage 3 보고 — 전체 검증

| 게이트 | 결과 |
|---|---|
| `cargo test --profile release-test --tests` 전체 | **exit 0** — 워크스페이스 편입 무회귀 |
| `cargo clippy --all-targets -- -D warnings` | **exit 0** |
| `cargo fmt --check` | 통과 |
| `cargo build -p rhwp-native-ffi` | 성공 (1.95초) |
| `cargo clippy -p rhwp-native-ffi --all-targets -- -D warnings` | 통과 |
| 워크플로 YAML 문법 | 통과 |

## 워크스페이스 편입의 부수 영향 — 실측

계획서 §5 가 지목한 위험을 확인했다.

| 항목 | 실측 |
|---|---|
| `Cargo.lock` 변경 | **7줄** — `rhwp-native-ffi` 등록만. **외부 의존 0 추가**(의존이 `rhwp` 코어 하나뿐) |
| 빌드 시간 | FFI 단독 1.95초. 전체 테스트 exit 0, 체감 증가 없음 |
| `cargo test` 대상 | FFI unittests 바이너리가 추가됨(현재 테스트 0개이나 타깃은 편입됨) |
| 캐시 영향 | lock 해시 변경으로 rust-cache 새 세대 1회 발생 예상. 어제 도입한 [#3684](https://github.com/edwardkim/rhwp/issues/3684) 스윕이 구 세대를 정리하므로 누적되지 않는다 |

## 범위 밖 (기록)

- **`rhwp-subsecond` 는 `cargo clippy --workspace` 에서 실패**한다
  (`E0433: could not find subsecond_dev` — feature 조건부라 기본 빌드에서 깨짐).
  이 작업과 무관한 기존 성질이며, 가드를 `-p` 로 좁힌 이유다. 별도 정리가 필요하면
  후속 이슈로 다룬다.
- **C#·Swift 래퍼 자체 검증**은 이번 범위 밖이다. 크레이트가 살아났으므로 그 위에서
  별도로 확인해야 하며, 필요 시 후속 이슈로 분리한다.
