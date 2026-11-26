# 인라인 중첩 표 뒤의 음수 줄 간격

공개 합성 입력 [`negative-spacing.hwp`](negative-spacing.hwp)는 [한컴에서 저장한 중첩 표 예제](https://github.com/edwardkim/rhwp/blob/fedc716cbe810985c798254b370962e24aa4ffcf/samples/pr9042/nested-tail-margins-hancom.hwp)에 제목 표와 `Tail line` 문단을 추가한 파일이다. 둘 다 원본에 있던 한 칸 표의 문단 서식을 사용한다. 제목 표와 본문 표를 담은 바깥 줄의 간격은 [#9042의 제목·본문 표 예제](https://github.com/puckqook/rhwp/blob/3ae4eeb332c3b0edbb3a8fbe334daf2b0a48115b/mydocs/pr/assets/coanchored-table-pagination/float-pagination.hwp)에서 가져왔다.

중첩 표를 담은 저장 줄은 높이가 1,648 HWPUNIT이다. 이 중 하단 바깥 여백은 283 HWPUNIT이다. 줄 간격은 -56 HWPUNIT이고, 다음 줄은 중첩 표가 그려지는 영역 아래에서 시작한다.

[Windows 한컴 2020 Print](../../../pdf/coanchored-inline-tail-negative-spacing/negative-spacing-2020.pdf)는 2쪽이다. 1쪽에 제목 표, 본문 표, `Tail line`이 있고 2쪽에 `Next page` 표가 있다. 공개 원본에서 비어 있던 스크립트 저장소를 제거했고 나머지 스트림은 바이트 단위로 유지했다. [Print 영수증](../../../mydocs/pr/assets/coanchored-inline-tail-negative-spacing/print-receipt.json)

- HWP SHA-256 `5ce948af486c1c745153fa21e49479dcba1031bd2eefa97f1a889795b8261f18`
- PDF SHA-256 `56f20f5abfdce8ccf3555d6e73b487de2c603b9b0d647f04fdc91d6cd3c4763d`

## 한컴 재저장 대조군

- [160% 재저장본](negative-spacing-hancom-resaved.hwp)과 [독립 Print PDF](../../../pdf/coanchored-inline-tail-negative-spacing/negative-spacing-hancom-resaved-2020.pdf)
- [90% 재저장본](negative-spacing-90-percent-hancom-resaved.hwp)과 [독립 Print PDF](../../../pdf/coanchored-inline-tail-negative-spacing/negative-spacing-90-percent-hancom-resaved-2020.pdf)

한컴 2020 11.0.0.9172의 재저장·인쇄 영수증과 Native·fresh WASM 전쪽 TSV는
[통합 검토 증거](../../../mydocs/pr/assets/puckqook-9042-integration/README.md#한컴-재저장-대조군)에 있다.
