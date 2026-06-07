# 구현 계획서 — Task M100 #1321

## 개요

**이슈**: #1321 — #1298 후속 — HWPX 시리얼라이저: 빈 문단의 0-length field fieldEnd/fieldBegin 순서 역전  
**브랜치**: `local/task1321`  
**수정 파일**: `src/serializer/hwpx/section.rs`

---

## 문제 구조 정밀 분석

### 빈 문단에서의 실행 흐름

```
para.text = ""
para.char_count = 17  (fieldBegin 8cu + fieldEnd 8cu + para_end 1cu)
para.field_ranges = [{ start: 0, end: 0, control_idx: 0 }]
para.controls = [Field { field_id: 55, ... }]
```

`inferred_control_slot_count` 계산:
- `text_units = 0`
- `from_char_count = (17 − 1 − 0) / 8 = 2`
- `from_offsets = 0` (text 루프 0회)
- `slot_count = max(2, 0) − 1(field_ranges) = 1`
- `slots = [&Field]` (slot_count 1 == controls.len() 1)

`render_run_content` 실행 순서:

| 단계 | 코드 위치 | 동작 |
|------|-----------|------|
| 1 | line 264 메인 루프 | **0회 실행** — pre-char 0-length 검사 건너뜀 |
| 2 | line 326 | `flush_text_fragment` (text_buf 비어있음, no-op) |
| 3 | line 329~339 post-loop fieldEnd | `!field_end_emitted[0]` → **`<fieldEnd>` 방출** ← 먼저 |
| 4 | line 341~344 remaining slots | slot_idx=0 < 1 → **`<fieldBegin>` 방출** ← 나중 |

결과: `<fieldEnd> <fieldBegin>` — 순서 역전

---

## 수정 전략

### 채택 방법: 메인 루프 진입 전 빈 문단 전용 블록 추가

비어있는 `para.text` 를 감지하여, **루프 진입 전에** slots(fieldBegin)를 모두 방출한 뒤 0-length fieldEnd를 방출한다. 이후 slot_idx와 field_end_emitted가 갱신되므로 post-loop 두 블록(329~344)은 no-op이 된다.

```rust
// line 263 직후 (field_end_emitted 초기화 다음)에 삽입:

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

### 수정 후 실행 흐름

| 단계 | 동작 |
|------|------|
| pre-loop 블록 | `while slot_idx < slots.len()` → **`<fieldBegin>` 방출**, slot_idx=1 |
| pre-loop 블록 | `start==end && !emitted` → **`<fieldEnd>` 방출**, emitted[0]=true |
| 메인 루프 | 0회 실행 (no-op) |
| post-loop fieldEnd (329~339) | `field_end_emitted[0]` == true → 건너뜀 |
| remaining slots (341~344) | `slot_idx == slots.len()` → 건너뜀 |

결과: `<fieldBegin> <fieldEnd>` ✓

### 기존 경로에 대한 영향

`para.text.is_empty()` 조건 아래 블록이 실행되므로 비어있지 않은 문단의 처리 경로에는 **전혀 영향 없음**.

---

## 구현 단계

### Stage 1 — 코드 수정 (`render_run_content`)

**파일**: `src/serializer/hwpx/section.rs`  
**위치**: line 262 (`field_end_emitted` 초기화) 직후

위 "채택 방법"의 블록을 삽입한다.  
변경 범위: ~15줄 추가, 기존 코드 수정 없음.

### Stage 2 — 단위 테스트 추가

**파일**: `src/serializer/hwpx/section.rs` `mod tests` 내부  
**추가 테스트**:

#### `task1321_zero_length_field_in_empty_paragraph`

```
para.text = ""
para.char_count = 17  (8+8+1)
para.char_offsets = []
field_ranges = [{ start: 0, end: 0, control_idx: 0 }]
```

검증 항목:
- `fieldBegin` 이 XML에 존재
- `fieldEnd` 이 XML에 존재
- `fieldBegin` 위치 < `fieldEnd` 위치

#### `task1321_multiple_zero_length_fields_in_empty_paragraph` (선택)

field 2개(`id=10`, `id=20`)가 모두 0-length인 빈 문단.  
각 fieldBegin-fieldEnd 쌍의 순서 검증.

### Stage 3 — 회귀 검증

```bash
cargo test -p rhwp -- section::tests 2>&1 | tail -5
cargo clippy -- -D warnings
```

기존 `task1298_*` 테스트 포함 전체 section 테스트 통과 확인.

---

## 단계별 커밋 계획

| 단계 | 커밋 메시지 |
|------|------------|
| Stage 1 | `fix: 빈 문단 0-length field fieldBegin/fieldEnd 순서 역전 수정 (#1321)` |
| Stage 2 | `test: 빈 문단 0-length field 인터리빙 회귀 테스트 추가 (#1321)` |
| Stage 3 | 보고서 커밋 (소스 변경 없음) |

---

## 검증 기준

- `task1321_zero_length_field_in_empty_paragraph` 통과
- 기존 `task1298_zero_length_field_at_para_start`, `task1298_zero_length_field_mid_text` 통과
- `cargo clippy -- -D warnings` 경고 0건
