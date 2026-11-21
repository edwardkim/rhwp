# samples/issue9217 — 본문 폭을 채운 사각형의 평행 그림자

[#9217](https://github.com/edwardkim/rhwp/issues/9217)의 정식 회귀 fixture다.

공개 연구용역 과업지시서(경찰청, 「검사 독점적 영장 신청권의 합리적 개선방향」, 정책연구관리시스템 공개, hwp5, 14쪽) 원본이다.

| 항목 | 값 |
| --- | --- |
| 대상 | 구역 0 `pi=15`, 글자처럼 취급한 묶음 개체(47591×69305HU) |
| 목차 사각형 | 묶음의 첫 자식, 폭 47591HU(본문 폭 45128HU 보다 넓다) |
| 그림자 | 오른쪽 아래 평행 그림자, 기본 이동량 600HU + 저장 offset 0 |

- 회귀 계약: 전체 14쪽. 2쪽에서 목차 사각형을 그림자 이동량만큼 옮긴 상자가 본문 clip 안에 있다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `lastSavedWith.product` 가 `hancom-office-2010` 이라 저장소 정책의 2022 이하 버킷이다.

기준 PDF: [`pdf/issue9217/research-task-order-toc-shadow-2020.pdf`](../../pdf/issue9217/research-task-order-toc-shadow-2020.pdf)
— 515,130 bytes, SHA-256 `2ff3412577a02231f4b075fcbce8702e812e910e8d40e93e589c64cd638f54c7`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), 14쪽.
