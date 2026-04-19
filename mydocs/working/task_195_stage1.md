# Task #195 단계 1 완료보고서 — 스펙 조사 및 IR 설계

> 구현계획서: [task_195_impl.md](../plans/task_195_impl.md)
> 단계: 1 / 5 (문서만, 코드 변경 없음)

## 작업 결과

### 산출 문서 (2건)

| 파일 | 내용 |
|------|------|
| `mydocs/tech/hwp_chart_spec.md` | CHART_DATA 레코드 구조, ChartType enum, DataSeries/Axis/Legend IR, 파싱 플로우, 렌더링 방침 |
| `mydocs/tech/hwp_ole_spec.md` | SHAPE_COMPONENT_OLE 필드 설계, BinData 스트림 구조, 프리뷰 추출 전략, OleShape IR |

### 추가 조사 결과

**1.hwp의 OLE 스트림 실측**:
```
BinData/BIN0001.OLE  size=30110  magic=ec975b4c
BinData/BIN0002.OLE  size=22204  magic=ec965b6c
```
표준 CFB 매직(`d0cf11e0`)과 불일치 → **BinData 스트림 압축됨**. 단계 3에서 DocInfo의 `BinDataItem` compressed 플래그 확인 후 압축 해제 로직 필요.

**기존 재사용 가능 자원 확인**:
- `src/wmf/` WMF 파서 존재 → OLE 프리뷰 WMF 디코드 시 재사용
- `Cargo.toml`에 `cfb` crate 존재 → 중첩 CFB 파싱에 직접 사용 가능

**차트/OLE 재분류 로직 확정**:
- `parse_gso_control`에서 child_records에 `HWPTAG_CHART_DATA`가 있으면 → **Chart**
- shape_tag_id == `HWPTAG_SHAPE_COMPONENT_OLE` → **Ole**
- 차트와 OLE는 상호 배타적이지 않을 수 있음(OLE 내부에 MS Graph가 있고 그 위에 CHART_DATA가 덧씌워짐). 이 경우 **CHART_DATA 우선**으로 Chart 분기.

## 코드 수정

없음 (하이퍼-워터폴 규칙: 스펙 조사 단계는 문서만).

## 테스트 결과

해당 없음 (문서만).

## IR 설계 요약

```rust
// 차트
pub struct ChartShape {
    pub common: CommonObjAttr,
    pub drawing: DrawingObject,
    pub chart_type: ChartType,
    pub title: Option<String>,
    pub legend: Option<Legend>,
    pub x_axis: Option<Axis>,
    pub y_axis: Option<Axis>,
    pub series: Vec<DataSeries>,
    pub raw_records: Vec<Record>,     // 라운드트립
    pub caption: Option<Caption>,
}

// OLE
pub struct OleShape {
    pub common: CommonObjAttr,
    pub drawing: DrawingObject,
    pub extent: (i32, i32),
    pub flags: u8,
    pub drawing_aspect: u16,
    pub bin_data_id: u32,
    pub preview: Option<OlePreview>,   // 1차 None
    pub raw_tag_data: Vec<u8>,
    pub caption: Option<Caption>,
}
```

## 미해결 이슈 (다음 단계 또는 별도 이슈)

- [ ] CHART_DATA 하위 태그(80~95)의 정확한 바이트 오프셋 — 단계 3에서 pyhwp 교차 확인
- [ ] HWPTAG_SHAPE_COMPONENT_OLE 레코드의 정확한 필드 오프셋 — 단계 3에서 확정
- [ ] BinData 스트림 압축 포맷(zlib raw deflate vs. HWP 자체) — 단계 3에서 DocInfo BinDataItem 플래그로 판정
- [ ] 실제 OLE 프리뷰 이미지 추출 (WMF/EMF 경로) — **별도 이슈로 분리 제안**
- [ ] 3D/복합 차트 렌더 — 별도 이슈

## 승인 요청

1. 스펙 문서 2건과 IR 설계가 다음 단계(Model 구현) 착수에 충분한지
2. 차트/OLE 재분류 우선순위(CHART_DATA 우선) OK인지
3. OLE 프리뷰 실제 추출을 별도 이슈로 분리 OK인지
4. 승인 시 **단계 2(Model 계층)** 착수

## 커밋 대상 파일

- `mydocs/tech/hwp_chart_spec.md` (신규)
- `mydocs/tech/hwp_ole_spec.md` (신규)
- `mydocs/plans/task_195.md` (단계 0 수행계획서, 신규)
- `mydocs/plans/task_195_impl.md` (구현계획서, 신규)
- `mydocs/working/task_195_stage1.md` (본 문서, 신규)

**커밋 메시지**: `Task #195: 스펙 조사 및 IR 설계 (단계 1)`
