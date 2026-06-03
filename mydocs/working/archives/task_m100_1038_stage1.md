# Stage 1 완료보고서 — Task M100 #1038

## 작업
HWPX `slash`/`backSlash` 핸들러를 대각선 **방향 비트 전담**으로 수정하고,
선 종류/굵기/색은 `<hh:diagonal>` 요소가 단독 책임지도록 분리.

## 변경 (`src/parser/hwpx/header.rs`)
1. **`parse_slash_shape_code(attr) -> u8` 신규**: slash 형태 enum 문자열 → HWP5 attr 3비트 방향 코드.
   - `NONE`→0, `CENTER`→0b010, `CENTER_BELOW`→0b011, `CENTER_ABOVE`→0b110, 기타→0b111.
2. **`set_diagonal_attr_bits(bf, shift, code)` 변경**: 받은 3비트 `code`를 해당 shift에 그대로 기록
   (기존: nonzero를 무조건 0b010으로 축소). `code==0`이면 비트 클리어.
3. **`b"slash"` / `b"backSlash"` 핸들러 수정**:
   - `parse_border_line_type_code` 호출 → `parse_slash_shape_code` 로 교체.
   - `bf.diagonal.diagonal_type = …` 할당 **제거** (방향 비트만 설정).
   - slash/backSlash 의 `width`/`color` 분기 제거 (OWPML에서 slash는 선 스타일 미보유).

`<hh:diagonal>` 핸들러(선 종류/굵기/색)와 `parse_diagonal_width`/`parse_border_line_type_code`는
그대로 유지 (diagonal 요소가 계속 사용).

## 검증 (예비)
- `cargo build` 통과, 미사용 경고 없음.
- `export-svg -p 3` 재생성:
  - 검정 대각선(`stroke="#000000"`) line: **3 → 0**
  - 전체 line: 9 → 6 (파란 박스 테두리만 잔존)
  - 헤딩 "Ⅰ 사업안내" 시각 확인 → 한컴 2022 PDF p4 정합.

정식 단위 테스트/전체 회귀는 Stage 2~3에서 수행.
