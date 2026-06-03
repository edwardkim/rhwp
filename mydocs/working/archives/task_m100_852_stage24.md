# Task M100-852 Stage 2.4 완료 보고서 — Form 컨트롤 BodyText 직렬화

- 이슈: [#852](https://github.com/edwardkim/rhwp/issues/852)
- 브랜치: `local/task852`
- 구현 계획서: `mydocs/plans/task_m100_852_stage24.md` (승인)
- 일시: 2026-05-20

## 1. 변경 요약

HWPX → HWP 변환 시 Form 컨트롤이 BodyText/Section0 에 사실상 미작성되어 한컴 손상 판정되던 문제를 정답지 (samples/form-01.hwp) byte-level reverse engineering 결과 기반으로 정공법 직렬화 구현.

### 1.1 코드 변경 4 개 파일

- `src/serializer/control.rs` — Form 분기 분리 + `serialize_form_control` + `build_form_prop_str` + 5 헬퍼 + thread_local form_order 카운터
- `src/serializer/body_text.rs` — `serialize_section` 진입 시 form_order 카운터 reset
- `src/parser/hwpx/section.rs` — HWPX Form 자식 요소 보존 11 속성 추가 (listBoxRows / autoSz / borderTypeIDRef 등)
- `src/parser/hwpx/contract_streams.rs` — (Stage 2.1+2.2 기존, 본 stage 무변경)

### 1.2 직렬화 정합

| Form | 정답지 size | 변환 size | byte-perfect |
|------|-------------|-----------|--------------|
| [13] PushButton  | 756 | 756 | ✅ |
| [22] CheckBox    | 838 | 838 | ✅ |
| [31] ComboBox    | 858 (form-01) / 850 (form-02) | 동일 | ✅ |
| [40] RadioButton | 900 | 900 | ✅ |
| [49] Edit        | 1024 | 1024 | ✅ |

**form-01.hwp / form-02.hwp 모두 5/5 Form 레코드 byte-level 정합**.

## 2. 핵심 진단 정정 (Stage 1 후속)

이슈 본문 가설 "Stage 40 = `Control::Form` 직렬화 + `CTRL_HEADER(form)` + `HWPTAG_FORM_OBJECT` 추가" 는 미머지/미구현 상태였음 (`mydocs/working/task_m100_841_stage40.md` 부재). 본 stage 가 처음으로 완전 구현.

### 2.1 정확한 근본 누락 위치

`src/serializer/control.rs:115` 의 "미구현 컨트롤" 분기:
```rust
Control::Hyperlink(_) | Control::Ruby(_) | Control::Form(_) | Control::Unknown(_) => {
    let ctrl_id = match ctrl {
        Control::Unknown(u) => u.ctrl_id,
        _ => 0,  // ← Form 은 0 진입
    };
    if ctrl_id != 0 {  // ← 0 이므로 CTRL_HEADER 미생성
```

Form 이 ctrl_id=0 으로 진입 → CTRL_HEADER 마저 미작성. 정답지 BodyText/Section0 의 약 4376 bytes (5×FORM_OBJECT) + 230 bytes (5×CTRL_HEADER 46) 가 변환 결과에서 완전 누락.

### 2.2 정답지 reverse engineering — CTRL_HEADER 구조 (46 bytes)

```
0..4:   ctrl_id "form" (LE: "mrof")
4..8:   attr = 0x002a6211 (HWP5 common ctrl property flag, 정답지 고정값)
8..12:  y_offset = 0 (i32)
12..16: x_offset = 0 (i32)
16..20: width  (u32, HWPUNIT)
20..24: height (u32, HWPUNIT)
24..28: order  (u32, z-order 0..N-1)
28..36: zero (8 bytes)
36..40: instance_id (u32, 0x7dcd59d6 + order)
40..46: zero (6 bytes)
```

### 2.3 정답지 reverse engineering — HWPTAG_FORM_OBJECT 구조

```
0..4:   type_id (ASCII "tbp+"/"tbc+"/"boc+"/"tbr+"/"tde+")
4..8:   type_id 중복 (magic marker)
8..12:  wchar_count (u32)
12..14: wchar_count (u16, 8..12 와 동일 값)
14..:   UTF-16 LE 속성 문자열 (wchar_count chars)
```

속성 문자열 포맷:
```
CommonSet:set:{N1}:{common_body} CharShapeSet:set:{N2}:{char_body} {TypeSet}:set:{N3}:{type_body}
```
- `set:N` 의 N 은 **chars 수 (UTF-16 wchar 수)**
- TypeSet 은 form_type 별 ButtonSet / ComboBoxSet / EditSet

## 3. HWPX 속성 보존 강화

HWPX parser 가 직렬화에 필요한 11 속성을 신규 보존 (이전엔 name/caption/foreColor/backColor/enabled/value/selectedValue 만 보존):

- ComboBox: `listBoxRows`, `listBoxWidth`, `editEnable`
- 공통: `groupName`, `tabStop`, `editable`, `tabOrder`, `borderTypeIDRef`, `drawFrame`, `printable`, `command`
- formCharPr: `charPrIDRef`, `followContext`, `autoSz`, `wordWrap`

## 4. 검증

### 4.1 byte-level diff (form-01.hwpx + form-02.hwpx)

```
form-01: 5/5 Form byte-perfect (Section0 정답지 6277 vs 변환 6229, 차이 48 bytes = SECD CTRL_HEADER + %clk 매크로 + sosns PARA_TEXT — 본 task 범위 외)
form-02: 5/5 Form byte-perfect (Section0 정답지 6269 vs 변환 6219)
```

### 4.2 회귀 부재 (전체 fixture 4 개)

| 파일 | 변환 결과 | CFB 스트림 | 비고 |
|------|-----------|------------|------|
| form-01.hwp (13KB) | ✅ | 9 | Form 5 byte-perfect |
| form-02.hwp (13KB) | ✅ | 9 | Form 5 byte-perfect |
| form-002.hwp (106KB) | ✅ | 9 | 10 페이지 대형 fixture |
| tbox-v-flow-01.hwp (19KB) | ✅ | 9 | HWPX 글상자 (form 무관) |
| hy-001-rt.hwp (141KB) | ✅ | 12 | HWP→HWP roundtrip (form 무관) |

### 4.3 CI 패턴 검증

| 항목 | 결과 |
|------|------|
| `cargo test --release --lib` | **1309 passed, 0 failed** |
| `cargo test --release --tests` | **모든 통합 테스트 passed, 0 failed** |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --all-targets --release` | 본 PR 변경 4 파일 0 warnings (기존 `src/wasm_api/tests.rs` 56 warnings 는 본 PR 무관) |

## 5. 잔존 / 후속

### 5.1 본 task 범위 내 잔존

- **`%clk` (click 매크로) + 0x57 레코드 26 bytes** — Form 컨트롤의 onclick 핸들러 메시지명 (myMsg01 등). 한컴 손상 영향 여부 Stage 2.4.3 한컴 재판정 후 결정
- **SECD CTRL_HEADER 10 bytes 차이** — Section Definition 의 미세 차이, Form 무관

### 5.2 후속 task 후보

- HWPX content.hpf opf:metadata → HwpSummaryInformation 정밀 패치
- ComboBox listItem 다중 항목 지원 (현재 listItem0 만 사용)

## 6. 메모리 룰 정합

- ✅ `feedback_self_verification_not_hancom` — rhwp byte 일치 확인 후 Stage 2.4.3 작업지시자 한컴 게이트 대기
- ✅ `feedback_diagnosis_layer_attribution` — Stage 2.4 분석에서 control.rs:115 의 ctrl_id=0 분기 정확 식별
- ✅ `feedback_hancom_compat_specific_over_general` — Form 5 타입 명시적 분기 (PushButton/CheckBox/RadioButton/ComboBox/Edit 각각 별도)
- ✅ `feedback_push_full_test_required` — cargo test --tests + clippy + fmt 전체 CI 패턴 통과
- ✅ `reference_authoritative_hancom` — samples/form-01.hwp / form-02.hwp 정답지 baseline
- ✅ `feedback_fix_scope_check_two_paths` — HWP roundtrip (정상, hy-001-rt 12 스트림) vs HWPX 변환 (5 Form 추가) 두 경로 분리 확인

## 7. 다음 단계

**Stage 2.4.3** — 작업지시자 한컴 에디터 재판정 (`feedback_visual_judgment_authority`):

| 파일 | 경로 | 기존 판정 | 재판정 |
|------|------|-----------|--------|
| form-01.hwp | output/poc/task852/stage24/form-01.hwp | 손상 | ? |
| form-02.hwp | output/poc/task852/stage24/form-02.hwp | 손상 | ? |
| form-002.hwp | output/poc/task852/stage24/form-002.hwp | 손상 | ? |
| tbox-v-flow-01.hwp | output/poc/task852/stage24/tbox-v-flow-01.hwp | 성공 | ? (회귀 확인) |
| hy-001-rt.hwp | output/poc/task852/stage24/hy-001-rt.hwp | 성공 | ? (회귀 확인) |
