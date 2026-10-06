# Issue #6279 Stage 1: review-only tail Full CI evidence reuse

## 배경

PR #6253과 #6275처럼 코드 변경 커밋에서 Full CI가 성공한 뒤 PR 검토 문서와 추가된 PDF
증적만 포함한 trailing commit이 추가될 수 있다. trailing commit 자체는 fast-pass를 적용할 수
있지만, merge 후 `devel` push 정책이 최종 head의 CI run만 찾으면 직전 Full CI와 측정 artifact를
놓쳐 전체 CI를 다시 실행하게 된다.

## 변경 방향

- 최종 head에서 시작해 허용된 review-only commit을 역추적하고, 선형 이력의 직전 코드 commit을
  Full CI 증적 후보로 선택한다.
- 허용 경로는 `mydocs/**`, 새로 추가된 `samples/` 기준 자료, 새로 추가된
  `pdf/`, `pdf-2020/`, `pdf-large/` PDF로 한정한다.
- 파일 목록 누락, 300개 이상 파일, 비선형 이력, 허용되지 않은 변경은 재사용하지 않고 기존
  fail-closed 전체 CI를 실행한다.

## 사전 회귀 방지

- verifier 단위 테스트가 review-only tail에서 직전 코드 commit의 성공 run을 선택하는지 확인한다.
- parent 연결이 맞지 않으면 재사용을 거부하는지 확인한다.
- reusable workflow 계약 테스트가 PR commit 상세 조회와 동일한 review-only 분류기를 요구한다.
