# PR #9246 검토 — fix(typeset): 단 영역을 넘친 각 단 아래 각주는 같은 쪽 다음 단에 잇고, 오른쪽 단 아래 각주는 오른쪽 단을 다 쓴다

## 통합 후보의 검토 결과

**메인테이너 보정 후 수용 가능.** 다단 각주의 다음 단 이월과 오른쪽 단의 가용 폭을 원본·편집본의 모든 쪽에서 대조했다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](pr_9292_review.md)에 기록했다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9246](https://github.com/edwardkim/rhwp/pull/9246)의 head는 `c5cbd9851e24832d037fe0ec69fbbb8ac21e3dae`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

8개 HWP가 포함되어 있고 기여자가 측정한 사례는 Native 최저 98.07%다. 구역 끝 overflow-e2-tail의 Native 실루엣과 fresh WASM 전쪽 검증이 빠졌다. 8개 원본의 대응 한컴 Print와 각주 이월·줄 재배치·HWPX 재열기 상태의 양 backend 결과를 보완한다.

## 메인테이너 보완 검증 중간 결과

각주 원본 전 11개 중 overflow·refold 8개의 한컴 2020 Print를 확보했다. Native와 첫 WASM 비교는 각 사례의 모든 쪽이 90% 이상이다. [쪽별 비교 원장](../assets/semanticist21-20261121/maintainer-comparison.tsv)

## 최신 제품 소스의 기능 검증

### 오른쪽 단 각주가 밀어낸 본문의 실제 소속

후보 `703a908900`에서는 오른쪽 단의 긴 각주가 본문을 밀면 2쪽 왼쪽 단에서 세 번째 문단 전체와 참조 번호가 사라졌다. 첫 쪽 첫 문단도 한 줄로 표시됐다. 원 HWP의 세 줄에 저장한 폭은 각각 7,308·12,788·18,272 HWPUNIT이다. `RhwpLineSegTags`의 Section0 해시가 일치하므로 이 파일의 bit 31은 rhwp가 만든 줄을 식별한다. 조판은 세 번째 문단을 다음 쪽의 좁은 단에 배정했지만, 그리기는 이전 줄 폭이 다르다는 이유로 그 문단을 건너뛰었다.

`9c8e2626b0`에서 공통 줄 구성과 실제 배치가 목적지 폭을 사용하도록 보완했다. 한컴 2020 인쇄본에서 첫 단의 본문은 ‘1단¹⁾ 본문입니’와 ‘다’ 두 줄이고, 2쪽 첫 단은 ‘3단³⁾ 본문입니’와 ‘다..’ 두 줄이다. 원본과 HWP·HWPX로 각각 저장해 다시 연 파일에서 이 줄과 참조를 직접 대조했다. 각주 본문은 ‘나400’ 뒤에 두 번째·세 번째 각주가 순서대로 이어졌다. 세 상태의 각 2쪽 모두 Native와 fresh WASM의 96 DPI raw가 100.00000%였다. [쪽별 측정과 원본·Print 해시](../assets/semanticist21-20261121/footnotes/right-column-body.tsv) · [본문과 참조의 전후 비교](../assets/semanticist21-20261121/footnotes/right-column-body-before-after.png)

다단 각주의 다음 단 이월과 오른쪽 단의 가용 폭을 원본·편집본의 모든 쪽에서 대조했다. 같은 원본의 2쪽 실루엣은 Native 100.00%, fresh WASM 100.00%다. [Native 비교](../assets/semanticist21-20261121/validation/final/representative/pr9246/native/review_002.png) · [WASM 비교](../assets/semanticist21-20261121/validation/final/representative/pr9246/wasm/review_002.png) [17건 통합 검토](pr_9292_review.md)

## 병합 후 기여자 PR 댓글 계획

각 출력에서 24쪽을 확인했다. 전쪽 최저 raw는 Native 99.25640%, fresh WASM 99.25632%다. 대표 페이지는 `overflow-r3u` 2쪽이다. 오른쪽 단으로 이어지는 각주의 첫 줄과 본문의 마지막 줄 사이에 겹침이 없고, 다음 번호의 각주도 원래 순서로 이어진다.

대표 페이지의 Native raw는 100.00%, 확대 확인할 자동 진단 항목은 0개다. fresh WASM은 100.00%, 0개다. 상세 색상과 안티앨리어싱은 인쇄 엔진에 따라 달라 96 DPI의 2px raw와 실제 잉크·위치를 함께 판독했다.

임시 비교 파일은 로컬 `output/pr-review/semanticist21-20261121/resume-1407/protection-correction/verified-entry-count/representative/`에 있다. 댓글에서는 병합 커밋의 원본 이미지를 사용한다. [검증 방법](../../manual/verification/visual_sweep_guide.md#github-merge-comment)

| 출력 | 비교 패널 | 겹침 |
| --- | --- | --- |
| native | [비교](../assets/semanticist21-20261121/validation/final/representative/pr9246/native/review_002.png) | [겹침](../assets/semanticist21-20261121/validation/final/representative/pr9246/native/overlay_002.png) |
| wasm | [비교](../assets/semanticist21-20261121/validation/final/representative/pr9246/wasm/review_002.png) | [겹침](../assets/semanticist21-20261121/validation/final/representative/pr9246/wasm/overlay_002.png) |

댓글 본문의 이미지 URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/semanticist21-20261121/validation/final/representative/pr9246/` 아래의 해당 파일로 고정한다. 실제 병합 SHA로 바꿔 원 PR에 UTF-8 본문 파일을 게시한 뒤, GitHub API를 통해 본문과 이미지 해시를 확인한다.

## 통합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)를 `devel`에 병합했다. 원 기여 커밋 `c5cbd9851e24832d037fe0ec69fbbb8ac21e3dae`이 병합 이력에 포함된 것을 확인했다. [원 PR #9246](https://github.com/edwardkim/rhwp/pull/9246) · [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [전체 CI와 검토](pr_9292_review.md)
