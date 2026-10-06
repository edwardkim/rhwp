# task_m100_2988 처리결과 보고서 — HWPX 체크박스 `value` 3상태(UNCHECKED/CHECKED/INDETERMINATE) 라운드트립

- **이슈**: [#2988](https://github.com/edwardkim/rhwp/issues/2988)
- **브랜치**: `task/m100-2961-form-attr` (base `devel`)
- **범위**: `src/parser/hwpx/section.rs`, `src/serializer/hwpx/form.rs`
- **분류**: 결함 수정 (파서·직렬화기 양쪽 2상태 취급 — 하드코딩성 값 대체)

## 1. 문제

HWPX `hp:btn`/`hp:checkBtn`/`hp:radioBtn` 는 모두 OWPML `AbstractButtonObjectType`
(`mydocs/manual/OWPML SCHEMA/ParaList XML schema.xml` 2586~2611행)을 쓰며, 이 타입의
`value` 속성은 다음 3값 열거형이다.

```xml
<xs:attribute name="value">
  <xs:simpleType>
    <xs:restriction base="xs:string">
      <xs:enumeration value="UNCHECKED"/>
      <xs:enumeration value="CHECKED"/>
      <xs:enumeration value="INDETERMINATE"/>
    </xs:restriction>
  </xs:simpleType>
</xs:attribute>
```

`INDETERMINATE`는 `triState="1"` (3상태 허용) 체크박스가 "부분 선택/알 수 없음" 상태일 때
한컴이 실제로 저장하는 값이다. 그런데 파서 `src/parser/hwpx/section.rs` 의
`parse_form_object` 는 이 값을 불리언 2값으로만 읽는다(수정 전):

```rust
b"value" => form.value = if attr_str(&attr) == "CHECKED" { 1 } else { 0 },
```

`"INDETERMINATE"` 는 `"CHECKED"` 문자열과 다르므로 `else` 분기로 떨어져 `0`
(=`UNCHECKED` 와 동일한 정수)으로 저장된다. 직렬화기 `src/serializer/hwpx/form.rs` 의
`write_form` 도 대칭적으로 2값만 다룬다(수정 전):

```rust
let checked = if form.value == 1 { "CHECKED" } else { "UNCHECKED" };
```

## 2. 재현 — 라운드트립 데이터 유실

tri-state 체크박스가 `value="INDETERMINATE"` 로 저장된 HWPX 문서를 rhwp 로 읽고
그대로 다시 HWPX 로 저장하면:

1. 파서: `"INDETERMINATE"` → `form.value = 0` (`"CHECKED"` 가 아니므로 else 분기)
2. 직렬화기: `form.value == 1` 이 거짓이므로 다시 `value="UNCHECKED"` 로 방출

원본이 `INDETERMINATE` 였던 체크박스가 저장 한 번만에 `UNCHECKED` 로 영구 대체된다.
표 pageBreak(#1393)·표 widthRelTo/heightRelTo/protect/numberingType(#2697)·
fieldBegin/fieldEnd fieldid(#2884) 등과 동일한 계열의 "파서/직렬화기가 원본 값의 전체
도메인을 다루지 못해 특정 값이 다른 값으로 조용히 대체되며 유실되는" 라운드트립 버그다.

## 3. 변경

### `src/parser/hwpx/section.rs` (`parse_form_object`)

`value` 속성 매칭을 `match` 로 바꾸고 `"INDETERMINATE"` 분기를 추가해 `2` 로 보존한다.
기존 `"CHECKED"→1`, 그 외(`"UNCHECKED"` 포함)→`0` 매핑은 그대로 유지해 기존 문서의
동작을 바꾸지 않는다.

```rust
b"value" => {
    form.value = match attr_str(&attr).as_str() {
        "CHECKED" => 1,
        "INDETERMINATE" => 2,
        _ => 0,
    }
}
```

### `src/serializer/hwpx/form.rs` (`write_form`)

`checked` 계산을 `match` 로 바꿔 `form.value == 2` 를 `"INDETERMINATE"` 로 되돌린다.
`0`/`1` 은 종전과 동일하게 `"UNCHECKED"`/`"CHECKED"` 로 출력되므로 기존 라운드트립
동작은 그대로다.

```rust
let checked = match form.value {
    1 => "CHECKED",
    2 => "INDETERMINATE",
    _ => "UNCHECKED",
};
```

두 파일 모두 각 함수 내부 몇 줄로 국한되는 최소 변경이며, 나머지 속성(캡션·라디오
그룹명·위치·여백 등)의 보존 로직은 건드리지 않았다.

## 4. 검증

### 4-1. 신규 테스트

`src/serializer/hwpx/form.rs` 기존 `#[cfg(test)] mod tests` 에
`checkbox_emits_indeterminate_value` 추가 — `form.value = 2` 인 `FormObject` 를
직렬화했을 때 `value="INDETERMINATE"` 가 출력되는지 확인한다.

### 4-2. red → green 실증

수정 전 코드(`if form.value == 1 {"CHECKED"} else {"UNCHECKED"}`)로 되돌려
`cargo test --lib checkbox_emits_indeterminate_value` 를 실행:

```
running 1 test
test serializer::hwpx::form::tests::checkbox_emits_indeterminate_value ... FAILED

---- serializer::hwpx::form::tests::checkbox_emits_indeterminate_value stdout ----
thread '...' panicked at src\serializer\hwpx\form.rs:255:9:
<hp:checkBtn caption="" value="UNCHECKED" ... name="CheckBox" ...>

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2522 filtered out
```

수정 복원 후 재실행:

```
running 6 tests
test serializer::hwpx::form::tests::color_inverts_parse_color_str ... ok
test serializer::hwpx::form::tests::checkbox_emits_indeterminate_value ... ok
test serializer::hwpx::form::tests::checkbox_emits_checked_value_and_caption ... ok
test serializer::hwpx::form::tests::edit_emits_text_child ... ok
test serializer::hwpx::form::tests::combobox_emits_list_items_and_selected_value ... ok
test serializer::hwpx::form::tests::nondefault_pos_outmargin_sz_listdisplay_are_emitted_from_props ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2517 filtered out
```

### 4-3. 빌드

`cargo check --lib` — 경고 없이 통과.

### 4-4. 미실행 항목 (투명 고지)

- **전체 회귀(`cargo test --lib` 전체 스위트)·`cargo clippy -- -D warnings`**: 저장소
  규약(`mydocs/manual/codex/docs_and_git_workflow.md`)상 작업지시자 별도 승인 사항이라
  실행하지 않았다. 영향 범위가 `form.rs`/`section.rs` 의 `value` 속성 한 곳으로 국한되고
  해당 모듈 테스트 6건 전부 통과했으므로 별도 회귀 위험은 낮다고 판단했다.
- **한컴 오피스 실물 대조**: `value="INDETERMINATE"` 체크박스를 한컴 오피스에서 실제로
  열어 시각적으로 "부분 선택" 표시가 되는지는 확인하지 않았다. 스키마 열거값과 파서·
  직렬화기의 왕복 일치만 코드로 검증했다.
- **HWP5(`src/serializer/control.rs`) 경로 미변경**: `form.value` 를 그대로
  `Value:int:{}` 로 내보내는 HWP5 `ButtonSet` 직렬화(`control.rs:2669,2689`)는 이번
  수정으로 `2` 값이 자연스럽게 실려 나가지만, HWP5 스펙상 3상태 값의 정확한 int 코드가
  `INDETERMINATE=2` 와 일치하는지는 별도 확인이 필요해 이번 변경 범위(HWPX 파서/
  직렬화기)에서는 다루지 않았다.

## 5. 잔여 (범위 밖)

- HWP5 `ButtonSet.Value:int` 필드의 3상태 정수 코드가 OWPML `INDETERMINATE` 와 정확히
  대응하는지 hwplib/한컴 실파일 기준 검증이 필요하다.
- `src/renderer/{web_canvas,svg,skia/renderer}.rs` 의 `form.value != 0` 렌더링 분기는
  `INDETERMINATE`(2) 도 "선택됨" 취급으로 렌더링한다. 중간 상태를 별도 시각(예: 사각형
  채움)으로 구분하려면 렌더러 쪽 확장이 별도로 필요하다.
