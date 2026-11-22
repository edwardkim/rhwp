# PR #9271 검토 — fix(char-overlap): 글자 없는 테두리 없는 포갠 겹침은 폭 없이 캐럿 한 칸만 둔다

## 통합 후보의 검토 결과

**독립 인쇄 검증 보완 후 수용 가능.** 글자가 없는 펼침과 포갬 상태를 각각 HWPX로 저장해 두 인쇄 결과를 확인했다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](pr_9292_review.md)에 기록했다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9271](https://github.com/edwardkim/rhwp/pull/9271)의 head는 `b6922ffed647ec34f0789da0dc329ef9e11cee9c`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

기여자가 빈 포갠 겹침의 폭을 Native 1쪽 100.00%로 보고했다. 같은 입력의 독립 PDF와 fresh WASM 전쪽 증거가 제출되지 않았다. 빈 겹침과 일반 겹침의 폭·캐럿을 같은 독립 출력과 비교하고 양 backend 증거를 보완한다.

## 메인테이너 보완 검증 중간 결과

빈 글자 겹침의 두 입력을 각각 HWPX로 저장하고 한컴으로 인쇄했다. 두 경우 모두 Native 1쪽은 raw 100.00%다. [쪽별 비교 원장](../assets/semanticist21-20261121/maintainer-comparison.tsv)

## 독립 Print와 두 렌더러 대조

제품 소스 `c6fb9159bf`에서 측정했다. 빈 문자열에 각각 펼침(`SPREAD`)과 포갬(`OVERLAP`)을 설정한 두 저장 입력을 한컴 2020에서 각각 인쇄했다. 두 입력의 1쪽이 Native와 fresh WASM 모두 100.00000%다. [페이지별 입력·PDF 해시와 raw 측정값](../assets/semanticist21-20261121/empty-overlap/pages.tsv)

## 최신 제품 소스의 기능 검증

글자가 없는 펼침과 포갬 상태를 각각 HWPX로 저장해 두 인쇄 결과를 확인했다. 같은 원본의 1쪽 실루엣은 Native 100.00%, fresh WASM 100.00%다. [Native 비교](../assets/semanticist21-20261121/validation/final/representative/pr9271/native/review_001.png) · [WASM 비교](../assets/semanticist21-20261121/validation/final/representative/pr9271/wasm/review_001.png) [17건 통합 검토](pr_9292_review.md)

## 병합 후 기여자 PR 댓글 계획

각 출력에서 2쪽을 확인했다. 전쪽 최저 raw는 Native 100.00000%, fresh WASM 100.00000%다. 대표 페이지는 `overlap-true-hwpx` 1쪽이다. 글자가 없는 겹침 상태에서 앞뒤 글자가 보존된다. 펼침과 겹침의 두 저장본에서 독립 인쇄본과 비교했다.

대표 페이지의 Native raw는 100.00%, 확대 확인할 자동 진단 항목은 0개다. fresh WASM은 100.00%, 0개다. 상세 색상과 안티앨리어싱은 인쇄 엔진에 따라 달라 96 DPI의 2px raw와 실제 잉크·위치를 함께 판독했다.

임시 비교 파일은 로컬 `output/pr-review/semanticist21-20261121/resume-1407/protection-correction/verified-entry-count/representative/`에 있다. 댓글에서는 병합 커밋의 원본 이미지를 사용한다. [검증 방법](../../manual/verification/visual_sweep_guide.md#github-merge-comment)

| 출력 | 비교 패널 | 겹침 |
| --- | --- | --- |
| native | [비교](../assets/semanticist21-20261121/validation/final/representative/pr9271/native/review_001.png) | [겹침](../assets/semanticist21-20261121/validation/final/representative/pr9271/native/overlay_001.png) |
| wasm | [비교](../assets/semanticist21-20261121/validation/final/representative/pr9271/wasm/review_001.png) | [겹침](../assets/semanticist21-20261121/validation/final/representative/pr9271/wasm/overlay_001.png) |

댓글 본문의 이미지 URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/semanticist21-20261121/validation/final/representative/pr9271/` 아래의 해당 파일로 고정한다. 실제 병합 SHA로 바꿔 원 PR에 UTF-8 본문 파일을 게시한 뒤, GitHub API를 통해 본문과 이미지 해시를 확인한다.

## 통합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)를 `devel`에 병합했다. 원 기여 커밋 `b6922ffed647ec34f0789da0dc329ef9e11cee9c`이 병합 이력에 포함된 것을 확인했다. [원 PR #9271](https://github.com/edwardkim/rhwp/pull/9271) · [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [전체 CI와 검토](pr_9292_review.md)
