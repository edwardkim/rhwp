## 요약

- HWP/HWPX 표 셀의 `셀 보호`, `필드 이름`, `양식 모드에서 편집 가능` 속성을 파싱/직렬화/API/Studio UI에서 보존하도록 했습니다.
- 보호 셀 위 hover 표시, 보호 셀 클릭 시 셀 선택 전환, 보호 셀 입력 차단을 구현했습니다.
- 보호 셀 선택 상태에서 `셀 속성...`, 표 객체 선택 상태에서 `표 속성...`에 진입할 수 있도록 했습니다.
- 표 외곽선 클릭으로 표 객체 선택이 가능하도록 보정했습니다.
- `표/셀 속성` 대화상자는 탭 이동 시 모달 크기가 변하지 않도록 고정했습니다.

## 검증

- `cargo test --test issue_493_cell_attrs`
- `cargo test --test issue_493_hwpx_cell_field_name`
- `cargo test --test issue_258_clickhere_form_mode`
- `cargo test set_cell_field_text_updates_text_metadata --lib`
- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `wasm-pack build --target web --out-dir pkg`
- `git diff --check`
- `cd rhwp-studio && npm run build`
- `samples/셀보호.hwp` Studio 동작 검증
- 작업지시자 시각 검증 완료

## 참고

- 작업 모드: 기여자 모드. 오늘할일 문서는 생성하지 않았습니다.

Closes #493
