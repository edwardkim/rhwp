# samples/issue9009 — 이어진 표 조각 뒤 [단 나누기] 문단

이 폴더는 [#9009](https://github.com/edwardkim/rhwp/issues/9009)의 정식 회귀 fixture다.

공개 연구보고서 「진도보고서(특허청) 산업재산권」(한국법제연구원, 정책연구관리시스템 PRISM 공개, HWP, 8쪽) 원본이다.

| 항목 | 값 |
| --- | --- |
| 표 | pi=14, 116행×3열, 비-TAC 자리차지, 5쪽에서 시작해 7쪽까지 이어짐(표를 가진 문단의 줄은 5쪽) |
| 7쪽 | 표 끝 조각(65~116행)만, 아래끝 922.0px(본문 941.1px) |
| 뒤 문단 | pi=15 빈 문단, [단 나누기], 저장 vpos 69148HU(921.97px) |
| 그 뒤 | pi=16 「나. 연구의 진척도」, [쪽 나누기], 저장 vpos 0 |

한/글은 pi=15 를 7쪽 표 조각 아래에 두고 8쪽을 pi=16 으로 시작한다(전체 8쪽). 수정 전 rhwp 는
pi=15 의 단 나누기로 새 쪽을 열어 8쪽에 빈 문단만 두고 9쪽이 됐다.

- 회귀 계약: 전체 8쪽. 8쪽은 「나. 연구의 진척도」로 시작하고, 7쪽 뒤에 글자 없는 쪽이 없다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `rhwp info --json` 의 `lastSavedWith.product` 가 없음(`9.1.1.5499`)이라
  저장소 정책의 2022 이하 버킷이다.

기준 PDF: [`pdf/issue9009/1430000-column-break-after-table-continuation-2020.pdf`](../../pdf/issue9009/1430000-column-break-after-table-continuation-2020.pdf)
— 378,118 bytes, SHA-256 `66c18d4b85e52aafe7b63e6885eea8a1b26e62dcee78577ece62f0ec007781f3`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), Producer `Hancom PDF 1.3.0.547`, 8쪽.
