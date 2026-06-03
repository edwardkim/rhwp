# Task #517 Stage 1 보고서 — RHWP_LAYOUT_DEBUG env logging

**이슈**: #517
**브랜치**: `local/task517`
**Stage**: 1 / 2

## 1. 변경 내용

`src/renderer/layout/paragraph_layout.rs`:
- `layout_debug_enabled()` helper 추가 (env var 체크)
- `layout_inline_table_paragraph` 시작 부분에 진단 로깅 (env-var-checked)

## 2. 출력 형식

```
LAYOUT_INLINE_TABLE_PARA: pi={N} sec={S} col_x={X} col_w={W} y_start={Y} y={Y'} sb={SB} sa={SA} ml={ML} mr={MR} align={A} ls_count={N} tables={T}
  LAYOUT_LS[i]: vpos={V} lh={LH} ls={LS} bl={BL} text_start={TS} sw={SW}
  LAYOUT_INLINE_TBL[i]: ctrl_idx={CI} rows={R} cols={C} w={W} h={H} vert={V} horz={H} wrap={WR}
```

## 3. 검증

### 3-1. exam_science p2 pi=61 (#496 재현 케이스)

```
LAYOUT_INLINE_TABLE_PARA: pi=61 sec=0 col_x=534.8 col_w=422.6 y_start=1176.8 y=1176.8 sb=0.0 sa=6.7 ml=15.1 mr=0.0 align=Justify ls_count=3 tables=1
  LAYOUT_LS[0]: vpos=74118 lh=2864 ls=460 bl=1432 text_start=0 sw=18939
  LAYOUT_LS[1]: vpos=77442 lh=1150 ls=460 bl=575 text_start=13 sw=18939
  LAYOUT_LS[2]: vpos=79052 lh=1150 ls=460 bl=575 text_start=60 sw=30562
  LAYOUT_INLINE_TBL[0]: ctrl_idx=0 rows=2 cols=1 w=14745 h=2864 vert=Top horz=Left wrap=TopAndBottom
```

핵심 정보:
- ls_count=3 (다중 줄 paragraph)
- table rows=2 (다중행 인라인 표)
- ls[2].text_start=60 (HWP 인코딩 break 위치)

→ #496 결함 분석에 직접 활용 가능.

### 3-2. 회귀 검증

`scripts/svg_regression_diff.sh build devel HEAD` 결과:
- TOTAL: pages=170 same=170 diff=0

env-var-checked 로깅이므로 기본 동작 변경 없음. **회귀 0건**.

### 3-3. 단위 + 통합 테스트

- `cargo test --release --lib`: 1103 passed
- `cargo test --release --tests`: 모든 통과

## 4. 잔여

Stage 2 (회귀 검증 도구 정형화) 로 진행.
