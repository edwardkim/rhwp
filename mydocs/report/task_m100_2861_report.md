# Task m100-2861: HWPX `hp:pic@reverse`(좌우 반전) 파싱 누락 + 직렬화 하드코딩 수정

## 배경

이슈 #2861. `<hp:pic reverse="...">` 는 그림의 좌우 반전 여부를 나타내지만, 파서는 이 속성을
전혀 읽지 않았고(`_ => {}` 로 조용히 폐기) `Picture` IR 에도 값을 담을 필드가 없었다. 시리얼라이저는
IR 값과 무관하게 항상 리터럴 `"0"` 을 방출했다. 결과적으로 `reverse="1"` 로 저장된(좌우 반전
삽입된) 그림을 rhwp 로 열었다가 그대로 저장하면 반전이 풀려 원본과 다르게 렌더링됐다.

`lock`(#2840), `lockForm`(#2839), `affectLSpacing`(#2784) 등과 동일한 "파싱 누락 + 직렬화
하드코딩" 결함 유형이다.

## 조사

- `mydocs/plans/archives/task_43_feature_def.md:177` — `InsertPicture` 옵션 목록에 `reverse` 명시.
- `mydocs/report/archives/task_m100_182_report.md:164` — `<hp:pic>` 방출 속성 표에 `reverse` 포함.
- `src/parser/hwpx/section.rs` `parse_picture()` 의 `<hp:pic>` 속성 매칭 루프가 `reverse` 를
  다루지 않음을 코드로 확인.
- `src/serializer/hwpx/picture.rs` `write_picture()` 의 `start_tag_attrs` 호출이
  `("reverse", "0")` 로 리터럴 고정돼 있음을 코드로 확인.
- 바탕쪽(masterPage) `type`/`pageDuplicate`/`pageNumber`/`pageFront`/`hasTextRef`/`hasNumRef`/
  `textDirection` 은 모두 기존 코드와 테스트에서 정상 왕복함을 `src/parser/hwpx/section.rs`,
  `src/serializer/hwpx/master_page.rs` 를 직접 읽어 확인했다(추가 결함 없음). 이번 수정은 별도
  요소인 `hp:pic@reverse` 를 대상으로 한다.

## 수정

- `src/model/image.rs`: `Picture` 에 `reverse: bool` 필드 추가.
- `src/parser/hwpx/section.rs`: `parse_picture()` 가 `hp:pic@reverse` 속성을 읽어
  `pic.reverse` 에 반영(`"0"` 이외 값은 `true`).
- `src/serializer/hwpx/picture.rs`: `write_picture()` 가 `pic.reverse` 값을 `"1"`/`"0"` 으로
  방출(종전 하드코딩 `"0"` 제거).
- `src/wasm_api/tests.rs`: `Picture` 리터럴 초기화 2곳에 `reverse: ref_pic.reverse,` 필드 추가
  (컴파일 유지, 참조 파일 값 그대로 전달).

## 테스트 (red → green)

`src/serializer/hwpx/picture.rs::tests::task2861_reverse_true_round_trips`

- 수정 전: `pic.reverse` 필드 자체가 없어 컴파일 불가(구조적 red) — 필드 추가와 동시에
  시리얼라이저가 `"0"` 하드코딩이면 `assert!(xml.contains(r#"reverse="1""#))` 가 실패.
- 수정 후: `write_picture` 가 `pic.reverse` 를 반영해 `reverse="1"` 방출 → 통과.

```
cargo test --lib task2861_reverse_true_round_trips
test serializer::hwpx::picture::tests::task2861_reverse_true_round_trips ... ok
```

## 검증

- `cargo build --lib` — 통과
- `cargo test --lib task2861_reverse_true_round_trips` — 통과
- `cargo clippy --all-targets --profile release-test -- -D warnings` — 경고 없음
- `rustfmt --edition 2021` (변경 파일만: `src/model/image.rs`, `src/parser/hwpx/section.rs`,
  `src/serializer/hwpx/picture.rs`, `src/wasm_api/tests.rs`)

## 범위

변경 파일 4개, 이슈 #2861 범위로 한정. 다른 리팩터링 없음.
