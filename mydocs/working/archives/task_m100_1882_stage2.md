# task_m100_1882 Stage 2 완료 보고서 — 갭④ Y축 headroom + step 기반 눈금

- 이슈: #1882 (C1c, #1431 Track C)
- 브랜치: `local/task1882`
- 단계: Stage 2 / 5 — 축 스케일 (`src/ooxml_chart/renderer.rs` 단독)

## 변경 내용

1. **`nice_range(min,max,ticks)` → `nice_axis(min,max) -> (min', max', step)`**:
   - `floor_nice_step`(종전 임계 1.5/3/7 유지) / `ceil_nice_step`(1/2/5/10 상향) 헬퍼 분리.
   - **경계 headroom**: 데이터 max가 step 경계에 정확히 걸리면 +1 step 확장 후 step을
     ceil-nice로 **재계산**, 확장이 없으면 step 유지(무조건 재계산 시 scatter X의 0.5 간격이
     1.0으로 승격되는 회귀 방지 — 조건부 규칙이 실측 앵커 3점을 동시에 만족하는 유일 규칙).
2. **범위 함수 triple화**: `value_range_for` / `value_range` / `scatter_range`가 step 포함 반환.
   소비처 전부 이관(`estimate_axis_label_width`, `render_bars`, `render_line`, `render_scatter`,
   `render_combo`). 격자 전용 함수 분리 없음 — 막대 기하가 격자와 같은 (vmin,vmax) 공유.
3. **`render_value_grid` step 기반 눈금**: 5등분 고정 → `step` 파라미터 + 정수 루프
   (`v = vmin + step*i`, 부동소수 누적 드리프트 방지). **비정수 step은 자동 decimal 라벨**
   (format_num 정수 반올림이 0.5 간격 라벨을 "0,1,1,2…"로 손상시키는 잠재 결함 차단).
4. **percentStacked 예외 유지**: `(0.0, 100.0, 20.0)` 명시 전달 → 종전 5등분 라벨
   0%,20%,…,100%와 동일 출력(무회귀).

## 실측 앵커 재현 (한컴 2022 정답지)

| 앵커 | 종전 | 이후 | 정답지 |
|---|---|---|---|
| 막대 max 5.0 (경계) | 0~5, 라벨 0,1,…,5 | **0~6, 라벨 0,2,4,6** | 0~6 라벨 0,2,4,6 ✓ |
| scatter Y max 4.0 (경계) | 0~4 | **0~5, step 1** | 0~5 ✓ |
| scatter X max 2.6 (비경계) | 0~3, 라벨 0.6 간격 | **0~3, step 0.5 유지** | 0~3 0.5 간격 ✓ |

## 테스트 (TDD — 구현 전 실패 확인)

- 신규 3: `test_axis_headroom_bar_max_on_boundary`(0,2,4,6 존재 + 1,3,5 부재),
  `test_axis_headroom_scatter_y_on_boundary`(4.0→`>5<`),
  `test_axis_no_headroom_when_max_off_boundary`(X 0.5 간격 보존).
- 기존 1 갱신(계획 명기 사항): `test_render_scatter_decimal_axis_labels` `">2.4<"`→`">2.5<"`
  (X축 라벨이 0.6 간격→0.5 간격으로 바뀜 — 의도된 스펙 변경).

## 검증

```
cargo test --lib ooxml_chart                              → 36 passed, 0 failed
cargo test --test issue_1431_scatter                      → 1 passed  (placeholder 0건)
cargo test --test issue_1453_chart_3d_ofpie_routing       → 2 passed  (percent "100%" 가드 포함)
```

**시각 확인** (`output/poc/chart_c1c/stage2/` ↔ 정답지): 묶은세로막대형 축 0~6 라벨 0,2,4,6,
표식만있는분산형 Y 0~5·X 0~3(0.5 간격) — 앵커 3점 전부 정답지와 일치.

## 알려진 편차 (구현계획서 명기 사항)

- 콤보 보조축(테스트 데이터 max 10) → 축 0~15 (50% headroom). 실측 앵커 없는 케이스로 수용.
- 콤보 기본/보조축 눈금 수가 달라질 수 있음 — 보조축은 라벨만 출력하므로 격자선 불일치 없음.

## 다음 단계

Stage 3 — 갭① 자동 제목: 모델 플래그 + `autoTitleDeleted` 파싱 + "차트 제목" 정책 + weight 400.
