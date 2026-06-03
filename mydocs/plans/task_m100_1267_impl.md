# 구현 계획서 — #1267 HWPX 시리얼라이저: 탭 폭 + 사선 수정

- **이슈**: #1267
- **브랜치**: `local/task1267`
- **작성일**: 2026-06-03

---

## 단계 구성

### Stage 1 — 버그 2: slash/backSlash 사선 직렬화 수정 (`header.rs`)

**변경 파일**: `src/serializer/hwpx/header.rs`

1. `slash_shape_code_str(code: u8) -> &'static str` 헬퍼 추가 (파서 `parse_slash_shape_code` 역함수)
2. `write_diag_line` 시그니처에 `code: u8` 파라미터 추가
3. 호출부에서 `bf.attr` bits 2-4(slash), 5-7(backSlash) 추출 후 전달

**커밋**: `fix: HWPX 시리얼라이저 slash/backSlash 사선 방향 코드 복원 (closes 일부 #1267)`

---

### Stage 2 — 버그 1: 탭 폭/leader/type 직렬화 수정 (`section.rs`)

**변경 파일**: `src/serializer/hwpx/section.rs`

1. `render_hp_t_content` 시그니처 변경:
   ```
   fn render_hp_t_content(text: &str, tab_extended: &[[u16; 7]], tab_idx: &mut usize) -> String
   ```
2. 탭 문자 처리 블록에서 `tab_extended.get(*tab_idx)` 참조 후 idx 증가
3. `flush_text_fragment` 에 동일 파라미터 추가
4. `render_run_content` 에 `tab_idx` 초기화 + 모든 호출부 전파
5. `render_paragraph_parts_for_text` 호출부에 `&[]`, `&mut 0` 전달

**커밋**: `fix: HWPX 시리얼라이저 탭 폭/leader/type IR 기반 출력 (closes #1267)`

---

### Stage 3 — 테스트 추가 + 전체 검증

**변경 파일**: `tests/issue_1267_hwpx_tab_and_diagonal.rs`

1. 탭 roundtrip 테스트: `<hp:tab width="17283" leader="3" type="2"/>` → HWPX 재직렬화 → 동일 값 보존 확인
2. slash/backSlash roundtrip 테스트: slash `type="CENTER"` 있는 borderFill → 재직렬화 → 동일 type 보존 확인
3. `cargo test` 전체 통과 확인
4. golden SVG 이상 없음 확인

**커밋**: `test: issue_1267 탭 폭 + 사선 HWPX roundtrip 검증`

---

승인 요청합니다.
