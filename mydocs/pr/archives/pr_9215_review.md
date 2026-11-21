# PR #9215 검토 — feat(page-hide): 캐럿 쪽의 감추기를 이어지는 문단·표 셀에서도 고친다

## 통합 후보의 검토 결과

**메인테이너 보정 후 수용 가능.** 감추기를 잘못된 셀로 옮기면 오류가 나고 기존 머리말 감추기와 속성이 보존된다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](pr_9292_review.md)에 기록했다.

## 접수 당시 판정

**머지 보류.** 대상 셀 경로가 잘못되어 편집이 실패할 때 이미 설정된 머리말 감추기가 삭제된다.

## 접수와 검토 범위

- 작성자 및 브랜치 `semanticist21/pr/page-hide`
- 검토 head `710f46dcfe9d94de1060f418f6c06806c48bec5e`
- 리뷰어 `edwardkim`, 요청 시각 `2026-11-20T10:37:25Z` (UTC), GitHub 요청 목록 재확인
- 최신 devel `0a437a7eefb19bfe81fc12866887e4548ea9554d` 기준 개별 및 번호순 누적 병합 시뮬레이션 성공
- 원 head의 Build & Test 및 CI Impact Policy 성공. [CI 집계](https://github.com/edwardkim/rhwp/actions/runs/44676303706/job/136099345003)

페이지 감추기 위치를 옮기는 명령은 대상 문단을 확인한 후 기존 감추기를 제거한다. 새 위치의 셀 경로는 그 뒤에 검사하므로, 유효하지 않은 경로에 대한 오류 반환 전 기존 상태가 변경될 수 있다. 본문 검증 SHA 616b6f4f5e 뒤에 활성 필드 인덱스 보정이 추가됐다.

## 메인테이너 실행 결과

누적 적용 2번째, 적용 직후 `4c1fdb4155`. 충돌 없이 적용됐다. 처음 10건을 합친 `c6407bf83c`에서 이 PR의 집중 검사 4개가 통과했다.

빈 문서의 0쪽에 머리말 감추기를 설정하고 같은 쪽에 존재하지 않는 셀 경로 `[(91, 0, 0)]`를 넘겼다. 명령은 오류를 반환했지만 `get_page_hides_on_page_native(0)`의 결과가 1개에서 0개로 바뀌었다. 대상 셀을 먼저 검증한 뒤 교체하고, 실패 전후 설정과 문서 내용이 같은지 회귀 검사로 확인해야 한다.

공개 API 대조 대상은 `getPageHidesOnPage, setPageHideOnPage`이다. 직접 빌드한 WASM 패키지에서는 메서드 존재와 TypeScript 컴파일을 확인했다. 추적 중인 `rhwp-studio/public/rhwp.d.ts`를 import한 같은 소비자 코드는 타입 검사에서 실패했다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9215](https://github.com/edwardkim/rhwp/pull/9215)의 head는 `710f46dcfe9d94de1060f418f6c06806c48bec5e`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

기존 머리말 감추기가 있는 상태에서 잘못된 표 셀로 옮길 때 오류를 반환하면서 기존 감추기까지 지운다. 새 대상의 유효성을 확인한 뒤 상태를 변경하고 실패 전후 기존 감추기가 보존되는 회귀를 추가한다.

## 메인테이너 보완 검증 중간 결과

잘못된 셀 경로에 감추기를 옮기는 입력을 다시 실행했다. 오류 뒤에도 원래 본문 감추기와 속성이 보존됐다. [쪽별 비교 원장](../assets/semanticist21-20261121/maintainer-comparison.tsv)

## 병합 후 기여자 PR 댓글 계획

캐럿이 놓인 쪽의 감추기를 셀에서도 편집할 수 있게 됐습니다. 대상 셀 경로가 잘못되면 기존 설정을 먼저 지우던 경우를 보완해, 오류가 난 뒤에도 머리말 감추기가 보존되는 것을 확인했습니다. 통합 PR과 확정된 병합 커밋, 이 PR의 검토 기록을 함께 연결한다.

## 통합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)를 `devel`에 병합했다. 원 기여 커밋 `710f46dcfe9d94de1060f418f6c06806c48bec5e`이 병합 이력에 포함된 것을 확인했다. [원 PR #9215](https://github.com/edwardkim/rhwp/pull/9215) · [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [전체 CI와 검토](pr_9292_review.md)
