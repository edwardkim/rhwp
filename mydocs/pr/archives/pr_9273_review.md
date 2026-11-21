# PR #9273 검토 — fix(table): 쪽보다 길게 자란 중첩 표 행을 쪽 경계의 글줄 사이에서 나눈다

## 통합 후보의 검토 결과

**메인테이너 보정 후 수용 가능.** 70줄로 편집한 중첩 표가 한컴과 같은 21·58·70줄에서 끊긴다. CBTA 문서 59쪽의 뒤 문단도 354.0px에 위치한다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](pr_9292_review.md)에 기록했다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9273](https://github.com/edwardkim/rhwp/pull/9273)의 head는 `7d8d6dc56caaef30893e1ed1cb6442a27597dfee`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

기여자가 보고한 70문단 사례는 3쪽이 한컴보다 한 줄 일찍 끝난다. 4쪽 Native 87.67%로 페이지별 90% 기준에 미달한다. 이어받은 행의 점유와 줄 경계를 보정한 뒤 4쪽까지 줄의 소속·누락·중복을 독립 Print와 Native·fresh WASM으로 재검증한다.

## 메인테이너 보완 검증 중간 결과

30줄 입력은 두 저장 형식에서 모두 한컴 3쪽, Native 4쪽이다. 70줄은 모두 4쪽이지만 4쪽 Native 87.15%이고, 한컴은 2쪽 줄 21, 3쪽 줄 58에서 끝난다. 한컴 재저장본의 줄과 원본의 분할 예산을 대조 중이다. [쪽별 비교 원장](../assets/semanticist21-20261121/maintainer-comparison.tsv)

## 최신 devel 병합 후 재검증

최신 devel의 #9277을 반영한 뒤 30줄과 70줄을 각각 HWP와 HWPX로 저장하고 다시 열었다. 한컴 2020 Print와 같은 3쪽·4쪽이다. 두 형식 모두 Native 최저는 30줄 98.05%, 70줄 98.12%다. 70줄의 2쪽은 줄 21, 3쪽은 줄 58에서 끝난다.

편집 직후에는 2쪽 끝이 줄 25이고 70줄의 3쪽 끝이 줄 62다. 동일 내용의 저장본과 행 8개의 측정 높이는 일치한다. 실행 추적에서 저장 재열기 경로가 앞 조각에서 소비한 행 높이를 기록하는 동안 편집 중 경로는 이 기록을 건너뛴다. 편집 직후 페이지 경계의 판정은 보류한다.

연속된 저장 줄을 가진 rowspan 셀의 유효성은 해당 셀의 재조판 여부와 저장 높이·줄 순서로 확인한다. 문서 전체의 편집 플래그가 다른 셀의 유효한 높이 기록까지 무효화하던 분기를 제거했다. 같은 원본의 중첩 표에서 30문단을 추가하면 편집 직후와 HWP/HWPX 재열기 모두 2쪽 끝이 줄 21이다. 70문단의 편집 직후에도 2쪽 줄 21, 3쪽 줄 58, 4쪽 줄 70으로 한컴 Print와 일치했다.

## 독립 Print와 두 렌더러 대조

제품 소스 `c6fb9159bf`에서 측정했다. 편집 직후와 HWP·HWPX 재열기를 구분해 두 렌더러에서 각각 21쪽을 대조했다. 30줄 편집본의 2쪽은 Native 99.97482%, fresh WASM 100.00000%이며, 70줄의 2쪽도 같은 값이다. 70줄의 실제 쪽 경계는 21·58·70줄이다. 저장 재열기까지 포함한 전체 최저는 Native 98.05486%, fresh WASM 97.94231%다. [페이지별 입력·PDF 해시와 raw 측정값](../assets/semanticist21-20261121/nested-row-split/pages.tsv)

### 59쪽 표 뒤 문단의 회귀

전체 회귀에서 행 이월 검사와 본문 하단 넘침 검사가 같은 CBTA 문서 59쪽에서 실패했다. 중첩 표의 저장 개체가 이미 세 번째 행 끝에서 닫혔는데도, 측정 높이와 선언 높이가 다르면 같은 행을 다시 분할할 수 있었다. 바깥 표가 늘어나 독립 출력의 354.05px에 있어야 할 후속 문단이 371.9px로 내려갔다.

저장 개체의 끝 행이 확인된 경우 그 행의 소속을 유지했다. 최소 글꼴 환경에서 후속 문단은 354.0px이며 문서 전체에서 본문 하단을 2px 넘는 항목은 0개다. 70줄 중첩 표의 편집 직후·클릭·HWP/HWPX 재열기 검사 4건, CBTA 후속 문단 검사, 각주 뒤 들여쓰기 문단의 38줄 소속 검사도 통과했다. 이 시점에는 전체 회귀와 최종 바이너리의 시각 검증을 진행하고 있었다.

## 기여자의 후속 커밋과 저장 줄 원점

기여자가 올린 `b1047824674564474907e76972100fb6a072e699`를 통합했다. 앞선 후보는 한컴과 같은 쪽에 줄을 표시했지만 저장 파일의 세로 좌표가 다음 쪽에서도 계속 증가했다. 한컴 2020이 다시 저장한 HWPX에서 21·22·23번째 줄은 각각 36960·0·1760 HWPUNIT이었다. 70줄 문서의 58·59·60번째 줄은 63360·0·1760이었다.

편집으로 재조판한 칸의 실제 캐럿 소속을 사용해 HWP와 HWPX 저장 사본에 쪽 원점을 기록한다. 기존에 원점이 기록된 칸은 그 값을 읽는다. [통합 커밋](https://github.com/edwardkim/rhwp/commit/c522f226f0fe8420c1afa09d10de0899373765aa) · [양 입력 형식의 편집 상태 추적](https://github.com/edwardkim/rhwp/commit/97c6616968b844e6162fec0efd9e6fb8b2306bee)

### 기여자 후속 커밋의 저장·재열기 경계

후보 `97c6616968`에서 같은 HWP 편집본을 HWP와 HWPX로 각각 저장했다. 저장한 30줄과 70줄의 줄 좌표는 한컴 2020이 다시 저장한 값과 전부 일치했다. 두 형식의 70줄 파일을 rhwp로 다시 열면 58번째 줄이 4쪽으로 넘어갔다. 같은 바이트의 한컴 Print에서는 58번째 줄이 3쪽에 있었다. 저장한 원점 경계에서 자식 줄의 후행 간격 8.8px가 부모 행 유닛에 전달되는지, 앞 조각의 가용 높이와 함께 대조한다.

원본 `samples/inner-table-01.hwp`를 한컴 2020에서 HWPX로 다시 저장한 뒤 같은 칸을 편집했다. 70줄 편집 직후에는 70개 캐럿이 모두 2쪽에 남았다. 바깥 표 마지막 행에서 높이 측정은 580.5px, 분할 유닛의 합은 2,216.9px였다. 독립 Print는 2쪽이 21번째 줄, 3쪽이 58번째 줄에서 끝났다. 입력 형식별로 실제 편집한 칸의 높이가 부모 측정과 분할에서 같은 근거를 쓰는지 확인한다.

### 편집한 안쪽 표와 부모 칸의 높이

`c862dcc652`에서 측정과 배치가 함께 쓰는 저장 높이 판정을 보완했다. 자식 표의 글줄을 편집하면 부모 칸의 마지막 저장 줄은 편집 전의 끝을 가리킨다. 이때 실제 자식 표를 포함해 계산한 높이를 사용한다. 저장본의 평문 행이 다음 쪽 원점에서 다시 시작할 때는 자식 칸들이 함께 소유한 마지막 줄간격을 부모 컷에 전달한다.

한컴 2020에서 다시 저장한 HWPX에 30줄과 70줄을 입력한 후 두 형식으로 저장했다. 앞선 후보에서 달랐던 세로 좌표 116건이 모두 독립 한컴 재저장본과 일치했다. HWP·HWPX 각각의 편집 직후와 재열기에서 200개 줄별 캐럿은 같은 쪽에 있었고 셀을 벗어난 캐럿은 0개다. 70줄 문서의 58번째 줄은 3쪽, 59번째 줄은 4쪽에 놓였다. [쪽 소속과 저장 좌표 200행](../assets/semanticist21-20261121/nested-row-split/hwpx-edited-lines.tsv)

### 3쪽 마지막 줄과 두 괘선

70줄을 입력한 문서의 3쪽에서 줄 58의 쪽 소속은 맞았지만, 재열기 시 바깥 표 아래 괘선이 글자 위를 지나갔다. 한컴 PDF의 벡터 좌표를 96 DPI로 환산하면 안쪽 괘선 중심은 1,000.03px, 바깥 괘선 중심은 1,003.86px다. 보완 전 재열기에서는 바깥 괘선이 안쪽보다 3.2px 높았다.

자식 표의 글줄 행을 중간에서 열고 닫을 때, 자식 행의 분할 상자는 마지막 줄에서 다음 쪽으로 넘어가는 전진을 이미 제외했다. 부모가 그 전진을 한 번 더 빼면서 괘선을 위로 그렸다. `703a908900`에서 같은 자식 컷의 물리 높이와 블록 표의 위·아래 바깥여백을 부모의 수용 높이와 정렬 높이에 함께 적용했다. 수정 후 HWP·HWPX 편집 직후와 재열기에서 두 괘선 사이 간격은 3.8px다. [보완 전후의 실제 줄·괘선 좌표](../assets/semanticist21-20261121/nested-row-split/edited/frame-contract.tsv)

## 최신 제품 소스의 기능 검증

70줄로 편집한 중첩 표가 한컴과 같은 21·58·70줄에서 끊긴다. CBTA 문서 59쪽의 뒤 문단도 354.0px에 위치한다. 같은 원본의 3쪽 실루엣은 Native 100.00%, fresh WASM 100.00%다. [Native 비교](../assets/semanticist21-20261121/validation/final/representative/pr9273/native/review_003.png) · [WASM 비교](../assets/semanticist21-20261121/validation/final/representative/pr9273/wasm/review_003.png) [17건 통합 검토](pr_9292_review.md)

원본 HWP와 한컴이 재저장한 HWPX에 각각 30줄·70줄을 입력하고, 편집 직후와 HWP·HWPX 저장 후 재열기를 대조했다. 각 백엔드의 전 42쪽에서 한컴 Print 대비 최저 raw는 Native 98.00640%, fresh WASM 97.86924%다. 한컴 HWPX에서 편집한 200개 줄의 저장 원점과 재열기 후 캐럿의 쪽 소속도 일치했다. [두 형식의 검증 결과](../assets/semanticist21-20261121/nested-row-split/edited/README.md) · [가장자리 확대 비교](../assets/semanticist21-20261121/nested-row-split/edited/nested-last-line.png)

## 병합 후 기여자 PR 댓글 계획

각 출력에서 21쪽을 확인했다. 전쪽 최저 raw는 Native 98.06894%, fresh WASM 98.06533%다. 대표 페이지는 `nested-70-hwp` 3쪽이다. 70줄로 편집한 중첩 표는 21·58·70줄 단위로 이어지고 마지막 줄과 표 뒤의 본문이 보존된다.

대표 페이지의 Native raw는 100.00%, 확대 확인할 자동 진단 항목은 0개다. fresh WASM은 100.00%, 0개다. 상세 색상과 안티앨리어싱은 인쇄 엔진에 따라 달라 96 DPI의 2px raw와 실제 잉크·위치를 함께 판독했다.

임시 비교 파일은 로컬 `output/pr-review/semanticist21-20261121/resume-1407/protection-correction/verified-entry-count/representative/`에 있다. 댓글에서는 병합 커밋의 원본 이미지를 사용한다. [검증 방법](../../manual/verification/visual_sweep_guide.md#github-merge-comment)

| 출력 | 비교 패널 | 겹침 |
| --- | --- | --- |
| native | [비교](../assets/semanticist21-20261121/validation/final/representative/pr9273/native/review_003.png) | [겹침](../assets/semanticist21-20261121/validation/final/representative/pr9273/native/overlay_003.png) |
| wasm | [비교](../assets/semanticist21-20261121/validation/final/representative/pr9273/wasm/review_003.png) | [겹침](../assets/semanticist21-20261121/validation/final/representative/pr9273/wasm/overlay_003.png) |

댓글 본문의 이미지 URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/semanticist21-20261121/validation/final/representative/pr9273/` 아래의 해당 파일로 고정한다. 실제 병합 SHA로 바꿔 원 PR에 UTF-8 본문 파일을 게시한 뒤, GitHub API를 통해 본문과 이미지 해시를 확인한다.

## 통합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)를 `devel`에 병합했다. 원 기여 커밋 `b1047824674564474907e76972100fb6a072e699`이 병합 이력에 포함된 것을 확인했다. [원 PR #9273](https://github.com/edwardkim/rhwp/pull/9273) · [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [전체 CI와 검토](pr_9292_review.md)
