---
kind: working-note
status: completed
issue: 5605
---

# #5605 Stage 1 - 환경 인식형 Rust test thread 지침

## 관측

`target/pr-review`을 각 실행 전에 비운 macOS Native Skia lib 측정에서 다음 결과를 얻었다. Cargo registry와
OS 파일 cache는 유지됐으므로 target-cold 측정이며, 다른 host에 고정값으로 일반화하지 않는다.

| libtest threads | 전체 시간 | 핵심 3,950개 테스트 |
| ---: | ---: | ---: |
| 1 | 190.64초 | 37.94초 |
| 4 | 159.81초 | 16.91초 |
| 8 | 144.37초 | 11.85초 |

현재 host는 논리 CPU 10개, 성능 코어 4개였다. 이 표본에서는 8 threads가 빨랐지만, 컴파일·메모리·동시
부하가 다른 host에는 같은 값이 적합하다는 뜻이 아니다.

## 명령 의미 보정

`cargo test --features native-skia skia --lib`의 `skia`는 Cargo feature가 아니라 test filter다. 이 명령은
236.49초에 58개만 통과하고 4,087개를 filter했다. 따라서 전체 Native Skia lib 회귀는
`cargo test --features native-skia --lib`로 실행하고, libtest 동시성을 조정할 때만 `--` 뒤에
`--test-threads <현재 환경에 맞는 값>`을 둔다.

## 적용

- 활성 가이드의 고정 nextest thread 수를 제거하고 host 기본 동시성을 사용한다.
- 조정이 필요하면 사용자가 논리 CPU·메모리·동시 작업을 확인해 값을 선택하도록 안내한다.
- 완료된 계획·보고서의 과거 명령과 수치는 증적이므로 변경하지 않는다.
