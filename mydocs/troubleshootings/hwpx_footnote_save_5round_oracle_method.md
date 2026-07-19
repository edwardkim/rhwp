---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# HWPX 각주 저장 — 5 라운드 정답지 oracle 방법론 (Task #1058)

| 항목 | 내용 |
|------|------|
| 발견일 | 2026-05-21 ~ 2026-05-22 |
| 이슈 | [#1058](https://github.com/edwardkim/rhwp/issues/1058) (reopen) |
| 선행 | [#1050](https://github.com/edwardkim/rhwp/issues/1050), [#1052](https://github.com/edwardkim/rhwp/issues/1052) |
| 정답지 | `samples/footnote-01.hwp` (한컴 저장 oracle) |
| 산출물 | `output/poc/issue_1058/repro_round5.hwp` |
| 분석 도구 | `examples/dump_footnote_inner.rs`, `examples/repro_1058_footnote_insert.rs` |
| 관련 | [`hwpx2hwp-rule.md`](hwpx2hwp-rule.md), [`task178_hwpx_to_hwp_first_attempt_failure.md`](task178_hwpx_to_hwp_first_attempt_failure.md) |

## 증상

`samples/hwpx/footnote-01.hwpx` 를 rhwp-studio 에서 열어 HWP 로 저장 후 한컴 한글
2020 에서 열면:

1. (Round 1) 각주 사이에 신규 각주 추가 시 본문 문단번호 "1.1.1.1.1.1" 자동 부여
2. (Round 2) 신규 각주 추가 시 본문 왼쪽 여백 60.0pt + 줄간격 160% 자동 설정
3. (Round 3) ParaShape attr1 강제 set 부작용 — 일반 문단 글머리표 자동 부여
4. (Round 4) HWPX `<hh:bullet>` 파싱 누락 — 한컴 default 글머리표 자동 부여
5. (Round 5) 신규 각주 본문에 입력한 텍스트 저장 후 한컴이 "2)" 표시 안 함

핵심: **rhwp-studio 자기 정합은 모든 라운드에서 정상이지만 한컴 호환 거부**.
`feedback_self_verification_not_hancom` 의 결정적 입증.

## 방법론 — Stage 4-pivot "HWP IR oracle 방식"

### 근본 통찰

```text
HWP → IR → HWP 라운드트립이 한컴 호환을 이미 보장하므로
HWPX → IR 만 HWP → IR 과 동일한 IR 을 생성하면 자동으로 한컴 호환.
```

즉, 한컴 거부의 본질은 **HWPX → IR 변환에서 한컴이 요구하는 record contract
누락 또는 잘못된 값 설정**. 다음 절차로 본질 정확 식별:

1. 작업지시자 한컴 시각 판정 보고 (어느 증상이 어디서 발생)
2. 정답지 HWP (한컴 직접 저장본) vs rhwp 저장본 record-level diff 분석
3. 본질 영역 식별 + 정정
4. 시각 판정 재요청 + 통과 확인

### 정답지 vs 저장본 비교 도구

문제 영역별 비교 도구를 명시적으로 작성한다 (절대 임의 추측 금지):

| 영역 | 도구 |
|------|------|
| Style record raw byte | hexdump + `dump_style_records` |
| ParaShape attr / line_spacing | `rhwp dump -s N -p M` + DocInfo Tag 17/18 byte 비교 |
| PARA_HEADER instance_id | `dump_para_header_raw` (raw_header_extra 추출) |
| BULLET record 개수/내용 | DocInfo Tag 24 (BULLET) count + char 추출 |
| Footnote inner_para contract | `examples/dump_footnote_inner.rs` (본 작업 신규) |

## 5 라운드 정정 영역 누적

### Round 1 (Stage 5~9) — PARA_HEADER instance_id

**본질**: HWPX `<hp:p id="2147483648">` (= 0x80000000) 의 instance_id 매핑 누락.

| 항목 | 정답지 (samples/footnote-01.hwp) | rhwp (정정 전) |
|------|--------------------------------|--------------|
| PARA_HEADER raw_header_extra[6..10] | UINT32 LE = 0x80000000 (MSB set) | 0x00000000 |

**정정**: `src/parser/hwpx/section.rs::parse_paragraph` — HWPX `<hp:p id>` 속성을
`raw_header_extra` offset 6..10 에 UINT32 LE 작성.

**한컴 시각 판정 통과**: "각주 추가 시 1.1.1.1.1.1 부여 사라짐".

### Round 2 (Stage 10~12) — Style record lang_id

**본질**: HWP5 spec 표 47 — Style record 의 `INT16 lang_id` + trailing 2 byte 누락.

| 항목 | 정답지 | rhwp (정정 전) |
|------|--------|--------------|
| Style record size | 32 byte | **28 byte** (4 byte 부족) |

**정정 (4 파일)**:
- `src/model/style.rs` — `Style.lang_id: i16` 필드 추가
- `src/parser/doc_info.rs::parse_style` — lang_id INT16 읽기 + trailing 2 byte 흡수
- `src/serializer/doc_info.rs::serialize_style` — lang_id + trailing UINT16 zero 작성
- `src/parser/hwpx/header.rs::parse_style` — HWPX `langID` 매핑 (default 1042)

추가: `parse_para_shape` 의 `line_spacing_v2` 보정 (5.0.2.5 이상).

**한컴 시각 판정 통과**: "각주 삽입 시 정상 동작".

### Round 3 (Stage 13) — ParaShape attr1 부작용 제거

**본질**: `ps.attr1 |= 0x80` (bit 7) 강제 set 시도가 정답지 회귀 유발.

| 항목 | 정답지 ps[5] | rhwp (정정 전 ps[5]) |
|------|------------|------------------|
| attr1 bit 7 | 0x00000000 | **0x00000080** (강제 set) |

**정정**: `parse_para_shape` 에서 `attr1 |= 0x80` 제거 — HWPX attr1 값 그대로 보존.

**한컴 시각 판정 통과 + 부작용 발견**: "성공이지만 일반 문단 시작에 글머리표 부여".

### Round 4 (Stage 14) — HWPX `<hh:bullet>` 파싱

**본질**: HWPX `<hh:bullets>` 의 `<hh:bullet>` 4개 (char ❏/※/­/❍) 파싱 누락 →
한컴이 default 글머리표 부여.

| 항목 | 정답지 | rhwp (정정 전) |
|------|--------|--------------|
| DocInfo Tag 24 (BULLET) 개수 | 4 | **0** |

**정정**: `src/parser/hwpx/header.rs` — `bullet` Empty event 분기 + 신규
`parse_bullet_hwpx` 헬퍼.

**한컴 시각 판정 통과**: "성공입니다".

### Round 5 (Stage 16) — 신규 각주 본문 입력 contract

**본질**: 신규 각주 inner_para 의 `char_offsets` 가 정답지 패턴과 불일치.

| 항목 | 정답지 (Footnote #0) | rhwp 저장본 신규 각주 |
|------|--------------------|-------------------|
| text | "  플라스틱 액체란" (placeholder ×2 + 본문) | "  기술이란?  " (placeholder ×2 + 본문 + ?) |
| char_offsets | [0, 8, 9, 10, 11, ...] | **[0, 1, 2, 3, ..., 8]** ← jump 누락 |
| char_count | 18 (10 + 8 AutoNumber) | 17 |
| style_id | 11 | 11 ✓ |
| controls | [AutoNumber] | [AutoNumber] ✓ |

**정정 (3 영역)**:

1. `src/document_core/commands/object_ops.rs::insert_footnote_native` —
   inner_para 초기 contract:
   ```rust
   Paragraph {
       text: "  ".to_string(),    // placeholder ×2 (정답지 정합)
       char_count: 10,             // 2 + 8 (AutoNumber inline ctrl)
       char_offsets: vec![0, 8],   // AutoNumber 8 cu 차지 jump
       style_id: 11,
       control_mask: 1u32 << 0x12,
       controls: vec![Control::AutoNumber(auto_num)],
       has_para_text: true,
       ...
   }
   ```

2. `src/model/paragraph.rs::insert_text_at` + `control_text_positions` —
   `Control::AutoNumber` 를 inline 컨트롤 분기에 추가:
   ```rust
   | Control::Footnote(_)
   | Control::Endnote(_)
   | Control::AutoNumber(_)  // 신규 (8 cu 약속)
   ```

3. `rhwp-studio/src/engine/cursor.ts::enterFootnoteMode` — caret 초기 위치
   `_fnCharOffset = 2` (placeholder 뒤, 실제 본문 작성 영역).

**한컴 시각 판정 통과**: "동작 테스트 통과입니다".

## 핵심 학습

### 1. rhwp 자기 정합 ≠ 한컴 호환 (5 라운드 입증)

rhwp-studio 의 자기 라운드트립 (HWP/HWPX 로드 → 편집 → 저장 → 재로드) 이
정상이어도 한컴이 거부할 수 있다. **시각 판정 게이트 없이 호환 주장 금지**.

본 task 의 모든 5 라운드에서 rhwp-studio 정상 / 한컴 거부. 자기 검증의
한계가 결정적으로 입증됨.

### 2. 정답지 oracle 방법론의 결정성

추측이나 spec 단순 참조 대신 **정답지 vs 저장본 raw byte 비교** 가 본질 식별의
유일한 결정적 방법:

- Round 1: PARA_HEADER raw_header_extra byte 비교 → 0x80 vs 0x00
- Round 2: Style record size 비교 → 32 vs 28
- Round 3: ParaShape ps[5] 비교 → 0x00 vs 0x80 (정정의 부작용 발견)
- Round 4: DocInfo Tag 24 count 비교 → 4 vs 0
- Round 5: inner_para char_offsets 비교 → [0, 8, ...] vs [0, 1, ...]

매 라운드 raw byte 비교 없이는 본질 식별 불가능했을 것 (특히 Round 3 의
attr1 부작용은 정정 시도 후 회귀로 발견).

### 3. ParaShape attr1 같은 강제 set 의 위험 (Round 3)

"spec 추정 + 직관" 으로 bit 강제 set 시도가 다른 케이스 회귀를 유발한다.
한컴은 정확한 byte 보존을 요구. **정답지에 없는 값을 강제 set 금지** —
HWPX 의 attr 그대로 보존이 안전.

`feedback_hancom_compat_specific_over_general` 정합.

### 4. inline 컨트롤 contract — 8 cu 약속의 시스템 영향 (Round 5)

AutoNumber 같은 inline 컨트롤은 utf16 8 cu (16 byte) 를 차지하는 약속.
이 약속이 다음 모든 곳에 일관되게 반영되어야 한다:

| 위치 | 처리 |
|------|------|
| inner_para 초기 char_offsets | `[0, 8]` (placeholder ×2 사이 8 cu 점유) |
| `insert_text_at` | 입력 위치가 컨트롤 뒤일 때 char_offsets jump 8 보존 |
| `control_text_positions` | 갭 분석에서 8 cu 인식 |
| serializer | char_offsets 그대로 보존 |
| caret 위치 | 사용자가 컨트롤 자리에 입력 못 하도록 placeholder 뒤로 |

한 곳이라도 누락되면 한컴 거부 (Round 5 의 5 영역 모두 점검 필요).

### 5. 시각 판정 라운드의 누적성

각 라운드가 다음 라운드의 부작용을 노출시키는 패턴 — Round 2 정정으로
Round 1 의 다단계 목록 부작용 해소되었지만, Round 3 시도로 새 부작용 발생.
**한 라운드 통과 = 다음 부작용 발견 시작점**. 모든 라운드 시각 판정 게이트 통과
전까지 task 종료 금지.

## 회귀 가드

`tests/issue_1058_textbox_list_header.rs` 누적 10 가드:

| Test | Round |
|------|-------|
| `issue_1058_textbox_list_header_size_33` | (이전 #1058 1차) |
| `issue_1058_textbox_list_header_byte_contract` | (이전 #1058 1차) |
| `issue_1058_hwp_textbox_roundtrip` | (이전 #1058 1차) |
| `issue_1058_footnote_list_header_size_16_preserved` | (Task #1050 양립) |
| `issue_1058_paragraph_instance_id_mapped` | Round 1 |
| `issue_1058_para_shape_line_spacing_v2_synced` | Round 2 |
| `issue_1058_style_lang_id_preserved` | Round 2 |
| `issue_1058_bullet_records_preserved` | Round 4 |
| `issue_1058_serialized_bullet_count` | Round 4 |
| `issue_1058_new_footnote_inner_para_contract` | Round 5 |

## 후속 작업

- Tag 30 (FORBIDDEN_CHAR) + 31 (COMPATIBLE_DOCUMENT) + 94 (LAYOUT_COMPATIBILITY)
  영역 (별개 task)
- 광범위 HWPX → HWP 호환 (Task #178 영역) — 별도 task
- 본 방법론을 다른 HWPX 출처 fixture 에도 적용 (sweep)

## 관련 commits

- `bc1cd4db` Round 1 — PARA_HEADER instance_id
- `f812abcf` Round 2~4 머지 (Stage 5~15)
- `749048c9` Round 5 머지 (Stage 16)

## 관련 메모리 룰

- `feedback_self_verification_not_hancom` — 5 라운드 결정적 입증
- `feedback_diagnosis_layer_attribution` — 정답지 vs 저장본 raw byte 비교 패턴
- `feedback_hancom_compat_specific_over_general` — attr1 강제 set 회귀 사례
- `feedback_visual_judgment_authority` — 매 라운드 시각 판정 게이트
- `project_hwpx_to_hwp_adapter_limit` — 단순 어댑터 한계 점진 돌파
- `feedback_search_troubleshootings_first` — 본 문서 사전 검색 의무
