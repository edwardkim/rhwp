# PR #2764: fix(hwp5): PARA_SHAPE 말미 4바이트 개요 수준 파서 누락 및 직렬화 하드코딩 수정

## 개요

HWP5 `PARA_SHAPE` 레코드 말미에 위치한 4바이트(INT32) 개요 수준(outline level)
필드를 파서가 읽지 않고 직렬화기가 0으로 하드코딩하는 문제를 수정한다.

## 문제 증상

1. **문단 서식 편집 한 번에 개요 수준 리셋**: 한컴 편집기에서 문단 정렬/여백 등
   서식을 변경하면 `PARA_SHAPE`가 다시 쓰이는데, 이때 직렬화기가 개요 수준을
   항상 0으로 기록하여 저장 후 재실행 시 모든 제목 문단이 본문으로 강등된다.

2. **8~10수준 개요 읽기 붕괴**: 파서가 말미 4바이트를 읽지 않아 `outline_level`
   이 기본값 0이 되므로, 7수준을 초과하는 개요 수준(8~10)을 보존할 수 없다.

## 변경 파일

### 1. `src/model/style.rs`

- `ParaShape` 구조체에 `outline_level: i32` 필드 추가
  - 기본값 0(본문), 1~7(개요 수준), 그 이상도 보존 가능
- `PartialEq` 구현에 `outline_level` 비교 추가
- `ParaShapeMods` 구조체에 `outline_level: Option<i32>` 추가
- `ParaShapeMods::apply_to()`에서 outline_level 적용 처리

### 2. `src/parser/doc_info.rs`

- `parse_para_shape()` 함수末尾, `line_spacing_v2` 이후 남은 4바이트를
  `r.read_i32()`로 읽어 `outline_level` 필드에 저장
- 데이터가 부족하면 기본값 0 유지 (하위 호환)

### 3. `src/serializer/doc_info.rs`

- `serialize_para_shape()` 함수에서 하드코딩 `w.write_u32(0)` 대신
  `w.write_i32(ps.outline_level)`로 실제값 기록
- 기존 주석을 `#2734` 참조로 갱신

## 검증

- `cargo clippy --all-targets -- -D warnings` 통과
- `cargo fmt --all -- --check` 통과
- 기존 `ParaShape` 생성 코드(`..Default::default()`)는 `outline_level=0`으로
  동작不变

## 리뷰 포인트

- `outline_level`은 INT32로 파싱/직렬화 (u32가 아닌 `read_i32`/`write_i32` 사용)
- 빈약한 PARA_SHAPE(이전 HWP 버전 등)에서 말미 4바이트가 없으면 `remaining() >= 4`
  가드로 기본값 0 유지
- `ParaShapeMods`로 개요 수준 변경 가능 (IR 수정 경로 대응)
