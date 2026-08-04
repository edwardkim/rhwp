# task_m100_3017: HWP3 각주 분리선 길이(footnote_line_width) 미매핑 수정

## 이슈

edwardkim/rhwp#3017

## 문제

`src/parser/hwp3/mod.rs`의 HWP3 문서 정보 파서가 각주 옵션 필드
`footnote_line_width`(HWP3 스펙 §3.2, offset 111: 각주 분리선 길이 종류.
0=5cm, 1=본문 폭의 1/3, 2=단 너비, 3=없음)를 `Hwp3DocInfo`로 파싱만 하고,
IR(`SectionDef.footnote_shape.separator_length`)로 전혀 전달하지 않았다.
결과적으로 HWP3 문서의 실제 설정값과 무관하게 각주 분리선 길이가 항상
기본값 0(선 없음)으로 렌더링되는 버그였다.

## 원인

`section_def.page_def` 구성 직후, 섹션 정의(SectionDef)를 만드는 지점에서
`doc_info.footnote_line_width`를 읽어 쓰는 코드가 없었다. 필드 자체는
`records.rs`의 `Hwp3DocInfo`에 정의되고 바이트 스트림에서 정상적으로
읽히지만(`records.rs:90`), 이후 어디에서도 참조되지 않았다.

## 수정

- `hwp3_footnote_separator_length(footnote_line_width: u8, column_width_hu: i32) -> i32`
  순수 함수를 추가해 스펙 4종 값을 HWPUNIT 길이로 변환.
  - 0 → 5cm(≈14160 HWPUNIT, 1mm≈283.2 HWPUNIT)
  - 1 → 본문 폭(`column_width_hu`)의 1/3
  - 2 → 본문 폭 그대로(단 너비)
  - 3 이상 → 0(없음)
- `section_def.footnote_shape.separator_length` / `separator_line_type` /
  `separator_line_width`를 이 값으로 채우도록 `section_def.page_def` 구성
  직후에 3줄 추가.
- 단위 테스트 `hwp3_maps_footnote_separator_length`로 0/1/2/3 네 케이스
  매핑을 직접 검증.

## 검증

- `cargo check --lib`: 통과.
- `cargo test --lib hwp3_maps_footnote_separator_length`: 통과(1 passed).
- `rustfmt --edition 2021 src/parser/hwp3/mod.rs`: 적용.

## 범위

- 변경 파일: `src/parser/hwp3/mod.rs` (순수 함수 + 3줄 배선 + 단위 테스트).
- 렌더러, 레이아웃, 공통 IR 등 다른 계층에는 손대지 않음(HWP3 전용 해석은
  `src/parser/hwp3/` 안에서 종료).
- 후속 과제: 각주 분리선의 `separator_line_type`/`separator_line_width`
  자체를 결정하는 HWP3 스펙 필드가 별도로 없어 관례값(실선/굵기 1)을
  사용했다. 실제 문서 대량 검증으로 값이 다르면 후속 조정이 필요할 수
  있다.
