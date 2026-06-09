# 구현 계획서 — Task M100 #1298

## 개요

**이슈**: #1298 — HWPX 시리얼라이저: 빈 필드(0-length field range) fieldBegin/fieldEnd 인터리빙 오류  
**브랜치**: `local/task1298`  
**수정 파일**: `src/serializer/hwpx/section.rs`

---

## 문제 구조 정밀 분석

### 현재 fieldEnd 방출 흐름

`render_run_content`의 메인 문자 루프:

```
for (idx, c) in para.text.chars().enumerate() {
    [1] 슬롯 방출 (fieldBegin 등): char_pos >= expected + 8 이면 flush → render_control_slot
    [2] text_buf.push(c)
    [3] expected_utf16_pos 갱신
    [4] fieldEnd 방출: fr.end_char_idx == (idx + 1) 이면 flush → fieldEnd 방출
}
// 루프 후
[5] flush
[6] 미방출 fieldEnd post-loop 처리
[7] 남은 슬롯 post-loop 처리
```

### 결함 1: `end_char_idx == 0` → fieldEnd가 텍스트 뒤로 밀림

0-length 필드 at position 0 (`start=0, end=0`):
- `char_offsets[0]` = 16 (fieldBegin 8cu + fieldEnd 8cu 갭)
- `inferred_control_slot_count` = 갭 2개 − field_ranges 1개 = 1 슬롯 (fieldBegin)
- **[1]**: idx=0에서 char_pos=16 ≥ 0+8 → fieldBegin 방출 ✓
- **[4]**: `end_char_idx=0 == next_idx(1,2,...)`는 **영원히 성립 안 됨** → 루프 내 방출 불가
- **[6]**: post-loop에서 방출 → 모든 텍스트 **뒤**에 fieldEnd 나옴 ✗

### 결함 2: 0-length mid-text 필드 (`start=N, end=N, N>0`) → 순서 역전

예) `text="ABCDE"`, `start=3, end=3`:
- `char_offsets` = [0, 1, 2, **19**, 20] (3번 문자 앞에 16cu 갭)
- **[4]** idx=2 (문자 C), next_idx=3, end_char_idx=3 → **매칭! fieldEnd가 C 뒤에 방출**
- **[1]** idx=3 (문자 D), char_pos=19 ≥ expected+8 → **fieldBegin 방출**
- 결과: `ABC` **fieldEnd** fieldBegin `DE` → fieldEnd가 fieldBegin 앞에! ✗

### 필요한 수정 요약

| 케이스 | 기대 출력 | 현재 오동작 |
|--------|----------|-----------|
| start=0, end=0 | `<fieldBegin/><fieldEnd/>` text | `<fieldBegin/>` text `<fieldEnd/>` |
| start=N, end=N | text[0..N] `<fieldBegin/><fieldEnd/>` text[N..] | text[0..N] `<fieldEnd/>` `<fieldBegin/>` text[N..] |

---

## 구현 단계

### Stage 1 — fieldEnd 방출 로직 수정 (section.rs)

**변경 위치**: `render_run_content` 함수 (section.rs 209~327)

#### 변경 1: 루프 내 — 문자 push 전에 0-length fieldEnd 방출

슬롯 방출([1]) 직후, `text_buf.push(c)` 직전에 다음 검사 추가:

```rust
// [NEW] 0-length 필드: start == end == idx → fieldBegin 방출 직후 fieldEnd 방출
for (i, fr) in para.field_ranges.iter().enumerate() {
    if fr.start_char_idx == fr.end_char_idx
        && fr.end_char_idx == idx
        && !field_end_emitted[i]
    {
        flush_text_fragment(&mut out, &mut text_buf, &para.tab_extended, &mut tab_idx);
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
```

#### 변경 2: 기존 post-char fieldEnd 검사에 guard 추가

기존 [4] 검사에 0-length 필드를 건너뛰는 조건 추가:

```rust
// 기존: fr.end_char_idx == next_idx
// 수정: start < end 인 경우에만 (0-length 필드는 변경 1에서 처리)
if fr.end_char_idx == next_idx
    && !field_end_emitted[i]
    && fr.start_char_idx < fr.end_char_idx  // ← 추가
{
    ...
}
```

#### 수정 전후 흐름 비교

**0-length 필드 at 0 (`start=0, end=0`, text="hello"):**

```
수정 전: <fieldBegin/> <hp:t>hello</hp:t> <fieldEnd/>   ✗
수정 후: <fieldBegin/> <fieldEnd/> <hp:t>hello</hp:t>   ✓
```

**0-length mid-text 필드 (`start=3, end=3`, text="ABCDE"):**

```
수정 전: <hp:t>ABC</hp:t> <fieldEnd/> <fieldBegin/> <hp:t>DE</hp:t>   ✗
수정 후: <hp:t>ABC</hp:t> <fieldBegin/> <fieldEnd/> <hp:t>DE</hp:t>   ✓
```

**기존 정상 필드 (start=0, end=5, text="hello"):**

```
수정 전: <fieldBegin/> <hp:t>hello</hp:t> <fieldEnd/>   ✓
수정 후: <fieldBegin/> <hp:t>hello</hp:t> <fieldEnd/>   ✓ (불변)
```

---

### Stage 2 — 단위 테스트 추가 (section.rs)

다음 테스트 케이스를 `section.rs`의 `#[cfg(test)] mod tests`에 추가:

#### 테스트 1: `task1298_zero_length_field_at_para_start`

- `start=0, end=0`, text="hello"
- 검증: fieldBegin이 fieldEnd보다 앞에 위치하고, 두 마커가 텍스트보다 앞에 위치
- `begin_pos < end_pos < text_pos` 순서 검증

#### 테스트 2: `task1298_zero_length_field_mid_text`

- `start=3, end=3`, text="ABCDE", char_offsets=[0,1,2,19,20]
- 검증: `ABC`가 fieldBegin 앞, fieldBegin이 fieldEnd 앞, `DE`가 fieldEnd 뒤
- `text_abc_pos < begin_pos < end_pos < text_de_pos` 순서 검증

#### 테스트 3: 기존 비-0-length 필드 회귀 검증

- `task1289_field_begin_end_roundtrip` (기존) 통과 유지 확인
- 별도 추가 케이스: `start=0, end=3`, text="hello" → 필드 내 텍스트 포함 케이스

---

## 예외 케이스 (이번 범위 외)

**빈 문단(text == "") + 0-length 필드**: 루프 자체가 실행되지 않아 fieldBegin/fieldEnd 모두 post-loop 처리. 현재 post-loop 순서가 fieldEnd → fieldBegin으로 역전되는 기존 결함이 있으나, 이는 별도 이슈로 처리한다.

---

## 검증 방법

```bash
cargo test -p rhwp -- section::tests 2>&1 | grep -E "ok|FAILED"
cargo clippy -- -D warnings
```

---

## 단계별 커밋 계획

| 단계 | 커밋 내용 |
|------|----------|
| Stage 1 | fix: 0-length field range fieldEnd 방출 위치 수정 (#1298) |
| Stage 2 | test: 0-length field range 인터리빙 회귀 테스트 추가 (#1298) |
