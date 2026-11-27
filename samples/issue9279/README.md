# samples/issue9279 — 글자처럼 취급 표 줄 뒤의 도형 줄

[#9279](https://github.com/edwardkim/rhwp/issues/9279)의 정식 회귀 fixture다.

산림청 보도자료 공개 문서(「DMZ의 숨결을 품은 북방계식물, 한 달간 국민과 만나다」, HWPX, 3쪽)
원본이다.

| 항목 | 값 |
| --- | --- |
| 대상 문단 | 구역 0 문단 14 — 글 없음, 컨트롤 2개, 문단 뒤 간격 1400HU(9.33px) |
| 저장 줄 0 | `vpos=20020`, `lh=3696`, `ls=840` — 글자처럼 취급 표(2×6 「담당 부서」) |
| 저장 줄 1 | `vpos=24556`, `lh=4287`, `ls=780` — 글자처럼 취급 사각형(로고 줄) |
| 저장 줄 전진 | 24556 − 20020 = 4536HU = 60.48px |

- 회귀 계약: 전체 3쪽. 2쪽에서 표 줄 위에서 사각형 위까지의 거리가 저장 줄 전진과 같고,
  문단 뒤 간격은 사각형 줄 뒤에 온다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `version.xml` 의 `appVersion="10, 0, 0, 13462"`(2024 저장본이 아니므로
  engine 2020).

기준 PDF: [`pdf/issue9279/forest-press-release-tac-table-shape-tail-2020.pdf`](../../pdf/issue9279/forest-press-release-tac-table-shape-tail-2020.pdf)
— 1,266,253 bytes, SHA-256 `4ad23be4027b145e32f32caa104b972a06c562fb3a1a88817cbc636c529dc9e8`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), 3쪽.
