# Task #4837 Stage 1 - HWPX field axis maintainer correction

## 목적

PR #4837 검토에서 확인된 HWPX 문단 축 결함 두 건을 메인터너 보정으로 처리한다.

## 원인

- `titleMark`는 텍스트가 아닌 8 UTF-16 unit 슬롯이지만, field range 계산은 내부 센티널
  `"\\u{0008}1"` 또는 `"\\u{0008}0"`의 두 글자를 visible text로 더했다. 따라서 뒤따르는
  field begin/end가 두 글자 늦은 위치로 저장됐다.
- HWPX의 다문단 `fieldEnd`는 `beginIDRef`만 보존했다. HWP5 저장은 field control fourcc가
  있어야 종료 슬롯을 쓰는데, 이 값을 앞 문단 `fieldBegin`에서 연결하지 않아 종료 마커를
  생략했다.

## 변경

- title marker 센티널을 visible-text field-range 축에서 0 길이로 처리한다.
- 문단 목록을 순서대로 훑어 HWPX orphan field end에 짝 field begin의 control id를 연결한다.
- titleMark+field 및 HWPX 다문단 field 연결을 회귀 테스트로 고정한다.

## 검증 계획

- `cargo test --lib parser::hwpx::section::tests::title_mark_does_not_shift_following_field_range`
- `cargo test --lib parser::hwpx::section::tests::task1556_orphan_field_end_recorded_in_end_paragraph`
