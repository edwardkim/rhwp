# Task M100-1251 Stage 3 완료 보고서

- **이슈**: [#1251](https://github.com/edwardkim/rhwp/issues/1251)
- **단계**: Stage 3 — fixture 최소 데이터 추출
- **브랜치**: `task-1251-ole-chart`
- **작성일**: 2026-06-03

## 1. 변경 내용

`src/ole_chart/parser.rs`의 legacy HWP chart `Contents` 파서가 #1251 fixture에서 `OleChart` IR을 생성하도록 보강했다.

추출 방식:

1. `VtDataGrid` marker를 찾는다.
2. `VtBackdrop` 또는 `VtChartSection` 직전까지를 data grid 범위로 본다.
3. data grid 범위에서 CP949 문자열 payload를 추출한다.
4. 같은 범위에서 정수형 f64 값 후보를 추출한다.
5. 라벨은 첫 숫자 라벨 전까지를 series 이름, 숫자 라벨을 category로 분리한다.
6. 값 배열은 row-major로 읽고 series-major `OleChartSeries`로 전치한다.

제목은 `VtChartTitle`부터 `VtList` 전까지의 문자열 후보 중 가장 긴 라벨을 사용한다. CP949 full-width space는 일반 공백으로 정규화한다.

## 2. Fixture 추출 결과

`samples/143E433F503322BD33.hwp`의 `BinData #2` `/Contents`에서 다음 데이터를 추출했다.

- 제목: `연금 재정 전망`
- 차트 종류: `Unknown`
- 카테고리: `2010년`, `2020년`, `2030년`, `2040년`
- 시리즈:
  - `적립금`: `328`, `812`, `1702`, `1477`
  - `수입`: `50`, `70`, `189`, `191`
  - `지출`: `11`, `15`, `201`, `289`

차트 종류는 아직 `VtSeries`/plot 레코드에서 안정적으로 식별하지 못했다. Stage 3에서는 임의로 column/bar/line을 단정하지 않고 `OleChartType::Unknown`으로 둔다.

## 3. 제한 사항

- 공개 스펙이 확인되지 않아 legacy HWP chart object graph 전체를 해석하지 않는다.
- f64 값 추출은 data grid 범위 안의 `>= 1.0` 정수형 값으로 제한했다. #1251 fixture에는 0 또는 소수 값이 없어 안정적으로 동작한다.
- 다른 legacy chart fixture에서는 값 범위나 레코드 shape가 다를 수 있으므로, shape가 맞지 않으면 성공 파싱하지 않고 `UNSUPPORTED_CONTENTS_LAYOUT`으로 실패한다.
- 차트 종류와 세부 스타일은 아직 렌더링에 반영하지 않는다.

## 4. 테스트

수정한 테스트:

- `tests/issue_1251_ole_chart_contents.rs`
  - 기존 stable error 검증을 성공 파싱 검증으로 변경
  - 제목, 카테고리, 시리즈 이름, 시리즈 값 검증 추가

추가한 단위 테스트:

- CP949 payload가 null pair 전까지 올바르게 디코딩되는지 확인
- CP949 full-width space가 일반 공백으로 정규화되는지 확인

## 5. 실행 결과

```text
cargo fmt --check
```

통과.

```text
cargo test --test issue_1251_ole_chart_contents -- --nocapture
```

통과: 4 passed.

```text
cargo test --lib ole_chart -- --nocapture
```

통과: 5 passed.

```text
cargo build
```

통과.

```text
cargo check --target wasm32-unknown-unknown --lib
```

통과.

## 6. 다음 단계

Stage 4에서는 renderer 통합을 진행한다.

목표:

1. nested OLE `raw_contents`가 있을 때 `ole_chart` parser를 호출한다.
2. native target에서는 `OleChart`를 `charming` chart로 변환해 SVG 문자열을 생성한다.
3. wasm 또는 실패 경로에서는 구체적인 fallback label을 유지한다.
4. #1251 fixture의 export-svg 결과에서 generic `OLE 개체 (BinData #2)` placeholder를 제거한다.
