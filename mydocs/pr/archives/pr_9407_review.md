# PR #9407 검토 · 떠 있는 중첩 표의 행 분할과 이어받기

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 기여 `950d846040769a7180532de1e5132f902b2ed7a4`의 기능에 떠 있는 중첩 표의 행 소속, 바깥 칸의 물리 높이와 마지막 행 뒤 이어받기를 보완한 제품 소스는 `b12b85ad6ec037fff9ce41ac6fc34b8865d93920`이다. 대조 기준은 `50324d194d1cd7c46d168f07a58515086530f03b`이다. 원 기여에서 남았던 증상과 보정본의 측정·배치 경로를 아래에 기록했다. 문서별로 승인된 글꼴 및 인쇄 조건으로 [통합 PR #9412](https://github.com/edwardkim/rhwp/pull/9412)를 `a5f01c69392dd4cf6919c8291023ddc84466e181`에서 검토했다. 필수 CI와 [셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9412#pullrequestreview-6063886947)를 마치고 [커밋 `5fa8cede504e`](https://github.com/edwardkim/rhwp/commit/5fa8cede504eca188d2e003931a50e9a8434d6da)으로 `devel`에 병합했다. [세 PR의 최종 검토](hyunmyungjanelee_20261125_review.md) · [원 기여와 보정 후 쪽별 측정](../assets/hyunmyungjanelee-20261125/maintainer/original-pr-comparison.tsv)

#9415의 재저장 글상자와 크기 보호 처리를 병합한 제품 소스 `f3d137d9cbe36cae93f859baac3a29ae4a543f75`에서 같은 입력·한컴 Print로 다시 검증했다. 전체 회귀 14,166건, Native Skia 4,227건과 세 PR 및 새 기준의 집중 회귀 64건을 통과했다. [현재 기준에서 세 PR의 쪽별 결과](../assets/hyunmyungjanelee-20261125/maintainer/current-9415/README.md)

#9411을 병합한 제품 소스 `8d1403eb14bbefb35690a50b6734b0c2ef9ba5cb`에서 이 기여의 입력·인쇄본과 전체 보호 문서를 다시 검증했다. [최신 기준에서의 검증 결과](../assets/hyunmyungjanelee-20261125/maintainer/current-develop/README.md)

원 기여자는 hyunmyungJaneLee이고 원 head는 `950d846040769a7180532de1e5132f902b2ed7a4`이다. 부모 #9352에 이어 #9406의 쪽수, 행의 소속, 중첩 표의 다음 쪽 내용, 바깥 괘선과 후속 문단을 검토한다. [원 PR](https://github.com/edwardkim/rhwp/pull/9407) · [연결 이슈](https://github.com/edwardkim/rhwp/issues/9406)

기준은 `50324d194d`이고 세 원 기여를 보존한 누적 소스는 `f11b8624fd`이다. 원 head의 CodeQL 요약은 devel에 있는 세 분석 구성을 찾지 못해 NEUTRAL이다. 제출된 PDF 네 개는 HWPX 입력에 대응한다. 제출된 `overlay_nested_anchor_offset.hwp`는 독립 한컴 2020에서 열기 경고로 인쇄가 중단됐다. [기여자의 인쇄 기록](../assets/pk40991_overlay_nested/hancom-print-provenance.json)

`overlay_nested_grow_split.hwpx`에서 중첩 표의 아래 바깥 여백 한 속성만 283 HU에서 0 또는 2,835 HU로 바꾸고 한컴 2020에서 각각 인쇄했다. 두 대조본 모두 2쪽이고, 1쪽 바깥 칸의 아래 괘선은 원본과 동일한 766.32 pt에 있다. 여백 변화는 안쪽 표와 이어지는 내용의 위치에 반영된다. 아래 정렬로 바꾼 별도 대조본도 인쇄해 비교한다. [이번 검토 계획](hyunmyungjanelee_20261125_review_plan.md)

#6976의 쪽 끝 접기는 #7568 통합 커밋 `731de9e1b4`에서 마지막 줄의 저장 줄간격과 실제 남는 높이를 사용했고, 후속 #7685는 행 경계에서 이어지는 표의 위 여백을 보완했다. 이번 `decoration_table::row_fragment::query`가 외부 칸 프레임에서 중첩 표의 아래 여백을 빼는 경로와 구분해 검토한다. [#6976](https://github.com/edwardkim/rhwp/issues/6976)

중첩 표 4종의 제출 HWP 네 파일 모두 동일 바이트로 한컴 2024에서 열고 직접 인쇄했다. 같은 네 파일은 한컴 2020에서 문서 열기 경고가 발생했다. 따라서 제출 HWP의 독립 검사에는 파일별 2024 인쇄본과 2024 호환 모드를 사용하고, HWPX의 2020 인쇄본을 사용한 기여자 수치는 별도 대조로 기록한다. [제출 HWP의 버전별 인쇄 기록](../assets/hyunmyungjanelee-20261125/submitted-hwp-print.tsv)

## 메인테이너 보완

중첩 행의 소유 범위와 바깥 칸의 물리 높이를 분리해 첫 쪽과 이어받기에서 같은 컷을 사용한다. 아래 여백 10mm 대조군의 두 번째 쪽은 중첩 행 26번에서 시작한다. 중첩 표를 전부 수용한 뒤 뒤 행만 넘기는 경우에는 다음 쪽에 `TAIL ROW`만 남고 `END`는 첫 쪽에 있다. 한컴 2024 실제 재저장본도 직접 다시 인쇄했다. [측정과 배치의 연결](../assets/hyunmyungjanelee-20261125/maintainer/placement.md)

최종 소스 `b12b85ad6e`에서 직접 대응하는 문서 27개·48쪽의 최저는 Native 99.67%, fresh WASM 99.76%다. 아래 여백 10mm인 두 번째 쪽은 74.86%에서 두 경로 모두 100.00%가 됐다. 한컴으로 저장하고 다시 연 두 분할 HWP는 인쇄와 같은 두 쪽이다. 용지 높이만 바꾼 추가 대조의 원래 실루엣, 프린터 용지 크기, 각 쪽의 중첩 행·뒤 행·본문 소속도 기록했다. [두 번째 쪽 확대](../assets/hyunmyungjanelee-20261125/maintainer/9407-placement.png) · [용지 높이별 수치](../assets/hyunmyungjanelee-20261125/maintainer/README.md)

## Merge 후 contributor PR comment 계획

[기여 PR](https://github.com/edwardkim/rhwp/pull/9407)에 기여와 공개 재현 자료에 대한 감사를 전하고 수용 PR·병합 SHA와 [연결 이슈](https://github.com/edwardkim/rhwp/issues/9406)를 연결한다. 원본·한컴 재저장본과 아래 여백·정렬·용지 높이 경계를 각각 인쇄했다. 2쪽의 첫 중첩 행과 마지막 행, 표 밖 본문 END, 뒤쪽 바깥 행 TAIL ROW가 한컴 인쇄와 같은 쪽에 있다.

원 PR의 입력·인쇄 조합에서 Native·fresh WASM 각각 18쪽을 검토했다. 자동 표시 0/36개이며, 픽셀 일치율 범위는 95.11~99.51%, 본문 픽셀 보조값은 21.59~41.23%다. 측정 대상과 보조값의 분모가 다르다. 각 페이지의 검토 그림과 오버레이 파일명·SHA-256은 [쪽별 검토표](../assets/hyunmyungjanelee-20261125/maintainer/full-review.tsv)에 있다. [Visual Sweep의 GitHub 댓글 절차](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)

첫 댓글에는 각 입력·쪽·출력 경로별 `reviews/<backend>/<case>/review_NNN.png`와 `overlay_NNN.png`를 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/hyunmyungjanelee-20261125/maintainer/` 아래로 연결한다. 실제 병합 SHA의 이미지 형식과 바이트 해시를 대조한 뒤 UTF-8 본문 파일로 게시하고 공개 댓글의 본문과 이미지를 확인한다.
