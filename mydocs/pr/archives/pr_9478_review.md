# PR #9478 검토

[semanticist21 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9478)

## 검토 버전

- 원 PR head `1e5b0e34add901db069ad8f54e929b9512593060`
- 기준 devel `3a6b7d2b318287361f443b0814702783eb973533`
- 통합 코드 후보 `8550d0698f54fda3d66bee84232e753bb4599185`
- 로컬 회귀 실행 head `b45dfda9a3de46438d2dd14ea473bd96ec6e6c81`
- 검토 브랜치 `review/integration-8506-native-layout`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `1e5b0e34add901db069ad8f54e929b9512593060` | `250bd01a035ee1a8eeb12982cc942dc6f40d9929` |

## 구현과 메인터너 보정

HWP5 EncryptVersion 4의 CFB1 암호화는 입력 비트마다 앞 암호문에 의존합니다. 같은 유도 키·길이·평문으로 저장할 때 결정적인 암호문을 재사용해 큰 그림 stream의 반복 연산을 줄입니다. 4 KiB 이상을 대상으로 하며, 스레드의 암호문 총량이 256 MiB를 초과하면 이전 항목부터 제거합니다.

캐시의 동등성 판정을 SHA-256으로 보정했습니다. HWP5 규격의 SHA-1 키 유도와 CFB1 변환은 그대로 사용합니다. 다른 비밀번호의 암호문 구분과 부분 블록의 열기·재저장을 회귀 검사로 확인합니다. [캐시 구현](../../../crates/rhwp-password-crypto/src/lib.rs) · [암호 형식 통합 계약](../../../tests/password_crypto_multiformat_contract.rs)

## 검증 결과

약 4MB 회귀 입력을 같은 Mac의 Node.js에서 `wasm-pack --target web --no-opt` 비교 패키지로 실행했습니다. 첫 암호 저장은 devel 11.375초, 후보 8.503초였고, 같은 문서의 반복 암호 저장은 devel 20.114초, 후보 0.033초였습니다. 암호 파일은 3,975,680바이트였으며 양쪽 첫 저장과 반복 저장의 SHA-256이 모두 `c12d3e7d67f1d39170861e3e2b5a90480f401caed8eb337911b891c84923adab`로 일치했습니다.

후보에서 `hwp5_large_streams_keep_their_ciphertext_across_open_and_save`를 실행해 통과했습니다. 암호화 crate의 단위 테스트에서 기존 HWP5 외부 벡터 계약도 통과했습니다.

## 병합 후 원 PR 안내

같은 조건의 반복 저장 실측, 암호 결과의 바이트 일치와 캐시 식별자 보정 범위를 안내합니다.

[통합 검증 결과](../assets/integration-8506-20261128/local-verification.json)
