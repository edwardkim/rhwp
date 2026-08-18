---
kind: review
status: active
source_pr: 5514
---

# PR #5514 검토

## 접수

| 항목 | 값 |
| --- | --- |
| 원본 PR | #5514 `V-replay: 제3자 재실행으로만 노동을 인정 (#5502)` |
| 작성자 | `kevin9327` |
| 원본 head | `bc16a0573e97411a2e35eeabdadd52fd3ed0d2ce` |
| 누적 체리픽 | `daf986000503e317a761c7a6f998d7e17231f963` |
| 관련 이슈 | #5502 |
| 검토 경로 | maintainer 일반 + 접수·리뷰 기록 + 로컬 검증 + 다수 PR 누적 |

## 변경 및 검토

- `tools/llm_verifier/third_party_replay/`에 제3자 replay 봉투, 해시·도구 버전 검증, 결정표와 corpus 검증기를 추가한다.
- 변경 경로는 다른 누적 PR의 파일과 겹치지 않아 충돌 없이 원본 저자를 보존해 적용했다.
- 대규모 정적 corpus는 기능 코드와 fixture 검증 코드가 같은 디렉터리에 있다. 메인터너 보정으로 CI Lint job에 해당 Python test discovery를 추가했다.

## 검증 상태

- 로컬 테스트: 이번 단계에서는 실행하지 않았다.
- 원본 PR CI: 오래된 기준선의 상태이므로 통합 후보의 최종 근거로 재사용하지 않는다.
- 최종 조건: 누적 통합 PR의 최신 head CI와 추가된 verifier fixture 검증이 성공해야 한다.

## 권고

**통합 후보에 포함, 검증 대기.**
