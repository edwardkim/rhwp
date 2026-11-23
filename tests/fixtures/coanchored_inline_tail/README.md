# 인라인 중첩 표 뒤의 음수 줄 간격

공개 합성 입력 [`negative-spacing.hwp`](negative-spacing.hwp)는 [한컴에서 저장한 중첩 표 예제](https://github.com/edwardkim/rhwp/blob/fedc716cbe810985c798254b370962e24aa4ffcf/samples/pr9042/nested-tail-margins-hancom.hwp)에 제목 표와 `Tail line` 문단을 추가한 파일이다. 둘 다 원본에 있던 한 칸 표의 문단 서식을 사용한다. 제목 표와 본문 표를 담은 바깥 줄의 간격은 [#9042의 제목·본문 표 예제](https://github.com/puckqook/rhwp/blob/3ae4eeb332c3b0edbb3a8fbe334daf2b0a48115b/mydocs/pr/assets/coanchored-table-pagination/float-pagination.hwp)에서 가져왔다.

중첩 표를 담은 저장 줄은 높이가 1,648 HWPUNIT이다. 이 중 하단 바깥 여백은 283 HWPUNIT이다. 줄 간격은 -56 HWPUNIT이고, 다음 줄은 중첩 표가 그려지는 영역 아래에서 시작한다.

[한컴오피스 한글 Viewer 12.31.9에서 인쇄한 참고 PDF](../../../pdf/coanchored-inline-tail-negative-spacing/hancom-mac-viewer-print.pdf)는 2쪽이다. 1쪽에 제목 표, 본문 표, `Tail line`이 있고 2쪽에 `Next page` 표가 있다.

- HWP SHA-256 `5fe29c822bcb48cd326d5e5ebf7581899478507e6634db8ac1f8d118478659a1`
- PDF SHA-256 `a3b93b9f4a2bd811ee11a82ed2e8f2a247aa2fab0bc5ecfe47bf3c0840850e19`
