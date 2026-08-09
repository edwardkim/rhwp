---
kind: review-implementation
status: completed-local
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-06
---

# PR #4073 메인터너 보정 기록

## 대상과 commit 경계

| 순서 | SHA | 역할 |
| --- | --- | --- |
| 1 | `a59f8bc36aff9acabca566e75e54dda874735bec` | contributor의 schema-validator 구현·문서·15개 회귀 테스트 |
| 2 | `f106912300ef79504ca0bf272e6565e3902967b4` | 선택된 `oneOf` 대안 WARNING 보존, API·CLI 회귀, README 보정 |
| 3 | `49d2511e9` | 최신 `upstream/devel`을 같은 가시성 브랜치에 update merge |

`maintainerCanModify=true`를 확인했다. contributor 원 commit은 rebase, amend, reset, force-push하지
않았고, 보정은 `review/kevin9327-4073-20260806` 위의 별도 single-parent commit으로만 추가했다.

## 완료한 보정

1. `_validate_one_of()`가 정확히 하나와 일치한 대안의 scratch WARNING을 최종 결과로 보존하게 했다.
2. 매치하지 않은 대안의 ERROR/WARNING은 계속 버려 `ONEOF_FAILED`와 `ONEOF_AMBIGUOUS`의 기존
   의미론을 바꾸지 않았다.
3. 실제 Rust parser가 거부하는 text 블록의 미지 `bold`와 image 전용 `ref`를 회귀 테스트로 추가했다.
4. CLI JSON이 경고 입력에서 `valid:true`, `error_count:0`, `warning_count:1`, 종료 코드 0을 유지하도록
   고정했다.
5. README의 `UNKNOWN_FIELD` 설명을 현재 객체 또는 선택된 `oneOf` 대안 기준으로 정정했다.

## 검증 결과

| 항목 | 결과 |
| --- | --- |
| Python 구문 검사 | 통과 |
| `test_schema_validator.py` | 17 passed |
| canonical sample 2종 | 오류 0·경고 0·종료 0 |
| `bold` / `ref` CLI | 각 경고 1·`valid:true`·종료 0 |
| `git diff --check` | 통과 |
| 보정 head GitHub Actions | 전체 CI, Build & Test aggregate, CodeQL 성공 |

## 원격 반영과 merge 전 조건

보정 code/test commit `f10691230`은 contributor source `pr/tool-schema-validator`에 fast-forward push했다.
최신 `devel` update merge와 archive review·오늘할일 commit도 같은 source branch에 순서대로 반영한다.
update merge가 포함됐으므로 마지막 review-only commit은 fast-pass 후보를 재사용하지 않고 최신 head의 전체
GitHub Actions를 다시 확인한다. 그 뒤 최신 head의 mergeability와 작업지시자 승인에 따라 merge 및 후속
정리를 수행한다.
