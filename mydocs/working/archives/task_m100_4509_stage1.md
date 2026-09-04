# task_m100_4509 stage1 — 4년 축 귀속(서명) 전체 구현 (Y4-M1~M3 통합)

- 이슈: #4509 ([#4447 착공]) / 브랜치: task_m100_4509 (base devel)
- 산출물: src/capsule_sign.rs(코어) + 명령 2(keygen·verify-signature) + 플래그
  2(replay --sign-key·lineage --keyring) + 7표면 등재 + 계약 테스트 5 + 기술
  문서(mydocs/tech/capsule_signing.md)

## 설계서 대비 실행 범위

설계서(horizon_year4_signing.md, devel 642d826ef)의 M1(발급·검증)+M2(keyring·
폐기)+M3(lineage signerOk)를 한 번에 구현. M4 등재는 이 저장소 규약상 명령
신설과 분리 불가라 함께 (7표면: capabilities·MCP·help·프로필 + node 래퍼·
지식지도 사전·출처 지도/스윕).

## 설계 결정 (문서 §1 상세)

파일 바이트 서명(정규화 금지 — 계보 해시와 동일 대상) / 분리 서명(캡슐 불변)
/ Ed25519(결정론 — 계획→산출→캡슐→서명 결정론 사슬 완성) / 폐기>유효(판정과
암호학적 사실 분리 병기) / PEM 회피 자체 키 JSON(의존성 절제: ed25519-dalek 2
단 1개 추가, 엔트로피는 기존 getrandom).

## 실측 (tests/signing_contract.rs 5/5 + 회귀 전건)

- 왕복 valid·exit 0 / **결정론 서명**(캡슐 재발급=같은 바이트=같은 서명 실측)
- 1바이트 변조=invalid / 미등록=unknownKey / 폐기=revoked(+signatureOk:true 병기)
- lineage 2링크 서명 체인 valid·signerOk true×2, 사이드카 변조=brokenAt,
  **opt-in 무파손**(--keyring 없으면 signerOk 필드 부재)
- 가드: cli_json 31·router·capabilities_schema·provenance(레시피 2종 추가)·
  knowledge_map(사전 11행 신설, 유니크 198 — verdict 중복 1건 실측 정정)·
  replay·audit·lineage 회귀 전건 green / node 466 pass(래퍼 2·옵션 3 추가,
  봉투 42 재생성) / clippy 0

## 함정 실록

- 지식지도 사전은 **유니크 필드 수** 기준 — verdict 가 기존 행과 중복이라
  헤딩은 +11 이 아니라 +10 (가드가 잡음, 199→198 정정).
- capabilities 는 단독 호출이 자기서술(--json 은 --search 전용) — 21차 실측
  재확인.

## 한계 (정직 조항)

S3(키 유출 소급)·S4(signedAt 증명)는 이 축 단독으로 불가 — 5년 축(앵커)
결합이 완성한다. 문서 §4 위협 표에 설계↔구현 대조로 명시.
