# samples/issue6795 — 쪼개진 자리차지 표의 형제 표 회귀 입력

이 폴더는 [#6795](https://github.com/edwardkim/rhwp/issues/6795)의 정식 회귀 fixture다.

빈 host 문단(`pi=113`)에 자리차지(`wrap=위아래`, `vert=문단`, 오프셋 0) 표 두 장이
매달려 있다. 첫 표(`ci=0`, 27×10)가 커서 쪽을 넘기면 그 조각은
`PageItem::PartialTable` 로 나가는데, 뒤 표(`ci=1`, 19×6)를 배치하는 조판 경로 두
곳이 모두 그 조각을 못 봐 **같은 앵커에 겹쳐** 놓았다.

```text
회귀 시 31쪽
  pi=113 ci=0  y=143.6..560.1   (조각)
  pi=113 ci=1  y=158.2..854.2   → 548.0 × 401.9px 겹침, 아래 표가 통째로 가려진다
```

- 회귀 계약: 31쪽에 `pi=113 ci=0` 조각 단독, 32쪽에 `pi=113 ci=1` 단독(본문 안),
  33쪽에 `pi=114 ci=0`. 즉 겹침 없음 + 문서 순서 보존.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `rhwp info --json` 의 `lastSavedWith.product` 가 `null`
  (`version 6.7.6.1002`)이라 저장소 정책
  [§3.5.1](../../mydocs/manual/pr_review/visual_fixture_evidence.md) 의 2022 이하 버킷이다.

기준 PDF: [`pdf/1341000-201100013-cyber-university-application-2020.pdf`](../../pdf/1341000-201100013-cyber-university-application-2020.pdf)
— 556,998 bytes, SHA-256 `3c1d4b0ae00b0f89169a0168b93f27ff4ec975c02743d9655e47bc58c7d289c5`.

| MCP 산출 증적 | 값 |
| --- | --- |
| job id | `d4219ac3-9c05-43ee-a03b-848cf8c988dd` |
| status | `succeeded` → `success` (phase `completed`) |
| 요청 engine / 응답 engine | `2020` / `2020` |
| `engine_profile` / `hancom_version` | `2020` / `12.0.0.4605` |
| backend | `hwp-managed-direct-dll-host` |
| blob 검증 | client·server byte 수·SHA-256 일치 (`verified`) |
| `pdf_page_count` | **45** |

이 PDF 는 **1-up 45쪽**이라 쪽 인덱스가 rhwp 와 1:1 로 맞는다. 앞서 증적으로 쓰던
PDF 는 인쇄 모아찍기(2-up, 23 물리 장)라 논리 쪽을 손으로 사상해야 했고, 이 MCP
`engine: 2020` 산출물이 그것을 대체한다.

## 회귀 계약과 정본 대조

| 쪽 idx (0-based) | 한/글 2020 정본 | rhwp 수정 후 |
| --- | --- | --- |
| 30 | `- 27 -` `ⅩIII. 평가영역별 심사의견 … 행정` | `pi=113 ci=0` 조각 단독 (`y=143.6..560.1`) |
| 31 | `- 28 -` `현장실사 … 위원회 심의결과` **단독** | `pi=113 ci=1` 단독 (`y=143.6..839.6`) |
| 32 | `- 29 -` `ⅩⅣ. 종합의견` | `pi=114 ci=0` 단독 |

쪽수도 정본과 같아진다.

```text
  수정 전   44쪽   idx 31 이 곧바로 'ⅩⅣ. 종합의견' — 형제 표가 자기 쪽을 못 받는다
  수정 후   45쪽   idx 31 이 형제 표 단독
  한/글     45쪽
```

- `product` 가 `null` 이므로 이 메타데이터는 원 작성 제품의 증명이 아니다
  (재저장·삭제·변조 가능).
