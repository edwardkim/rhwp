# samples/issue9233 — 정렬 배치 그림 채우기와 영역보다 큰 그림

[#9233](https://github.com/edwardkim/rhwp/issues/9233)의 정식 회귀 fixture다.

서울시교육청 공개 문서(「2018학년도 후기고등학교 입학전 전학 시행 계획(안)」, HWP5, 22쪽) 원본이다.

| 항목 | 값 |
| --- | --- |
| 대상 | 구역 0 문단 2 의 3×1 표, 첫 행·셋째 행 칸 |
| 칸 | 46775×2596HU(623.7×34.6px), 글 없음 |
| 칸 배경 | 그림 채우기 — BinData 2 `LeftTop`(첫 행), BinData 3 `RightBottom`(셋째 행) |
| 그림 | 344×131px PNG(RGBA) |

- 회귀 계약: 전체 22쪽. 1쪽의 두 채우기 그림 사각형은 칸 안에 들어 있고, 높이는 칸 높이로 줄며 폭은 그림 픽셀 폭 그대로다. 첫 그림은 칸 왼쪽, 둘째 그림은 칸 오른쪽에 붙는다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `lastSavedWith.product` 가 `hancom-office-2010` 이라 저장소 정책의 2022 이하 버킷이다.

기준 PDF: [`pdf/issue9233/high-school-transfer-plan-cover-band-2020.pdf`](../../pdf/issue9233/high-school-transfer-plan-cover-band-2020.pdf)
— 457,395 bytes, SHA-256 `8537af724977c49e283cecb6fd162a89bd096884fe36f23cc9f1172f11b694f4`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), 22쪽. 쪽 맨 아래 「파일 이름 … 22쪽 중 1쪽」 줄은 한 쪽씩 인쇄한 경로의 인쇄 머리말이며 문서 내용이 아니다.
