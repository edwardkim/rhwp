# Task #1587 — Stage 2 완료보고서

**단계**: 모델 + 파서 확장
**브랜치**: `local/task1587`

## 변경 내용

| # | 파일 | 변경 |
|---|------|------|
| C1 | `src/model/control.rs` Ruby | `alignment`(u8) 제거 → `main_text`, `pos_type`, `align`, `sz_ratio`, `option`, `style_id_ref` 추가 |
| C2 | `src/parser/hwpx/section.rs` parse_dutmal | posType/align 분리 보존 + szRatio/option/styleIDRef 파싱 + mainText 보존(종전 skip 제거) |

## 검증

- `cargo build` 성공 — `alignment` 제거가 **외부 파급 0**(parse_dutmal 한 곳만 사용 확인 입증).
- Stage 1 RED 테스트 `task1587_ruby_control_roundtrips` 는 `..Default::default()` 사용으로
  모델 변경 후에도 컴파일되며, 직렬화기 미수정이므로 **여전히 RED**(Ruby 드롭). 의도된 상태.

## 다음 단계

Stage 3 — 직렬화기(`write_ruby` + `render_control_slot` arm). Stage 1 GREEN 전환 +
신규 필드 무손실 단언 추가 + baseline 회귀 0 확인.
