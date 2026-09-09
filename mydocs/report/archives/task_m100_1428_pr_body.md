## 요약

- rhwp-studio 개체 속성 기본 탭에 `비율 유지` 설정을 추가했습니다.
- `비율 유지` OFF 상태에서는 너비/높이를 독립 입력하고, ON 상태에서는 기존 비율에 맞춰 반대 축을 보정합니다.
- `비율 유지` 선택값은 문서 포맷 속성이 아니라 `rhwp-settings`에 저장되는 사용자 UI 설정으로 처리했습니다.
- 개체 속성 및 주요 모달이 외부 클릭만으로 닫히지 않도록 정리했습니다.
- 누름틀 고치기 완료 후 포커스/캐럿 복귀, 빈 guide 클릭, 경계 바깥 클릭, 인접 누름틀 guide hit-test, 누름틀 붙여넣기 후 입력 위치를 보정했습니다.

## 검증

- `git diff --check upstream/devel..HEAD`
- `cargo fmt --check`
- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm test`
- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo clippy --all-targets -- -D warnings`
- `wasm-pack build --target web --out-dir pkg`
- `cd rhwp-studio && npm run build`

## 참고

- 작업 모드: 기여자 모드. 오늘할일 문서는 생성하지 않았습니다.
- `비율 유지`는 HWP/HWPX 파일 저장 속성이 아니라 Studio 대화상자 사용자 설정입니다.

Closes #1428
