# Task #195 단계 3 완료보고서 — Parser 계층 구현

> 구현계획서: [task_195_impl.md](../plans/task_195_impl.md)
> 단계: 3 / 5

## 작업 결과

### 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| src/parser/control/shape.rs | HWPTAG_SHAPE_COMPONENT_OLE 분기, HWPTAG_CHART_DATA 감지, parse_ole_shape 신규, Chart/OLE 우선 분류 |
| src/main.rs | dump 출력에서 하드코딩 "도형" → `s.shape_name()` 사용 |

### 분기 로직

```
parse_gso_control
  1) 자식 레코드 순회하며:
     - HWPTAG_SHAPE_COMPONENT_OLE (GSO level+1) → shape_tag_id + ole_tag_data 기록
     - HWPTAG_CHART_DATA          → chart_data_bytes 기록
  2) 차트 우선: chart_data_bytes.is_some() → ChartShape (raw 보존)
  3) OLE 분기:  shape_tag_id == OLE → parse_ole_shape
  4) 그림/묶음/일반 도형 (기존 경로)
  5) 미지 태그 → Rectangle 폴백 (기존, 이제 차트/OLE는 여기로 오지 않음)
```

### OLE 파싱 필드

```rust
pub fn parse_ole_shape(common, drawing, tag_data) -> OleShape {
    extent_x:        i32,    // HWPUNIT
    extent_y:        i32,
    flags:           u8,
    drawing_aspect:  u16 → OleDrawingAspect (Content/Icon/Thumbnail/DocPrint)
    bin_data_id:     u32,
    raw_tag_data:    전체 보존 (라운드트립)
}
```

## 1.hwp 검증 결과

```
$ rhwp dump 1.hwp | grep -E "차트|OLE|도형: tac"
  [0]       ctrl[0] OLE: tac=false, wrap=Square
  [0]       ctrl[0] OLE: tac=false, wrap=Square
```

1.hwp의 2개 차트 컨트롤이 **OLE로 정상 분류**됨 (기존: "도형"으로 표시되며 내부적으로는 Rectangle 폴백이었음).

**주의**: 1.hwp에는 네이티브 HWP CHART_DATA 레코드가 없음(Section0 0개). 한컴오피스에서 Excel/Graph 차트를 OLE 객체로 임베드한 형태 — 이 케이스가 1.hwp 뿐 아니라 실무에서 다수. CHART_DATA 경로는 한컴 네이티브 차트용으로 유지.

## 테스트 결과

```
cargo test --release --lib
  test result: ok. 878 passed; 0 failed; 1 ignored
```

신규 단위 테스트 3건:
- `test_parse_ole_shape_minimal` — 전체 필드 파싱
- `test_parse_ole_shape_content_aspect` — aspect=0 (Content) 기본값
- `test_parse_ole_shape_truncated_graceful` — 부분 데이터에서 기본값 폴백

기존 875개 테스트 전원 통과 (회귀 없음).

## 설계 확정 사항

- **1.hwp의 OLE 매직 `ec975b4c...`**: BinData 스트림 압축 여부는 단계 4에서 DocInfo의 BinDataItem compressed 플래그를 읽어 해제 (현재는 raw preview 추출 범위 외)
- **CHART_DATA + OLE 동시 존재 시**: 차트 우선 (순수 HWP 차트로 렌더, OLE는 원본 보존만)
- **아직 기본값인 ChartShape 필드들** (chart_type/series 등): 단계 4에서 raw_chart_data 해석하여 채움. 현재는 Unknown + empty로 둠

## 미해결 이슈 (단계 4)

- [ ] OLE placeholder 전용 SVG 렌더 (현재는 Rectangle과 동일)
- [ ] CHART_DATA 하위 태그 파싱으로 실제 차트 렌더
- [ ] BinData 압축 해제 및 프리뷰 이미지 추출

## 커밋 대상

- src/parser/control/shape.rs
- src/main.rs (dump 출력)
- mydocs/working/task_195_stage3.md

**커밋 메시지**: `Task #195: Parser — OLE/CHART_DATA 분기 + parse_ole_shape (단계 3)`
