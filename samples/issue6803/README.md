# samples/issue6803

`#6803` — **조각 시작 행에서 시작한 rowspan 셀이 끝 컷을 잃어 다음 쪽 몫까지 그린다.**

## 재현물

| | |
|---|---|
| 원본 이름 | `1376496_[별표 6] 근린생활시설용지 건축물 용도 · 건폐율 · 용적률 · 높이(전주·완주 혁신도시개발사업 실시계획 승인).hwp` |
| 출처 | 국가법령정보센터 행정규칙(건설교통부) 공개 문서 |
| 크기 | 32,768 bytes (SHA-256 은 `MANIFEST.json`) |
| 쪽 수 | 5 |
| 저장 버전 | `6.7.10.1058` (제품 표기 없음) |

## 형상

`pi=6 ci=0` 표는 14행×5열, `쪽나눔=RowBreak`. **셀[21] `r=8,c=0` `rs=6`** 이 문단 67개
(대부분 빈 문단으로 세로 가운데를 만든 수작업 서식, 높이 156,749HU = 2,090px)를 담고
3·4쪽에 걸친다.

```
DIAG_FRAG pi=6 ci=0 rows=8..11  start_cut=[1] end_cut=[1]   ← end_cut 은 행 10 의 부기
DIAG_FRAG pi=6 ci=0 rows=10..14 start_cut=[1] end_cut=[5]
```

셀은 조각의 **시작 행**(8)에서 시작하므로 `is_split_start_row` 로 잡히지만, `end_cut` 은
`end_row-1`(행 10)의 부기라 그 셀이 없다 → `eu = usize::MAX`. `#1748` 의 높이-기반 구제는
`!is_in_split_row` 를 요구해 닿지 않는다.

## 계약

3쪽 조각은 **자기 셀 상자 높이까지만** 그린다(`pi=0..33`). 4쪽이 `pi=34` 에서 이어받는
동작은 이미 옳으므로 건드리지 않는다.

## 기준 엔진

**`2020`** — `lastSavedWith.product` 가 `null` 이므로
[시각·fixture 증적](../../mydocs/manual/pr_review/visual_fixture_evidence.md) §3.5.1 에 따라
`hwp2024Convert` MCP 의 `engine: 2020` 이다.

기준 PDF: [`pdf/1376496-neighborhood-facility-land-table-2020.pdf`](../../pdf/1376496-neighborhood-facility-land-table-2020.pdf)
— 153,835 bytes, SHA-256 `ef7ac0c12d37f8b96f1af5e2d67e812151d9b9862c4f42824fee8bc04a5dda12`.

| MCP 산출 증적 | 값 |
| --- | --- |
| job id | `586dd5c8-8ffe-469e-9b79-d3023ed5b6af` |
| status | `succeeded` → `success` (phase `completed`) |
| 요청 engine / 응답 engine | `2020` / `2020` |
| `engine_profile` / `hancom_version` | `2020` / `12.0.0.4605` |
| backend | `hwp-managed-direct-dll-host` |
| blob 검증 | client·server byte 수와 SHA-256 일치 (`verified`) |
| `pdf_page_count` | 5 (rhwp 와 동일) |

`product` 가 `null` 이므로 이 메타데이터는 원 작성 제품의 증명이 아니다 —
재저장·삭제·변조될 수 있다.

## 판정 수치

`A1` 쪽별 계수가 결함을 직접 준다.

```
        수정 전   수정 후   한/글 2020
p2         2        2          2
p3         6        2          2      ← 결함
p4         3        3          3
p5         1        1          1
```

`export-pdf` 전체 글자 수도 수정 후 **3,299 = 한/글 3,299** 로 일치한다
(공백·점 리더 제거 기준). 쪽수는 셋 다 5다.

세 지표 모두 engine 2024 로 먼저 잰 값과 같았다 — 이 문서에서 두 엔진은
`A1` 계수·글자 수·쪽수가 동일하다.
