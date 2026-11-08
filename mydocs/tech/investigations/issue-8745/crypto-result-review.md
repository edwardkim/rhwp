---
kind: investigation
status: active
canonical: mydocs/tech/investigations/issue-8745/crypto-result-review.md
last_verified: 2026-11-07
---

# 기존 암호 테스트·키 유도 결과의 구분

대상은 analysis `2117586342`의 logging 외 18개 결과다. [원장](crypto-result-inventory.json)에
각 alert·rule·함수·현재 기준 source blob을 기록했다. 이 검토는 기존 GitHub 분류를 변경하지 않는다.

| 구분 | Alert | 실제 목적과 처리 |
| --- | --- | --- |
| 테스트 암호 | #27·#37–#42·#124·#125·#151·#173–#175 | `#[cfg(test)]` 안의 공개·합성 입력. 오암호, 지원 버전, 압축 해제 한도, 전체 스트림 왕복, 외부 벡터 복호화를 검증한다. 운영 인증 정보가 아니며 정상 고정 벡터를 무작위 값이나 다른 인코딩으로 바꾸지 않는다. |
| 공개 NIST AES 벡터 | #136 | `distribute_aes_matches_nist_vector`의 알려진 입력·출력이다. 비교 기준을 유지한다. |
| HWP5 작업 배열 | #176 | `derive_hwp5_key`의 `[0_u8; 16]`은 SHA-1 digest 앞 16바이트로 전부 덮어쓴 뒤 반환한다. 운영 고정 키로 사용되는 배열이 아니다. |
| HWP3 작업 배열 | #177 | `derive_hwp3_legacy_des_key`가 rolling password 결과의 56개 비트를 8바이트에 조립한다. 키 유도 규칙에 따른 parity 위치의 초기 0과 고정 키를 구분한다. |
| HWPX 키 유도 전처리 | #205 | `derive_hwpx_key`의 SHA-256 결과는 최종 저장/인증 hash가 아니라 manifest salt·iteration·PRF를 쓰는 PBKDF2의 입력이다. 이 전처리만 임의의 다른 KDF로 바꾸면 기존 문서를 열 수 없다. |
| HWP3 legacy DES | #178 | 실제 DES 사용을 가리킨다. 이를 현대적 암호 알고리즘이라고 주장하지 않는다. 기존 HWP3 호환 기능과 보안 한계의 구분을 유지한다. |

테스트 14개 결과는 위 첫 두 행의 합이다. 반복된 입력·출력에서 동일 공개 값이 두 결과로
보고된 경우도 별도로 센다. `src/**` 안의 테스트라는 이유로 분석 경로를 제외하지 않는다.

실행 검증은 이 함수들이 속한 기존 crypto·parser 회귀를 재사용한다. 16개 결과를 0개로 만드는
것을 보안 개선의 목표로 두지 않는다. 실제 민감 입력의 진단 경계와 새로운 운영 고정 credential의
양성 대조를 함께 지키는 것이 목표다. [합성 양성 대조](command-query-controls-result.json)에서
`rust/hard-coded-cryptographic-value`가 운영 credential 전달 사례를 계속 검출했으며, 같은 설정에서
`password` 변수의 직접 로그도 검출됐다. 두 값은 실제 계정의 비밀이 아니다.
