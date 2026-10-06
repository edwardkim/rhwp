---
kind: working
status: completed
issue: 3604
stage: 5
last_verified: 2026-08-01
---

# #3604 Stage 5: HWP3·HWPX 공통 암호 모듈 이관

## 목표

- HWP3 DES-ECB, raw-DEFLATE, CRC32/ISIZE 검증을 `src/password_crypto.rs`로 이관한다.
- HWPX ODF manifest, SHA-256 start key, PBKDF2-HMAC-SHA1, AES-256-CBC no-padding
  패키지 암·복호화를 같은 공통 파일에 둔다.
- HWP3/HWPX parser는 형식별 오류 변환과 호출만 담당한다.
- HWPX serializer에 password 저장 entrypoint를 추가한다.

## 구현 계획

1. parser의 HWP3/HWPX 암호 처리 public 계약과 오류 유형을 먼저 확인한다.
2. 공통 오류 유형과 HWP3/HWPX 암·복호화 함수를 추가하고 parser를 얇은 adapter로 바꾼다.
3. 암호 HWPX 저장 후 일반 열기 거부, 올바른 비밀번호 재열기, 오류 비밀번호 거부를
   integration test로 검증한다.
4. HWP3는 기존 보호 fixture 재열기와 재암호화 가능한 raw HWP3 payload 계약을 검증한다.
5. 한컴 2020 교차 검증은 Rust 계약 테스트가 끝난 뒤 별도 Stage에서 수행한다.

## 범위 밖

- MCP tool·client·배포 변경은 이 Stage에서 다루지 않는다.
- CLI 암호 입력과 외부 API 공개는 모든 Rust 형식 계약이 고정된 뒤 Stage 6에서 다룬다.

## 구현 결과

- HWP3의 DES-ECB, HWP5의 AES bit-CFB, HWPX의 ODF AES-256-CBC 구현을
  `src/password_crypto.rs` 한 파일로 통합했다.
- `src/parser/hwp3/crypto.rs`와 `src/parser/hwpx/crypto.rs`는 알고리즘 없이
  공통 오류를 기존 parser 오류 유형으로 변환하는 adapter가 됐다.
- `encrypt_hwp3_password_document()`는 압축되지 않은 raw HWP3에서
  256-byte confirmation 영역, raw-DEFLATE, CRC32/ISIZE, DES-ECB payload를 조립한다.
- `encrypt_hwpx_package()`는 header, section, settings, preview, BinData와
  master-page XML을 암호화하고, ODF manifest 및 ZIP `Stored` 암호문 엔트리를 쓴다.
- `serialize_hwpx_with_password()`로 Document IR의 HWPX password 저장 경로를 추가했다.

## 테스트 결과

| 명령 | 결과 | 해석 |
|---|---|---|
| `cargo check` | 성공 | 새 난수 의존성과 공통 HWP3/HWPX 모듈 컴파일 확인 |
| `cargo check --lib --target wasm32-unknown-unknown` | 성공 | `getrandom`의 wasm JS backend를 포함한 공통 library 호환성 확인 |
| `cargo test --lib password_crypto::tests --no-fail-fast` | 2 passed | HWP5 외부 벡터와 부분 block 왕복 유지 |
| `cargo test --test hwp3_password_fixture --no-fail-fast` | 11 passed | 기존 실제 HWP3 암호 열기·CLI 회귀 유지 |
| `cargo test --test hwpx_password_fixture --no-fail-fast` | 3 passed | 기존 실제 ODF HWPX 암호 열기·CLI 회귀 유지 |
| `cargo test --test password_crypto_multiformat_contract --no-fail-fast` | 2 passed | 실제 HWP3 재암호화와 HWPX 새 암호 패키지/serializer 재열기 확인 |
| `cargo test --test password_encryption_write_contract --no-fail-fast` | 1 passed | HWP5 password 저장 계약 유지 |

참고: `cargo check --target wasm32-unknown-unknown`의 binary 대상은 기존
`src/main.rs`가 wasm API에 없는 네이티브 전용 메서드를 호출해 실패한다. 이 Stage의
`password_crypto` library와 새 난수 의존성은 `--lib` 대상에서 정상 컴파일됐다.

## 다음 Stage

Hancom Office 2020 직접 열기 교차 검증과 외부 CLI/MCP 노출은 Stage 6에서 진행한다.
