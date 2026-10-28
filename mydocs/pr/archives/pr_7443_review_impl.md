---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-27
---

# PR #7443 collaborator 보정 기록

1. 동일 가시성 branch `codex/pr-7443-review`에서 contributor 원 head
   `df0f400fa008008e39ec009da04e0be328166486`를 유지했다. 원 기여의 author·commit을 재작성하지 않았다.
2. 원 head의 section 비교 누락과 leaf 셀 이동에 따른 bbox 재조회에 대해 새 Studio 검사 3개의 실패를 확인했다.
   section 경계 검사와 표 단위 cache identity를 보정해 관련 21개 및 Studio 전체 1,803개의 통과를 확인했다.
3. 첫 HWP에서 일반 드래그는 before/after 모두 통과했다. 이를 결함 재현으로 세지 않고 F5 선택 모드의 실제
   드래그로 음성 대조를 다시 구성했다. 최종 E2E는 before의 셀 너비 assertion에서 실패하고 after에서 통과했다.
4. core grid 검사의 문단 불일치 skip을 없애고 bbox 페이지·section·조상 및 안쪽 표 경로를 검사하도록 보강했다.
   같은 깊이의 텍스트 우선 10점은 before/after 경로·offset이 동일함을 확인했다. leaf 일치 가정으로 기존 계약을
   바꾸지 않았다. 알려진 빈 영역의 정확한 leaf, 텍스트 offset, 바깥 표 대조군은 유지·강화했다.
5. code/test 보정은 `7af493cb68a6dd94f6dc087e723d44b0738b7939` 한 커밋으로 분리했다.
   최종 검사와 CI 결과는 [리뷰 본문](pr_7443_review.md#검증-결과)에 기록한다.
6. 사용자 승인에 따라 PR·이슈를 @lidge-jun에 assign하고 기존 태그 `bug`, `rhwp-studio`, `table`을 붙였다.
   reviewer는 @postmelee다. 조직 fork의 실제 push 권한이 없어 dry-run은 403으로 거부됐다.
7. section guard와 cache identity 문제는 base에도 존재함을 확인했다. 보정 없는 원 head의 별도 서버에서
   핵심 F5 드래그·Undo와 대조군 E2E도 PASS했다. 따라서 원 PR의 독립 수용과 기존 문제의 후속 보정을 분리했다.
8. 사용자는 원 PR 승인·병합 후 별도 PR에 보정 코드·추가 테스트·리뷰 문서를 함께 제출하기로 결정했다.
   승인 리뷰에 이 계획, 비교 영상의 촬영 범위, 두 번째 문서 미검증을 명시한다. 기여자에게 수정 요청은 하지 않는다.
9. 후속 PR은 원 PR 병합 후 최신 base에서 준비하고 검증·CI를 다시 확인한다. 원 PR 승인 후 사용자 지시에 따라
   원 PR을 `013bc846fca3434ccb1e4167744bed9440c6da6a`로 병합했고 이슈 자동 종료를 확인했다.
   최신 devel에서 `codex/pr-7443-followup`을 만들고 보정 commit만 cherry-pick했다.
   새 코드 commit `9e1cffccf`의 tree는 이전 검증본 `7af493cb6`와 동일하다.
