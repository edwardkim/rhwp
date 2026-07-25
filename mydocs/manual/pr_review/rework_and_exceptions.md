---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-25
---

# 재작업과 예외

이 가이드는 기본 경로를 대체하지 않는다. 재작업, close, 특수 PR에서 선택한 기본 경로와 함께 적용한다.

## 10. 재작업 요청

base가 main이거나, 관련 issue가 없거나, metadata 변경이 섞였거나, 최종 문서가 초기 가설과 모순되면
구체적인 수정 요청을 남기고 close·재제출 경로를 안내할 수 있다.

1. PR에 정중하지만 단호한 feedback과 수정 목록을 남긴다.
2. 작업지시자 승인 뒤 PR을 close한다.
3. 재제출은 새 PR 번호로 다시 접수한다.

결정은 명확히, 사유는 구체적으로, devel target 재제출 경로와 contributor credit 보존을 함께 안내한다.

### 10.1 영어 contributor 응답 언어

영어 contributor에게는 한글 문단 뒤에 같은 의미의 영어 문단을 ---로 구분해 병기한다. CI 실패 원인,
수정 요청, draft인 경우 Ready for review 요청도 두 언어에 같은 수준으로 담는다.

## 11. 예외

### 11.1 Dependabot PR

dependabot npm_and_yarn branch가 main을 target으로 하면 .github/dependabot.yml의 target-branch를 devel로
보정한다. 현재 main target PR은 close하고 필요한 version bump는 devel에서 처리한다.

### 11.2 오래된 base와 대량 commit 혼입

수십 commit 전 base에서 갈라져 이미 merge된 과거 commit이 대량 diff에 섞이면 contributor의 신규 commit만
저자를 보존해 cherry-pick한다. 원 PR과 중복 PR은 설명 comment 뒤 close한다.

### 11.3 대형 PR (>1000 라인)

1,000줄 초과 PR은 즉시 admin merge하지 않는다. 코드 review, simulation, 필요한 시각 검증과 작업지시자
판단을 별도 cycle로 진행한다.

## 12. 참조 memory

작업 전 troubleshootings와 관련 memory를 검색한다. 현행 규칙은 이 canonical workflow와 자식 가이드에서
확인하며, 과거 memory는 근거·피드백 출처로만 사용한다.

## 13. 참고 archive

재작업 사례는 mydocs/pr/archives/pr_234_review.md, 다양한 review 패턴은 pr_235_review.md와
pr_237_review.md, 재제출 후 merge 사례는 pr_251_review.md를 참고한다.
