# Stage 2 완료보고서 — Task M100 #1038

## 작업
`src/parser/hwpx/header.rs` 테스트 모듈에 slash 형태 파싱·diagonal_type 불변 회귀 가드 추가.

## 신규 테스트 (4건)
1. **`test_parse_slash_shape_code`** — 형태 enum → 3비트 방향 코드
   - NONE→0, CENTER→0b010, CENTER_BELOW→0b011, CENTER_ABOVE→0b110, ALL(미지)→0b111.
2. **`test_set_diagonal_attr_bits`** — 3비트 코드를 shift 2(slash)/5(backSlash)에 정확히 기록,
   타 비트 보존, code 0 → 클리어.
3. **`test_slash_center_without_diagonal_no_line`** — #1038 핵심 회귀 가드
   - `slash type="CENTER"` + `<hh:diagonal>` 없음 → slash 비트 0b010, **diagonal_type==0** (미표시).
4. **`test_diagonal_element_sets_line_independent_of_slash`**
   - `slash type="NONE"` + `<hh:diagonal type="SOLID">` → slash 비트 0, **diagonal_type==1**.

테스트 헬퍼: `slash_code()`(Attribute 직접 파싱), `parse_single_border_fill()`(헤더 XML 래핑 후
`parse_hwpx_header` 경유 end-to-end 파싱).

## 검증
- `cargo test --lib parser::hwpx::header`: **13 passed, 0 failed** (신규 4건 포함).
- 핵심 회귀 가드(#3)가 버그 재발 시 실패하도록 고정.

## 비고
- raw string 내 `color="#000000"` 의 `"#` 충돌로 해당 테스트만 `r##"..."##` 구분자 사용.
