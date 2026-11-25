# samples/issue9396 — 마지막 줄 앵커 자리차지 그림이 다음 쪽 host 줄과 함께 있는 저장 띠

[#9396](https://github.com/edwardkim/rhwp/issues/9396)의 정식 회귀 fixture다.

10k 공개 코퍼스의 소방청 「소방방재 시스템 점검메뉴얼」 Ch2(HWP 5.0.0.6, 558쪽)에서 71~75쪽만
남긴 **파생본**(7쪽)이다. 원본 전체에는 이 이슈와 무관한 쪽 흐름 차이가 많아, 결함 쪽 앞뒤만
이 이슈의 범위로 검증한다.

| 항목 | 값 |
| --- | --- |
| 원본 | `1660000-200700020_D0150004-2-001_2_Ch2.소방방재 시스템 점검메뉴얼.hwp`, 5,494,272 bytes, SHA-256 `c89121c5…5215e`, 558쪽 |
| 생성 1 | `rhwp extract-pages <원본> <출력> --from 71 --to 75` (쪽 범위 문단만 남김, 구역마다 최소 1문단) |
| 생성 2 | `rhwp edit delete-control --section 0 --para 0 --ctrl 6` (표지 제목 글 상자 삭제 — 상용 표지 글꼴 대체 차이만 있는 범위 밖 요소) |
| 보존 | 원본 73·74쪽 내용이 4·5쪽이 된다. 구역 1 문단 28 = 원본 1211 |
| 그림 | 문단 28 마지막 줄(ts 415, vpos 18000)의 유일한 글자, 자리차지·문단 기준 오프셋 2161·높이 9900 |
| 저장 사다리 | 다음 문단 29 vpos 30061 = 18000 + 2161 + 9900 |

- 회귀 계약: [MANIFEST.json](MANIFEST.json)의 `contract`, 검사는
  `tests/cases/issue_9396_last_line_anchor_picture_band.rs`.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기.
- 기준 엔진: **2020** — 원본 저장 제품(한/글 2007 계열 5.0.0.6)에 맞춰 한/글 2020 으로 인쇄했다.

기준 PDF: [`pdf/issue9396/fire_inspection_manual_p71-75-2020.pdf`](../../pdf/issue9396/fire_inspection_manual_p71-75-2020.pdf)
— 190,794 bytes, SHA-256 `d3cf952ee105a522d30dd4dacbbb5112ece7d0a28c7896d2963019fb9180f159`,
한/글 2020(11.0.0.1623) `Hancom PDF` Print(`hancom2020_pdf_driver_one_up`), 7쪽.
