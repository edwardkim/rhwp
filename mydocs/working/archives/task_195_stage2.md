# Task #195 단계 2 완료보고서 — Model 계층 확장

> 구현계획서: [task_195_impl.md](../plans/task_195_impl.md)
> 단계: 2 / 5

## 작업 결과

### 신규 타입 (src/model/shape.rs)

- `ChartShape` — common/drawing + chart_type/title/legend/axes/series + raw_chart_data
- `OleShape` — common/drawing + extent/flags/drawing_aspect/bin_data_id + preview + raw_tag_data
- `ChartType` enum — Bar/Column/Line/Pie/Area/Scatter/Unknown
- `LegendPosition` enum — 8방위 + Hidden
- `Axis` / `Legend` / `DataSeries` / `OlePreview` / `OlePreviewFormat` / `OleDrawingAspect`
- `ShapeObject` enum에 `Chart(Box<ChartShape>)`, `Ole(Box<OleShape>)` 추가

### 매치 사이트 확장 (8곳)

| 파일 | 라인 | 내용 |
|------|------|------|
| src/model/shape.rs | 271~ | common / common_mut / drawing / drawing_mut / shape_attr / shape_name 각 매치에 Chart/Ole arm |
| src/renderer/layout/shape_layout.rs | 951 | 렌더: placeholder Rectangle로 대체 (단계 4에서 실제 렌더) |
| src/serializer/control.rs | 1042 | 직렬화(최상위): raw_chart_data/raw_tag_data로 라운드트립 |
| src/serializer/control.rs | 1020 | 그룹 자식 ctrl_id 매치 |
| src/serializer/control.rs | 1276 | 그룹 내 직렬화 |
| src/document_core/commands/object_ops.rs | 2754, 2765 | scale_shape_coords + shape_attr 동기화 |
| src/document_core/commands/object_ops.rs | 2889 | 그룹 진입 시 shape_attr |
| src/document_core/commands/object_ops.rs | 3138 | 그룹 해제 시 shape_attr |
| src/main.rs | 936 | dump 명령 출력 |
| src/parser/control/tests.rs | 453, 526 | 테스트 헬퍼 |

## 테스트 결과

```
cargo build --release    # OK
cargo test --release --lib
  test result: ok. 875 passed; 0 failed; 1 ignored
```

단계 3 파서가 아직 Chart/Ole을 생성하지 않으므로, 실제 런타임에서 신규 arm이 동작하지는 않음 (컴파일러 exhaustiveness 충족 목적).

## 설계 확정 사항

- **라운드트립 전략**: 파서가 채우는 `raw_chart_data` / `raw_tag_data`를 직렬화 시 그대로 내보냄 → 1차 구현에서 IR 필드(title/series 등)가 비어 있어도 바이너리 손실 없음
- **렌더링 전략(임시)**: 단계 2에서 Chart/Ole은 Rectangle 대체 렌더와 동일하게 처리 (단계 4에서 교체)

## 미해결 이슈 (단계 3 이후)

- [ ] 파서에서 CHART_DATA / OLE 레코드 인식 및 실제 변환
- [ ] shape.rs의 `is_container` 분기에 차트/OLE가 끼지 않는지 재검증

## 커밋 대상

- src/model/shape.rs (enum 확장 + 신규 타입)
- src/renderer/layout/shape_layout.rs (placeholder 렌더)
- src/serializer/control.rs (라운드트립 직렬화)
- src/document_core/commands/object_ops.rs (shape_attr 매치 확장)
- src/main.rs (dump 출력)
- src/parser/control/tests.rs (테스트 헬퍼 매치)
- mydocs/working/task_195_stage2.md (본 문서)

**커밋 메시지**: `Task #195: Model 계층 확장 — ChartShape/OleShape (단계 2)`
