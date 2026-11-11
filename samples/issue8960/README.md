# samples/issue8960 — 글자 높이가 정한 빈 문단 줄의 TAC 표

이 폴더는 [#8960](https://github.com/edwardkim/rhwp/issues/8960)의 정식 회귀 fixture다.

관악구의회 「후보자 등록 신청서 및 후보자 사퇴서」 공개 서식(HWPX, 2쪽)이다. 두 쪽 모두
「주 : 2. 한글로 …」 줄 아래에 회색 구분선이 있고, 그 아래가 「접수 : 사무국 -」 줄이다.

| 문단 | 내용 | 저장 vpos 차이 | lh · th · bl · ls |
| --- | --- | --- | --- |
| 앞 | 「2. 한글로 표기된 성명은 …」 | — | 1200 · 1200 · 1020 · 960 |
| 구분선 | 13pt 빈 문단 + 1×1 TAC 표(높이 482HU, 바깥 여백 0) | +2160 | **1300 · 1300 · 1105** · 392 |
| 뒤 | 「접수 : 사무국 -」 | +1692 | 1200 · 1200 · 1020 · 1200 |

구분선 줄은 글자 높이(1300)가 표 띠(482)보다 커서 글자가 줄 높이를 정했다(`th` 가 표 띠가
아니다). 한/글 engine 2020 Print 는 표 아래끝을 기준선(줄 위 + 1105HU)에 두고, 「접수」 줄을
저장 vpos 에 둔다. 수정 전 rhwp 는 표를 줄 맨 위에 두고 줄 끝을 표 아래끝으로 잘라 「접수」
줄 이하가 약 9px 위로 올라왔다.

- 회귀 계약: 전체 2쪽. 각 쪽에서 앞 글줄 위끝 → 구분선 위끝 2783HU, 구분선 아래끝 → 「접수」
  줄 위끝 587HU(각 ±1px).
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `rhwp info --json` 의 `lastSavedWith.product` 가
  `hancom-office-2020`(`11.0.0.8227`)이라 저장소 정책의 2022 이하 버킷이다.

기준 PDF: [`pdf/issue8960/36439173-empty-tac-line-baseline-2020.pdf`](../../pdf/issue8960/36439173-empty-tac-line-baseline-2020.pdf)
— 105,116 bytes, SHA-256 `ebbb58a6454508fc9bb4bc55b305bdaf1aa0fdd9d8783cf590275e7ccaa7fa94`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), 2쪽.
