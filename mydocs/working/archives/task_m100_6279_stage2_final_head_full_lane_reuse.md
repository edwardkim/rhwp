# Task M100 #6279 Stage 2: PR 최종 head Full CI 재사용

## 배경

PR #6283은 코드 변경과 PR review, 오늘할일, PDF 증적을 같은 PR에 포함했다. 최종
head `85b8ed2235d20c95fd0ec67d7f0bf99439106f31`의 CI는 Full lane으로 성공했고
B/C/D nextest duration artifact 및 CodeQL 성공 결과를 남겼다. 그러나 기존 verifier는
review tail 직전의 코드 commit만 후보로 찾아, 최종 head의 유효한 Full CI를 재사용하지
못하고 merge 뒤 full lane을 다시 실행했다.

## 목표

다음 두 운영 경로를 모두 fail-closed로 지원한다.

1. 코드, PR review, 오늘할일, PDF 증적을 한 PR에 함께 넣고 최종 head에서 Full CI를
   통과한 경우: 최종 head run을 우선 재사용한다.
2. 코드 head의 Full CI 성공 뒤 허용된 review, 오늘할일, PDF tail만 추가한 경우:
   직전 코드 head run을 fallback으로 재사용한다.

## 신뢰 경계

- 동일 원본 저장소의 `devel` 대상 merged PR, merge 전 완료된 `pull_request` run,
  PR 최종 tree와 merge tree 일치, CI enforcement surface 무변경을 공통으로 요구한다.
- CI 후보는 성공 상태만으로 신뢰하지 않고 B/C/D duration artifact가 모두 있을 때만
  Full lane 증적으로 인정한다.
- CodeQL 후보는 성공한 `Analyze (...)` job이 최소 하나 있을 때만 실제 분석 증적으로
  인정한다.
- 증적이 없는 최종 head는 재사용하지 않고 review-tail 코드 후보를 시도한다. 두 후보가
  모두 불충분하면 full lane으로 fail-closed 한다.

## 검증 계획

- shared verifier Node 계약 테스트에 최종 head 우선과 증적 부재 거부를 추가한다.
- shared workflow Python 계약 테스트에 CI artifact 및 CodeQL analysis-job 검증을 추가한다.
- PR CI 후 코드+문서 동시 포함 사례와 문서 tail 사례가 모두 각각의 증적 run을 선택하는지
  로그로 확인한다.
