# Task #1050 Stage 4-pivot 보고서 — HWP IR contract 정합 (작업지시자 통찰)

- 이슈: [#1050](https://github.com/edwardkim/rhwp/issues/1050)
- 단계: Stage 4-pivot (작업지시자 통찰 — HWP 코드 참조)
- 일시: 2026-05-21

## 1. 작업지시자 핵심 통찰

> "HWP 에서는 편집기에서 각주를 새로 생성하거나 삭제하고 저장하면 정상적으로
> 한컴편집기에서 동작합니다. HWPX 를 이 코드를 참조해서 처리하면 어떨까요?"

→ **HWP → IR → HWP 라운드트립이 한컴 호환을 이미 보장**. HWPX → IR 만 HWP → IR 과
동일하게 만들면 자동으로 한컴 호환.

## 2. HWP IR vs HWPX IR 전수 비교 결과

`tests/scratch_diff_hwp_hwpx_ir.rs` 임시 진단 테스트로 두 IR 의 정확한 차이 추출:

### 2.1 FootnoteShape (SectionDef) — Stage 4a 정정 후 16 필드 모두 정합 ✓

### 2.2 본문 paragraph (p3 "사람들은" + footnote control) — 추가 정정 후 완전 정합 ✓

| 항목 | HWP 정답지 | Stage 4a 후 HWPX | 정정 후 |
|------|----------|------------------|---------|
| char_offsets | [0, 1, 10, 11] | [0, 1, 2, 3] | **[0, 1, 10, 11]** ✓ |
| char_count | 13 | 5 | **13** ✓ |
| control_mask | 131072 | 131072 | 131072 |

### 2.3 각주 안 paragraph (text "  글상자 내부 각주") — 추가 정정 후 완전 정합 ✓

| 항목 | HWP 정답지 | Stage 4a 후 HWPX | 정정 후 |
|------|----------|------------------|---------|
| text | "  글상자 내부 각주" | " 글상자 내부 각주" (1 공백) | **"  글상자 내부 각주"** ✓ |
| char_count | 19 | 12 | **19** ✓ |
| char_offsets | [0, 8, 9, 10, ..., 17] | [0, 1, 2, ..., 10] | **[0, 8, 9, 10, ..., 17]** ✓ |
| control_mask | 262144 | 0 | 0 (serializer 자동 재계산) |

## 3. HWP PARA_TEXT contract 규명 — hwplib 권위 참조

HWP `body_text.rs:326` 의 `parse_para_text` 처리:

```rust
if ch == 0x0012 {  // AUTO_NUMBER inline 컨트롤
    char_offsets.push(code_unit_pos);  // 현재 pos 추가 (예: 0)
    text.push(' ');                    // text 에 placeholder 공백
    char_count += 1;
}
pos += 16;  // 8 code unit (16 byte) 점프 → 다음 pos = 8
```

→ **AUTO_NUMBER 의 정확한 contract**:
- text 의 한 char (placeholder space) 차지
- char_offsets 에 한 자리 push
- pos 8 점프

본 sample 의 footnote 안 PARA_TEXT (UTF-16 code units):
```
[0..7] AUTO_NUMBER 컨트롤 8 cu (0x12 + 'on','ta' + 0,0,0,0 + 0x12)
[8]    placeholder space (0x0020)
[9..]  본문 ' 글상자 내부 각주'
[18]   문단 끝 (0x000d)
```

## 4. 정정 — 2 영역

### 4.1 `src/parser/hwpx/section.rs`

**1) `parse_paragraph` 의 inline ctrl 분기**:
```rust
b"autoNum" => {
    let ctrl = parse_ctrl_autonum(ce, reader)?;
    controls.push(ctrl);
    // [Task #1050] AUTO_NUMBER (0x12) — HWP PARA_TEXT 정합:
    //   char_offsets.push(pos) + text.push(' ') + pos += 8 (16 byte)
    text_parts.push("\u{0012}".to_string());  // ← Empty 분기에도 동일
}
```

**2) `visual_text` 조립 로직**:
```rust
"\u{0012}" => {
    // [Task #1050] AUTO_NUMBER — HWP PARA_TEXT 정합
    char_offsets.push(utf16_pos);
    visual_text.push(' ');
    utf16_pos += 8;
}
```

### 4.2 `src/serializer/body_text.rs::serialize_para_text`

**AutoNumber placeholder 검출 분기**:
```rust
let is_autonum_placeholder = *ch == ' '
    && offset == prev_end
    && ctrl_idx < para.controls.len()
    && matches!(
        control_char_code_and_id(&para.controls[ctrl_idx]).0,
        0x0011 | 0x0012  // FOOTNOTE | AUTO_NUMBER
    )
    && next_offset.map_or(false, |n| n >= offset + 8);
if is_autonum_placeholder {
    // placeholder ' ' 대신 컨트롤 8 cu 작성
    let (ctrl_code, ctrl_id) = control_char_code_and_id(&para.controls[ctrl_idx]);
    push_extended_ctrl(&mut code_units, ctrl_code, ctrl_id);
    ctrl_idx += 1;
    prev_end = offset + 8;
    continue;
}
```

## 5. 정량 입증 — HWP 정답지 PARA_TEXT byte 정합

3 케이스 hex 완전 동일:

```
ORACLE:        12006f6e746100000000000000001200200000aec1c090c72000b4b080bd...
HWP-roundtrip: 12006f6e746100000000000000001200200000aec1c090c72000b4b080bd...
HWPX→HWP-v5:   12006f6e746100000000000000001200200000aec1c090c72000b4b080bd...
```

## 6. "HWP 도 잘못된 순서로 저장하고 있었나?" 답변 (작업지시자 질문)

**아니요. HWP 처리는 본래 정확합니다**. HWP→HWP 라운드트립의 PARA_TEXT 는 정답지 byte 정합.

HWP parser/serializer 는 같은 약속 (AUTO_NUMBER 의 placeholder space + jump 8) 을
이해해서 동작. 본 결함은 **HWPX parser 가 다른 약속 (autoNum → single space 또는
\u{0002} marker)** 으로 IR 을 만들어 HWP serializer 가 잘못된 PARA_TEXT 생성.

## 7. 다음 단계

Stage 5 작업지시자 한컴 재검증 → 통과 시 Stage 6 회귀 가드 + merge.
