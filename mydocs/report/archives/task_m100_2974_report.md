# 완료 보고서 — Task M100-2974

- 이슈: #2974
- 제목: hp:composeText(글자겹치기) 본문이 CDATA로 인코딩된 경우 파서가 겹침 텍스트를 소실함
- 작성일: 2026-07-22
- 브랜치: `task/m100-2974-compose-cdata-loss`

## 1. 문제 요약

`src/parser/hwpx/section.rs`의 `read_compose_text`(글자겹치기 `hp:compose`의
`composeText` 본문을 읽는 헬퍼)가 `Event::Text`와 `Event::GeneralRef`만 처리하고
`Event::CData`를 처리하지 않았다. `quick_xml` 리더는 `<![CDATA[...]]>` 섹션을
`Event::Text`가 아닌 별도의 `Event::CData` 이벤트로 전달하므로, `<composeText>`
본문이 CDATA로 인코딩된 경우(겹침 문자에 `<`, `>`, `&` 등 XML 특수문자가 포함되어
저장기가 CDATA로 이스케이프한 경우) 겹침 텍스트가 빈 문자열로 소실되던 결함이다.

이는 이 코드베이스에서 이미 세 차례 발견·수정된 것과 동일한 결함 클래스다.

- #2916 — `hp:script` 본문 CDATA 파싱 누락
- #2935 — `stringParam` 본문 CDATA 파싱 누락
- #2951 — `hp:dutmal`의 `mainText`/`subText` CDATA 파싱 누락 (PR #2966)

`read_compose_text`는 위 세 지점과 동일한 "텍스트 본문을 직접 읽는 소형 리더 루프"
패턴이며, CDATA 분기만 아직 누락되어 있었다.

## 2. 재현 (수정 전, red)

```rust
let xml = r#"...
<hp:compose circleType="CHAR" charSz="100" composeType="OVERLAP">
  <composeText><![CDATA[a<b]]></composeText>
</hp:compose>..."#;
let section = parse_hwpx_section(xml).unwrap();
// Control::CharOverlap(co).chars 가 빈 Vec — "a<b" 세 글자가 전부 소실됨
```

수정 전에는 `co.chars`가 빈 벡터가 되어 테스트가 실패했다(red).

## 3. 수정 내용

`read_compose_text`의 `match` 문에 `Event::CData` 분기를 추가해
`String::from_utf8_lossy`로 디코딩한 뒤 누적하도록 했다. #2916/#2935/#2951과
완전히 동일한 패턴을 그대로 적용했다.

- `src/parser/hwpx/section.rs`
  - `read_compose_text`에 `Ok(Event::CData(ref cdata))` 분기 6줄 추가
  - 회귀 테스트 `compose_text_preserve_cdata` 추가 (CDATA로 인코딩된 `a<b`가
    `['a', '<', 'b']`로 정확히 파싱되는지 검증)

## 4. 검증 결과 (수정 후, green)

통과:

- `cargo check --lib`
- `cargo test --lib compose_text_preserve_cdata`
  - 1 passed
- `rustfmt --edition 2021 src/parser/hwpx/section.rs`

## 5. 리스크

- 변경 범위가 `read_compose_text` 내부 CDATA 분기 추가로 한정되어 있어 기존
  Text/GeneralRef 경로에는 영향이 없다.
- 동일 클래스의 CDATA 누락 지점이 코드베이스 다른 곳에 더 있을 수 있으나, 본
  이슈의 범위는 `composeText`로 한정한다.

## 6. 결론

Task M100-2974 구현과 검증을 완료했다. 이슈를 close할 수 있다.
