# 최종 결과보고서 — Task M100 #2085: 라운드 10 (typeset_block_table, goal 루프 1/4)

- 이슈: #2085 / 브랜치: `local/task2085` / 2026-07-09 / goal 루프 (close 는 루프 말미 일괄 승인)

## 결과

| 지표 | 시작 (r9) | 완료 (r10) |
|---|---|---|
| `typeset_block_table` 공식 CC | **129** (전체 1위) | **37** (1,550→1,221줄) |
| 신규 `scan_block_table_split_rows` | — | **93** (381줄) — §5 예외 심사 아래 |
| 전체 최대 CC | 129 | **124** (`layout_table_cells` — R11 대상) |
| CC>25 예외 | 87 | 88 (**+1**, §5 과도기 허용 +1~2 내) |
| 행동 회귀 | — | **0건** |

## §5 예외 심사 — scan_block_table_split_rows (CC 93)

행-스캔 분할점 산출은 단일 국면(행/블록/셀-컷 결정)으로 응집돼 있고, 캐리 5·탈출 16이
전부 루프-지역이라 이번 회전은 통이동이 옳다. 내부의 rowspan 보호/RowBreak 판정
연쇄는 **후속 분해 후보**로 등재 (R9 의 judge_* 패턴 적용 가능).

## 수행·게이트

- 359줄 while 루프 통이동: `BlockTableRowScan`(캐리 5, 값 왕복) + `BlockRowScanVars`(Copy 12).
- 소스분기(is_hwpx_source landscape 허용치) caller 잔류 (§1). 지역 const 2건 모듈 승격.
- 게이트 전수 통과: 테스트 2,945/0 · issue_1116 13/13 · OVR 5샘플 회귀 0 · clippy 0.
