# Task #2989 Report — hp:edit passwordChar 스키마 기본값 하드코딩 수정

## 요약

이슈 #2989 에서 지적한 대로, `src/serializer/hwpx/form.rs` 의 `write_form()` 이
`<hp:edit>`(양식 개체 편집 상자)를 직렬화할 때 `passwordChar` 속성의 폴백 기본값을
빈 문자열(`""`)로 하드코딩하고 있었다. OWPML 공식 스키마
(`mydocs/manual/OWPML SCHEMA/ParaList XML schema.xml:2632`, `EditType.passwordChar`)는
`default="*"` 를 명시하므로, 원본 문서에 `passwordChar` 속성이 없던 `<hp:edit>` 컨트롤을
저장할 때 마스킹 문자 `*` 대신 빈 문자열이 방출되어 저장 후 재로딩 시 마스킹 문자가
소실되는 결과를 낳고 있었다.

같은 `EditType` 의 다른 모든 속성(`multiLine`/`scrollBars`/`tabKeyBehavior`/`numOnly`/
`readOnly`/`alignText`)은 이미 스키마 default 값을 정확히 폴백으로 쓰고 있어, `passwordChar`
하나만 어긋난 실수로 판단했다.

## 원인

- 파서(`src/parser/hwpx/section.rs:5573-5576`): `<hp:edit passwordChar="...">` 속성이
  XML 에 실제로 존재할 때만 `FormObject.properties["PasswordChar"]` 를 채운다. 속성이
  없으면 이 키 자체가 생기지 않는다(스펙상 정상 — 생략 시 스키마 기본값 적용을 기대).
- 직렬화기(`src/serializer/hwpx/form.rs:112`, 수정 전):
  ```rust
  attrs.push(("passwordChar", prop(form, "PasswordChar", "")));
  ```
  `properties` 에 키가 없을 때(신규 생성 폼, 또는 속성이 생략된 원본) `""` 로 폴백해
  스키마 기본값 `"*"` 와 어긋난 값을 명시적으로 출력했다.

## 수정

`src/serializer/hwpx/form.rs:112` 한 줄만 변경했다.

```rust
// 변경 전
attrs.push(("passwordChar", prop(form, "PasswordChar", "")));
// 변경 후
attrs.push(("passwordChar", prop(form, "PasswordChar", "*")));
```

`passwordChar` 속성이 원본에 명시적으로 있던 경우는 `properties` 에 값이 이미 채워져
있으므로 이번 수정과 무관하게 그대로 보존된다(라운드트립 영향 없음). 영향을 받는 대상은
(1) 원본에 `passwordChar` 속성이 없던 `<hp:edit>` 컨트롤, (2) rhwp API 로 새로 생성한
`FormObject::Edit` 뿐이다.

## 검증 (red → green)

`src/serializer/hwpx/form.rs` 의 `tests` 모듈에 회귀 테스트를 추가했다.

```rust
#[test]
fn task3001_edit_password_char_defaults_to_asterisk() {
    let form = FormObject {
        form_type: FormType::Edit,
        name: "Edit".to_string(),
        enabled: true,
        ..Default::default()
    };
    let xml = to_string(|w| write_form(w, &form));
    assert!(xml.contains(r#"passwordChar="*""#), "{xml}");
}
```

- 수정 전(red): `properties` 가 비어 있으므로 `prop(form, "PasswordChar", "")` 폴백이
  적용되어 `passwordChar=""` 가 출력되고, 위 단언이 실패한다.
- 수정 후(green): 폴백이 `"*"` 로 바뀌어 단언이 통과한다.

```bash
cargo test --lib task3001_edit_password_char_defaults_to_asterisk
# running 1 test
# test serializer::hwpx::form::tests::task3001_edit_password_char_defaults_to_asterisk ... ok
cargo check --lib
# Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## 범위

`src/serializer/hwpx/form.rs` 1개 파일만 수정(로직 1줄 + 테스트 12줄). 다른 파일은
건드리지 않았다.
