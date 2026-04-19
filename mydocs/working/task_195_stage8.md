# Task #195 단계 8 완료보고서 — OOXML 차트 네이티브 SVG 렌더

> 구현계획서: [task_195_impl.md](../plans/task_195_impl.md)
> 단계: 8 / 8

## 작업 결과

### 신규 모듈: `src/ooxml_chart/`

| 파일 | 내용 |
|------|------|
| `mod.rs` | `OoxmlChart` 데이터 모델, `OoxmlChartType`, `OoxmlSeries` |
| `parser.rs` | DrawingML XML → 데이터 모델 (`quick-xml` 사용) |
| `renderer.rs` | 데이터 모델 → SVG 조각 |

### 데이터 모델
```rust
pub struct OoxmlChart {
    pub chart_type: OoxmlChartType, // Column/Bar/Line/Pie/Unknown
    pub title: Option<String>,
    pub series: Vec<OoxmlSeries>,   // name, values[], color
    pub categories: Vec<String>,
}
```

### 지원 범위 (1차)
- `c:barChart` + `c:barDir val="col"` → 세로 막대
- `c:barChart` + `c:barDir val="bar"` → 가로 막대
- `c:lineChart` → 꺾은선
- `c:pieChart` → 원형

### 범위 외 (폴백)
- 3D 차트, 영역, 산점도, 복합 차트 → placeholder
- 제목/범례 스타일 세밀 재현
- OOXMLChartContents 부재 파일 → placeholder

### SVG 렌더링 특징
- 기본 팔레트 8색 순환
- 자동 min/max 축 (0 포함)
- 4분할 격자 + 축 레이블
- 범례 하단 (파이는 카테고리 범례)
- 폰트: sans-serif
- `<g class="hwp-ooxml-chart">` 래퍼

### 렌더 트리 확장
- `RenderNodeType::RawSvg(RawSvgNode)` 신규 — 사전 생성 SVG 조각 삽입용
- `svg.rs`에 대응 분기 추가

### shape_layout.rs Ole arm 업데이트
```rust
ShapeObject::Ole(ole) => {
    // BinDataContent 조회 → parse_ole_container → parse_chart_xml → render_chart_svg
    // OOXML 없거나 파싱 실패 시 기존 placeholder로 폴백
}
```

## 1.hwp 실제 렌더 검증

| 페이지 | 차트 | 결과 |
|--------|------|------|
| 3 | 월별 기부 금액/건수 2 시리즈 × 12개월 (barChart col) | ✅ 실제 막대 차트 렌더 |
| 4 | 시간대별 기부 건수 1 시리즈 × 24개 (barChart col) | ✅ 실제 막대 차트 렌더 |

SVG `hwp-ooxml-chart` 클래스 마커 각 페이지에 1개씩 확인. PNG 변환 후 육안 검증 완료.

### 기존 "OLE 개체 (BinData #NNN)" placeholder → 실제 차트

이전: 회색 박스 + 텍스트 라벨
현재: 2 시리즈 색상 구분 막대 + 축 레이블 + 범례 + 제목 (있으면)

## 테스트 결과
- `cargo build --release` OK
- `cargo test --release --lib` **890 passed; 0 failed; 1 ignored**
  - 기존 878 + ole_container 4 + ooxml_chart 8 = 890

### 신규 단위 테스트 (8건)
- parser: bar / horizontal_bar / pie / malformed
- renderer: empty / column / pie / color_hex

## 범위 외 이월 (별도 이슈 제안)

1. **EMF 네이티브 SVG 변환기** — OOXMLChartContents 부재 OLE를 벡터 렌더
2. **3D / 영역 / 산점도 / 복합 OOXML 차트**
3. **OOXML 차트 스타일 세밀 재현** (축 포맷, 보조축, 추세선)
4. **CHART_DATA 네이티브 HWP 차트 파싱**

## 커밋 대상

- src/ooxml_chart/mod.rs (신규)
- src/ooxml_chart/parser.rs (신규)
- src/ooxml_chart/renderer.rs (신규)
- src/lib.rs (`pub mod ooxml_chart`)
- src/renderer/render_tree.rs (`RawSvg` variant)
- src/renderer/svg.rs (`RawSvg` emit)
- src/renderer/layout/shape_layout.rs (Ole arm OOXML 분기)
- src/parser/control/shape.rs (`parse_ole_shape` bin_data_id 오프셋 수정)
- mydocs/working/task_195_stage6.md / _stage7.md / _stage8.md

## 최종 성과 (단계 1~8 전체)

| 항목 | 이전 | 현재 |
|------|------|------|
| OLE 차트 | 빈 사각형 | 실제 바/선/원형 차트 네이티브 SVG |
| 차트 데이터 접근 | 없음 | OOXML DrawingML 파싱 후 구조화 |
| BinData 스트림 | Embedding만 | Embedding + Storage(OLE) 모두 |
| 파싱 코드 | — | CFB 중첩 / OLE Presentation / OOXML XML |
| 테스트 | 875 | 890 (+15) |
