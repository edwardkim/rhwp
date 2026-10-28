# Task M100-493 Stage 1 — 셀 속성 보존 구현

- 이슈: #493
- 브랜치: `local/task_m100_493`
- 작성일: 2026-06-19

## 1. 진행 상태

셀 보호, 필드 이름, 양식 모드에서 편집 가능 속성의 파서/직렬화/API/UI 1차 구현을 완료했다.

작성 문서:

- `mydocs/plans/task_m100_493.md`
- `mydocs/plans/task_m100_493_impl.md`

## 2. 사전 확인

- 제공 샘플 `samples/셀보호.hwpx`는 5×5 표를 포함한다.
- HWPX 원본의 셀 속성:
  - 셀[0], 셀[1], 셀[2]: `protect="1"`
  - 셀[2]: `name="name"`, `editable="1"`, 텍스트 `12334`
- 현재 `hwpx-roundtrip`은 IR 차이 0건으로 통과하지만, 생성된 `셀보호.rt.hwpx`에서는
  `protect="1"`과 `editable="1"`이 모두 `"0"`으로 바뀐다.
- 원인은 HWPX 파서가 `hp:tc/@protect`, `hp:tc/@editable`을 읽지 않고,
  HWPX serializer가 두 속성을 고정 `"0"`으로 방출하기 때문이다.
- HWP5 경로는 `Cell::list_header_width_ref` bit 1/3으로 셀 보호와 양식 모드 편집 가능을
  표현할 수 있는 기존 저장소가 있다.

## 3. 구현 내용

- `Cell::list_header_width_ref` bit 0~3에 대한 상수와 helper를 추가했다.
  - bit 0: 안 여백 지정
  - bit 1: 셀 보호
  - bit 2: 제목 셀
  - bit 3: 양식 모드에서 편집 가능
- HWPX `hp:tc/@protect`, `hp:tc/@editable`을 파싱해 공통 bit 저장소에 반영했다.
- HWPX serializer가 `protect`, `editable`을 고정 `"0"`이 아니라 셀 bit 값으로 방출하도록 수정했다.
- `getCellProperties()` 결과에 `fieldName`, `editableInForm`을 추가했다.
- `setCellProperties()` 입력에서 `fieldName`, `editableInForm`을 반영하도록 확장했다.
- 셀 `field_name` 기반 가상 필드가 `editableInForm`을 셀 bit 3과 맞추도록 수정했다.
- 표/셀 속성 다이얼로그의 필드 이름, 양식 모드 편집 가능 컨트롤을 활성화하고 값을 주고받도록 수정했다.
- 제공 샘플 기반 회귀 테스트 `tests/issue_493_cell_attrs.rs`를 추가했다.

## 4. 검증 결과

통과:

- `cargo test --test issue_493_cell_attrs`
- `cargo test --test issue_493_hwpx_cell_field_name`
- `cargo test --test issue_258_clickhere_form_mode`
- `cargo test set_cell_field_text_updates_text_metadata --lib`
- `cargo fmt --check`
- `npm run build` (`rhwp-studio`)
- `cargo build --release`
- `cargo test --release --lib`

참고:

- `npm run build` 중 Vite가 기존 대용량 chunk 경고를 출력했으나 빌드는 성공했다.

## 5. 남은 확인

- 전체 PR 전 검증은 `dev_environment_guide.md`의 macOS 검증 흐름을 따른다.
- Stage 1 설계상 HWPX→HWP 저장은 `list_header_width_ref` bit를 그대로 쓰므로 별도 adapter 수정은 필요 없어 보인다. 다만 PR 전 전체 검증 중 이상이 있으면 Stage 2에서 보강한다.
