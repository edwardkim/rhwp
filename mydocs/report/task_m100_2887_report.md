# 완료 보고서 — Task M100-2887

- 이슈: #2887
- 제목: `hp:pic` `dropcapstyle` 라운드트립 유실 — 파서 미인식으로 저장 시 무조건 None
- 작성일: 2026-07-22
- 브랜치: `task/m100-2887-pic-dropcapstyle`

## 1. 완료 내용

`<hp:pic>`(그림 개체) 를 감싼 문단의 `dropcapstyle`(드롭캡 표시 방식:
`None`/`DoubleLine`/`TripleLine`/`Margin`, OWPML Core 스키마 `DropCapStyleType`)
속성이 파서에서 아예 읽히지 않아, 직렬화기가 항상 `dropcapstyle="None"`을
하드코딩 방출하던 버그를 고쳤다. `lock`(개체 잠금) 라운드트립 버그
(#2840 수식, #2855 표)와 동일한 유형("파서 미인식 속성 → 직렬화기 상수
하드코딩")이며, 이번 PR은 `hp:pic` 1건으로 범위를 좁혔다.

## 2. 근거 (수정 전)

- `src/serializer/hwpx/picture.rs::write_picture()` — `("dropcapstyle", "None")`
  리터럴 고정 방출.
- `src/parser/hwpx/section.rs::parse_picture()` — `<hp:pic>` 속성 루프
  (`id`/`zOrder`/`textWrap`/`textFlow`/`instid`/`href`/`groupLevel`)에
  `dropcapstyle` 분기 없음.
- `src/model/shape.rs::CommonObjAttr` — 드롭캡 상태를 보존할 필드 자체가
  없었음.

## 3. 주요 변경

- `src/model/shape.rs`
  - `CommonObjAttr`에 `drop_cap_style: DropCapStyle` 필드 추가.
  - `DropCapStyle` enum 추가 (`None`/`DoubleLine`/`TripleLine`/`Margin`,
    OWPML `DropCapStyleType` 그대로 매핑).
- `src/parser/hwpx/section.rs`
  - `parse_picture()`의 `<hp:pic>` 속성 루프에 `b"dropcapstyle"` 분기 추가
    — `DoubleLine`/`TripleLine`/`Margin`/그 외(`None`)로 파싱.
- `src/serializer/hwpx/shape.rs`
  - `drop_cap_style_str()` 헬퍼 추가 (기존 `numbering_type_str()`과 동형).
- `src/serializer/hwpx/picture.rs`
  - `write_picture()`가 하드코딩 `"None"` 대신
    `super::shape::drop_cap_style_str(pic.common.drop_cap_style)` 방출.
- `src/document_core/converters/common_obj_attr_writer.rs`
  - 테스트 헬퍼 `make_sample()`의 `CommonObjAttr` 리터럴 초기화에
    `drop_cap_style: DropCapStyle::None` 필드 추가 (신규 필드로 인한
    컴파일 에러 해소, 동작 변경 없음).

## 4. 테스트 (red → green)

`src/serializer/hwpx/picture.rs::tests::dropcapstyle_round_trips_instead_of_always_none`

- Red (수정 전): `pic.common.drop_cap_style = DropCapStyle::TripleLine`로
  직렬화해도 출력이 `dropcapstyle="None"`으로 고정돼 실패.
- Green (수정 후): 출력에 `dropcapstyle="TripleLine"` 포함, 통과.

## 5. 검증 결과

통과:

- `cargo build --lib`
- `cargo test --lib dropcapstyle_round_trips_instead_of_always_none`
  (1 passed)
- `cargo clippy --all-targets --profile release-test -- -D warnings`
- `rustfmt --edition 2021` (변경 파일만)

## 6. 범위 제한 / 후속 과제

`dropcapstyle="None"` 하드코딩은 `hp:rect`/`hp:line`/`hp:tbl`/`hp:equation`
등 다른 개체 직렬화기에도 동일하게 있으나, 이번 PR은 `hp:pic` 단독으로
범위를 좁혔다. 나머지 개체 종류로의 확장은 후속 이슈로 남긴다.

또한 `origin/devel` 현재 상태를 `git log -S`·`git grep`으로 확인한 결과
`CommonObjAttr.locked` 필드는 아직 병합되지 않았고(#2840, #2855 모두
devel 미반영), `hp:tbl`의 `lock`도 여전히 `bool01(false)` 상수 방출
중이다. 따라서 `lock` 필드와 충돌 없이 독립적인 `dropcapstyle` 속성을
선택했다.
