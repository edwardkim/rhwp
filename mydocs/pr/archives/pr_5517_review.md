---
kind: review
status: active
source_pr: 5517
---

# PR #5517 검토

## 접수

| 항목 | 값 |
| --- | --- |
| 원본 PR | #5517 `feat(v-decomp): 검증 기준을 원자 항목으로 분해한다 (#5504)` |
| 작성자 | `kevin9327` |
| 원본 head | `442f83e15ecccbfeb130a51704d99b10a545e57b` |
| 누적 체리픽 | `cc0b2e418a753a4e31d83223ac7606247e3759b4` |
| 관련 이슈 | #5504 |
| 검토 경로 | maintainer 일반 + 접수·리뷰 기록 + 로컬 검증 + 다수 PR 누적 + 재작업 예외 |

## 변경 및 메인터너 보정

- `tools/llm_verifier/criteria_decomp/`에 원자 기준 분해 crate, corpus, schema와 테스트를 추가한다.
- 원본 PR의 GitHub merge ref는 오래된 workspace member 목록 때문에 충돌했다.
- 체리픽 시 기존 `verdict_protocol`·`claim_bind` workspace member와 lockfile package를 유지하고, `criteria_decomp`를 추가해 세 crate를 함께 등록했다.
- 이 보정은 최신 `devel`의 이미 병합된 verifier crate를 보존하기 위한 호환 보정이며 원본 기능 범위를 바꾸지 않는다.

## 검증 상태

- 로컬 테스트: 이번 단계에서는 실행하지 않았다.
- 최종 조건: 통합 PR CI의 workspace build·clippy와 criteria_decomp crate 테스트가 성공해야 한다.

## 권고

**메인터너 호환 보정 반영, 검증 대기.**
