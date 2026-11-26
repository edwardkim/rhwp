# samples/issue9419 — 한/글 저장 HWPX 의 쪽 첫 글자처럼 취급 표

[#9419](https://github.com/edwardkim/rhwp/issues/9419)의 회귀 fixture다.

| 파일 | 만든 방법 |
| --- | --- |
| `kaf_appendix_2020.hwpx` | [`samples/issue9379/kaf_appendix.hwp`](../issue9379/README.md)(54쪽)를 한/글 2020(11.0.0.1623)으로 열어 HWPX 로 다시 저장. 바이트 수정 없음 |

## 대상

21·23쪽(가로 쪽)은 쪽 나누기 뒤 빈 문단 하나가 글자처럼 취급 표(자리차지, 쪽 나눔 허용) 하나를 가진다.

| 쪽 | 문단(구역 1) | 위·아래 간격 | 저장 첫 줄 |
| --- | --- | --- | --- |
| 21 | 12 | 600·600 HU | vpos 600, 줄 높이 46150, 줄간격 960 |
| 23 | 14 | 600·600 HU | vpos 600, 줄 높이 47254, 줄간격 840 |

저장 첫 줄 vpos 600 은 쪽 맨 위에서도 적용한 문단 위 간격이다. 한/글 2020 Print 는 HWP 와 HWPX 모두
표 테두리를 단 위끝에서 위 간격과 바깥 위 여백만큼 아래에 그린다(96 DPI 래스터 내용 윗변 122px).
두 형식의 이 문단 저장값(문단 모양·줄·표)은 같다.

## 계약과 기준

- 회귀 계약: [MANIFEST.json](MANIFEST.json)의 `contract`, 검사는
  `tests/cases/issue_9419_hwpx_page_top_tac_table_keeps_spacing_before.rs`.
- 기준 엔진: **2020** — 저장 제품(한/글 2020)과 같은 엔진의 `Hancom PDF` Print(`hancom2020_pdf_driver_one_up`).
  기준 PDF 는 `pdf/issue9419/kaf_appendix_2020-2020.pdf`(54쪽, Creator `Hwp 2018 11.0.0.1623`).
