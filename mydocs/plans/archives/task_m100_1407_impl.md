# Task M100 #1407 구현계획서 — newNum 슬롯/fieldEnd 우선순위 정정

- 이슈: #1407, 마일스톤 M100, 브랜치 `local/task1407`
- 작성일: 2026-06-14
- 수행계획서: `mydocs/plans/task_m100_1407.md`

> **1단계 구현 정정(2026-06-14)**: 단위 추적 결과 실제 가로채기는 한 idx 앞에서
> 일어났다. fieldEnd 는 post-char(line 525~)에서 이미 방출되나 `expected` 를 +8
> 진행하지 않아 다음 idx 에서 newNum 이 그 갭을 차지했다. 따라서 아래 pre-char 가드
> 대신 **post-char fieldEnd 방출에 `expected += 8`** 1줄로 정정. 상세: stage1 보고서.

## 1. 수정 본체 — `render_runs` 슬롯 루프에 fieldEnd-자리 가드

### 1.1 현재 동작 (`section.rs:431~449`)

```rust
for (idx, c) in para.text.chars().enumerate() {
    let char_pos = para.char_offsets.get(idx)...;
    while slot_idx < slots.len() && char_pos >= expected_utf16_pos.saturating_add(8) {
        // ← 8유닛 갭이 보이면 controls 순서대로 다음 슬롯 무조건 방출
        flush_text_fragment(...);
        splitter.cut_before(expected_utf16_pos);
        render_control_slot(..., slots[slot_idx], ...);
        slot_idx += 1;
        expected_utf16_pos += 8;
    }
    // fieldEnd 방출은 문자 push 뒤 (line 525, post-char) — 슬롯보다 늦음
```

문단 0.14: idx=3, char_pos=27, expected=16. 27 ≥ 24 → slots[2]=NewNumber 방출.
그러나 그 27 자리는 **fieldEnd**(field_range 0..3, control_idx=1) 의 8유닛 갭이다.

### 1.2 수정 — 슬롯 방출 전에 "이 갭이 fieldEnd 자리인지" 판별

슬롯 방출 `while` 조건에, **현재 expected 위치가 미방출 field_range 의 fieldEnd 자리**
이면 슬롯 방출을 보류하는 가드를 추가한다. fieldEnd 자리는 `fr.end_char_idx == idx`
(현재 문자가 필드 범위 직후 첫 문자)이고 `fr.start_char_idx < fr.end_char_idx`
(0-length 아님 — 0-length 는 pre-char 경로가 처리)이며 `!field_end_emitted[i]` 인 경우다.

도우미:

```rust
// 현재 idx 위치에서 닫혀야 할(미방출) fieldEnd 가 있으면 true — 이 8유닛 갭은
// 슬롯이 아니라 fieldEnd 소유이므로 슬롯 방출을 양보한다.
let field_end_due_here = para.field_ranges.iter().enumerate().any(|(i, fr)| {
    fr.end_char_idx == idx
        && fr.start_char_idx < fr.end_char_idx
        && !field_end_emitted[i]
});
while slot_idx < slots.len()
    && char_pos >= expected_utf16_pos.saturating_add(8)
    && !field_end_due_here
{
    ... // 기존 슬롯 방출
}
```

그리고 슬롯 루프 **직후**에, fieldEnd 가 이 위치에서 닫혀야 하면 먼저 방출한다
(현재는 line 525 의 post-char 검사가 문자 push 뒤에 닫지만, newNum 같은 텍스트-끝
슬롯이 그 사이에 끼지 않도록 pre-char 로 끌어올린다):

```rust
// fieldEnd pre-char 방출 — 슬롯 가로채기 방지(#1407). field_range 끝 위치의
// 8유닛 갭은 fieldEnd 소유. 0-length 필드는 별도 pre-char 경로(line 454)가 처리.
for (i, fr) in para.field_ranges.iter().enumerate() {
    if fr.end_char_idx == idx
        && fr.start_char_idx < fr.end_char_idx
        && !field_end_emitted[i]
    {
        flush_text_fragment(...);
        splitter.cut_before(expected_utf16_pos);
        emit_field_end(&mut splitter.content, para, fr.control_idx);
        expected_utf16_pos = expected_utf16_pos.saturating_add(8);
        field_end_emitted[i] = true;
    }
}
```

이렇게 하면:
- idx=3 에서 fieldEnd(27) 가 먼저 방출 → expected 24→ (cut_before 후) → 텍스트 " 기자"
  가 27 부터 정상. NewNumber 는 루프를 못 빠져나가고 **남은 슬롯 일괄 방출**(line 559)
  로 텍스트 끝(54)에 배치 → 원본 XML 동형.

### 1.3 기존 post-char fieldEnd 검사(line 525)와의 정합

pre-char 로 끌어올렸으므로 같은 field 가 중복 방출되면 안 된다. `field_end_emitted[i]`
플래그가 이미 가드 → post-char 검사는 `!field_end_emitted[i]` 로 skip 된다. 기존
0-length(line 454)·루프후(line 551) 경로도 동일 플래그로 안전.

회귀 우려: fieldEnd 와 슬롯이 **같은 위치**일 때 순서. 원본은 fieldEnd 가 먼저(텍스트
중간), 텍스트-끝 슬롯이 나중. pre-char fieldEnd 가 슬롯 가드와 함께 이 순서를 보장.

## 2. 단계별 구현

### 1단계 — 슬롯/fieldEnd 우선순위 정정 + 단위 테스트
- `render_runs` 에 fieldEnd-자리 가드 + pre-char fieldEnd 방출 추가.
- 단위 테스트:
  - `task1407_newnum_after_text_field`: 머리말+필드(텍스트 래핑)+newNum(텍스트 끝)
    문단 → 방출 XML 에서 newNum 이 fieldEnd·텍스트 뒤(텍스트 끝)에 오는지.
  - `task1407_field_end_not_stolen_by_slot`: fieldEnd 갭이 후속 슬롯에 가로채이지
    않는지 (char_offsets 보존).
- `cargo test --lib serializer::hwpx` 그린.

### 2단계 — 전수 검증 + 페이지 수 귀속
- 143E ir-diff `-s 0 -p 14` → 차이 0.
- 143E RT 페이지 수 재측정 (export-svg 페이지 수). 1→2 해소 여부 → 귀속 확정.
  - 해소: 보고서에 동반 해소 기록.
  - 미해소: 별도 원인 — 새 이슈 분리(슬롯 정정만으로 종결 안 함).
- `cargo test --test hwpx_roundtrip_baseline` (B=0) + `--batch` 신규 실패 0.

### 3단계 — 문서 + 보고서
- 트러블슈팅 `hwpx_newnum_slot_after_text.md` 해소 반영 + 체크리스트 완료.
- 매뉴얼 `hwpx_roundtrip_baseline.md` known-limitations 에 #1407 해소 행.
- 단계별 보고서(stage1~3) + 최종 보고서.

## 3. 검증

- 단위 테스트(신규 2건) + `--lib serializer::hwpx` 전체
- ir-diff 143E -s 0 -p 14 → 0
- baseline B=0, `--batch` 신규 실패 0
- CI급: `cargo test --profile release-test --tests` + fmt + clippy

## 4. 위험과 대응

| 위험 | 대응 |
|------|------|
| pre-char fieldEnd 중복 방출 | `field_end_emitted` 플래그 가드 (1.3) |
| 같은 위치 fieldEnd vs 슬롯 순서 회귀 | 단위 테스트 + 전수 배치 + baseline |
| 페이지 수 1→2 별도 원인 | 2단계 재측정 후 별도 이슈 분리 (귀속 명시) |
