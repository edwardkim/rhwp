# PR #9229 검토 — feat(object): 표 셀·글상자·머리말 안 직선 끝점을 경로로 옮긴다

## 통합 후보의 검토 결과

**메인테이너 보정 후 수용 가능.** 공개 TypeScript 선언에서 새 API 17개를 호출했다. 셀·글상자 안 직선의 끝점은 HWP·HWPX로 두 번씩 저장해 다시 연 뒤에도 보존된다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](pr_9292_review.md)에 기록했다.

## 접수 당시 판정

**메인터너 보정 후 수용 가능.** 표 셀과 글상자의 직선 편집·저장은 통과했다. Studio에 제공하는 TypeScript 선언에 새 메서드를 반영한 후보가 필요하다.

## 접수와 검토 범위

- 작성자 및 브랜치 `semanticist21/pr/line-endpoint-path`
- 검토 head `4714549cc641fd511a9646a1f9efb4102e7ea46f`
- 리뷰어 `edwardkim`, 요청 시각 `2026-11-20T10:37:36Z` (UTC), GitHub 요청 목록 재확인
- 최신 devel `0a437a7eefb19bfe81fc12866887e4548ea9554d` 기준 개별 및 번호순 누적 병합 시뮬레이션 성공
- 원 head의 Build & Test 및 CI Impact Policy 성공. [CI 집계](https://github.com/edwardkim/rhwp/actions/runs/44687712997/job/136138261364)

끝점 이동의 공통 계산과 머리말 경로의 저장 검사는 추가됐지만, 제출한 새 검사는 머리말에 집중돼 있다. 표 셀 및 글상자 경로의 실제 편집과 저장 후 끝점 방향을 추가로 확인해야 한다.

## 메인테이너 실행 결과

누적 적용 6번째, 적용 직후 `0ffd1e476b`. 충돌 없이 적용됐다. 처음 10건을 합친 `c6407bf83c`에서 이 PR의 집중 검사 1개가 통과했다.

머리말 회귀 1개 외에 표 셀과 글상자에 각각 직선을 삽입해 끝점을 반대로 이동했다. HWP와 HWPX를 각각 두 차례 저장·재열기한 모든 경우 끝점 `(7000, 0, 0, 4000)`이 보존됐다. 새 WASM에는 `moveLineEndpointByPath`가 있지만 저장소의 공개 선언을 import하는 클라이언트는 TS2551로 컴파일되지 않았다. 같은 호출이 갱신된 선언으로 컴파일되는지 검증할 수 있는 제한 보정이다. 보정 커밋은 아직 없다.

공개 API 대조 대상은 `moveLineEndpointByPath`이다. 직접 빌드한 WASM 패키지에서는 메서드 존재와 TypeScript 컴파일을 확인했다. 추적 중인 `rhwp-studio/public/rhwp.d.ts`를 import한 같은 소비자 코드는 타입 검사에서 실패했다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9229](https://github.com/edwardkim/rhwp/pull/9229)의 head는 `4714549cc641fd511a9646a1f9efb4102e7ea46f`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

직선 끝점을 이동한 중첩 개체는 반복 저장 후에도 좌표를 보존한다. 추적 중인 공개 타입 선언에서는 moveLineEndpointByPath를 사용할 수 없다. 공개 선언을 생성된 WASM 서명과 맞추고 직접 임포트하는 TypeScript 소비 예제로 검증한다.

## 메인테이너 보완 검증 중간 결과

공개 TypeScript 선언에서 직접 가져온 소비 예제의 새 API 17개가 strict 컴파일을 통과했다. 표와 글상자 속 직선 끝점은 각각 HWP·HWPX를 두 번 다시 열어도 (7000, 0, 0, 4000)을 유지했다. [쪽별 비교 원장](../assets/semanticist21-20261121/maintainer-comparison.tsv)

## 병합 후 기여자 PR 댓글 계획

셀과 글상자 안 직선의 끝점을 움직이는 기능을 반영했습니다. 공개 TypeScript 선언을 보완했고, 두 형식으로 각각 두 차례 저장하고 다시 열어도 바꾼 끝점의 방향과 좌표가 보존됐습니다. 통합 PR과 확정된 병합 커밋, 이 PR의 검토 기록을 함께 연결한다.

## 통합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)를 `devel`에 병합했다. 원 기여 커밋 `4714549cc641fd511a9646a1f9efb4102e7ea46f`이 병합 이력에 포함된 것을 확인했다. [원 PR #9229](https://github.com/edwardkim/rhwp/pull/9229) · [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [전체 CI와 검토](pr_9292_review.md)
