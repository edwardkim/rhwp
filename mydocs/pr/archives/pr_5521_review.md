---
kind: review
status: active
source_pr: 5521
---

# PR #5521 검토

## 접수

| 항목 | 값 |
| --- | --- |
| 원본 PR | #5521 `feat(verifier): 문서 파생 텍스트가 검증 기준이 되지 못하게 (#5491)` |
| 작성자 | `kevin9327` |
| 원본 head | `4559297cdc3fb37518365bb251004b628c45a66b` |
| 누적 체리픽 | `86a04afdb2890753fc61a4ab401084e6454d6ae5` |
| 관련 이슈 | #5491 |
| 검토 경로 | maintainer 일반 + 접수·리뷰 기록 + 로컬 검증 + 다수 PR 누적 |

## 변경 및 검토

- `tools/llm_verifier/untrusted_sandbox/`에 문서 파생 텍스트와 검증 입력을 분리하는 봉투·결정·corpus 검증기를 추가한다.
- 다른 누적 verifier 기능의 파일과 겹치지 않아 원본 저자와 변경 범위를 유지했다.
- 외부 입력을 거부하는 경계는 fixture로 고정돼 있으므로, 최종 CI에서 envelope·prose 관련 테스트가 실행돼야 한다.

## 검증 상태

- 로컬 테스트: 이번 단계에서는 실행하지 않았다.
- 최종 조건: 통합 PR CI와 untrusted_sandbox fixture 검증 성공.

## 권고

**통합 후보에 포함, 검증 대기.**
