# Task M100-1251 Stage 1 완료 보고서

- **이슈**: [#1251](https://github.com/edwardkim/rhwp/issues/1251)
- **단계**: Stage 1 — OLE `/Contents` 구조 진단과 최소 파서 골격
- **브랜치**: `task-1251-ole-chart`
- **작성일**: 2026-06-03

## 1. 변경 내용

OLE `/Contents` 전용 차트 모듈 골격을 추가했다.

- `src/ole_chart/mod.rs`
- `src/ole_chart/parser.rs`

추가한 public API:

- `probe_ole_chart_contents(bytes)`
- `parse_ole_chart_contents(bytes)`
- `OleChartContentsProbe`
- `OleChartParseError`
- `OleChart`, `OleChartType`, `OleChartSeries`

Stage 1의 `parse_ole_chart_contents`는 아직 실제 차트 IR 생성을 하지 않는다. 대신 #1251 fixture의 legacy HWP chart `Contents` 레이아웃을 안정적으로 감지하고, 다음 오류 코드로 고정한다.

```text
UNSUPPORTED_CONTENTS_LAYOUT
```

## 2. Fixture 확인

`samples/143E433F503322BD33.hwp`의 `BinData #2` 확인 결과:

- DocInfo BinData #2: `Storage`, `storage_id=2`, extension `OLE`
- `/BinData/BIN0002.OLE`: CFB magic으로 시작
- nested OLE 내부 stream: `Contents`만 존재
- `Contents` 길이: `9,876 bytes`
- `Contents` 첫 16바이트:

```text
00 00 01 00 ec 2e 00 00 ec 2e 00 00 60 00 00 00
```

현재 추출되지 않는 항목:

- `OOXMLChartContents`
- `OlePres000` EMF preview
- native image preview

## 3. 테스트

신규 테스트:

- `tests/issue_1251_ole_chart_contents.rs`

검증 항목:

1. `BinData #2`가 OLE storage이고 nested OLE에서 `Contents`만 존재함
2. `probe_ole_chart_contents`가 길이, 첫 u32 words, legacy layout 후보를 안정적으로 반환함
3. `parse_ole_chart_contents`가 Stage 1 기준 안정적인 `UNSUPPORTED_CONTENTS_LAYOUT` 오류를 반환함

## 4. 실행 결과

```text
cargo fmt --check
```

통과.

```text
cargo test --lib ole_chart -- --nocapture
```

통과: 3 passed.

```text
cargo test --test issue_1251_ole_chart_contents -- --nocapture
```

통과: 3 passed.

```text
cargo test --test issue_1156_chart_column_flow -- --nocapture
```

통과: 2 passed.

## 5. 다음 단계

Stage 2에서는 `charming` 네이티브 SSR 도입 가능성을 실제 dependency 빌드와 SVG smoke test로 검증한다.

주의: Stage 2는 `Cargo.toml`/`Cargo.lock` 변경과 dependency 다운로드 가능성이 있으므로 작업지시자 승인 후 진행한다.
