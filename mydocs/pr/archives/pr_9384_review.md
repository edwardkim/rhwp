# PR #9384 검토 · 줄 정보가 없는 칸의 사진 배치

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 기여 `c3c76e6c2d2b07389714dbafbb40505747b47ec6`의 기능에 칸 사진의 실제 가로 범위와 정렬·오프셋에 따른 겹침 판정을 보완한 제품 소스는 `b12b85ad6ec037fff9ce41ac6fc34b8865d93920`이다. 대조 기준은 `50324d194d1cd7c46d168f07a58515086530f03b`이다. 원 기여에서 남았던 증상과 보정본의 측정·배치 경로를 아래에 기록했다. 문서별로 승인된 글꼴 및 인쇄 조건으로 [통합 PR #9412](https://github.com/edwardkim/rhwp/pull/9412)를 `a5f01c69392dd4cf6919c8291023ddc84466e181`에서 검토했다. 필수 CI와 [셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9412#pullrequestreview-6063886947)를 마치고 [커밋 `5fa8cede504e`](https://github.com/edwardkim/rhwp/commit/5fa8cede504eca188d2e003931a50e9a8434d6da)으로 `devel`에 병합했다. [세 PR의 최종 검토](hyunmyungjanelee_20261125_review.md) · [원 기여와 보정 후 쪽별 측정](../assets/hyunmyungjanelee-20261125/maintainer/original-pr-comparison.tsv)

#9415의 재저장 글상자와 크기 보호 처리를 병합한 제품 소스 `f3d137d9cbe36cae93f859baac3a29ae4a543f75`에서 같은 입력·한컴 Print로 다시 검증했다. 전체 회귀 14,166건, Native Skia 4,227건과 세 PR 및 새 기준의 집중 회귀 64건을 통과했다. [현재 기준에서 세 PR의 쪽별 결과](../assets/hyunmyungjanelee-20261125/maintainer/current-9415/README.md)

#9411을 병합한 제품 소스 `8d1403eb14bbefb35690a50b6734b0c2ef9ba5cb`에서 이 기여의 입력·인쇄본과 전체 보호 문서를 다시 검증했다. [최신 기준에서의 검증 결과](../assets/hyunmyungjanelee-20261125/maintainer/current-develop/README.md)

원 기여자는 hyunmyungJaneLee이고 원 head는 `c3c76e6c2d2b07389714dbafbb40505747b47ec6`이다. #9383의 가로 나란한 사진, 단일 사진, 가로로 겹쳐 세로로 쌓이는 사진을 검토한다. 선행 #9352를 포함하며 #9407과는 병렬이다. [원 PR](https://github.com/edwardkim/rhwp/pull/9384) · [연결 이슈](https://github.com/edwardkim/rhwp/issues/9383)

접수 당시 devel `50324d194d`에 #9352와 이 원 head를 반영한 로컬 커밋은 `c9b3a8ed9d`이다. 세 원 PR의 누적 검토 소스는 `f11b8624fd`이다. 한컴 2020 원본 인쇄 영수증은 제출된 HWP 및 PDF와 일치한다. [기여자의 인쇄 기록](../assets/side_by_side_photo_cell/hancom-print-provenance.json)

창에서 좌우 기준을 바꿔 같은 위치를 입력하는 편집을 재현하기 위해 마지막 사진 한 장의 가로 정렬과 오프셋만 바꿨다. 한컴 2020에서 원본(왼쪽, 4,000 HU)과 대조본(오른쪽, 5,718 HU)의 96 DPI 전쪽 색상 픽셀이 동일했다. 두 문서 모두 마지막 사진을 그 앞 사진 바로 아래에 배치했다. 용지 높이만 줄인 대조본은 한컴에서 같은 쪽 아래에서 일부 사진이 잘리므로 분할 행 쌓기의 정답으로 삼기 어렵다. [이번 검토 계획](hyunmyungjanelee_20261125_review_plan.md)

선행 #9365의 보정 `de678c8325`는 어울림(Square) 그림에서 각 그림의 가로 정렬과 겹침 허용을 반영했다. 이번 원 PR의 자리차지(TopAndBottom) 경로는 `unstored_cell_no_overlap_topbottom_picture`에서 왼쪽 정렬로 대상을 제한한다. 물리적으로 같은 위치인 대조본에서 한컴처럼 세로로 쌓이는지를 별도로 확인한다. [선행 보정 기록](../pr_9339_review.md)

## 메인테이너 보완

보정본은 사진 한 장의 가로 정렬을 바꿔도 실제 가로 위치와 점유 폭을 사용한다. 왼쪽·가운데·오른쪽 정렬, 음수 오프셋, 두 사진이 정확히 닿는 경계를 한컴 2020으로 각각 직접 인쇄했다. 가로로 나란한 두 장과 세로로 쌓는 두 장의 순서를 검사한다. [측정과 배치의 연결](../assets/hyunmyungjanelee-20261125/maintainer/placement.md)

최종 소스 `b12b85ad6e`에서 원본과 한 속성씩 바꾼 7개 HWP 모두 한컴 인쇄와 Native·fresh WASM 각각 100.00%였다. 오른쪽 정렬 반례의 세로 간격은 원 기여에서 −80px, 한컴과 보정 후 두 경로에서 0px였다. 가로로 나란히 놓는 사진과 단일 사진 행도 검사했다. [그림 사이의 실제 간격](../assets/hyunmyungjanelee-20261125/maintainer/picture-spacing.tsv) · [같은 위치의 정렬 대조](../assets/hyunmyungjanelee-20261125/maintainer/9384-placement.png)

## Merge 후 contributor PR comment 계획

[기여 PR](https://github.com/edwardkim/rhwp/pull/9384)에 기여와 공개 재현 자료에 대한 감사를 전하고 수용 PR·병합 SHA와 [연결 이슈](https://github.com/edwardkim/rhwp/issues/9383)를 연결한다. 원본의 가로 사진 두 장과 세로 사진 두 장, 오른쪽·가운데 정렬과 맞닿는 경계를 한컴 인쇄로 대조했다. 가로로 나란한 두 장은 같은 높이에 있고 겹치는 두 장은 80px 간격의 위·아래 원점에 있다.

원 PR의 입력·인쇄 조합에서 Native·fresh WASM 각각 1쪽을 검토했다. 자동 표시 0/2개이며, 픽셀 일치율 범위는 99.09~99.09%, 본문 픽셀 보조값은 56.48~56.63%다. 측정 대상과 보조값의 분모가 다르다. 각 페이지의 검토 그림과 오버레이 파일명·SHA-256은 [쪽별 검토표](../assets/hyunmyungjanelee-20261125/maintainer/full-review.tsv)에 있다. [Visual Sweep의 GitHub 댓글 절차](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)

첫 댓글에는 각 입력·쪽·출력 경로별 `reviews/<backend>/<case>/review_NNN.png`와 `overlay_NNN.png`를 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/hyunmyungjanelee-20261125/maintainer/` 아래로 연결한다. 실제 병합 SHA의 이미지 형식과 바이트 해시를 대조한 뒤 UTF-8 본문 파일로 게시하고 공개 댓글의 본문과 이미지를 확인한다.
