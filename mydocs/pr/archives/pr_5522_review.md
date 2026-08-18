---
kind: review
status: active
source_pr: 5522
---

# PR #5522 검토

## 접수

| 항목 | 값 |
| --- | --- |
| 원본 PR | #5522 `V-lineage: 부모 산출=자식 입력 해시 사슬만 인정` |
| 작성자 | `kevin9327` |
| 원본 head | `6bbe72504954ec5a3a5414056ab4272eb3fc08ce` |
| 누적 체리픽 | `0fa3d68b2961dee4f30afcef99100a878752e4bb` |
| 관련 이슈 | PR 본문에 자동 종료 참조 없음 |
| 검토 경로 | maintainer 일반 + 접수·리뷰 기록 + 로컬 검증 + 다수 PR 누적 |

## 변경 및 검토

- `tools/llm_verifier/lineage_chain/`에 parent-output/child-input hash chain 검증, schema, corpus와 결정표를 추가한다.
- 다른 verifier PR과 경로 중복 없이 누적 적용됐다.
- lineage 깊이와 해시 불일치 fixture가 별도 입력으로 고정돼 있으므로, 통합 CI에서 corpus 검증기를 독립 실행해야 한다.

## 검증 상태

- 로컬 테스트: 이번 단계에서는 실행하지 않았다.
- 최종 조건: 통합 PR CI와 lineage_chain test·corpus 검증 성공.

## 권고

**통합 후보에 포함, 검증 대기.**
