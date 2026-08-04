# planet6897 열린 PR 통합 검토 - Stage 4

## 목적

이번 통합 대상 원 PR 8건(#2662, #2663, #2664, #2665, #2666, #2669, #2671,
#2706)의 review 기록과 시각 증적을 PR 번호별로 분리해 준비한다. Stage 3 중 새로 열린
#2714는 작업지시자 지시에 따라 명시적으로 제외한다.

## 작업 범위

1. `mydocs/pr/archives/pr_{N}_review.md`를 원 PR별로 작성한다. 각 문서에는 원 commit,
   누적 체리픽 commit, 충돌 여부, 선행 의존, 검증과 merge 판단을 기록한다.
2. 시각 검증을 실제 판단에 사용한 #2663, #2665, #2669의 대표 PNG를
   `mydocs/pr/assets/pr_{N}/`에 안정 파일명으로 복사하고 review 문서에서 연결한다.
3. 페이지 수 불일치나 브라우저 정책 제한을 과장하지 않고, 해당 원 PR의 범위와 별도 잔여
   issue를 구분한다.
4. 오늘할일은 collaborator-mediated 통합 PR 생성 승인 직전에 최신 `upstream/devel`에서만
   갱신한다. 이 단계에서는 생성하지 않는다.

## 완료 기준

- review 문서가 8개이고, 누적 검토가 한 문서로 뭉치지 않는다.
- 필요한 visual asset이 PR 번호별 경로에 존재하며 review 문서의 상대 링크가 유효하다.
- #2714가 체리픽/문서/asset 범위에 섞이지 않는다.

## 결과

- `mydocs/pr/archives/`에 #2662, #2663, #2664, #2665, #2666, #2669, #2671,
  #2706의 개별 review 기록을 작성했다.
- 시각 판단에 사용한 대표 PNG 5개를 `mydocs/pr/assets/pr_2663/`,
  `mydocs/pr/assets/pr_2665/`, `mydocs/pr/assets/pr_2669/`에 복사했고, 각 review
  문서의 상대 링크가 해당 안정 경로를 가리키는 것을 확인했다.
- #2714는 작업지시자 지시에 따라 체리픽, review 문서, 증적 자산 모두에서 제외했다.
- `git diff --check`와 자산 파일 존재·해상도 확인을 통과했다. 원 PR head와 CI의 최신성은
  remote push/통합 PR 생성 직전에 다시 확인한다.
