# Stage 2 완료보고서 — Task M100 #1199

**단계**: 회귀 테스트
**브랜치**: `local/task1199`

## 추가 테스트 (`src/parser/hwpx/section.rs` tests 모듈)

1. `test_parse_note_prefix_char_maps_to_before_decoration_letter`
   - `<hp:endNote prefixChar="47928" suffixChar="65289">` / `<hp:footNote ...>` 파싱.
   - 검증: `before_decoration_letter == 47928`(='문'), `after_decoration_letter == 65289`(='）') — 미주·각주 양쪽.

2. `test_parse_note_without_prefix_char_keeps_zero_before_letter`
   - prefixChar 없는 `<hp:endNote suffixChar="41">`.
   - 검증: `before_decoration_letter == 0`(접두 없음 유지), `after_decoration_letter == 41`(=')'). 회귀 방지.

## 결과

```
cargo test --release --lib test_parse_note
test result: ok. 2 passed; 0 failed
```

전체 스위트(`cargo test --release`) 통과, 회귀 0건.
