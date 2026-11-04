# 제목 표 아래에서 시작하는 쪽의 독립 Print 기준

`samples/title-table-then-inline-table.hwpx`(#8144 원 재현물)와, 그 제목 표 문단 앞에
빈 문단(구역 정의만 든 문단)을 넣은 뒤 한글 2024가 HWP·HWPX로 각각 다시 저장한
`samples/title-table-after-empty-paragraph.hwp`·`.hwpx`의 Print 출력이다. 두 경계 표본의
저장 줄은 한/글이 다시 계산한 값이다(빈 문단 / 제목 표 host / 글자처럼 취급 표 문단의 첫
vpos 12246 / 13846 / 15446HU). 저장한 사람 이름만 지웠다.

- 출력: 한글 2024 `13.0.0.711`, 자동화(`HWPFrame.HwpObject`)의 `Print` 액션,
  프린터 `Microsoft Print to PDF`, 파일로 인쇄, 전체 범위, 1부, 한 장에 한 쪽 100%.
  각 PDF는 아래 표본 파일 바이트 그대로를 연 출력이다.
- 모두 1쪽이며 한컴 쪽 수와 rhwp 쪽 수가 같다.

| 표본 | 표본 SHA-256 | PDF | PDF SHA-256 |
| --- | --- | --- | --- |
| `samples/title-table-after-empty-paragraph.hwp` | `465d4f8f4b69f17bb519be5fea4dd73dd31b1ef01e9f2b9ca1daa2ca81761560` | `title-table-after-empty-paragraph-hwp-2024.pdf` | `e7b2c919f339506aeebc94223d60b0c5364914853a04ffa800182062423bc5cf` |
| `samples/title-table-after-empty-paragraph.hwpx` | `f0bd8b6931c80e86a38ebfb182df654b445380af82720ff1aa63f51a376768f0` | `title-table-after-empty-paragraph-hwpx-2024.pdf` | `c1cf793ad1da717b292ac8a9dc3ef64e65f160aacc009faad2c4053cd9d3e5ff` |
| `samples/title-table-then-inline-table.hwpx` | `a8c41628c068aa9266c742153aebde21296667235d6569c89c3e8915470e0f3d` | `title-table-then-inline-table-hwpx-2024.pdf` | `749d340a8cc4bfbdac7317acc2635a323c6008be82507ca1220e25221432f02e` |

#8144 리뷰는 한글 2024 `13.0.0.3901`로 같은 경계를 만들어 같은 vpos를 보고했다. 이
기준은 `13.0.0.711` 출력이며 빌드 사이의 차이는 따로 검증하지 않았다.
