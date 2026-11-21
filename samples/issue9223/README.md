# samples/issue9223 — 단 폭을 채운 어울림 개체와 밀린 글줄

[#9223](https://github.com/edwardkim/rhwp/issues/9223)의 정식 회귀 fixture다.

공개 보도자료(산업통상자원부, 「이산화탄소저장활용법 하위법령 제정안 입법예고」, 정책브리핑 공개, HWPX, 3쪽) 원본이다.

| 항목 | 값 |
| --- | --- |
| 대상 | 구역 0 `pi=16`, 컨트롤 둘 |
| [0] 사각형 글상자 | 어울림, 세로 문단 기준 오프셋 2943HU, 가로 문단 기준 0, 48797×4287HU(단 폭 48188HU 이상) |
| [1] 글자처럼 1×3 표 | 저장 줄 vpos 0(2쪽 맨 위), 줄 높이 3013 |
| 앞 문단 `pi=15` | 1쪽 63449HU 에서 끝남(본문 높이 70020HU) |

- 회귀 계약: 전체 3쪽. 사각형은 1쪽에서 앞 표 아래에 있고 2쪽에는 없다. 2쪽에서 넘어온 표는 제목 표(`pi=18`) 위에 있다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `lastSavedWith.product` 가 `hancom-office-2020` 이라 저장소 정책의 2022 이하 버킷이다.

기준 PDF: [`pdf/issue9223/ccus-press-release-square-band-2020.pdf`](../../pdf/issue9223/ccus-press-release-square-band-2020.pdf)
— 245,820 bytes, SHA-256 `e0d19eee4dc9af599b59638a0cfbf9a5d382901e50591c30210d22533c7ca9e4`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), 3쪽.
