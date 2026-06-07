# Task M100 #1321 — Stage 1 완료보고서

## 개요

| 항목 | 내용 |
|------|------|
| 이슈 | #1321 — 빈 문단(text == "") 0-length field fieldBegin/fieldEnd 순서 역전 수정 |
| 단계 | Stage 1: 코드 수정 |
| 브랜치 | `local/task1321` |
| 커밋 | `7127a7c9` |

## 수행 내용

### 근본 원인

`render_run_content` 함수의 pre-char 검사는 `for (idx, c) in para.text.chars().enumerate()` 루프 **내부**에 위치한다. `para.text == ""`인 빈 문단에서는 이 루프가 0회 실행되므로 pre-char 검사가 전혀 실행되지 않는다.

결과적으로 post-loop 처리 순서가 역전된다:
1. 후처리 1 (lines 329–339): 남은 `field_ranges`의 `fieldEnd` 방출 → **fieldEnd 먼저**
2. 후처리 2 (lines 341–344): 남은 slots의 `fieldBegin` 방출 → **fieldBegin 나중**

HWPX 스펙은 `<fieldBegin>` → `<fieldEnd>` 순서를 요구하므로, 빈 문단 + 0-length field 조합에서 XML이 유효하지 않게 된다.

### 수정 내용

`src/serializer/hwpx/section.rs` — `render_run_content` 함수

`field_end_emitted` 초기화 직후, 메인 루프 이전에 다음 블록을 삽입했다:

```rust
// 빈 문단(text == "")의 0-length 필드: 메인 루프가 실행되지 않아
// pre-char 검사를 통과하지 못하므로 루프 전에 slots → fieldEnd 순으로 방출한다.
if para.text.is_empty() {
    while slot_idx < slots.len() {
        render_control_slot(&mut out, slots[slot_idx], ctx);
        slot_idx += 1;
    }
    for (i, fr) in para.field_ranges.iter().enumerate() {
        if fr.start_char_idx == fr.end_char_idx && !field_end_emitted[i] {
            if let Some(Control::Field(f)) = para.controls.get(fr.control_idx) {
                if let Ok(xml) = writer_to_string(|w| write_field_end(w, f.field_id)) {
                    out.push_str("<hp:ctrl>");
                    out.push_str(&xml);
                    out.push_str("</hp:ctrl>");
                }
            }
            field_end_emitted[i] = true;
        }
    }
}
```

이 블록 실행 후 `slot_idx == slots.len()` 이고 `field_end_emitted[i] == true`가 되어, 두 post-loop 블록은 no-op이 된다.

## 확인 사항

- `src/serializer/hwpx/section.rs` 수정: 1파일, +66줄
- 커밋: `fix: 빈 문단 0-length field fieldBegin/fieldEnd 순서 역전 수정 (#1321)`

## 상태

완료
