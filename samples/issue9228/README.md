# samples/issue9228 — 어울림 표 양옆 차선에 이어진 뒤 문단

[#9228](https://github.com/edwardkim/rhwp/issues/9228)의 정식 회귀 fixture다.

공개 보도자료(산업통상자원부, 「산업융합 규제샌드박스 500건 돌파, 성과확산 위해 샌드박스 2.0 체제로 전환」, 정책브리핑 공개, HWPX, 21쪽) 원본이다.

| 항목 | 값 |
| --- | --- |
| 대상 | 구역 0, 6쪽 `pi=76·78·80·82`(host)와 `pi=77·79·81·83`(뒤 문단) |
| host 의 표 | 어울림 1×1, 글자처럼 취급 아님, 세로 문단 기준 280·800·780·300HU, 가로 문단 기준 3480·3540HU, 6306×2131HU, 바깥 여백 141HU |
| 저장 줄 | host·뒤 문단 모두 같은 `vertical_pos` 에 두 조각 — `cs=0, sw=3480`(왼쪽 차선), `cs=10068, sw=38120`(오른쪽 차선) |
| 저장 사다리 | host 줄만 전진 — `pi=76` vpos 45493(lh 400, ls 120) → `pi=77` vpos 46013 |

- 회귀 계약: 전체 21쪽. 6쪽에서 각 host 표의 다음 문단 첫 줄이 표와 세로로 겹치고 조각이 표 왼쪽·오른쪽에 하나씩 놓인다. 네 문단의 줄은 서로 겹치지 않고 뒤 제목 문단(`pi=85`)은 마지막 표 아래에 있다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — 저장 버전 `11.0.0.6350`(`hancom-office-2020` 버킷)이라 저장소 정책의 2022 이하 버킷이다.

기준 PDF: [`pdf/issue9228/regulatory-sandbox-press-release-flanking-box-2020.pdf`](../../pdf/issue9228/regulatory-sandbox-press-release-flanking-box-2020.pdf)
— 1,933,742 bytes, SHA-256 `2e99ad20b7fff4f26af41e2c31c5e30f03f73afeef8b8627b10e3b25a12a133c`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), 21쪽.
