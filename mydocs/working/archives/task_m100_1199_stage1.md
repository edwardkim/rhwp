# Stage 1 완료보고서 — Task M100 #1199

**단계**: 파서 수정 (prefixChar → before_decoration_letter)
**브랜치**: `local/task1199`

## 변경

`src/parser/hwpx/section.rs`:
- `parse_ctrl_footnote()` — `b"prefixChar"` 분기 추가 → u16(코드포인트) 파싱 → `note.before_decoration_letter`.
- `parse_ctrl_endnote()` — 동일 분기 추가.

`suffixChar`(→ `after_decoration_letter`)와 대칭 구현. prefixChar 속성이 없으면 `before_decoration_letter`는 `default()`의 0(접두 없음) 유지.

## 검증

- `cargo build --release` 성공.
- 변경: 추가만(2개 분기), 기존 경로 불변.

## 비고

근본 원인은 렌더러가 아니라 파서 입력 누락이었음. 렌더 경로(`format_endnote_marker_text`, typeset.rs:60)는 이미 `before_decoration_letter`로 접두를 조립하고 있어 변경 불필요.
