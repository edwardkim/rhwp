# Task #1587 — Stage 3 완료보고서

**단계**: 직렬화기 (write_ruby + arm)
**브랜치**: `local/task1587`

## 변경 내용

| # | 파일 | 변경 |
|---|------|------|
| C3a | `serializer/hwpx/section.rs` | `render_dutmal(r: &Ruby)` 추가 — `<hp:dutmal>` 역매핑(속성 순서 posType/szRatio/option/styleIDRef/align + mainText/subText) |
| C3b | `serializer/hwpx/section.rs` | `render_control_slot` 에 `Control::Ruby(r) => render_dutmal(r)` arm 추가 |
| — | import | `Ruby` 타입 use 추가 |

Ruby 는 이미 `is_hwpx_inline_slot` 포함 → 슬롯 위치 자동, arm 추가만으로 방출.

## 검증 (모두 GREEN)

| 검사 | 결과 |
|------|------|
| `task1587_ruby_control_roundtrips` (전 필드 무손실) | **PASS** — main_text/ruby_text/pos_type/align/sz_ratio/option/style_id_ref 보존 |
| `cargo test --lib` 전체 | **1961 passed, 0 failed** |
| `hwpx_roundtrip_baseline` | **4/4 PASS** |
| `cargo clippy --lib` (변경 파일) | 무경고 |

## 다음 단계

Stage 4 — fidelity 전수 통제 비교: 3건(36384160·36399208·36389301) 해소 + 악화 0 +
순효과>0 확인, opengov 가드 편입.
