---
kind: plan
status: active
issue: 3604
last_verified: 2026-08-01
---

# #3604 후속: HWP3·HWP5·HWPX 비밀번호 암호화 저장 계획

## 목표

비밀번호로 연 HWP3, HWP5, HWPX 문서를 평문으로 강하하지 않고, 지정한 새 비밀번호로
다시 암호화해 저장할 수 있게 한다. HWP3 원형 저장, HWP5 `EncryptVersion=4`, HWPX ODF
AES-256-CBC/PBKDF2를 모두 대상으로 한다.

## 범위와 순서

1. `src/password_crypto.rs` 하나에 세 포맷의 비밀번호 암호화·복호화 primitive와 패키지
   조립을 둔다. parser/serializer의 포맷별 파일은 얇은 호출 어댑터만 유지한다.
2. HWP5 암호화 저장을 먼저 연결한다. 기존 암호 HWP5 재열기와 올바른/오류 비밀번호
   회귀 테스트로 CFB 스트림 범위를 고정한다.
3. HWP3 원형 암호화와 HWPX ODF package 암호화를 같은 모듈에 추가한다.
4. 세 포맷의 자체 재열기, 한컴 2020 교차 열기, 평문 누출 검사를 완료한 뒤 CLI와 MCP
   출력 비밀번호 입력을 추가한다.

## 비범위

- DRM, 배포용 문서, HWP5 `EncryptVersion` 1~3은 지원하지 않는다.
- 이번 첫 단계에서는 CLI, WASM, MCP schema를 바꾸지 않는다.
- 비밀번호를 명령행 인자, JSON 응답, 로그, 작업 문서에 기록하지 않는다.

## 수용 기준

- HWP5 출력은 header encrypted flag와 `EncryptVersion=4`를 갖고, 평문 parser는
  비밀번호 필요 오류, 올바른 비밀번호 parser는 원문과 동등한 IR을 반환한다.
- HWP3/HWPX도 동일한 올바른/오류/누락 비밀번호 계약을 가진다.
- 암호문 파일에서 고유 본문 평문이 발견되지 않는다.
- 한컴 2020 Linux가 세 출력 형식을 비밀번호로 열 수 있음을 별도 단계에서 실증한다.

## 단계

| 단계 | 내용 | 상태 |
|---|---|---|
| 4 | C++ 조사, 공통 모듈 도입, HWP5 암호화 저장 | 완료 |
| 5 | HWP3 원형 암호화와 HWPX ODF 암호화 | 대기 |
| 6 | 한컴 교차 검증 및 CLI/MCP 출력 비밀번호 표면 | 대기 |
