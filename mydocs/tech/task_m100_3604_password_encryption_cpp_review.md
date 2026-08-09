---
kind: decision
status: active
canonical: mydocs/tech/task_m100_3604_password_encryption_cpp_review.md
issue: 3604
last_verified: 2026-08-01
---

# #3604 후속: Hancom C++ 암호 저장 표면 조사

## 조사 대상

`/home/tsjang/hwp-convert`의 C++ direct-link probe, Hancom 2020 Linux shared library
조사 기록, HWPX password container 결과를 검토했다.

## 확인된 사실

- 직접 호출 probe의 저장은 `SaveDocument`/필터 형식 선택 경로다. 이 경로는 HWP, HWPX,
  PDF 변환에는 사용되지만 비밀번호를 설정하는 안정된 공개 ABI 계약은 없다.
- `libHncFoundation.so`에는 `HNC_PBKDF2_SHA1`, SHA-256, `CHncAES` primitive가 있고,
  암호 HWPX는 `SHA-256 start key -> PBKDF2-HMAC-SHA1 -> AES-256-CBC no-padding ->
  raw-DEFLATE` 순서임을 실제 fixture로 확인했다.
- 이 사실은 Rust HWPX 구현의 cipher/KDF 선택을 뒷받침하지만, private C++ ABI로 암호
  저장을 구현하면 Hancom 업데이트마다 재검증해야 한다.

## 결정

암호화·복호화의 제품 구현은 Rust 공통 모듈에서 파일 형식 계약을 직접 생성한다. C++ probe는
한컴 2020 교차 검증 및 모르는 계약의 정적/동적 조사에만 사용한다.
