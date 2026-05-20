# Task M100-852 Stage 2.5 완료 보고서 — JavaScript 일관 직렬화

- 이슈: [#852](https://github.com/edwardkim/rhwp/issues/852)
- 브랜치: `local/task852` (기반 commit: a93c0cf7 Stage 2.4)
- 구현 계획서: `mydocs/plans/task_m100_852_stage25.md` (승인)
- 작업지시자 관찰 (2026-05-20 Stage 2.4 후): "form-01.hwp 에 자바스크립트가 포함되지 않은 것으로 보임"

## 1. 변경 요약

Stage 2.4 의 5/5 한컴 성공 후 잔존한 "JavaScript 미포함" 문제 해소. 정답지 (samples/form-01.hwp) reverse engineering 결과 기반:
1. **Scripts/DefaultJScript** = HWPX `headerScripts` (var 선언) + `sourceScripts` (함수) **결합** + **raw deflate** + length-prefix
2. **BodyText 의 `%clk` (ClickHere field)** = `Control::Field` 직렬화에서 ctrl_id/properties/extra_properties/instance_id 정확 채움
3. **BodyText 의 0x57 (CTRL_DATA `myMsg01`)** = ClickHere field 의 CTRL_DATA 자식 레코드 자동 합성

### 1.1 코드 변경 3 개 파일

- `src/parser/hwpx/contract_streams.rs` — Scripts/DefaultJScript 구조 (header+u32+source+trail) + raw_deflate 헬퍼
- `src/parser/hwpx/section.rs` — Field parser 가 ctrl_id/field_id/properties/ctrl_data_name 채움 (`<hp:fieldBegin>` 의 attribute 완전 수집)
- `src/serializer/control.rs` — Field ClickHere 의 instance_id 정답지 패턴 (form_order_counter 기반) + CTRL_DATA 자동 합성

## 2. 정답지 reverse engineering

### 2.1 Scripts/DefaultJScript 구조 (1580 bytes uncompressed raw deflate)

```
0..4        u32 LE = headerScripts wchar count (430)
4..864      headerScripts (UTF-16 LE, 860 bytes) — HWPX 와 byte-identical
864..868    u32 LE = sourceScripts wchar count (350)
868..1568   sourceScripts (UTF-16 LE, 700 bytes) — HWPX 와 byte-identical
1568..1580  trailing 12 bytes: 8 bytes zero + 4 bytes 0xFFFFFFFF
```

이전 Stage 2.1 의 `zlib_deflate(sourceScripts)` 는 sourceScripts 만 zlib 압축 → headerScripts 누락 → JS 실행 불능 (var Documents/PushButton/CheckBox/... 변수 부재).

### 2.2 %clk CTRL_HEADER 구조 (151 bytes)

```
0..4    ctrl_id "%clk" (= tags::FIELD_CLICKHERE)
4..8    properties = 0x00000001 (bit 0 = editable in form)
8..9    extra_properties = 0x09
9..11   u16 LE wchar_count
11..143 UTF-16 LE 속성 문자열 ("Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:  ")
143..147 instance_id (u32) — 정답지: form 마지막 instance_id + 1 = 0x7dcd59db
147..151 memo_index (u32) = 0
```

이전 변환 결과는 모든 4 필드 (ctrl_id/properties/extra_properties/instance_id) 가 0 으로 작성 → 한컴이 무효 필드로 해석 → JS 핸들러 reference 끊김.

### 2.3 0x57 CTRL_DATA 구조 (26 bytes for "myMsg01")

```
0..2    0x021b (HWP5 CTRL_DATA magic)
2..6    0x00000001
6..8    0x4000 (HWP5 CTRL_DATA flag)
8..10   0x0001
10..12  u16 LE wchar_count (7)
12..26  UTF-16 LE 필드 이름 ("myMsg01")
```

이전엔 ClickHere field 의 CTRL_DATA 자식 레코드 자체 미생성.

## 3. byte-level 정합

### 3.1 form-01.hwp / form-02.hwp — 7/7 byte-perfect

| 레코드 | 정답지 | 변환 | 상태 |
|--------|--------|------|------|
| [13] PushButton FORM_OBJECT (756) | ✅ | ✅ | byte-perfect |
| [22] CheckBox FORM_OBJECT (838) | ✅ | ✅ | byte-perfect |
| [31] ComboBox FORM_OBJECT (858/850) | ✅ | ✅ | byte-perfect |
| [40] RadioButton FORM_OBJECT (900) | ✅ | ✅ | byte-perfect |
| [49] Edit FORM_OBJECT (1024) | ✅ | ✅ | byte-perfect |
| [57] %clk CTRL_HEADER (151) | ✅ | ✅ | **NEW** byte-perfect |
| [58] 0x57 CTRL_DATA (26) | ✅ | ✅ | **NEW** byte-perfect |

### 3.2 Scripts/DefaultJScript

| fixture | 정답지 raw | 변환 raw | identical |
|---------|-----------|----------|-----------|
| form-01 | 1580 bytes | 1580 bytes | ✅ |
| form-02 | 2388 bytes | 2388 bytes | ✅ |

압축된 길이는 정답지 vs 변환이 약간 다름 (406 vs 390 등) — deflate 알고리즘 차이지만 uncompressed 내용 byte-identical.

### 3.3 전체 fixture (회귀 가드)

| fixture | 결과 | CFB 스트림 | 비고 |
|---------|------|------------|------|
| form-01.hwp (14KB) | ✅ | 9 | 7/7 byte-perfect + JS 결합 |
| form-02.hwp (14KB) | ✅ | 9 | 7/7 byte-perfect + JS 결합 |
| form-002.hwp (106KB) | ✅ | 9 | 10페이지 대형 |
| tbox-v-flow-01.hwp (19KB) | ✅ | 9 | HWPX 글상자 (Form 무관) |
| hy-001-rt.hwp (141KB) | ✅ | 12 | HWP→HWP roundtrip 회귀 없음 |

## 4. CI 패턴 검증

| 항목 | 결과 |
|------|------|
| `cargo test --release --lib` | **1309 passed, 0 failed** |
| `cargo test --release --tests` | **모든 통합 테스트 passed, 0 failed** |
| `cargo fmt --all -- --check` | clean (적용 후) |
| `cargo clippy --all-targets --release` | 본 PR 변경 0 warnings (`wasm_api/tests.rs` 56 기존 warnings 본 PR 무관) |

## 5. 다음 단계 — 작업지시자 한컴 재판정 (`feedback_visual_judgment_authority`)

| 파일 | 경로 | Stage 2.4 판정 | 재판정 |
|------|------|----------------|--------|
| form-01.hwp | [output/poc/task852/stage25/form-01.hwp](output/poc/task852/stage25/form-01.hwp) | 성공 (JS 없음 관찰) | ? (JS 포함 확인) |
| form-02.hwp | [output/poc/task852/stage25/form-02.hwp](output/poc/task852/stage25/form-02.hwp) | 성공 | ? (JS 포함 확인) |
| form-002.hwp | [output/poc/task852/stage25/form-002.hwp](output/poc/task852/stage25/form-002.hwp) | 성공 | ? |
| tbox-v-flow-01.hwp | [output/poc/task852/stage25/tbox-v-flow-01.hwp](output/poc/task852/stage25/tbox-v-flow-01.hwp) | 성공 | ? |
| hy-001-rt.hwp | [output/poc/task852/stage25/hy-001-rt.hwp](output/poc/task852/stage25/hy-001-rt.hwp) | 성공 | ? |

핵심 검증: **form-01.hwp 의 PushButton 클릭 또는 ComboBox 동작 시 JavaScript 핸들러 (OnDocument_Open / OnComboBox_Click / ComboBox 초기화 코드) 가 정상 동작**.

## 6. 메모리 룰 정합

- ✅ `feedback_self_verification_not_hancom` — Stage 2.5.5 한컴 게이트 대기
- ✅ `feedback_diagnosis_layer_attribution` — Stage 2.1 zlib(sourceScripts) → 정답지 raw deflate(header+source) 차이 정확 식별
- ✅ `feedback_hancom_compat_specific_over_general` — Form/ClickHere 정공법 직렬화 (일반화 회피)
- ✅ `feedback_push_full_test_required` — cargo test + clippy + fmt CI 패턴 통과
- ✅ `reference_authoritative_hancom` — samples/form-01.hwp / form-02.hwp 정답지 baseline byte-level 정합
- ✅ `feedback_fix_scope_check_two_paths` — Field 직렬화 (HWP→HWP 정상) vs HWPX→HWP (parser ctrl_id 누락) 두 경로 식별

## 7. 잔존

- form-01 Section0 정답지 6277 vs 변환 6259 (18 bytes 차이) — SECD CTRL_HEADER (28 vs 38 bytes) + 첫 PARA_TEXT (16 vs 8 bytes). Form 본질과 무관, Stage 3 회귀 가드에는 영향 없음
- 다른 FieldType (Date / DocDate / MailMerge 등) 의 정답지 byte-level 정합은 본 task 범위 외 — ClickHere 만 검증
