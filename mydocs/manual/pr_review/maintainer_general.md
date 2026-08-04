---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-25
---

# Maintainer 일반 PR 처리

이 경로는 admin 또는 branch-protection bypass 권한을 가진 maintainer가 외부 contributor PR을 일반 방식으로
검토·merge하는 기본 경로다. collaborator 경로는 이 문서를 대체하지 않는다.

## review 문서 위치

처리 중에는 active 경로를 사용한다.

~~~text
mydocs/pr/pr_N_review.md
mydocs/pr/pr_N_review_impl.md
~~~

원 코드 PR merge 뒤에는 [merge 후속 처리](post_merge.md)의 판단에 따라 archive로 이동한다. 원 코드 PR의
review 기록만 따로 남길 것이 확정되었으면 처음부터 archive 경로를 쓰거나 같은 후속 기록 commit에서 이동한다.

## 4.5 devel 규약 변경의 열린 PR 일괄 파급

devel에 테스트 규약 변경이 merge되면, 이미 열린 PR의 신규 파일은 merge ref에서도 구 규약을 유지해 여러
PR이 일괄 실패할 수 있다.

- 영향받는 열린 PR과 신규 파일을 전수 분류한다.
- 공통 규약을 충족하는 메인터너 보정 commit을 각 PR에 적용하고, 규약 변경과 보정 내용을 contributor 안내
  comment로 남긴다.
- 보정 뒤 최신 head의 required check를 PR별로 재검증한다.

## 5. 작업지시자 승인 요청

접수·검증·필요한 시각 증적을 review 문서에 남긴 뒤 승인 요청을 한다. 다음 값은 요청 시점 참고값이고
merge 직전에 다시 확인한다.

~~~text
PR #N 검토 결과 · admin merge 준비 완료.

- mergeable: <참고값, merge 전 재확인>
- 충돌 simulation: <결과>
- 선택한 local 검증: <결과>
- review 문서: mydocs/pr/pr_N_review.md
- merge 전 조건: 최신 PR head의 GitHub Actions 통과 + 작업지시자 승인
~~~

## 6. 승인 뒤 admin merge

작업지시자가 명시적으로 승인하고 최신 head·required check·mergeable을 재확인한 뒤 수행한다.

~~~bash
gh pr merge N --repo edwardkim/rhwp --merge --admin
~~~

admin은 BEHIND 상태도 강제 merge할 수 있으므로, 이 명령은 최신 상태 재확인과 승인 뒤에만 사용한다.
merge가 완료되면 즉시 [merge 후속 처리](post_merge.md)를 추가하고 merge SHA부터 확인한다.
