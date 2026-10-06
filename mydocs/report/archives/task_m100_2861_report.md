# Task #2861 처리 결과

## 개요

HWPX `<hp:pic reverse="...">` (그림 좌우 반전) 속성이 파서에서 아예 읽히지 않고, 직렬화기는
IR 과 무관하게 항상 `reverse="0"` 을 방출하던 결함을 수정했다. `lock`(#2840)·`lockForm`(#2839)·
`affectLSpacing`(#2784)·`widthRelTo`/`protect`(#2712) 와 동일한 유형("파싱 누락 + 직렬화
하드코딩")이다.

## 근거

- `mydocs/plans/archives/task_43_feature_def.md:177` — 한컴 Automation `InsertPicture` 옵션
  목록에 `reverse` 명시.
- `mydocs/report/archives/task_m100_182_report.md:164` — `<hp:pic>` 방출 속성 표에 `reverse`
  가 요소 속성 중 하나로 명시돼 있으나 코드 구현은 되지 않았음.

## 수정 내용

- `src/model/image.rs`: `Picture` 에 `reverse: bool` 필드 추가.
- `src/parser/hwpx/section.rs`: `<hp:pic>` 속성 파싱 루프에 `b"reverse"` 매칭 추가, `pic.reverse`
  에 반영.
- `src/serializer/hwpx/picture.rs`: `("reverse", "0")` 하드코딩을 `pic.reverse` 기반 `bool01()`
  출력으로 교체.
- `src/wasm_api/tests.rs`: `Picture` 구조체 리터럴 2곳에 `reverse: ref_pic.reverse` 필드 추가
  (신규 필드로 인한 컴파일 오류 수정, 로직 변경 없음).

## 테스트 (red → green)

`src/serializer/hwpx/picture.rs::tests::issue2861_pic_reverse_is_preserved_on_serialize` 추가.

- 수정 전: `Picture` 에 `reverse` 필드가 없어 컴파일 불가(혹은 필드 추가만 하고 직렬화기 미수정
  시 `reverse="0"` 하드코딩으로 실패).
- 수정 후: `pic.reverse = true` 설정 후 직렬화한 XML 에 `reverse="1"` 포함 확인 — 통과.

```
cargo test --lib issue2861_pic_reverse
running 1 test
test serializer::hwpx::picture::tests::issue2861_pic_reverse_is_preserved_on_serialize ... ok
```

## 검증

- `cargo build --lib` — 성공
- `cargo test --lib issue2861_pic_reverse` — 통과
- `cargo clippy --all-targets --profile release-test -- -D warnings` — 경고 없음
- `rustfmt --edition 2021` — 변경 파일 4개만 적용

## 관련

- Closes #2861
