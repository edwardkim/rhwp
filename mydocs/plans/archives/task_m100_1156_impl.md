# Task M100-1156 — 구현 계획서

- 이슈: [#1156](https://github.com/edwardkim/rhwp/issues/1156)
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task1156`
- 일시: 2026-05-29
- 수행 계획서: [`task_m100_1156.md`](task_m100_1156.md)

## 1. 본질 (사전 진단 + spec 교차 + SVG 렌더 확정)

### 1.1 차트 크기 — 80mm 확정, 점유 정상

- 한컴 확정 + spec(SHAPE_COMPONENT 공통 크기) + HWPX `hp:sz`(22677 HU) = **80mm 3중 일치**
- rhwp 레이아웃 점유: `resolve_object_size` 가 common(80mm) 사용 → **점유 크기 정상**
- 차트 그림: 본 fixture 차트는 **MS Graph OLE2** (`ole3.ole` = CFB `d0cf11e0`). ooxml_chart/EMF/native_image fallback 모두 실패 → **placeholder("OLE 개체 #3") 로 렌더** (회색 80mm 박스).

→ dump `eff=0.0mm` 는 shape_attr curr/orig(0) 표시값일 뿐, 실제 layout 점유는 placeholder 가 render_w/h(80mm) 사용. **크기는 이미 80mm 로 점유 중.**

### 1.2 핵심 미해결 — 다단 column flow

`dump-pages`:
```
단 0: used=785px / hwp_used=855px (diff -70px)
단 1: used=345px / hwp_used=741px (diff -396px)  ← 한컴보다 396px 적게 채움
```

→ **이슈 본질 = 80mm 차트가 2단에서 ① 단 이동(영역 부족 시 다음 단), ② 차트가 비운 공간에 후속 텍스트 back-fill 이 안 됨** (단1 diff -396px).

### 1.3 차트 그림 렌더 (MS Graph OLE2) — 별개 scope

MS Graph OLE2 차트의 실제 그래프 렌더링(축/막대/선)은 차트 렌더링 기능 enhancement 로, **본 task scope 밖** (placeholder 80mm 점유로 layout 검증 가능). 본 task 는 **layout(단 이동 + back-fill)** 에 집중.

## 2. Stage 구성 (4 단계)

### Stage 1 — 진단 확정 + 다단 flow 분석

- 차트(OLE placeholder) 80mm 점유 확정 (완료 — 사전 진단)
- 한컴 단0/단1 vpos 흐름 vs rhwp 흐름 정밀 매핑 (`dump-pages` + 한컴 PDF/편집기)
- 다단 column flow 의 객체(차트/Shape) 처리 + back-fill 코드 경로 추적 (`pagination/engine.rs` advance_column_or_new_page, `state.rs`)
- **back-fill 본질**: 객체가 단 이동 시 같은 단 후속 문단이 빈 공간 채우는 한컴 알고리즘
- 산출: `mydocs/tech/multicolumn_object_flow_backfill.md` + Stage 1 보고서

### Stage 2 — 차트(객체) 단 이동

- 차트(Shape/OLE)가 현재 단 잔여 영역 부족 시 다음 단 이동 처리
- 80mm 점유 기준 단 영역 판정
- 산출: Stage 2 보고서

### Stage 3 — 빈 공간 텍스트 back-fill

- 객체 단 이동 후 같은 단 후속 문단이 빈 공간 채우도록 column flow 정정
- 단1 used ≈ hwp_used (diff 축소)
- 산출: Stage 3 보고서

### Stage 4 — 검증 + 종결

- `dump-pages` 단1 diff 축소 확인
- `export-svg --debug-overlay` 시각 검증
- 작업지시자 한컴 한글 2020/2022 시각 판정 게이트
- 회귀 가드: `tests/issue_1156_*.rs` (다단 객체 flow) + 기존 다단(#1082) 회귀 없음
- HWPX fixture `samples/hwpx/143E433F503322BD33.hwpx` 커밋
- `cargo test --tests` + `clippy --lib` + `fmt`
- 최종 보고서 + commit + merge + close

## 3. 위험 분석

| 위험 | 평가 |
|------|------|
| 다단 flow 정정이 기존 다단 fixture 회귀 | #1082 등 기존 다단 회귀 가드 + 시각 판정 |
| back-fill 알고리즘 복잡도 | Stage 1 tech 문서로 한컴 동작 정밀 분석 선행 |
| 차트 그림(MS Graph) 미렌더 혼동 | placeholder 80mm 점유로 layout 분리 검증. 그림 렌더는 별개 scope 명시 |
| 객체 일반(Shape) vs 차트 한정 | 차트(OLE) 우선 + 일반 Shape 회귀 가드 |

## 4. 작업지시자 승인 요청

1. 본 구현 계획 (4 단계) 승인 여부
2. 차트 그림(MS Graph OLE2) 렌더링을 본 task scope 밖(별개 enhancement)으로 두는 것 동의 여부 — 본 task 는 layout(단 이동 + back-fill) 집중
3. Stage 1 tech 문서 (`multicolumn_object_flow_backfill.md`) 작성 권장 수용 여부
4. Stage 4 한컴 2020/2022 시각 판정 게이트 권장 수용 여부
