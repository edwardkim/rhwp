# Task M100-852 Stage 2.4 구현 계획서 — Form 컨트롤 BodyText 직렬화

- 이슈: [#852](https://github.com/edwardkim/rhwp/issues/852)
- 브랜치: `local/task852`
- 기반 보고서: `mydocs/working/task_m100_852_stage1.md` (승인), `task_m100_852_impl.md` (승인)
- 한컴 판정 (작업지시자, 2026-05-20):
  - form-01.hwp / form-02.hwp / form-002.hwp → **파일손상 판정**
  - tbox-v-flow-01.hwp / hy-001.hwp / hy-001-rt.hwp → 성공
  - 결론: Stage 2.1+2.2 (contract 9 스트림) 만으로는 Form 보유 HWPX 한컴 호환 불충분. **Form 직렬화 본질 결함 잔존**.

## 1. Root cause 정밀 식별

### 1.1 정답지 분석 — form-01.hwp BodyText/Section0 의 Form 5 개

`samples/form-01.hwp` 의 BodyText/Section0 (1620 → 6277 bytes uncompressed raw deflate) 의 65 레코드 중 5 개 Form:

| idx | tag | size | ctrl_id | width | height | order | instance_id |
|-----|-----|------|---------|-------|--------|-------|-------------|
| [12] | 0x47=CTRL_HEADER | 46 | "form" | 7087 | 1984 | 0 | 0x7dcd59d6 |
| [13] | 0x5b=FORM_OBJECT | 756 | tbp+ (PushButton) | — | — | — | wchars=371 |
| [21] | 0x47 | 46 | "form" | 9921 | 1984 | 1 | 0x7dcd59d7 |
| [22] | 0x5b | 838 | tbc+ (CheckBox) | — | — | — | wchars=412 |
| [30] | 0x47 | 46 | "form" | 6058 | 1450 | 2 | 0x7dcd59d8 |
| [31] | 0x5b | 858 | boc+ (ComboBox) | — | — | — | wchars=422 |
| [39] | 0x47 | 46 | "form" | 8504 | 1984 | 3 | 0x7dcd59d9 |
| [40] | 0x5b | 900 | tbr+ (RadioButton) | — | — | — | wchars=443 |
| [48] | 0x47 | 46 | "form" | 7087 | 1984 | 4 | 0x7dcd59da |
| [49] | 0x5b | 1024 | tde+ (Edit) | — | — | — | wchars=505 |

총 **Form 페이로드 ≈ 5306 bytes** (5×CTRL_HEADER 230 + 5×FORM_OBJECT 4376 + 레코드 헤더 60).

### 1.2 변환 결과 분석 — form-01 (rhwp 변환)

변환 결과 Section0 (418 → 1583 bytes uncompressed) 의 54 레코드 중 **Form 0 개**. CTRL_HEADER 마저 미작성.

### 1.3 코드 갭 (`src/serializer/control.rs:115-130`)

```rust
// 미구현 컨트롤은 최소한의 CTRL_HEADER만 생성
Control::Hyperlink(_) | Control::Ruby(_) | Control::Form(_) | Control::Unknown(_) => {
    let ctrl_id = match ctrl {
        Control::Unknown(u) => u.ctrl_id,
        _ => 0,  // ← Form 은 0 진입
    };
    if ctrl_id != 0 {  // ← 0 이므로 분기 미진입
        // CTRL_HEADER 작성
    }
    // HWPTAG_FORM_OBJECT 자식 미작성
}
```

Form 은 ctrl_id=0 으로 진입 → CTRL_HEADER 레코드조차 미작성. Stage 40 가설은 미머지/미구현.

## 2. CTRL_HEADER "form" 구조 (46 bytes, reverse engineering 확정)

```
0..4:   ctrl_id "form" (LE bytes: 6d 72 6f 66)
4..8:   attr = 0x002a6211 (HWP5 common ctrl property flag, 모든 form 동일)
8..12:  y_offset = 0 (i32)
12..16: x_offset = 0 (i32)
16..20: width  (u32, HWPUNIT)
20..24: height (u32, HWPUNIT)
24..28: order  (u32, z-order/탭 순서 0..N-1)
28..36: zero (8 bytes)
36..40: instance_id (u32, 0x7dcd59d6 + order — Form 별 고유 ID)
40..46: zero (6 bytes)
```

## 3. HWPTAG_FORM_OBJECT 구조 (가변, reverse engineering 확정)

```
0..4:   type_id (4 ASCII bytes: "tbp+"/"tbc+"/"boc+"/"tbr+"/"tde+")
4..8:   type_id 중복 (동일 4 bytes — magic marker)
8..12:  wchar_count (u32, 다음 UTF-16 문자열 길이 WCHAR 단위)
12..14: wchar_count (u16, 8..12 와 동일 값)
14..14+wchar_count*2: UTF-16 LE 속성 문자열
```

레코드 size = 14 + wchar_count\*2 (정확). trailing 데이터 없음.

### 3.1 속성 문자열 포맷 (parser 가 이미 디코딩, 역방향)

```
CommonSet:set:N1:                       — 공통 set (N1 bytes)
  Name:wstring:K:VALUE                  — 이름
  ForeColor:int:V                       — 글자색 (0xBBGGRR)
  BackColor:int:V                       — 배경색
  GroupName:wstring:K:VALUE             — 그룹명
  TabStop:bool:V                        — 탭 정지
  TabOrder:int:V                        — 탭 순서
  Enabled:bool:V                        — 활성화
  BorderType:int:V                      — 테두리 타입
  DrawFrame:bool:V                      — 테두리 그리기
  Command:wstring:K:VALUE               — 명령
  Editable:bool:V                       — 편집 가능
  Printable:bool:V                      — 인쇄 가능
 CharShapeSet:set:N2:                   — 글자 모양 set
  CharShapeID:int:V
  FollowContext:bool:V
  AutoSize:bool:V
  WordWrap:bool:V
 (타입별):set:N3:                       — ButtonSet/ComboBoxSet/EditSet 등
  ...
```

핵심: **`CommonSet:set:N` 의 N 은 자식 set 내부의 byte 길이** — 직렬화 시 계산 필요. set 들은 공백 두 개로 종결 (실제 정답지에서 확인).

## 4. 구현 전략 — 옵션 A 정공법 + D 정답지 참조

### 4.1 `src/serializer/control.rs:115` 분기 분리

Form 을 미구현 분기에서 빼내 별도 함수 `serialize_form_control(form, level, records)` 추가.

### 4.2 신규 함수 `serialize_form_control`

```rust
fn serialize_form_control(form: &FormObject, order: u32, level: u16, records: &mut Vec<Record>) {
    // 1. CTRL_HEADER (46 bytes)
    let mut hdr = Vec::with_capacity(46);
    hdr.extend_from_slice(b"mrof");           // "form" LE
    hdr.extend_from_slice(&0x002a_6211u32.to_le_bytes());
    hdr.extend_from_slice(&0i32.to_le_bytes());  // y_offset
    hdr.extend_from_slice(&0i32.to_le_bytes());  // x_offset
    hdr.extend_from_slice(&form.width.to_le_bytes());
    hdr.extend_from_slice(&form.height.to_le_bytes());
    hdr.extend_from_slice(&order.to_le_bytes());
    hdr.extend_from_slice(&[0u8; 8]);
    hdr.extend_from_slice(&(0x7dcd59d6 + order).to_le_bytes());
    hdr.extend_from_slice(&[0u8; 6]);
    records.push(Record { tag_id: tags::HWPTAG_CTRL_HEADER, level, size: 46, data: hdr });

    // 2. HWPTAG_FORM_OBJECT 자식 (level+1)
    let type_id = match form.form_type {
        FormType::PushButton  => b"tbp+",
        FormType::CheckBox    => b"tbc+",
        FormType::ComboBox    => b"boc+",
        FormType::RadioButton => b"tbr+",
        FormType::Edit        => b"tde+",
    };
    let prop_str = build_form_prop_str(form, order); // → 정답지 포맷 문자열
    let wchars: Vec<u16> = prop_str.encode_utf16().collect();
    let mut fo = Vec::with_capacity(14 + wchars.len() * 2);
    fo.extend_from_slice(type_id);
    fo.extend_from_slice(type_id);
    fo.extend_from_slice(&(wchars.len() as u32).to_le_bytes());
    fo.extend_from_slice(&(wchars.len() as u16).to_le_bytes());
    for w in &wchars { fo.extend_from_slice(&w.to_le_bytes()); }
    records.push(Record { tag_id: tags::HWPTAG_FORM_OBJECT, level: level + 1, size: fo.len() as u32, data: fo });
}
```

### 4.3 `build_form_prop_str(form, order)` — 속성 문자열 합성

정답지의 정확한 키 순서 + 인덱싱 (CommonSet → CharShapeSet → 타입별 Set):

```rust
fn build_form_prop_str(form: &FormObject, order: u32) -> String {
    // CommonSet 내부
    let common = format!(
        "Name:wstring:{}:{} ForeColor:int:{} BackColor:int:{} GroupName:wstring:0: TabStop:bool:1 TabOrder:int:{} Enabled:bool:{} BorderType:int:{} DrawFrame:bool:1 Command:wstring:0: Editable:bool:1 Printable:bool:1 ",
        form.name.chars().count(), form.name,
        form.fore_color, form.back_color,
        order + 1, // 한컴 정답지가 1-base
        if form.enabled { 1 } else { 0 },
        default_border_type(form.form_type),
    );
    let char_shape = " CharShapeID:int:0 FollowContext:bool:0 AutoSize:bool:0 WordWrap:bool:0 ".to_string();
    let type_set = match form.form_type {
        FormType::PushButton => format!(
            "Caption:wstring:{}:{} ",
            form.caption.chars().count(), form.caption,
        ),
        FormType::CheckBox => format!(
            "Caption:wstring:{}:{} Value:int:{} TriState:bool:0 BackStyle:int:1 ",
            form.caption.chars().count(), form.caption,
            form.value,
        ),
        FormType::ComboBox => format!(
            "ListBoxRows:int:4 Text:wstring:{}:{} ListBoxWidth:int:0 EditEnable:bool:1 ",
            form.text.chars().count(), form.text,
        ),
        FormType::RadioButton => format!(
            "Caption:wstring:{}:{} RadioGroupName:wstring:0: Value:int:{} TriState:bool:0 BackStyle:int:1 ",
            form.caption.chars().count(), form.caption,
            form.value,
        ),
        FormType::Edit => format!(
            "Text:wstring:{}:{} MultiLine:bool:0 PasswordChar:wstring:0: MaxLength:int:2147483647 ScrollBars:int:0 TabKeyBehavior:int:0 Number:bool:0 ReadOnly:bool:0 AlignText:int:0 ",
            form.text.chars().count(), form.text,
        ),
    };
    let type_set_name = match form.form_type {
        FormType::PushButton | FormType::CheckBox | FormType::RadioButton => "ButtonSet",
        FormType::ComboBox => "ComboBoxSet",
        FormType::Edit => "EditSet",
    };
    let common_bytes = common.chars().count();
    let char_shape_bytes = char_shape.chars().count();
    let type_set_bytes = type_set.chars().count();
    format!(
        "CommonSet:set:{}:{} CharShapeSet:set:{}:{} {}:set:{}:{} ",
        common_bytes, common.trim_end(),
        char_shape_bytes, char_shape.trim_end(),
        type_set_name, type_set_bytes, type_set.trim_end(),
    )
}
```

**중요**: `set:N` 의 N 은 정답지에서 **chars 수 (실제 문자 수)** 가 아니라 **byte 수** 일 가능성 — Stage 2.4 구현 시 byte-level diff 로 확인.

### 4.4 order 산정

FormObject 모델에 order 필드 없음. Section paragraph 순회 중 form 등장 순서로 0..N-1 부여 (전역 카운터). 한컴 정답지의 TabOrder 와 일치 (1-base 라 +1).

### 4.5 BorderType 기본값 (한컴 정답지 관찰)

| FormType | BorderType |
|----------|-----------|
| PushButton | 4 |
| CheckBox | 0 |
| RadioButton | 0 |
| ComboBox | 5 |
| Edit | 5 |

이는 `FormObject.properties` 에 키 `BorderType` 으로 보존되어 있으면 우선 사용, 없으면 위 기본값.

## 5. Stage 분해

### Stage 2.4.1 — Serializer 구현 (예상 1.5 시간)

1. `src/serializer/control.rs`:
   - Form 분기 분리 + `serialize_form_control(form, order, level, records)` 추가
   - `build_form_prop_str(form, order)` 헬퍼
   - Form order 카운터: serialize_paragraph 또는 serialize_section 단에서 전역 카운터 유지
2. `cargo test --lib`

### Stage 2.4.2 — Byte-level diff 검증 (예상 1 시간)

1. `rhwp convert samples/hwpx/form-01.hwpx output/poc/task852/stage24/form-01.hwp`
2. BodyText/Section0 raw deflate 디코딩 + 정답지와 record-by-record 비교
3. 각 form 의 CTRL_HEADER 46 bytes / FORM_OBJECT N bytes 1:1 일치 (또는 사소한 차이 식별)
4. Set:N 의 N 산정 (chars vs bytes) 정확화
5. `cargo test --release --tests` + `clippy -D warnings` + `fmt --all`

### Stage 2.4.3 — 한컴 재판정 (작업지시자 게이트)

1. form-01.hwp / form-02.hwp 변환 결과 → 한컴 에디터 손상 미판정 확인
2. **`feedback_self_verification_not_hancom` 정합 — rhwp byte 일치 ≠ 한컴 호환**

## 6. 회귀 위험

| 영역 | 위험도 | 근거 |
|------|--------|------|
| HWP→HWP roundtrip (Form 무관) | **낮음** | parser 와 serializer 모두 Form 분기 분리. 기존 비-Form 경로 무영향 |
| HWPX→HWP 일반 (Form 무관) | **낮음** | Form 없는 HWPX 는 본 함수 미호출 |
| HWPX→HWP Form 보유 (form-01/02) | **의도된 변경** | 5 Form 레코드 추가 ≈ 5306 bytes BodyText 보강 |
| HWP→IR Form parser | **영향 없음** | parser 무변경 |

## 7. 잠재 위험 — 한컴 비공개 필드

- **instance_id 0x7dcd59d6**: 한컴 내부 timestamp/PID 기반. 본 task 에서는 0x7dcd59d6 + order 로 hardcode. 한컴이 이 값 검증할 가능성 낮음 (정답지에서 모두 동일). 위험 시 0 으로 시도.
- **set:N 의 N 단위**: chars vs bytes 의 정확도. Stage 2.4.2 byte-level diff 게이트로 확정.
- **속성 키 누락**: HWPX 가 보존하지 않은 키 (예: BackStyle, AlignText) 의 기본값 추정. 한컴 호환 영향 시 정답지 hardcode.

## 8. 메모리 룰 정합

- `feedback_self_verification_not_hancom` — **핵심**: rhwp byte 일치 검증 ≠ 한컴 호환. 작업지시자 한컴 게이트 필수
- `feedback_diagnosis_layer_attribution` — Stage 2.4 정밀 진단 (control.rs:115 의 ctrl_id=0 분기 갭)
- `feedback_hancom_compat_specific_over_general` — Form 5 타입 명시적 분기 (일반화 회피)
- `feedback_push_full_test_required` — `cargo test --release --tests` + `clippy -D` + `fmt --all`
- `reference_authoritative_hancom` — `samples/form-01.hwp` / `form-02.hwp` 한컴 정답지 baseline
- `feedback_fix_scope_check_two_paths` — HWP roundtrip (정상) vs HWPX 변환 (누락) 두 경로 식별 — HWPX serializer 만 정정

## 9. 후속 잔존

- **`%clk` (click 매크로) + 0x57 레코드 26 bytes** — Form 컨트롤의 onclick 핸들러 메시지명 (myMsg01 등). 본 task 우선순위 후순위, 한컴 손상 영향 여부 Stage 2.4.3 확인 후 결정
- **HwpSummaryInformation 의 form-01.hwp 추출본 vs HWPX content.hpf 메타 패치** — 정밀 정합 후속 task
- **속성 키 순서 정확도** — `properties` HashMap 보존 키의 순서 불보장. 한컴 호환에 영향 시 정답지 hardcode order

## 10. 검증 기준

- form-01.hwpx / form-02.hwpx 변환 결과 BodyText/Section0 에 5 개 Form (CTRL_HEADER + FORM_OBJECT) 레코드 존재
- 각 레코드 size 정답지와 일치 (CTRL_HEADER 46, FORM_OBJECT ±5 bytes 이내)
- **작업지시자 한컴 에디터 손상 미판정** (`feedback_visual_judgment_authority`)
- 기존 회귀 가드 + 신규 회귀 가드 `tests/issue_852_hwpx_to_hwp_form_serialization.rs` 통과
- CI 패턴 (cargo test --release --tests + clippy + fmt --all) 통과
