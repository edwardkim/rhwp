# PR #2718: hwpx 표 hp:tbl 속성 3종 하드코딩 제거 — widthRelTo/heightRelTo/protect/numberingType

## 이슈
- **Issue**: #2697 — 표 `<hp:tbl>` 속성 3종이 하드코딩으로 라운드트립 유실

## 분석

표 HWPX 직렬화기에서 4개 속성이 IR 대신 리터럴로 방출되어 라운드트립 소실 발생:

| 속성 | 문제 | 수정 |
|------|------|------|
| `hp:sz@widthRelTo` | `"ABSOLUTE"` 하드코딩 | `size_criterion_width_str()`로 IR 방출 |
| `hp:sz@heightRelTo` | `"ABSOLUTE"` 하드코딩 | `size_criterion_height_str()`로 IR 방출 (Column/Para→Absolute) |
| `hp:sz@protect` | `"0"` 하드코딩 + 파서 arm 부재 | `bool01(size_protect)` + 파서 추가 |
| `hp:tbl@numberingType` | `"TABLE"` 하드코딩 + 파서 arm 부재 | `numbering_type_str()` + 파서 추가 |

## 변경

### 파서 (`src/parser/hwpx/section.rs`)
- `numberingType` 속성 파싱 추가
- `hp:sz protect` 속성 파싱 추가
- `materialize_hwpx_table_attrs` numbering bit 조건부 설정으로 수정

### 직렬화 (`src/serializer/hwpx/table.rs`)
- `size_criterion_width_str`/`height_str` 헬퍼 추가
- `write_sz`에서 IR 값 방출
- `numbering_type` IR 값 방출

## 검증
- `cargo test --lib -- hwpx::table` — 43/43 통과
