---
kind: investigation
status: active
canonical: mydocs/tech/investigations/issue-8745/issue-body.md
last_verified: 2026-11-07
---

# 이슈 본문 초안 — 암호 문서 보안 경계

## 문제

암호 문서 처리에 대한 CodeQL 경고를 개별 진단 로그 삭제·오탐 분류로 처리했으나 다른 sink에서
반복된다. PR #8739의 Rust analysis `2117586342`(CodeQL 2.27.4, merge
`8e8a09daf88cc6cd91d683c409757c08ddbc1fde`)에는 cleartext logging 107개 결과가 있다.
106개는 암호 문서 constructor 반환값 전체, 1개는 암호화 저장 결과의 바이트 수에서 시작한다.
이를 암호 값 자체의 유출 107건으로 해석하지 않는다.

별도로 `service::OpenOptions`의 자동 Debug가 암호를 가리지 않는 것을 제한 재현했다.
운영 로그 유출을 확인했다는 주장은 아니며 공개 API의 문자열 표현 경계를 보완한다.

## 선행 작업

- #6731·#6748: Studio 암호 open command와 metadata query 분리. Rust 분석에는 별도 실증 필요.
- #8517·#8548: 개별 조판 진단 제거와 일부 실제 암호 fixture 회귀. 반환값 source는 유지.
- #8739: #478·#483·#484를 인자·flow 확인 후 오탐 분류. source flow 자체는 남음.
- #7754의 암호 HWPX 피델리티 문제와는 원인·완료 조건이 다르다.

## 수행 범위

1. 암호 옵션 Debug 가림과 공개 API 회귀.
2. 암호 소비 command·완성 문서 query 경계의 Rust 분석 실험과 호환 구현.
3. CLI 완료 정보·저장 결과 경계 확인, 열기·저장·실패 복구 유지.
4. 하드코딩 결과 16건의 공개 테스트 벡터·작업 버퍼 구분과 보호 계약 확인.
5. 실제 암호 로그와 운영 키를 검출하는 양성 대조, 목표 flow 제거의 음성 대조, 필수 로컬 검증.

CodeQL 탐지 제외·광범위 sanitizer·자동 dismiss·조판 변경·암호 형식 변경은 범위에 포함하지 않는다.
변경 전후 같은 분석기·query의 SARIF로 효과를 확인하며 기존 dismissed 상태나 check 성공만으로
완료하지 않는다. 합성 비밀과 공개 fixture만 사용하고 운영 암호를 수집하지 않는다.

## 완료 조건

- Debug·오류·진단 채널로 암호가 노출되지 않는다.
- 정상 암호·오암호·재시도·평문·암호화 저장 재열기의 제품 계약을 보존한다.
- 대상 반환값 기반 flow가 사라지고 실제 비밀의 양성 대조 탐지는 유지된다.
- 기존 형식·공개 벡터의 남은 결과는 개별 근거와 실제 결함 여부를 기록한다.
- 로컬 필수 검사 및 승인된 PR의 CodeQL 분석을 통과하고, 이슈 종료의 독립 보안 분석 조건을 충족한다.
