# 검증 실행과 소스 동일성

실제 실행한 Rust/Native/WASM 제품 소스는 `c031a38a41670f5e4f6ab0268ef46c62ab1445f5`다. 이후 Rust 소스·검사·프로그램 자원·Studio 제품 경로는 Git object 단위로 동일함을 확인했다. 검증 호스트는 별도 `3ededa625581f4a4a052ef912dc30c0317fec0ed`이며, 이 변경 뒤 WASM 출력은 별도로 다시 산출했다.

| 검증 | 실제 결과 |
| --- | --- |
| release-test 전체 nextest | 12,424 PASS, 0 FAIL, 45 skip; 테스트 581.283초, 컴파일 포함 1303.629초 |
| 생성 suite 배정 정정 뒤 실제 컴파일·수집 | 12,469 선언, 12,424 실행 대상, 45 제외; 이전 실제 PASS 함수 집합과 동일 |
| 최신 Visual Sweep Python | 240 PASS |
| 현재 글꼴 공급·원본/암호 계약 Node | 34 PASS |
| `authenticated-final-fmt` | PASS, exit 0; 30.512초 |
| `authenticated-native-build` | PASS, exit 0; 2.347초 |
| `authenticated-fresh-wasm` | PASS, exit 0; 293.03초 |
| `authenticated-full-nextest-minfont` | PASS, exit 0; 1303.629초 |
| `authenticated-clippy-native` | PASS, exit 0; 137.778초 |
| `authenticated-clippy-wasm` | PASS, exit 0; 309.882초 |
| `authenticated-workspace-build` | PASS, exit 0; 997.325초 |
| `authenticated-clippy-all-targets` | PASS, exit 0; 614.663초 |
| `authenticated-skia-lib-minfont` | PASS, exit 0; 399.218초 |
| `authenticated-skia-png` | PASS, exit 0; 281.938초 |
| `authenticated-skia-pdf` | PASS, exit 0; 61.333초 |
| `authenticated-source-frame` | PASS, exit 0; 21.919초 |
| `authenticated-skia-missing-picture` | PASS, exit 0; 35.623초 |
| `authenticated-doc-tests` | PASS, exit 0; 919.946초 |
| `authenticated-release-build` | PASS, exit 0; 2186.625초 |
| `authenticated-release-lib` | PASS, exit 0; 1451.703초 |

원 main driver의 manifest 배정 drift 실패는 삭제하지 않았다. 정본 prepare 뒤 fmt·manifest·unit tier가 통과했고, 실제 재컴파일한 수집 집합을 이전 실행 결과와 대조하여 누락·추가가 없음을 확인했다. 검사 제거·완화로 통과한 것이 아니다.

Mac fresh WASM은 locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화 통과로 주장하지 않는다. 전체 시각 판정·직접 판독은 이 표와 별도 게이트다.

[실행 명령·종료 결과·Git object 근거](gate-results.json)
