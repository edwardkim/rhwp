# 단계 완료 보고 — Task M100 #2085 (R10) 2단계: 행-스캔 통이동

- 작성일: 2026-07-09 / goal 루프 1/4 (자체 검증)

## 수행 내용

행-스캔 `while` 루프 359줄을 `scan_block_table_split_rows` 로 통이동 (원본 무변경).
- 캐리 5종 = `BlockTableRowScan` **값 왕복** (Vec move), 초기값은 호출부 리터럴.
- 읽기 스칼라 12종 = `BlockRowScanVars` (Copy, §6).
- 소스분기(is_hwpx_source 기반 landscape 허용치 4종) caller 잔류 (§1).
- 보정 2건: 지역 const 2개(MIN_TOP_KEEP_PX 등) 모듈 승격(caller/callee 공용),
  캐리 재바인딩 E0384 → mut 디스트럭처 (컴파일러 검출).

## 게이트 (전수 통과)

fmt ✓ / clippy 0 / `--tests` **2,945/0** / issue_1116 13/13 / OVR 5샘플 회귀 **0건**.

## 계측 (표적 공식 CC)

| 함수 | 시작 (r9) | 현재 |
|---|---|---|
| `typeset_block_table` | **129** (전체 1위) | **37** (1,550→1,221줄) |
| 신규 `scan_block_table_split_rows` | — | **93** (381줄) — §5 예외 심사: 단일 국면(분할점 산출) 응집, 후속 분해 후보로 등재. 예외 목록 순증 +1 (§5 과도기 허용 +1~2 내) |
