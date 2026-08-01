---
kind: working
status: completed
issue: 3604
stage: 4
last_verified: 2026-08-01
---

# #3604 Stage 4: 공통 암호 모듈과 HWP5 암호화 저장

## 시작 상태

- 기준: `upstream/devel` `c588c8240` 위에 #3604 MCP 비밀번호 입력 커밋을 rebase했다.
- 기존 HWP3/HWP5/HWPX 복호화 코드는 각각 `src/parser/hwp3/crypto.rs`,
  `src/parser/crypto.rs`, `src/parser/hwpx/crypto.rs`에 나뉘어 있다.
- HWP5 serializer는 암호 문서를 저장할 때 암호 flag와 `EncryptVersion`을 제거한다.

## C++ 조사 결과

- `/home/tsjang/hwp-convert`의 Hancom 직접 호출 C++ probe는 `SaveDocument` 기반 형식
  변환 ABI를 사용한다.
- HWPX password open의 Hancom primitive와 ODF 계약은 `mydocs/tech`에 기록돼 있으나,
  재현 가능하고 안정적인 비밀번호 설정/암호 저장 ABI는 확인되지 않았다.
- 따라서 C++은 HWPX ODF no-padding과 PBKDF2-HMAC-SHA1의 교차 근거로만 사용하고,
  실제 암호 저장은 Rust 형식 writer로 구현한다.

## 구현 계획

1. HWP5 bit-CFB 암호화 primitive를 공통 모듈로 이동하고 제품 API로 공개한다.
2. HWP5 CFB serializer에 password 저장 entrypoint를 추가한다.
3. parser와 serializer의 focused 회귀 테스트를 추가한다.

## 성공 기준

- 암호 저장 HWP5가 올바른 비밀번호로 재파싱된다.
- 잘못된 비밀번호와 없는 비밀번호가 각각 기존 오류 계약을 유지한다.
- 평문 HWP5 저장 동작은 바뀌지 않는다.

## 테스트 결과

| 명령 | 결과 | 해석 |
|---|---|---|
| `cargo fmt --check` | 성공 | 공통 모듈·serializer·계약 테스트 형식 확인 |
| `cargo test --lib password_crypto::tests --no-fail-fast` | 2 passed | HWP5 외부 암호문 벡터와 마지막 부분 블록 왕복을 공통 모듈에서 확인 |
| `cargo test --lib parser::crypto::tests --no-fail-fast` | 3 passed | 배포용 AES 회귀와 HWP5 압축 상한 wrapper 유지 |
| `cargo test --test password_encryption_write_contract --no-fail-fast` | 1 passed | 새 HWP5가 `encrypted` flag와 `EncryptVersion=4`를 기록하고, 누락·오류 암호를 거부하며 올바른 암호로 재열림 |

## 결과

- `src/password_crypto.rs`를 추가하고 HWP5 SHA-1 key derivation과 AES bit-CFB
  암·복호화를 그 파일로 이관했다.
- `src/parser/crypto.rs`는 배포용 ViewText 계약 및 공통 HWP5 모듈 호출만 유지하도록
  축소했다. HWP5 비밀번호 알고리즘의 중복 구현은 제거했다.
- `serialize_hwp_with_password()`는 HWP5 header의 flag bit 1과 `EncryptVersion=4`를
  설정하고 DocInfo, BodyText, BinData, Scripts, 고아 BinData를 암호화한다.
- HWP3와 HWPX의 기존 로직 이관 및 암호 저장은 Stage 5에서 진행한다.
