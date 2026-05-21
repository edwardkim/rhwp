# Task #1050 최종 보고서 — [hwpx2hwp] 각주 저장 기능 구현

- 이슈: [#1050](https://github.com/edwardkim/rhwp/issues/1050) (closes)
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1050`
- assignee: @edwardkim
- 일시: 2026-05-21
- 수행 계획서: [task_m100_1050.md](../plans/archives/task_m100_1050.md)
- 구현 계획서: [task_m100_1050_impl.md](../plans/archives/task_m100_1050_impl.md)
- 단계별: [stage1](../working/task_m100_1050_stage1.md) / [stage2](../working/task_m100_1050_stage2.md) / [stage3](../working/task_m100_1050_stage3.md) / [stage4-pivot](../working/task_m100_1050_stage4_pivot.md) / [stage5](../working/task_m100_1050_stage5.md)

## 1. 이슈 본질

작업지시자 요청 (이슈 본문):
> hwpx 의 각주 를 hwp 로 저장하는 기능 구현. hwp 에서는 이미 각주 시리얼라이제이션이 구현되어 있습니다. hwpx 도 hwp 로 저장시 문제 없이 저장되도록 해야 합니다.

핵심 결함:
- HWPX → HWP 저장 시 한컴 한글 편집기에서 각주가 **본문 인라인 텍스트로 인식** (각주 영역으로 조판 안 됨)
- 자기 라운드트립 시 글상자 안 각주 본문 누락 (Stage 0 진단)
- 새 각주 추가 시 한컴에서 본문 다단계 목록 (1.1.1.1.1.) 으로 표시

## 2. 작업지시자 핵심 통찰 (Stage 4-pivot)

> "HWP 에서는 편집기에서 각주를 새로 생성하거나 삭제하고 저장하면 정상적으로 한컴편집기에서 동작합니다. HWPX 를 이 코드를 참조해서 처리하면 어떨까요?"

→ **HWP → IR → HWP 라운드트립이 한컴 호환을 이미 보장**. HWPX → IR 만 HWP → IR 과
동일한 IR 을 생성하면 자동으로 한컴 호환.

이 통찰이 결정적 — Stage 1~3 의 부분 정정 (CTRL_FOOTNOTE size + FootnoteShape) 만으로는
한컴 조판 미흡. Stage 4-pivot 에서 HWP IR vs HWPX IR 전수 비교 → AUTO_NUMBER inline
컨트롤의 PARA_TEXT contract 누락 식별 → 양쪽 IR 완전 정합.

## 3. 변경 사항 (5 영역)

### 3.1 `src/model/footnote.rs` (+10 라인)

Footnote / Endnote 구조체에 4 필드 추가 (`hwplib::CtrlHeaderFootnote` 정합):
- `before_decoration_letter: u16`
- `after_decoration_letter: u16` (default `0x0029` = `')'`)
- `number_shape: u32`
- `instance_id: u32`
- `list_header_property: u32`

### 3.2 `src/parser/control.rs` (+30 라인)

`parse_footnote_control` / `parse_endnote_control`:
- `number` u16 → u32 (UInt4) 변경
- CTRL_FOOTNOTE 16 byte payload 점진 파싱 (4/8/12/16 byte)
- `find_list_header_property_for_footnote_endnote` 신규 — LIST_HEADER property 추출

### 3.3 `src/serializer/control.rs` (+35 라인)

`serialize_footnote` / `serialize_endnote`:
- CTRL_FOOTNOTE 16 byte payload (record 헤더 제외, total size=20)
- after_decoration_letter == 0 → default `0x0029 ')'`

`serialize_footnote_endnote_list_header` 신규 — LIST_HEADER 16 byte 형식
(paraCount + property + 8 byte zero padding)

### 3.4 `src/parser/hwpx/section.rs` (+170 라인)

**`parse_ctrl_footnote` / `parse_ctrl_endnote`** (Task #1050 Stage 2):
- HWPX `suffixChar` → `after_decoration_letter`
- HWPX `instId` → `instance_id`

**`parse_sec_pr_children`** (Stage 3.5 / 4a):
- `b"footNotePr"` / `b"endNotePr"` 분기 추가

**`parse_note_pr_children`** 신규 헬퍼:
- `<autoNumFormat>` suffix/prefix/userChar → FootnoteShape
- `<noteLine>` length/type/width/color → separator_*
- `<noteSpacing>` betweenNotes/belowLine/aboveLine → raw_unknown/note_spacing/separator_margin_bottom
- `<numbering>` newNum → start_number

**`parse_paragraph`** inline ctrl 분기 (Stage 4-pivot):
- `b"autoNum"` (Start + Empty 양쪽) → `text_parts.push("\u{0012}")`
- `b"footNote"` / `b"endNote"` → `text_parts.push("\u{0002}")`

**`visual_text` 조립** (Stage 4-pivot):
- `\u{0012}` → `char_offsets.push(pos) + text.push(' ') + utf16_pos += 8`
- (HWP `body_text.rs:326` `parse_para_text` 정합)

### 3.5 `src/serializer/body_text.rs::serialize_para_text` (Stage 4-pivot, +18 라인)

AutoNumber placeholder 검출 분기:
- `text[i] == ' '` + `offset == prev_end` + 다음 offset `>= +8` + controls 에 AUTO_NUMBER/FOOTNOTE
- placeholder space 대신 컨트롤 8 cu 작성 + `prev_end = offset + 8`

## 4. 검증 결과

### 4.1 자동 검증

| 항목 | 결과 |
|------|------|
| cargo build --release --bin rhwp | OK |
| cargo build --lib | OK |
| cargo test --release --lib | **1319 passed** |
| cargo test --release --tests | FAILED 0 (전체 통합) |
| **회귀 가드** `cargo test --release --test issue_1050_footnote_serialize` | **7/7 passed** |
| cargo clippy --release --lib -D warnings | clean |
| cargo fmt --all --check | clean |
| WASM Docker 빌드 | OK (`pkg/rhwp_bg.wasm` 4.91 MB) |
| rhwp-studio 동기화 | OK |

### 4.2 광범위 sweep (10 fixtures, 149 SVG)

| Fixture | 페이지 수 | BEFORE/AFTER diff |
|---------|----------|------|
| samples/hwpx/footnote-tbox-01.hwpx | 1 | **1** (의도) |
| samples/footnote-tbox-01.hwp | 1 | 0 |
| samples/hwpx/footnote-01.hwpx | 6 | **3** (의도) |
| samples/footnote-01.hwp | 6 | 0 |
| samples/2010-01-06.hwp | 6 | 0 |
| samples/table-in-tbox.hwp | 2 | 0 |
| samples/aift.hwp | 74 | 0 |
| samples/KTX.hwp | 27 | 0 |
| samples/biz_plan.hwp | 6 | 0 |
| samples/exam_kor.hwp | 20 | 0 |

→ **HWPX 출처 2 fixture (의도된 본질 정정) 만 변동, HWP 출처 8 fixture 회귀 부재**.

### 4.3 정량 입증 — HWP 정답지 PARA_TEXT byte 정합

3 케이스 hex 완전 동일 (footnote 안 paragraph PARA_TEXT):
```
ORACLE:        12006f6e746100000000000000001200200000aec1c090c72000b4b080bd...
HWP-roundtrip: 12006f6e746100000000000000001200200000aec1c090c72000b4b080bd...
HWPX→HWP:      12006f6e746100000000000000001200200000aec1c090c72000b4b080bd...
```

### 4.4 작업지시자 한컴 한글 2020 시각 판정 — 2 sample 통과

| Sample | 판정 |
|--------|------|
| `samples/hwpx/footnote-tbox-01.hwpx` | ✓ "이제 잘 열리고 정확하게 한컴편집기에서 조판됩니다." |
| `samples/hwpx/footnote-01.hwpx` | ✓ "성공입니다." |

- 각주 영역 정상 조판 (separator + "N) 본문")
- 각주 추가/삭제 동작 정상

## 5. 성공 기준 충족

| 기준 | 내용 | 결과 |
|------|------|------|
| C1 | HWPX → HWP 저장 후 한컴 한글 정상 조판 | ✓ |
| C2 | 본 환경 자기 라운드트립 정합 | ✓ |
| C3 | HWP 출처 회귀 부재 | ✓ (sweep 8 fixture diff=0) |
| C4 | 회귀 가드 영구화 | ✓ tests/issue_1050_footnote_serialize.rs (7 tests) |
| C5 | 광범위 sweep 회귀 부재 | ✓ |
| C6 | 자동 검증 통과 | ✓ |
| C7 | hwp5-inventory-diff footnote 영역 정합 | ✓ |
| C8 | **작업지시자 한컴 시각 판정 통과 (2 sample)** | ✓ |

## 6. 메모리 룰 정합

- ✅ `feedback_search_troubleshootings_first` — `hwpx2hwp-rule.md` + `task178_*` 사전 정독
- ✅ `feedback_self_verification_not_hancom` — 자기 라운드트립 통과 ≠ 한컴 호환 입증 (Stage 0 시각 판정 부정합 → Stage 4-pivot 정정 후 통과)
- ✅ `feedback_visual_judgment_authority` — 작업지시자 한컴 한글 2020 시각 판정 게이트 통과
- ✅ `feedback_assign_issue_before_work` — 이슈 등록 직후 assignee 지정
- ✅ `feedback_process_must_follow` — 이슈 → 브랜치 → 수행 계획서 → 구현 계획서 → 단계별 → 보고서
- ✅ `feedback_diagnosis_layer_attribution` — Stage 1 contract 규명 (`hwplib` 권위) + Stage 4-pivot 본질 위치 (HWPX parser AUTO_NUMBER) 정확 식별
- ✅ `feedback_push_full_test_required` — lib + tests + clippy + fmt 모두 통과
- ✅ `project_hwpx_to_hwp_adapter_limit` 정합 — Task #178 의 광범위 한컴 거부 영역과 달리 각주 영역 한정 부분 해결 가능. **작업지시자 통찰 (HWP IR 정합) 이 단순 어댑터 한계 돌파**

## 7. 의의 — `hwpx2hwp-rule.md` 의 실제 적용

본 task 는 `mydocs/troubleshootings/hwpx2hwp-rule.md` 의 핵심 원칙 ("HWPX → HWP lowering
contract 를 한컴 oracle 정합 단위로 축적") 의 실제 contract unit 추가:

- **CTRL_FOOTNOTE 16 byte payload** (number/before/after/numberShape/instanceId)
- **footnote LIST_HEADER 16 byte 형식** (paraCount/property/8 byte padding)
- **FootnoteShape 16 필드** (separator_*, note_spacing, raw_unknown 등)
- **AUTO_NUMBER inline 컨트롤 PARA_TEXT contract** (8 cu + placeholder + jump 8)
- **HWPX `<hp:footNotePr>` ↔ HWP FOOTNOTE_SHAPE** 매핑

## 8. 잔여 / 후속

- FOOTNOTE_SHAPE tuple=2 (endnote shape) payload hash 1건 — footnote 외 영역. endnote 의 noteLine length/type/width 매핑 추가 시 정합 가능 (별도 issue 권고).
- 미주 (Endnote) 의 한컴 시각 검증 — 본 task 에서 Footnote 와 동일 contract 적용. Endnote 보유 sample 확보 시 후속 검증.
- glob 차원 광범위 HWPX → HWP 호환 (Task #178 영역) — 본 task 범위 외.
