# 구현 계획서: Task #898

수행계획서: [`task_m100_898.md`](task_m100_898.md)
브랜치: `local/task898`

## 0. 원인 분석 재확인 (수행계획 단계 추가 조사 결과)

수행계획서 단계의 가설 (단정의 separator 추가 필드 / 단 구분선 정지 위치) 은 **틀렸음** 을 확인.

- `parse_column_def_ctrl` 잔여 바이트 분석: `cold` ctrl_data 12바이트 전부 파싱됨 (`08 10 f4 0c 00 00 00 00 00 00 00 00`). `separator_type=0` → **단 구분선 없음**.
- 가운데 세로 선은 단 구분선이 아니라 **문서 본문 0.0 컨트롤 [5] 직선 Shape**.
  - 크기: `0×85039 HU` (≈0×300mm), 선 굵기 100 HU
  - 위치: 가로 = `용지/Center +105 HU` (≈가로 중앙), 세로 = `용지/Top +16842 HU` (≈59.4mm)
  - 렌더 범위 y=224.6 ~ 1358.4 → 페이지 비율 15.1% ~ 91.2%
- 쪽번호 박스는 **바탕쪽(MasterPage) 표 1×3**:
  - 위치: `vert=Paper/101954 HU` (≈359.55mm), `horz=Paper/85 HU` (≈Center)
  - 크기: `58408×2196 HU` (≈205.4×7.7mm)
  - 렌더 범위 y=1359.4 ~ 1388.7 → 페이지 비율 91.3% ~ 93.3%

PDF (한컴 2022, A3 1169×1653 px 렌더) 측정:
- 직선 끝 y=1509 (91.3%) — **우리 렌더와 거의 일치**
- 쪽번호 박스 상단 y=1530 (92.6%) — **우리 렌더(91.3%)보다 ≈1.3%(≈21px=5.3mm) 낮음**

**결론**: 단 구분선 / 직선 길이 문제가 아니라, **바탕쪽 쪽번호 표의 Y 위치가 약 5mm 위쪽으로 잘못 배치** 되어 직선 끝과 시각적으로 붙는 것이 원인.

## 1. 수정 대상 (예상)

- `src/renderer/layout.rs` `build_master_page` — 바탕쪽 표 컨트롤 배치 시 `Paper`/`vert` 좌표 해석
- `src/renderer/layout/shape_layout.rs` `compute_table_y_position` 또는 `compute_object_position` — Paper 기준 Y 좌표 + outer_margin / 크기 / VerticalAlign 처리
- 가설 후보:
  - (a) **outer_margin_top 미적용** — 표의 outer_margin_top 만큼 아래로 추가 이동해야 하는데 빠뜨림
  - (b) **vert offset 해석 오류** — `Paper/101954` 가 표 상단이 아니라 표 베이스라인/하단 기준일 가능성
  - (c) **이중 바탕쪽 합산** — 바탕쪽이 2개(`[0]`, `[1]`) 모두 그려지지만 둘 다 동일 좌표 → 동일 표 2회 겹쳐 그림. 위치 차이는 없으나 검증 필요
  - (d) **MasterPage paper_area 좌표계 오프셋** — `paper_area.y = 0` 기준 vert offset 합산 시 부정확

## 2. 단계 분할 (4단계)

### Stage 1 — 정밀 원인 규명 + 단위 테스트 추가

목표: 바탕쪽 표의 실제 SVG y 좌표가 어떤 계산식으로 산출되는지 추적하고, HWP/HWPX 스펙 및 한컴 동작과 합치하는 기대값 도출.

작업:
1. `src/renderer/layout.rs` `build_master_page` 에서 표 컨트롤 처리 경로 (`Control::Table → layout_table`) 추적
2. `compute_table_y_position` 동작 분석 — `vert_rel_to=Paper`, `vertical_offset=101954`, outer_margin_top 의 합산식 확인
3. 비교 자료:
   - HWP5 스펙 (hwp5 control common 부분) outer_margin 적용 규칙
   - HWPX 동등 문서가 있다면 대조 (현재 `samples/` 에 `exam_math.hwpx` 없음 — 한컴에서 추출하거나 수동 변환 필요 시 도움 요청)
4. 단위 테스트 신설: `tests/master_page_table_position.rs` 또는 기존 `src/renderer/page_layout.rs` 테스트 확장 — `Paper` 기준 + outer_margin 케이스 명시

산출물: `mydocs/working/task_m100_898_stage1.md` (원인 결론 + 기대 좌표 산출식)

### Stage 2 — 수정 구현

목표: Stage 1 에서 확정된 좌표 산출식으로 바탕쪽 표 Y 배치 로직 수정.

작업:
1. 해당 함수(`compute_object_position` 또는 `build_master_page` 분기) 수정
2. 변경 영향이 본문 표/그림 배치에 미치지 않도록 분리 — 바탕쪽 컨텍스트 전용 분기 또는 outer_margin 적용 누락 보강
3. Stage 1 단위 테스트 통과 확인

산출물: `mydocs/working/task_m100_898_stage2.md` (수정 diff + 테스트 결과)

### Stage 3 — 시각 회귀 검증

목표: `exam_math.hwp` 전 20페이지 SVG ↔ PDF 시각 비교 + 관련 문서 회귀 확인.

작업:
1. `rhwp export-svg samples/exam_math.hwp -o output/svg/exam_math/` 재생성
2. `rsvg-convert` + `pdftoppm` 으로 각 페이지 비교 (Python 스크립트로 vertical line / pagenum box y 좌표 측정)
3. `aift.hwp`, `shortcut.hwp`, `exam_math_8.hwp`, `exam_math_no.hwp` 회귀 검토
4. 골든 SVG (있다면) 갱신 — Issue #267/#617/#677 케이스 등

산출물: `mydocs/working/task_m100_898_stage3.md` (페이지별 측정 표, 회귀 결과)

### Stage 4 — 최종 마무리

작업:
1. `cargo test`, `cargo clippy --all-targets -- -D warnings` 통과
2. 골든 SVG 갱신 커밋 (필요 시)
3. 최종 결과보고서: `mydocs/report/task_m100_898_report.md`
4. 오늘할일(`mydocs/orders/`) 갱신 — **Orders 변경 금지** 메모리 룰에 따라 작업지시자 지시 확인 후만

산출물: `mydocs/report/task_m100_898_report.md`

## 3. 위험 / 주의 사항

- 바탕쪽 표 위치 수정이 **모든 문서의 머리말/꼬리말 영역 객체 위치에 영향**을 줄 수 있음. Stage 3 회귀 범위를 충분히 확보해야 함.
- 가설 (b) `vert offset 이 표 baseline 기준` 이 맞다면 **모든 Paper 기준 객체** 의 Y 계산을 일괄 수정해야 함 → 영향 범위 큼. 이 경우 Stage 2 직전에 작업지시자에게 추가 승인 요청.
- 가설 (a) outer_margin_top 미적용만 해결하면 영향 범위 작음 — 우선 검증.

## 4. 승인 요청

본 구현 계획에 대한 작업지시자의 승인을 요청합니다. 승인 후 Stage 1 부터 진행합니다.
