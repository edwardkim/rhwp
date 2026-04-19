# HWP 차트(CHART_DATA) 바이너리 스펙 정리

> Task #195 단계 1 산출물. 구현 전 스펙 조사 및 IR 설계 문서.

## 1. 차트 컨트롤의 전체 구조

HWP 5.0에서 차트는 **일반 도형(GSO, `b"gso "`) 컨트롤의 한 형태**로 저장된다.
```
CTRL_HEADER (ctrl_id=b"gso ")
├── SHAPE_COMPONENT (공통 도형 속성: 위치/크기/테두리/채우기)
├── HWPTAG_CHART_DATA (= HWPTAG_BEGIN + 79)
│   └── (하위 레코드들 — 차트 내부 구조)
└── [LIST_HEADER + 문단들]  (차트 내부 텍스트 — 타이틀/범례 등)
```

SHAPE_COMPONENT 바로 뒤에 `HWPTAG_CHART_DATA` 레코드가 오면 그 도형은 **차트**이다.
즉, 현재 rhwp의 `parse_gso_control`에서 `shape_tag_id` 미지 분기로 떨어지는 도형 중, child_records에 `HWPTAG_CHART_DATA`가 있으면 Chart로 재분류해야 한다.

참고: 한컴 HWP 파일 포맷 5.0 공식 문서(hwpFormat_HWP2002Revised.pdf) 및 pyhwp(pyhwp/hwp5/binmodel.py) 구현.

## 2. CHART_DATA 레코드

`HWPTAG_CHART_DATA` 레코드 자체는 **컨테이너**이며, 내부에 다음 하위 레코드들이 트리 구조로 저장된다.

| 하위 태그 (HWPTAG_BEGIN + offset) | 설명 |
|---|---|
| +80 ~ +95 | 차트 세부 (크기/ID/Legend/Axis/Series/Title 등) |

하위 태그의 정확한 바이너리 필드는 공식 문서에도 세부 기술이 빈약하여, 구현 시 다음 전략을 취한다:

1. **1차 구현**: `HWPTAG_CHART_DATA` 및 하위 레코드는 **raw 바이트로 보존**(`ChartShape::raw_records`) → 라운드트립 유지
2. **2차 구현**: 주요 필드만 파싱하여 IR에 채움
   - 차트 종류(`ChartType`)
   - 타이틀(`title: String`)
   - 범례 위치(`legend_position`)
   - 축 레이블(`x_axis_labels`, `y_axis_labels`)
   - 데이터 시리즈(`series: Vec<DataSeries>`)
   - 색상 팔레트

## 3. 차트 종류 (ChartType)

HWP 차트는 내부적으로 MS Graph OLE를 기반으로 하므로 종류가 방대하다.
1차 범위는 다음으로 제한한다:

| ChartType | 설명 |
|---|---|
| `Bar` | 막대(가로) |
| `Column` | 막대(세로) |
| `Line` | 선 |
| `Pie` | 파이 |
| `Area` | 영역 |
| `Scatter` | 분산형 |
| `Unknown(u16)` | 그 외 — raw만 보존, Rectangle placeholder 렌더 |

## 4. 데이터 시리즈

```rust
pub struct DataSeries {
    pub name: String,          // 시리즈 이름(범례 표시용)
    pub values: Vec<f64>,      // Y값 (또는 파이의 각 조각)
    pub categories: Vec<String>, // X축 레이블 (시리즈 간 공유)
    pub color: Option<u32>,    // RGB
}
```

## 5. rhwp IR 설계 — `ChartShape`

```rust
pub struct ChartShape {
    pub common: CommonObjAttr,          // GSO 공통 속성
    pub drawing: DrawingObject,         // 도형 기본(테두리/채우기/캡션)
    pub chart_type: ChartType,
    pub title: Option<String>,
    pub legend: Option<Legend>,
    pub x_axis: Option<Axis>,
    pub y_axis: Option<Axis>,
    pub series: Vec<DataSeries>,
    pub raw_records: Vec<Record>,        // 라운드트립용 원본 레코드 보존
    pub caption: Option<Caption>,
}

pub struct Legend { pub position: LegendPosition, pub visible: bool }
pub struct Axis { pub label: Option<String>, pub labels: Vec<String>, pub min: Option<f64>, pub max: Option<f64> }
```

## 6. 파싱 플로우

```
parse_gso_control(ctrl_data, child_records)
  ├── common = parse_common_obj_attr(ctrl_data)
  ├── drawing = parse_drawing(...)
  ├── CHART_DATA 존재 여부 확인
  │    └── 있으면 → parse_chart_shape(common, drawing, child_records)
  ├── SHAPE_COMPONENT_OLE 태그 확인
  │    └── 있으면 → parse_ole_shape(common, drawing, ole_tag_data, child_records)
  └── 기타 shape_tag_id별 분기 (기존 로직)
```

## 7. 렌더링 방침

- SVG `<g class="hwp-chart">` 컨테이너 내부에 축/격자/시리즈를 별도 요소로 렌더
- 축/격자: `<line>` / `<text>`
- 막대: `<rect>`
- 선: `<polyline>` / `<path>`
- 파이: `<path>` (arc)
- 범례: `<g class="hwp-chart-legend">`
- raw 바이트 미파싱 차트: 회색 사각형 + "차트(미지원 종류)" 텍스트로 폴백

## 8. 검증 체크리스트

- [ ] CHART_DATA 존재하는 GSO를 Chart로 재분류
- [ ] raw_records 보존으로 라운드트립 동일성
- [ ] 1.hwp(로컬 검증용) export-svg에서 차트 위치에 막대/선 그래프 렌더
- [ ] 자체 제작 samples/chart-basic.hwp로 회귀

## 9. 참고 자료

- HWP 5.0 파일 포맷 스펙 (한컴 공식)
- pyhwp: https://github.com/mete0r/pyhwp (BSD)
- hwplib: https://github.com/neolord0/hwplib (Apache 2.0)
- rhwp 기존 도형 파서: `src/parser/control/shape.rs` — PICTURE/LINE/POLYGON 등

## 10. 미해결 이슈 (단계 3에서 결정)

- 하위 태그 80~95의 정확한 바이트 레이아웃은 공식 문서 + pyhwp 코드 교차 확인 필요
- 데이터 시리즈가 많을 때 SVG 성능(단계 4에서 측정)
- 차트 애니메이션 속성(1차 범위 외)
