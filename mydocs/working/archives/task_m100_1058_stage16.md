# Task #1058 Stage 16 보고서 — Round 5 신규 각주 본문 입력 contract 정합

- 이슈: [#1058](https://github.com/edwardkim/rhwp/issues/1058) (reopen Round 5)
- 단계: Stage 16
- 일시: 2026-05-22

## 1. 배경

Stage 5~15 (Round 1~4) 머지 후 작업지시자 추가 보고:

> "saved/222footnote-01.hwp footnote-01.hwp 파일을 rhwp-studio 에서 열고,
>  새로운 각주를 추가한 다음 저장. 한컴에디터에서는 각주영역에 2)가 안보임.
>  rhwp-studio 에서 열면 2) 가 보임."

→ 한컴에서 각주 번호 "2)" 미표시. **rhwp-studio 정상 ≠ 한컴 호환** —
`feedback_self_verification_not_hancom` 모범 사례.

## 2. 본질 식별 — Stage 4-pivot 패턴 재적용

### 2.1 정답지 vs 저장본 record-level diff

`examples/dump_footnote_inner.rs` 신규 작성 — 정답지/저장본 inner_para
contract 비교:

| 항목 | 정답지 (samples/footnote-01.hwp) | 저장본 (saved/111footnote-01.hwp 신규 각주) |
|------|--------------------------------|-------------------------------------------|
| text | "  플라스틱 액체란" | "  기술이란?  " |
| char_offsets | [0, 8, 9, 10, 11, 12, 13, 14, 15, 16] | **[0, 1, 2, 3, 4, 5, 6, 7, 8]** ← jump 누락 |
| char_count | 18 | 17 |
| style_id | 11 | 11 ✓ |
| controls | [AutoNumber] | [AutoNumber] ✓ |
| control_mask | 0x40000 | 0x40000 ✓ |

→ Stage 16 정정 영역: **char_offsets 의 AutoNumber 8 byte jump 누락**.

### 2.2 본질

`insert_footnote_native` 의 inner_para contract 가 정답지와 불일치:
- 정답지: `text="  "` (placeholder ×2) + `char_offsets=[0, 8]` (AutoNumber 8 cu)
- 기존 코드: `text=" "` (placeholder ×1) + `char_offsets=[0]`

추가 — `Paragraph::insert_text_at` 의 `inserts_before_inline_control` 분기 +
`control_text_positions` 의 인라인 컨트롤 분기에 `Control::AutoNumber` 누락
→ 사용자 입력 시 AutoNumber 의 8 byte 약속 인식 못함.

추가 — rhwp-studio 의 `Cursor::enterFootnoteMode` 가 caret 초기 위치를
`_fnCharOffset = 0` 으로 설정 → 사용자 입력이 placeholder 자리에 삽입.

## 3. 3 영역 정정

### 3.1 `src/document_core/commands/object_ops.rs::insert_footnote_native`

inner_para contract 를 한컴 정답지 패턴 정합:

```rust
let inner_para = Paragraph {
    text: "  ".to_string(),  // placeholder ×2 (정답지)
    char_count: 10,          // 2 + 8 (AutoNumber)
    char_count_msb: true,
    control_mask: 1u32 << 0x12,
    char_offsets: vec![0, 8],  // AutoNumber 8 cu 차지 jump
    para_shape_id: 0,
    style_id: 11,
    char_shapes: vec![CharShapeRef { start_pos: 0, char_shape_id: default_char_shape_id }],
    controls: vec![Control::AutoNumber(auto_num)],
    line_segs: vec![LineSeg { ... }],
    has_para_text: true,
    ..Default::default()
};
```

### 3.2 `src/model/paragraph.rs`

**`insert_text_at`** — `inserts_before_inline_control` 분기에 `Control::AutoNumber` 추가:

```rust
| Control::Footnote(_)
| Control::Endnote(_)
| Control::AutoNumber(_)  // 신규
```

**`control_text_positions`** — 인라인 컨트롤 분기에 `Control::AutoNumber` 추가
(8 cu 차지 동일):

```rust
| Control::Footnote(_)
| Control::Endnote(_)
| Control::AutoNumber(_)  // 신규
```

### 3.3 `rhwp-studio/src/engine/cursor.ts::enterFootnoteMode`

caret 초기 위치 `_fnCharOffset = 2` (placeholder ×2 뒤, 실제 본문 작성 영역):

```typescript
this._fnCharOffset = 2;  // 기존: 0
```

## 4. 정량 입증

### 4.1 자동 reproduce 결과

`examples/repro_1058_footnote_insert.rs` — rhwp-studio 시나리오 자동 모사:

1. samples/footnote-01.hwp 로드
2. paragraph 5 의 char_offset 4 에 신규 각주 삽입
3. fnCharOffset=2 위치에 "기술이란?" 입력
4. `output/poc/issue_1058/repro_round5.hwp` 저장
5. 재파싱 후 신규 각주 inner_para 검증

```
== 재파싱 신규 footnote ==
  text="  기술이란?"
  char_offsets=[0, 8, 9, 10, 11, 12, 13]
  char_count=15
  text_chars=7
  style_id=11
  controls=1
  control_mask=0x40000
```

→ **정답지 패턴 정확 정합**: AutoNumber 8 cu jump 보존, char_count=15 (7+8).

### 4.2 회귀 가드 10/10 통과

`tests/issue_1058_textbox_list_header.rs`:
- 기존 9 가드 (Stage 5~15) 유지
- **신규: `issue_1058_new_footnote_inner_para_contract`** — inner_para contract 검증

### 4.3 CI 패턴

| 항목 | 결과 |
|------|------|
| cargo test --release --lib | **1323 passed** |
| cargo test --release --tests issue_ | 105 passed / 0 failed |
| cargo clippy --release --lib -D warnings | clean |
| cargo fmt --all --check | clean |

### 4.4 WASM Docker 빌드 + 동기화

- `pkg/rhwp_bg.wasm` 4.91 MB
- `rhwp-studio/public/` (rhwp.js + rhwp_bg.wasm + rhwp.d.ts + rhwp_bg.wasm.d.ts) 동기화

### 4.5 작업지시자 한컴 한글 2020 시각 판정 Round 5 통과

| 시나리오 | 결과 |
|---------|------|
| `output/poc/issue_1058/repro_round5.hwp` 한컴에서 열기 | ✓ |
| rhwp-studio 신규 각주 추가 + 입력 + 저장 → 한컴 "2)" 표시 | ✓ |

작업지시자 보고: **"동작 테스트 통과입니다"**.

## 5. 메모리 룰 정합

- ✅ `feedback_visual_judgment_authority` — Round 5 시각 판정 게이트 추가 통과
- ✅ `feedback_diagnosis_layer_attribution` — 정답지 vs 저장본 char_offsets diff 로 본질 정확 식별
- ✅ `feedback_self_verification_not_hancom` — rhwp-studio 정상 ≠ 한컴 호환 재입증
- ✅ `feedback_hancom_compat_specific_over_general` — AutoNumber inline 컨트롤 contract case-specific
- ✅ `feedback_push_full_test_required` — lib + tests + clippy + fmt 모두 통과
- ✅ `project_hwpx_to_hwp_adapter_limit` 정합 + **단순 어댑터 한계 점진 돌파**

## 6. `hwpx2hwp-rule.md` contract unit 추가 (Round 5)

Round 1~4 (Stage 5~15) 의 4 contract unit 에 더해:

- **신규 각주 inner_para placeholder ×2 + char_offsets=[0, 8]** (AutoNumber 8 cu 차지)
- **insert_text_at 의 AutoNumber inline 컨트롤 8 byte 약속 인식**
- **enterFootnoteMode caret 초기 위치 = 2** (placeholder 뒤 본문 작성 영역)

작업지시자 통찰 Stage 4-pivot 의 결정성 — 본 Round 5 에서도 정답지 vs
저장본 record-level diff 분석으로 본질 정확 식별 완료.

후속: 본 task #1058 완전 종결 (1~5 라운드 누적 정정 + 5 라운드 시각 판정 통과).
