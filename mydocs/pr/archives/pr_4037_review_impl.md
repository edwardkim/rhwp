---
kind: reference
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-06
---

# PR #4037 메인터너 보정 실행 기록 — 도구킷 출력·실패 계약

## 완료한 적용

1. contributor 원 head `48b41ca9c` 위에서 공통 출력 충돌 검사를 추가했다.
2. `form_filling`과 `table_harvest`는 이번 호출이 새로 확보한 산출물만 정리하고, 기존 산출물은 exit 2로
   보존하도록 바꿨다.
3. `archive_search`는 최종 exit와 batch 정보를 포함한 보고서를 저장하고, `bulk_sweep`은 레코드 없는
   batch 실패를 `batchFailures`로 분리했다.
4. 기존 출력 보존 5건, batch 프로세스 실패 1건, 보고서 종료 코드 검증을 추가해 회귀를 21건에서
   27건으로 확장했다.
5. 단순 보조 도구의 회귀는 핵심 CI에 추가하지 않고, 실제 release-test `rhwp`로 로컬 실행했다.
   README·가이드의 출력·실패 계약도 갱신했다.
6. 보정은 `6f8ee6d84`, CI 제외 정리는 `6049d2919`으로 원 PR source branch에 반영했다.
7. code head `6049d291`의 CI `31068823951`과 CodeQL `31068823815`가 모두 성공했다.
8. 이후 같은 날 `devel` 오늘할일 추가와 충돌해, 양쪽 기록을 보존한 update-branch merge
   `df3aefd70`을 만들었다. contributor 원 commit은 rewrite하지 않았다.
9. 첫 fast-pass preflight가 merge 뒤의 green candidate를 아직 찾지 못해 full CI로 전환됐고,
   `67aedc594`의 CI `31069850597`과 CodeQL `31069850507`가 모두 성공했다.

## 남은 절차

- `67aedc594` 뒤의 이 archive review·오늘할일 trailing commit을 같은 source branch에 push한다.
- `67aedc594`을 code candidate로 재사용한 fast-pass aggregate와 최신 `CLEAN` 상태를 확인한다.
- 병합 후 comment에는 기존 산출물 삭제와 batch 실패 은닉을 막기 위한 메인터너 보정임을 명시한다.

## rollback

- 출력 계약·batch 처리·회귀는 하나의 메인터너 보정 commit으로 되돌릴 수 있다.
- rollback 시 기존 PR 구현 commit `48b41ca9c`은 보존되며, 보정 commit만 역순 revert한다.
- review 문서만 archive로 이동하거나 제거할 때도 코드·CI 보정과 분리된 후속 commit으로 처리한다.
