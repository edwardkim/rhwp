# 구현 계획서 — Task #110

**이슈**: [#110](https://github.com/edwardkim/rhwp/issues/110)
**타이틀**: HWPX 양식 컨트롤 파싱 미구현 — checkBtn/btn/radioBtn/comboBox/edit IR 처리
**마일스톤**: M100
**작성일**: 2026-04-13
**브랜치**: `local/task110`

---

## 수정 대상

`src/parser/hwpx/section.rs` 단일 파일

---

## 구현 단계

### 1단계: `parse_form_object()` 함수 구현 + run 분기 추가

#### 1-1. `parse_form_object()` 함수 신규 작성

`<hp:run>` Start 이벤트 핸들러에서 호출할 함수. 태그명으로 `FormType`을 결정하고
자식 요소를 순회하여 `FormObject`를 구성한 뒤 `Control::Form`으로 반환한다.

```rust
fn parse_form_object(
    form_type: FormType,
    e: &BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError>
```

**속성 파싱** (`AbstractFormObjectType` 공통):
- `name` → `form.name`
- `caption` → `form.caption` (btn/checkBtn/radioBtn)
- `foreColor` → `parse_color()` → `form.fore_color`
- `backColor` → `parse_color()` → `form.back_color`
- `enabled` → `parse_bool()` → `form.enabled`
- `value` → `"CHECKED"` = 1, 그 외 = 0 → `form.value`
- `selectedValue` → `form.text` (comboBox)

**자식 요소 순회**:
- `<hp:sz width="..." height="..."/>` (Empty) → `form.width`, `form.height`
- `<hp:listItem value="..."/>` (Empty, comboBox) → `form.properties`에 누적
- `<hp:text>` (Start+End, edit) → 텍스트 읽어 `form.text`
- `<hp:formCharPr .../>` (Empty) → skip
- 기타 자식 → `skip_element()` 또는 End 태그까지 소비

#### 1-2. `<hp:run>` Start 이벤트 분기에 5개 태그 추가

```rust
// src/parser/hwpx/section.rs, run 파싱 Start 이벤트 match 분기 (line ~228 _ => {} 앞)
b"btn" => {
    let ctrl = parse_form_object(FormType::PushButton, ce, reader)?;
    text_parts.push("\u{0002}".to_string());
    para.controls.push(ctrl);
}
b"checkBtn" => {
    let ctrl = parse_form_object(FormType::CheckBox, ce, reader)?;
    text_parts.push("\u{0002}".to_string());
    para.controls.push(ctrl);
}
b"radioBtn" => {
    let ctrl = parse_form_object(FormType::RadioButton, ce, reader)?;
    text_parts.push("\u{0002}".to_string());
    para.controls.push(ctrl);
}
b"comboBox" => {
    let ctrl = parse_form_object(FormType::ComboBox, ce, reader)?;
    text_parts.push("\u{0002}".to_string());
    para.controls.push(ctrl);
}
b"edit" => {
    let ctrl = parse_form_object(FormType::Edit, ce, reader)?;
    text_parts.push("\u{0002}".to_string());
    para.controls.push(ctrl);
}
```

---

### 2단계: 빌드 + SVG 검증

#### 2-1. 빌드

```bash
cargo build --release --bin rhwp
```

#### 2-2. SVG 내보내기 및 시각 확인

```bash
cargo run --release --bin rhwp -- export-svg samples/hwpx/form-002.hwpx -p 0
```

`output/form-002_001.svg`에서 체크박스가 렌더링되는지 확인한다.

#### 2-3. IR 덤프로 파싱 결과 확인

```bash
cargo run --release --bin rhwp -- dump samples/hwpx/form-002.hwpx
```

`Form(CheckBox)` 컨트롤이 문단에 포함되어 있는지 확인한다.

#### 2-4. `cargo test` 회귀 확인

기존 테스트 전체 통과 여부 확인.

---

## 예상 diff 규모

- `parse_form_object()` 함수 신규: ~60줄
- run 분기 추가: ~20줄
- **합계**: ~80줄 순증

---

## 승인 요청

위 구현 계획서를 검토 후 승인해주시면 1단계 구현을 시작하겠습니다.
