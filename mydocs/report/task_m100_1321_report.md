# Task M100 #1321 — 최종 결과보고서

## 개요

| 항목 | 내용 |
|------|------|
| 이슈 | #1321 |
| 제목 | 빈 문단(text == "") 0-length field fieldBegin/fieldEnd 순서 역전 수정 |
| 브랜치 | `local/task1321` |
| 마일스톤 | M100 (v1.0.0) |
| 관련 이슈 | #1298 (선행 수정) |

## 배경

#1298 구현 계획서에는 "빈 문단(text == "") + 0-length 필드는 별도 이슈로 처리한다"라고 명시되어 있었다. #1298이 merge된 후 해당 케이스가 여전히 미수정임을 확인하고 #1321로 등록하여 수정했다.

## 버그 내용

### 재현 조건

- `para.text == ""` (빈 문단)
- `para.field_ranges`에 `start_char_idx == end_char_idx`인 0-length field 존재

### 원인

`render_run_content`의 pre-char 검사는 `for (idx, c) in para.text.chars().enumerate()` 루프 **내부**에 위치한다. 빈 문단에서는 이 루프가 0회 실행되므로 pre-char 검사가 전혀 실행되지 않는다.

post-loop 처리 순서:
1. 남은 `field_ranges`의 `fieldEnd` 방출 → **fieldEnd 먼저**
2. 남은 slots의 `fieldBegin` 방출 → **fieldBegin 나중**

HWPX 스펙은 `<fieldBegin>` → `<fieldEnd>` 순서를 요구하므로 유효하지 않은 XML이 생성된다.

## 수정 내용

### 변경 파일

`src/serializer/hwpx/section.rs`

### 수정 방식

`field_end_emitted` 초기화 직후, 메인 루프 이전에 pre-loop 빈 문단 블록을 삽입했다:

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

이 블록 실행 후 `slot_idx == slots.len()`, `field_end_emitted[i] == true`가 되어 두 post-loop 블록은 no-op이 된다.

## 검증 결과

### 단위 테스트

신규 테스트 `task1321_zero_length_field_in_empty_paragraph` 추가 — 51/51 통과

```
test serializer::hwpx::section::tests::task1321_zero_length_field_in_empty_paragraph ... ok
test result: ok. 51 passed; 0 failed; 0 ignored
```

### clippy

```
CARGO_INCREMENTAL=0 cargo clippy --lib -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 19s
```

경고 0, 오류 0.

## 커밋 이력

| 커밋 | 내용 |
|------|------|
| `7127a7c9` | fix: 빈 문단 0-length field fieldBegin/fieldEnd 순서 역전 수정 (#1321) |
| `1fc4313d` | docs: Task #1321 Stage 1~3 완료보고서 |

## 완료 기준 충족 여부

| 항목 | 결과 |
|------|------|
| 빈 문단 + 0-length field XML 순서 올바름 | ✓ |
| 단위 테스트 추가 및 통과 | ✓ |
| clippy 경고 없음 | ✓ |
| 기존 테스트 회귀 없음 (51/51) | ✓ |
