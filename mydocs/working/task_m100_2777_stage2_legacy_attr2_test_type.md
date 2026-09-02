---
kind: working
status: active
issue: 2777
stage: 2
last_verified: 2026-08-14
---

# #2777 Stage 2: 구규약 이관 회귀 단정의 정수 타입 고정

## 배경

Stage 1의 HWP5 구규약 이관 회귀 단정은 비트 조합 리터럴에 타입을 지정하지 않아,
`to_le_bytes()` 호출 시 Rust가 정수 타입을 추론하지 못했다. 프로덕션 빌드는 성공했지만
`#[cfg(test)]` 모듈을 포함하는 단위 테스트 빌드가 중단됐다.

## 변경

- 구 `attr2` 비트 조합을 `u32`로 명시해 HWP5 레코드의 4바이트 little-endian 값으로 안정적으로 기록한다.

## 검증 계획

- Stage 1에서 추가한 HWPX 파싱, HWPX 직렬화, HWP5 구규약 이관 단위 테스트를 다시 실행한다.
