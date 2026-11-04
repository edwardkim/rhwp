# PR #8203 검토 기록 — 텍스트 조회의 개체 자리표

## 최종 판정

**승인 — 읽기 API의 텍스트 일관성 범위로 검토하고 원 PR을 병합했다.**
원 기여자 [semanticist21](https://github.com/semanticist21)의 commit 이력을 일반 merge로 보존했다.
이 문서는 완료된 검토의 후속 기록이며 제품 코드나 새로운 검증 결과를 추가하지 않는다.

## 대상과 확정 이력

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8203](https://github.com/edwardkim/rhwp/pull/8203) |
| 검토자 | [postmelee](https://github.com/postmelee) |
| 원 head | `1dadce799b50d210a6db98037971f4c87a74115f` |
| 병합 전 고정 base | `1924d371d6757392d063b8dc2e9bbe9a298be817` |
| 로컬 합성 후보 | `5fcb93f9816046ea8153e95101debc67a4e6b66b` |
| 검증·실제 병합 tree | `91ab562f6607cf1d02eb828f685f4c83038cad3a` |
| 실제 merge | [`515412bef80a7ed9689dff3c4b68065419b055c5`](https://github.com/edwardkim/rhwp/commit/515412bef80a7ed9689dff3c4b68065419b055c5) |
| 병합 시각 | 2026-11-04 09:19:04 UTC |
| 승인 리뷰 | [5740103236](https://github.com/edwardkim/rhwp/pull/8203#pullrequestreview-5740103236) |

## 원 기여와 검토 범위

개체가 소유한 U+FFFC 자리표를 읽기 API에서 본문 글자로 다시 내보내는 문제를 수정했다.
Table/Shape/Picture/Equation이 소유하고 원본 오프셋에서 8n 코드 유닛이 확인되는 자리표만 소비한다.
일반 U+FFFC 문자와 누락·역전·불완전 오프셋은 보존한다. 같은 앵커의 비본문 컨트롤이 앞에 있어도
이미 소비한 위치를 되돌리지 않는다. 추가 제품·테스트·fixture 보정 없이 원 기여를 수용했다.

조판 원칙 검토의 적용 범위는 원본 코드 유닛과 개체 소유 정보를 사용하는 읽기 조회다.
페이지 배치·글꼴 메트릭·paint·serializer 앵커를 변경하는 PR이 아니므로 새 시각 일치율을
승인 근거로 사용하지 않았다. 일반 문자 보존과 오프셋 경계, 조회의 비변경성을 확인했다.

## 검증 근거와 재사용

- 관련 회귀 및 CLI 계약 13개: PASS. 집중 2개, CLI text 6개, scan 5개를 포함한다.
- suite manifest 도구 테스트 23개: PASS.
- 실제 `SO-SUEOP.hwp`의 HWP/HWPX 저장·재열기에서 text·scan·objects 일치와 조회 전후 문서 불변 확인.
- 앞선 11월 2일 왕복 기록은 text 48,295자, scan 1,273개, 최상위 objects 4개를 기록했다.

11월 2일 승인 리뷰는 devel `ea86816b5316dfeeb47ecf20df97abfd4b02da73` 기준의 검토였다.
11월 4일 병합 전에는 위 고정 base/합성 후보에서 집중 검사와 왕복 조회를 재검증한 완료 기록 및
실제 merge tree 일치 영수증을 확인했다. 후속 문서 작성 시 개별 worktree의 실행 로그·producer는
다시 열 수 없었으므로 보존된 후보·병합 영수증, 통합 검토 요약과 공개 승인 기록을 대조했다.
이는 검사를 이번 문서 작업에서 다시 실행했다는 의미가 아니다.

광범위 전체 회귀는 검토 당시 [정본의 재사용 조건](../../manual/pr_review_workflow.md#322-녹색-github-code-head의-중복-로컬-전체-회귀-생략)에
따라 정확한 원 head의 성공 CI를 재사용했다. 원 head 뒤 제품·테스트 보정이 없고 최신 base 합성이
충돌 없이 완료됐으며 focused 검증을 별도로 실행했다. 로컬 합성 후보에 새 GitHub CI가 실행됐다고
표현하지 않는다.

- [Full CI 40112330404](https://github.com/edwardkim/rhwp/actions/runs/40112330404)
- [CodeQL 40112330416](https://github.com/edwardkim/rhwp/actions/runs/40112330416)
- [Adapter inter-diff 40112330484](https://github.com/edwardkim/rhwp/actions/runs/40112330484)
- [Proptest 40112330332](https://github.com/edwardkim/rhwp/actions/runs/40112330332)
- [CI Impact Policy 40114405805](https://github.com/edwardkim/rhwp/actions/runs/40114405805)
- [병합 후 duration 갱신 41806484963](https://github.com/edwardkim/rhwp/actions/runs/41806484963): 성공.

## 한계와 후속 상태

별도 serializer fallback 앵커 이동, 렌더링 품질, AI 편집 품질까지 해결됐다고 판단하지 않았다.
개인 문서나 글꼴 파일을 이 기록에 포함하지 않으며 skipped 검사를 PASS로 세지 않는다.
관련 기록은 [#8289](pr_8289_review.md), [#8285](pr_8285_review.md)와 함께 반영한다.

## Merge 후 contributor PR comment 계획

별도 감사·병합 결과 댓글은 작업지시자 확인 전 게시하지 않는다. 게시 시 원 기여자의 수정,
실제 merge와 이 검토 문서를 연결하고 위 검증 범위를 존댓말로 설명한다. 새 시각 점수나 다른 PR의
이미지를 이 PR의 증거로 사용하지 않는다. 본 기록 작성은 새 댓글 게시나 issue 종료를 뜻하지 않는다.
