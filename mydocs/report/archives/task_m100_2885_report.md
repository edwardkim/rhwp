# Task m100-2885 처리 결과 — pageBorderFill type(BOTH/EVEN/ODD) 슬롯 배정 수정

## 이슈

[#2885](https://github.com/edwardkim/rhwp/issues/2885) — `hp:pageBorderFill`의 `type`
속성(BOTH/EVEN/ODD)이 파싱만 되고 슬롯 배정(`SectionDef.page_border_fill` /
`extra_page_border_fills`)에는 전혀 반영되지 않고, XML 등장 순서로만 슬롯이
결정되던 결함.

## 근본 원인

- `src/parser/hwpx/section.rs`의 `parse_page_border_fill_empty`가 `type` 속성을
  `apply_type` 지역변수로 읽었지만, `page_border_fill_attr()` 호출로 전달만 되고
  해당 함수 본문에서 한 번도 사용되지 않아 값이 그대로 폐기됨.
- 슬롯 배정 함수 `push_page_border_fill`은 `type` 값과 무관하게
  카운터(`count == 0` → BOTH 슬롯, 그 외 → EVEN/ODD 순서대로 append)만으로
  동작 — HWPX 스펙이 강제하지 않는 "BOTH → EVEN → ODD 등장 순서" 가정에 의존.

## 수정 내용

- `src/parser/hwpx/section.rs`
  - `parse_page_border_fill_empty` / `parse_page_border_fill`이 파싱한
    `PageBorderFill`과 함께 `type` 문자열도 반환하도록 시그니처 변경.
  - `push_page_border_fill`이 `apply_type: &str` 인자를 받아 `"BOTH"`/`"EVEN"`/
    `"ODD"` 값에 따라 정확한 슬롯에 배정하도록 재작성. 인식 불가/누락 값에
    한해서만 기존 등장 순서 기반 폴백 유지(회귀 방지).
  - 두 호출부(`Event::Start`, `Event::Empty`)를 신규 시그니처에 맞게 갱신.

`src/serializer/hwpx/section.rs`(직렬화 쪽 BOTH/EVEN/ODD 3-슬롯 템플릿 치환
로직)는 이번 수정 범위 밖 — IR 배정이 정확해지면 기존 직렬화 로직은 그대로
올바르게 동작함.

## 테스트 (Red → Green)

`src/parser/hwpx/section.rs::tests::test_parse_page_border_fill_slot_by_type_not_by_order`

- XML에 `type="EVEN"`(borderFillIDRef=7)을 먼저, `type="BOTH"`
  (borderFillIDRef=9)을 나중에 배치.
- 수정 전: 등장 순서 기반 로직이 첫 요소(EVEN, id=7)를 `page_border_fill`
  (BOTH 슬롯)에 잘못 배정 → `assert_eq!(..border_fill_id, 9)` 실패.
- 수정 후: `type` 값 기반 배정으로 `page_border_fill.border_fill_id == 9`
  (실제 BOTH 요소) 확인 → 통과.

## 검증

```
cargo build --lib                                             # 성공
cargo test --lib page_border_fill                              # 6 passed
cargo clippy --all-targets --profile release-test -- -D warnings  # 경고 0
rustfmt --edition 2021 src/parser/hwpx/section.rs               # 적용
```

## 변경 파일

- `src/parser/hwpx/section.rs`
