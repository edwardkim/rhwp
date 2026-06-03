# Task M100-1251 Stage 3.5 — 한컴 차트 사양 기반 parser 보강

- **이슈**: [#1251](https://github.com/edwardkim/rhwp/issues/1251)
- **브랜치**: `task-1251-ole-chart`
- **기준 문서**: `/Users/melee/Downloads/한글문서파일형식_차트_revision1.2.pdf`
- **상태**: 구현 및 1차 검증 완료

## 1. 공식 문서 확인 결과

한컴 차트 사양 revision 1.2의 `Chart Object의 기본 구조`는 차트 바이너리 데이터가 `ChartOBJ`의 순차 나열이라고 설명한다.

`ChartOBJ` 기본 필드:

| 필드 | 자료형 | 비고 |
|---|---|---|
| `id` | `long` | 객체 id |
| `StoredtypeId` | `long` | 객체 type id |
| `StoredName` | `char*` | variable data |
| `StoredVersion` | `int` | variable data |
| `ChartObjData` | `chartObject` | payload |

중요한 제약:

- `StoredName`과 `StoredVersion`은 variable data이다.
- 동일한 `StoredtypeID`가 앞에 나온 경우 뒤 객체에서는 variable data가 제외될 수 있다.
- 차트 tree는 `VtChart` 아래 `BackDrop`, `DataGrid`, `Footnote`, `Legend`, `Plot`, `PrintInformation`, `Title` 계열 object를 가진다.
- `DataGrid Object`는 `ColumnCount`, `ColumnLabel`, `RowCount`, `RowLabel` 등을 정의한다.
- `Title Object`는 `Text`를 기본 속성으로 갖는다.
- `ChartType Constants`는 0-26 값을 정의하지만, 현재 fixture의 `/Contents`에서 해당 property 위치는 아직 안정적으로 특정하지 못했다.

## 2. parser 반영 내용

`src/ole_chart/parser.rs` 변경:

- `OleChartContentsProbe`에 legacy chart 진단 필드 추가
  - `legacy_chart_object_start`
  - `has_vt_chart_marker`
  - `has_vt_data_grid_marker`
  - `has_vt_chart_title_marker`
- `Contents` 첫 16바이트의 네 번째 little-endian word를 `ChartOBJ` 시작 오프셋 후보로 기록
- legacy 판정은 `ChartOBJ` 시작 오프셋과 `VtDataGrid` marker 존재로 고정
- `VtDataGrid` marker 뒤의 `StoredVersion(int)` 4바이트를 건너뛰고 grid data 범위를 시작
- `VtBackdrop`, `VtBackDrop`, `VtChartSection`, `VtLegend`, `VtPlot`, `VtPrintInformation`, `VtChartTitle`, `VtTitle` 등을 grid boundary 후보로 사용
- grid value extraction은 dense f64 run을 우선 선택하고, 실패 시 기대 개수와 정확히 일치하는 sparse 후보만 허용

## 3. 현재 한계

- 공식 PDF는 object/property 표를 제공하지만, property serialization의 바이트 단위 레이아웃을 완전히 설명하지 않는다.
- `ChartType` property의 실제 offset/context는 #1251 fixture 하나만으로 확정하지 않았다.
- `VtChart` marker는 fixture probe의 필드로 노출하지만 legacy 판정 필수 조건으로 두지 않았다. 공식 문서의 variable data 생략 규칙 때문에 일부 object name은 stream에 직접 나타나지 않을 수 있다.

## 4. 검증 결과

통과:

```text
cargo fmt --check
cargo test --lib ole_chart -- --nocapture
cargo test --test issue_1251_ole_chart_contents -- --nocapture
```

fixture 유지 결과:

- title: `연금 재정 전망`
- categories: `2010년`, `2020년`, `2030년`, `2040년`
- series:
  - `적립금`: `328`, `812`, `1702`, `1477`
  - `수입`: `50`, `70`, `189`, `191`
  - `지출`: `11`, `15`, `201`, `289`

## 5. 다음 단계

Stage 4에서 renderer 통합을 진행한다.

렌더 통합 전 유지할 정책:

- `OOXMLChartContents`는 기존처럼 우선한다.
- legacy `Contents` parsing 성공 시 `OleChart`를 `charming` chart로 변환한다.
- parsing 실패 시 generic OLE placeholder로 조용히 묻지 않고 명시적인 `OLE 차트 미지원:` fallback을 유지한다.
