# PR #9234 검토 — fix(note): 다단 쪽의 단 각주 영역마다 누르기·캐럿·선택을 찾고, 각주 영역 폭이 바뀌면 각주 줄을 그 폭으로 다시 나눈다

## 통합 후보의 검토 결과

**메인테이너 보정 후 수용 가능.** 너비가 다른 세 단의 본문과 각주 참조가 처음 열기·편집·두 형식 다시 열기에서 한컴과 같은 줄에 남는다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](pr_9292_review.md)에 기록했다.

## 본문 줄바꿈 보완 결과

너비가 다른 세 단의 본문 줄바꿈과 각주 참조를 보완했다. 한컴 2020 Print 대비 원본의 Native 일치율은 89.87%에서 99.26%, 최적화한 fresh WASM은 89.59%에서 98.48%로 올랐다. 이 수치는 본문 줄바꿈을 처음 보완했을 때 측정했다. [원본·편집본 검증](../assets/semanticist21-20261121/footnotes/README.md)

## 접수와 검토 범위

- 작성자 및 브랜치 `semanticist21/pr/multi-column-footnote`
- 검토 head `fb144c2ecceee9f7db6288629121ed521257c16f`
- 리뷰어 `edwardkim`, 요청 시각 `2026-11-20T10:37:43Z` (UTC), GitHub 요청 목록 재확인
- 최신 devel `0a437a7eefb19bfe81fc12866887e4548ea9554d` 기준 개별 및 번호순 누적 병합 시뮬레이션 성공
- 원 head의 Build & Test 및 CI Impact Policy 성공. [CI 집계](https://github.com/edwardkim/rhwp/actions/runs/44700778799/job/136180876717)

두 단 아래, 너비가 다른 세 단 아래, 오른쪽 단 아래의 HWP 3개는 포함됐다. 테스트에 인용한 한컴 Mac Print 좌표는 있지만 해당 출력 PDF와 편집 후 Native 및 fresh WASM의 전쪽 TSV를 확인할 수 없다. 단 폭 변경은 실제 각주의 줄바꿈을 바꾸므로 편집 후 상태의 비교가 필요하다.

## 메인테이너 실행 결과

누적 적용 9번째, 적용 직후 `bb3cfa6101`. 충돌 없이 적용됐다. 처음 10건을 합친 `c6407bf83c`에서 이 PR의 집중 검사 5개가 통과했다.

두 단·너비가 다른 세 단·오른쪽 단 각주 사례를 포함한 집중 검사 5개는 통과했다. 각주 조판은 문서와 편집 상태에 따라 달라진다. 원 HWP와 동일하게 편집한 문서 각각의 독립 기준 출력·쪽수·페이지별 TSV와 각주 영역 확대 대조를 확보하면 보류 사유를 재검토할 수 있다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9234](https://github.com/edwardkim/rhwp/pull/9234)의 head는 `fb144c2ecceee9f7db6288629121ed521257c16f`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

다단 각주의 클릭·캐럿 및 너비 변경 회귀는 있으나 원본 및 폭 변경·편집 상태에 대응하는 독립 PDF와 양 backend 전쪽 시각 결과가 필요하다. 동일한 편집 상태의 한컴 Print와 Native·fresh WASM 전쪽 TSV 및 각주 영역 확대 대조를 보완한다.

## 메인테이너 보완 검증 중간 결과

11개 원본의 한컴 2020 독립 Print를 확보했다. 3단 원본의 본문 저장 줄 시작은 단 1에서 0·34·42, 단 2에서 0·32였다. 한컴 재저장 줄은 각각 0·31·40, 0·30이며 동일 Print 기준 Native는 89.87%에서 99.26%로 달라졌다. 편집·단 수·여백 변경 4개 상태 6쪽의 Native 최저는 99.63%다. [쪽별 비교 원장](../assets/semanticist21-20261121/maintainer-comparison.tsv)

## 최신 devel 병합 후 재검증

본문 참조의 표시 폭을 공통 줄 채움에 반영한 뒤 글자 삽입·삭제 시 원본 본문은 첫째 단 0·31·40, 둘째 단 0·30, 셋째 단 0·28에서 줄을 시작했다(원본 UTF-16 축). 독립 한컴 2020 재저장본의 세 단과 일치한다. 최초 열기의 초기 보정은 본문과 참조 번호의 합산 폭이 허용 폭을 넘는 저장 줄을 다시 나눴다. 이 규칙과 줄 배치가 소비하는 참조 번호 메트릭을 로컬 커밋 `f26e155ab6`에 반영했다. 2020과 2024의 독립 Print는 첫째 단의 두 번째 줄이 ‘다.’, 둘째 단은 ‘단 문장입니다’로 시작한다.

최신 devel에 들어온 #9277의 정책연구 문서 83쪽에도 본문 안에 각주 참조가 있다. 이 문단은 내어쓰기가 있는 저장 줄을 사용한다. 초기 보정은 그 줄의 폭까지 다시 추정해 ‘EU 지’에서 끝나던 줄의 ‘지’를 다음 줄로 밀었다. 같은 글꼴에서 보완 전후를 대조한 결과 Native 일치율이 99.67%에서 86.30%로 떨어졌다.

로컬 커밋 `bd5abb3360`은 저장 줄의 폭과 시작점이 일정하고, 각주 참조의 전진 폭만 빼고 돌린 공통 줄 채움이 원본의 모든 줄 경계를 그대로 재현할 때에만 저장 줄을 다시 나눈다. 내어쓰기·문단 머리·자동 하이픈·제외 영역이 있는 줄은 해당 저장 프레임의 처리 규칙을 따른다. Native로 다시 인쇄한 83쪽은 HWP 99.96%, HWPX 99.99%이고 두 파일 모두 대조군의 본문 38줄이 복원됐다. 세 단 원본에서는 참조가 포함된 11줄이 유지됐다.

## 단별 각주 사이 간격

`c2-eachColumn.hwp`의 각주 모양은 주석 사이 간격을 283 HWPUNIT으로 지정한다. 독립 한컴 2020 Print에서 오른쪽 단의 두 각주 기준선은 741.7494pt와 754.5754pt이며, 왼쪽 단 마지막 각주의 기준선도 754.5754pt다. 한 줄 높이와 주석 사이 간격의 합이 기준선 거리 12.826pt를 설명한다.

프레임 높이 측정은 오른쪽 단 안의 두 각주 사이에 이 간격을 예약했다. 실제 배치는 쪽 전체에서의 원본 각주 인덱스를 현재 단의 각주 수와 비교해 간격을 건너뛰었다. 마커와 히트 테스트는 원본 인덱스를 사용하고, 다음 각주의 존재 여부는 현재 단 안의 순번으로 판정한다.

## 최신 제품 소스의 기능 검증

너비가 다른 세 단의 본문과 각주 참조가 처음 열기·편집·두 형식 다시 열기에서 한컴과 같은 줄에 남는다. 같은 원본의 1쪽 실루엣은 Native 100.00%, fresh WASM 100.00%다. [Native 비교](../assets/semanticist21-20261121/validation/final/representative/pr9234/native/review_001.png) · [WASM 비교](../assets/semanticist21-20261121/validation/final/representative/pr9234/wasm/review_001.png) [17건 통합 검토](pr_9292_review.md)

## 병합 후 기여자 PR 댓글 계획

각 출력에서 24쪽을 확인했다. 전쪽 최저 raw는 Native 99.25640%, fresh WASM 99.25632%다. 대표 페이지는 `c2-eachColumn` 1쪽이다. 단별 각주 마지막 기준선과 오른쪽의 두 각주 간격을 대조했다. 너비가 다른 3단 본문은 처음 열기와 편집·저장 후에도 같은 줄 경계를 유지한다.

대표 페이지의 Native raw는 100.00%, 확대 확인할 자동 진단 항목은 0개다. fresh WASM은 100.00%, 0개다. 상세 색상과 안티앨리어싱은 인쇄 엔진에 따라 달라 96 DPI의 2px raw와 실제 잉크·위치를 함께 판독했다.

임시 비교 파일은 로컬 `output/pr-review/semanticist21-20261121/resume-1407/protection-correction/verified-entry-count/representative/`에 있다. 댓글에서는 병합 커밋의 원본 이미지를 사용한다. [검증 방법](../../manual/verification/visual_sweep_guide.md#github-merge-comment)

| 출력 | 비교 패널 | 겹침 |
| --- | --- | --- |
| native | [비교](../assets/semanticist21-20261121/validation/final/representative/pr9234/native/review_001.png) | [겹침](../assets/semanticist21-20261121/validation/final/representative/pr9234/native/overlay_001.png) |
| wasm | [비교](../assets/semanticist21-20261121/validation/final/representative/pr9234/wasm/review_001.png) | [겹침](../assets/semanticist21-20261121/validation/final/representative/pr9234/wasm/overlay_001.png) |

댓글 본문의 이미지 URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/semanticist21-20261121/validation/final/representative/pr9234/` 아래의 해당 파일로 고정한다. 실제 병합 SHA로 바꿔 원 PR에 UTF-8 본문 파일을 게시한 뒤, GitHub API를 통해 본문과 이미지 해시를 확인한다.

## 통합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)를 `devel`에 병합했다. 원 기여 커밋 `fb144c2ecceee9f7db6288629121ed521257c16f`이 병합 이력에 포함된 것을 확인했다. [원 PR #9234](https://github.com/edwardkim/rhwp/pull/9234) · [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [전체 CI와 검토](pr_9292_review.md)
