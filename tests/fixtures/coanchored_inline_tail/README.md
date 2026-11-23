# 인라인 중첩 표 뒤의 음수 줄 간격

`negative-spacing.hwp`는 [#9042의 제목·본문 표 예제](https://github.com/puckqook/rhwp/blob/3ae4eeb332c3b0edbb3a8fbe334daf2b0a48115b/mydocs/pr/assets/coanchored-table-pagination/float-pagination.hwp)와 [#9066의 한컴 저장 중첩 표 예제](https://github.com/edwardkim/rhwp/blob/fedc716cbe810985c798254b370962e24aa4ffcf/samples/pr9042/nested-tail-margins-hancom.hwp)를 바탕으로 만든 공개 합성 입력이다. 본문 표의 마지막 칸에 중첩 표와 `Tail line` 문단을 넣었다.

중첩 표를 담은 저장 줄의 높이는 1,648 HWPUNIT이다. 그중 아래쪽 바깥 여백은 283 HWPUNIT이다. 저장된 줄 간격은 -56 HWPUNIT이고, 다음 줄의 시작은 중첩 표가 그려지는 영역의 아래쪽보다 뒤에 있다.

[한컴오피스 한글 Viewer 12.31.9에서 인쇄한 PDF](../../../pdf/coanchored-inline-tail-negative-spacing/hancom-print.pdf)는 2쪽이다. 1쪽에 제목 표, 본문 표와 `Tail line`이 있다. 2쪽에는 `Next page` 표가 있다.

- HWP SHA-256 `5529400189cff18bf85f640c87c82bfb7643d4fe40540bcb8d5f9373e2312308`
- PDF SHA-256 `56d96cbdc88c645acf633aeefa0e7ccb98b95b617ed895fded044d92887af9bb`
