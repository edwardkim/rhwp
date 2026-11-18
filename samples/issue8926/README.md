# samples/issue8926 — TAC 표 줄 뒤 글줄의 저장 쪽 리셋

이 폴더는 [#8926](https://github.com/edwardkim/rhwp/issues/8926)의 정식 회귀 fixture다.

공개 정책연구 문서(PRISM) `6540000-202500049` 의 활용결과 보고서 서식이다. 문단 하나에
줄이 셋 있다.

| 줄 | 내용 | 저장 vpos | lh |
| --- | --- | --- | --- |
| 0 | 「<서식8>」 | 0 | 1100 |
| 1 | TAC 표 9×4 (167.6×240.4mm) | 988 | 68438 |
| 2 | 표 뒤 글줄 | **0** | 1100 |

줄 2 의 vpos 가 0 으로 되돌아간 것은 한/글이 그 줄을 다음 쪽 맨 위에 두었다는 저장
증거다. 한/글 engine 2020 Print 는 2쪽이다. 수정 전 rhwp 는 표 뒤 글줄을 1쪽 본문 하단
(933.6px) 밖 938.9px 에 남겨 1쪽이었다.

- 회귀 계약: 전체 2쪽, 「<서식8>」 줄과 표는 1쪽, 표 뒤 글줄은 2쪽에 한 번.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `rhwp info --json` 의 `lastSavedWith.product` 가
  `hancom-office-2020`(`11.0.0.8808`)이라 저장소 정책의 2022 이하 버킷이다.

기준 PDF: [`pdf/issue8926/6540000-202500049-post-table-line-stored-page-reset-2020.pdf`](../../pdf/issue8926/6540000-202500049-post-table-line-stored-page-reset-2020.pdf)
— 87,181 bytes, SHA-256 `d0917ecf26c5253658cb02d61429916b903223f795fa5f16f2e77ac8ac8a9982`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), 2쪽.
