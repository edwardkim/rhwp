# #2791 처리 결과 — hp:pic groupLevel 하드코딩 수정

## 문제

`src/serializer/hwpx/picture.rs`의 `write_picture()`에서 `<hp:pic>` 속성 `groupLevel`이
항상 문자열 리터럴 `"0"`으로 하드코딩되어 있었다. 파싱 단계(`src/parser/hwpx/section.rs`)는
`groupLevel` 속성을 읽어 `shape_attr.group_level`(IR)에 정확히 저장하지만, 저장 시 이 IR 값을
무시하고 항상 0을 출력했다. 결과적으로 그룹 해제(ungroup) 후 재저장하거나, 그룹 레벨이 있는
picture를 왕복 저장(round-trip)하면 그룹 레벨 정보가 유실된다.

동일 증상을 보고한 중복 이슈 #2781/#2783/#2789는 오늘 모두 #2791로 정리되어 닫혔다.

## 수정

`src/serializer/hwpx/picture.rs`:
- `write_picture()` 시작부에 `let group_level = pic.shape_attr.group_level.to_string();` 추가
- `start_tag_attrs`의 `("groupLevel", "0")`를 `("groupLevel", &group_level)`로 교체

`src/serializer/hwpx/shape.rs`(도형 직렬화기)는 이미 `sa.group_level.to_string()`을 사용하고
있어 이번 수정으로 picture.rs가 shape.rs와 동일한 패턴으로 통일됐다.

## 테스트 (red → green)

`src/serializer/hwpx/picture.rs`의 `mod tests`에 추가:

```rust
#[test]
fn issue2791_group_level_uses_shape_attr_not_hardcoded_zero() {
    let doc = make_doc_with_bin(1, "png");
    let mut ctx = SerializeContext::collect_from_document(&doc);
    let mut pic = make_picture(1);
    pic.shape_attr.group_level = 2;
    let xml = serialize(&pic, &mut ctx);
    assert!(
        xml.contains(r#"groupLevel="2""#),
        "groupLevel 은 shape_attr.group_level 을 반영해야 한다(하드코딩 0 아님): {xml}"
    );
}
```

수정 전 코드(`("groupLevel", "0")` 하드코딩) 상태에서 이 테스트를 실행하면 출력이 항상
`groupLevel="0"`이므로 `assert!` 실패(red) — 수정 후 `groupLevel="2"`가 출력되어 통과(green)를
로컬에서 직접 확인했다. (단, 빌드 도중 로컬 링커(dbghelp.lib) 손상으로 재확인 빌드가 실패해
디스크/툴체인 이슈로 재검증 실행은 `cargo check --lib` 통과로 대체했다 — 최초 1회 `cargo test --lib
issue2791_group_level` 실행에서는 정상적으로 `test ... ok` 를 확인함.)

## 검증 (디스크 제약으로 경량 검증만 수행)

- `cargo test --lib issue2791_group_level` — 최초 1회 실행: `test result: ok. 1 passed`
- `cargo check --lib` — 통과 (fmt 적용 후 재확인)
- `cargo build --lib`, 전체 `cargo test`, `cargo clippy --profile release-test`는 디스크
  공간 제약(~23GB 여유)으로 스킵함
- `rustfmt --edition 2021 src/serializer/hwpx/picture.rs` 적용, `git diff --name-only`로
  포맷팅 후 변경 파일이 의도한 1개 파일(`picture.rs`)로 유지됨을 확인

## 변경 파일

- `src/serializer/hwpx/picture.rs`
