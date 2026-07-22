# Task m100-2931: HWPX hp:chart/hp:ole lock(개체 잠금) 속성 왕복 유실 해소

## 이슈

https://github.com/edwardkim/rhwp/issues/2931

## 근본 원인

`src/parser/hwpx/section.rs`의 `parse_hp_chart_element`(`<hp:chart>` 전용)와
`parse_hp_ole_element`(`<hp:ole>` 폴백 전용)는 요소 최상위 속성을 개별 `match`로
처리하는데, 둘 다 `numberingType`/`zOrder`/`textWrap`/`textFlow`/`chartIDRef`(또는
`binaryItemIDRef`)/`instid`만 처리하고 `lock` 속성은 매치 대상이 없어
`_ => {}`로 조용히 버려졌다.

`CommonObjAttr`(`src/model/shape.rs`)에도 애초에 개체 잠금 상태를 담을 필드가
없었기 때문에, `src/serializer/hwpx/section.rs`의 `render_common_shape_xml`
(ellipse/arc/polygon/curve/chart 공용 경로 — `ShapeObject::Chart`가 이 경로로
들어간다)과 `src/serializer/hwpx/shape.rs`의 `write_ole`(OLE 전용)는 모두
`lock` 속성을 `"0"` 문자열 리터럴로 고정 방출했다.

결과적으로 원본 HWPX 문서에서 `<hp:chart lock="1" .../>` 또는
`<hp:ole lock="1" .../>`로 잠가 둔 차트·OLE 개체가 rhwp를 거쳐 재저장되면
항상 `lock="0"`(잠금 해제)으로 바뀌어 사용자가 지정한 개체 보호 설정이
조용히 사라진다.

같은 저장소에서 이미 수식(`<hp:equation>`, #2840), 표(`<hp:tbl>`, #2855),
그림(`<hp:pic>`, #2875) 각각에 대해 동일한 `lock="0"` 하드코딩 패턴이 별도로
확인·트래킹되고 있었지만, 차트(`<hp:chart>`)와 OLE(`<hp:ole>`) 경로는 어떤
이슈에서도 다뤄지지 않아 잔여로 남아 있었다.

## 변경 사항

- `src/model/shape.rs`: `CommonObjAttr`에 `locked: bool` 필드 추가
  (#2840에서 제안된 이름과 동일하게 두어 추후 통합을 쉽게 함).
- `src/parser/hwpx/section.rs`:
  - `parse_hp_chart_element`에 `b"lock" => common.locked = attr_str(&attr) == "1"`
    분기 추가.
  - `parse_hp_ole_element`에도 동일한 분기 추가.
- `src/serializer/hwpx/section.rs`: `render_common_shape_xml`의 하드코딩
  `lock="0"`을 `common.locked` 기반 `{lock}` 포맷 인자로 교체.
- `src/serializer/hwpx/shape.rs`: `write_ole`의 하드코딩 `("lock", "0")`을
  `common.locked` 기반 `lock` 변수로 교체.
- `src/document_core/converters/common_obj_attr_writer.rs`: 신규 필드 추가에
  따른 테스트 헬퍼 `make_sample()` 초기화 보정(`locked: false`).

## 테스트 (red → green)

`src/parser/hwpx/section.rs`의 `parser::hwpx::section::tests` 모듈에
`task2931_chart_lock_attr_roundtrips_into_common`을 추가했다. `<hp:chart
lock="1" .../>`를 포함한 최소 HWPX 문서를 파싱해 `ShapeObject::Ole(ole)`의
`ole.common.locked`가 `true`로 보존되는지 확인한다.

- **Red**: `locked` 필드 도입 전에는 `assert!(ole.common.locked, ...)`가
  `CommonObjAttr`에 해당 필드가 없어 `error[E0609]: no field \`locked\` on
  type \`model::shape::CommonObjAttr\`` 컴파일 오류로 실패했다(파서 미구현
  상태를 그대로 드러냄).
- **Green**: `locked` 필드 추가 + 파서 2곳(`parse_hp_chart_element`,
  `parse_hp_ole_element`)에 `lock` 매칭 추가 + 직렬화 2곳(`render_common_shape_xml`,
  `write_ole`) 하드코딩 제거 후 `cargo test --lib
  task2931_chart_lock_attr_roundtrips_into_common` 통과.

## 검증 명령과 결과

- `cargo check --lib`: 성공(경고 없음).
- `cargo test --lib task2931_chart_lock_attr_roundtrips_into_common`: 1개
  통과.
- `cargo test --lib -- chart lock ole`: 187개 통과, 0개 실패(회귀 없음;
  차트/락/OLE 관련 기존 테스트 전부 정상).
- `rustfmt --edition 2021 <변경 파일 5개>`: 실질 diff 없음(CRLF 개행 경고만
  발생).

## 완료 기준

차트(`<hp:chart>`)·OLE(`<hp:ole>`) 개체의 `lock` 속성이 IR(`CommonObjAttr.locked`)을
거쳐 HWPX 왕복 시 보존됨을 최소 단위 테스트로 확인했다. `render_common_shape_xml`이
공유하는 ellipse/arc/polygon/curve 경로도 동일한 필드를 참조하므로 함께 개선된다
(단, 이 경로들의 파서는 `parse_object_element_attrs`를 통해 별도 이슈(#2840 PR
방향)로 다뤄질 예정이며 본 작업 범위 밖이다).
