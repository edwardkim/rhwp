# 단계별 완료 보고서 — Task M100 #1298 Stage 1

## 개요

구현 계획서의 Stage 1(수정) + Stage 2(테스트)를 단일 커밋으로 완료했다.

**커밋**: `1f25588f`  
**변경 파일**: `src/serializer/hwpx/section.rs`

---

## 수행 내용

### 수정 1: 0-length fieldEnd pre-char 방출 추가

`render_run_content` 내 메인 문자 루프에서, 슬롯 방출(fieldBegin) 직후 `text_buf.push(c)` 직전에 0-length 필드 검사를 추가했다.

```rust
// 0-length 필드(start == end == idx): fieldBegin 방출 직후, 문자 push 전에 fieldEnd 방출.
for (i, fr) in para.field_ranges.iter().enumerate() {
    if fr.start_char_idx == fr.end_char_idx
        && fr.end_char_idx == idx
        && !field_end_emitted[i]
    {
        flush_text_fragment(...);
        // emit fieldEnd
        field_end_emitted[i] = true;
    }
}
```

### 수정 2: 기존 post-char fieldEnd 검사에 guard 추가

0-length 필드가 idx-1 시점의 post-char 검사에서 역순 방출되는 것을 차단했다.

```rust
if fr.end_char_idx == next_idx
    && !field_end_emitted[i]
    && fr.start_char_idx < fr.end_char_idx  // 추가
```

### 테스트 추가 (Stage 2)

| 테스트명 | 검증 내용 |
|---------|----------|
| `task1298_zero_length_field_at_para_start` | start=0, end=0: fieldBegin < fieldEnd < text 순서 |
| `task1298_zero_length_field_mid_text` | start=3, end=3: ABC < fieldBegin < fieldEnd < DE 순서 |

---

## 검증 결과

```
running 14 tests
... (all 14 passed)
test result: ok. 14 passed; 0 failed
```

`cargo clippy --lib -- -D warnings`: 경고 0건

---

## 수정 전후 비교

| 케이스 | 수정 전 | 수정 후 |
|--------|---------|---------|
| start=0, end=0 | `<fieldBegin/>` text `<fieldEnd/>` | `<fieldBegin/><fieldEnd/>` text |
| start=3, end=3 | AB `<fieldEnd/>` `<fieldBegin/>` CDE | ABC `<fieldBegin/><fieldEnd/>` DE |
| start=0, end=5 (정상) | `<fieldBegin/>` text `<fieldEnd/>` | `<fieldBegin/>` text `<fieldEnd/>` (불변) |
