# PR #9352 검토 · 칸 안 종이·쪽 기준 도형

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 기여 `04d560696bedf76ee589fd538deb7705bcca281d`의 기능에 종이를 기준으로 놓은 흰 도형의 앞뒤 그리기와 중첩 글상자 안쪽의 세로 정렬을 보완한 제품 소스는 `b12b85ad6ec037fff9ce41ac6fc34b8865d93920`이다. 대조 기준은 `50324d194d1cd7c46d168f07a58515086530f03b`이다. 원 기여에서 남았던 증상과 보정본의 측정·배치 경로를 아래에 기록했다. 문서별로 승인된 글꼴 및 인쇄 조건으로 [통합 PR #9412](https://github.com/edwardkim/rhwp/pull/9412)를 `a5f01c69392dd4cf6919c8291023ddc84466e181`에서 검토했다. 필수 CI와 [셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9412#pullrequestreview-6063886947)를 마치고 [커밋 `5fa8cede504e`](https://github.com/edwardkim/rhwp/commit/5fa8cede504eca188d2e003931a50e9a8434d6da)으로 `devel`에 병합했다. [세 PR의 최종 검토](hyunmyungjanelee_20261125_review.md) · [원 기여와 보정 후 쪽별 측정](../assets/hyunmyungjanelee-20261125/maintainer/original-pr-comparison.tsv)

#9415의 재저장 글상자와 크기 보호 처리를 병합한 제품 소스 `f3d137d9cbe36cae93f859baac3a29ae4a543f75`에서 같은 입력·한컴 Print로 다시 검증했다. 전체 회귀 14,166건, Native Skia 4,227건과 세 PR 및 새 기준의 집중 회귀 64건을 통과했다. [현재 기준에서 세 PR의 쪽별 결과](../assets/hyunmyungjanelee-20261125/maintainer/current-9415/README.md)

#9411을 병합한 제품 소스 `8d1403eb14bbefb35690a50b6734b0c2ef9ba5cb`에서 이 기여의 입력·인쇄본과 전체 보호 문서를 다시 검증했다. [최신 기준에서의 검증 결과](../assets/hyunmyungjanelee-20261125/maintainer/current-develop/README.md)

원 기여자는 hyunmyungJaneLee이고 원 head는 `04d560696bedf76ee589fd538deb7705bcca281d`이다. #9349의 빈 칸이 커지고 도형이 잘리는 증상, 글상자 안의 도형, 쪽을 넘는 행의 첫 조각을 검토한다. [원 PR](https://github.com/edwardkim/rhwp/pull/9352) · [연결 이슈](https://github.com/edwardkim/rhwp/issues/9349)

접수 당시 devel `50324d194d`에 원 head를 적용한 로컬 커밋은 `c16783823f`이다. 후속 #9384·#9407과의 누적 검토 소스는 `f11b8624fd`이다. 원 head의 CodeQL 요약은 devel에 있는 세 분석 구성을 찾지 못해 NEUTRAL이다. [검토 계획](hyunmyungjanelee_20261125_review_plan.md)

한컴 2020 PDF 네 개의 기록은 각각 HWPX를 입력으로 명시하며, 제출된 PDF의 SHA-256과 일치한다. 같은 이름의 HWP는 별도 입력이다. 한컴 2020에서 제출된 `paper_square_split.hwp`를 직접 인쇄할 때 문서 열기 경고로 변환이 중단됐다. HWPX의 인쇄 결과와 한컴으로 다시 저장한 HWP의 결과를 구분해 대조한다. [기여자가 기록한 입력·PDF](../assets/pk40991_cell_paper_shape/hancom-print-provenance.json)

#9298에 수용된 셀 내용 그림은 괘선이 그림 앞에 그려진다. 이번 독립 Print의 종이 기준 흰 사각형은 반대로 괘선을 덮는다. 분할 표 경로는 먼저 종이 기준 도형을 칸 뒤로 옮기고 뒤에서 표 괘선을 붙인다. 검토 중인 수정은 두 개체의 그리기 순서를 각각 보호해야 한다. [#9298의 셀 그림 검증](pr_9298_review.md)

칸 안 사각형 4종의 제출 HWP 네 파일 모두 동일 바이트로 한컴 2024에서 열고 직접 인쇄했다. 같은 네 파일은 한컴 2020에서 문서 열기 경고가 발생했다. 따라서 제출 HWP의 독립 검사에는 파일별 2024 인쇄본과 2024 호환 모드를 사용하고, HWPX의 2020 인쇄본을 사용한 기여자 수치는 별도 대조로 기록한다. [제출 HWP의 버전별 인쇄 기록](../assets/hyunmyungjanelee-20261125/submitted-hwp-print.tsv)

### HWP 5 영문 글꼴 표가 빈 경우

제출된 분할 HWP를 한컴 2024에서 직접 인쇄하면 영문과 숫자에 한컴바탕이 쓰인다. 원본의 영문 글꼴 표에 한컴바탕 한 항목만 넣은 대조본은 두 쪽 모두 원본 한컴 인쇄와 픽셀이 같다. 함초롬바탕을 명시하면 한컴의 인쇄 결과도 달라진다. rhwp는 빈 영문 표에서 한글 슬롯의 함초롬바탕을 선택한다. 원본 두 쪽의 Native 일치율은 95.98%·96.21%이며, 영문 슬롯을 명시한 대조본과 실제 한컴 재저장본에서는 두 쪽 모두 100.00%이다. 11월 25일 작업지시자가 빈 영문 글꼴 표의 대체 처리는 별도 검토로 보류했다. [입력·한컴 인쇄 해시와 Native·WASM 쪽별 측정](../assets/hyunmyungjanelee-20261125/maintainer/hwp5-latin-slot.tsv)

## 메인테이너 보완

보정본에서는 종이 기준 도형의 글 앞·글 뒤 속성을 괘선 그리기가 끝날 때까지 보존한다. 같은 흰 사각형을 앞과 뒤로 바꾼 인쇄본, 글상자 내부의 독립 도형, #9298의 셀 내용 그림을 함께 확인했다. [측정과 배치의 연결](../assets/hyunmyungjanelee-20261125/maintainer/placement.md)

원 기여에서 분할 HWPX의 흰 도형 안으로 괘선 130줄이 보였다. 보정 소스 `b12b85ad6e`의 Native·fresh WASM은 한컴처럼 도형 안에서 0줄, 위아래에서 65줄·60줄을 그린다. 글 뒤 배치와 기존 #9298의 일반 셀 그림도 각각 원래 그리기 순서를 유지했다. [앞뒤 배치 확대](../assets/hyunmyungjanelee-20261125/maintainer/9352-placement.png) · [쪽별 판정](../assets/hyunmyungjanelee-20261125/maintainer/README.md)

같은 호스트에서 원 기여 커밋도 다시 빌드해 Native 12쪽을 대조했다. 분할 HWP 자체를 한컴 2024로 인쇄한 두 쪽에서 일치율은 95.55%→95.98%, 96.21%→96.21%였다. 두 번째 쪽의 PNG는 보정 전후 바이트가 같다. [동일 입력·글꼴·인쇄본으로 재측정한 원 PR](../assets/hyunmyungjanelee-20261125/maintainer/pr9352-same-host.tsv)

## Merge 후 contributor PR comment 계획

[기여 PR](https://github.com/edwardkim/rhwp/pull/9352)에 기여와 공개 재현 자료에 대한 감사를 전하고 수용 PR·병합 SHA와 [연결 이슈](https://github.com/edwardkim/rhwp/issues/9349)를 연결한다. 분할 HWP 2쪽의 96.22%는 원 기여자가 보고한 값이다. 같은 호스트의 원 PR과 보정본은 이 쪽이 모두 96.21%이고 PNG의 바이트도 같다. 글 앞의 흰 사각형 아래는 괘선이 가려지고, 글 뒤의 사각형에서는 괘선이 보인다.

원 PR의 입력·인쇄 조합에서 Native·fresh WASM 각각 15쪽을 검토했다. 자동 표시 0/30개이며, 픽셀 일치율 범위는 95.78~98.39%, 본문 픽셀 보조값은 19.32~35.94%다. 측정 대상과 보조값의 분모가 다르다. 각 페이지의 검토 그림과 오버레이 파일명·SHA-256은 [쪽별 검토표](../assets/hyunmyungjanelee-20261125/maintainer/full-review.tsv)에 있다. [Visual Sweep의 GitHub 댓글 절차](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)

첫 댓글에는 각 입력·쪽·출력 경로별 `reviews/<backend>/<case>/review_NNN.png`와 `overlay_NNN.png`를 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/hyunmyungjanelee-20261125/maintainer/` 아래로 연결한다. 실제 병합 SHA의 이미지 형식과 바이트 해시를 대조한 뒤 UTF-8 본문 파일로 게시하고 공개 댓글의 본문과 이미지를 확인한다.
