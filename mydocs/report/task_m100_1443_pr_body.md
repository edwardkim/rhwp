## 요약

Closes #1443

- 표 셀 마우스 드래그 선택, 보호 셀 상태 전환, 표 선택 뒤 셀 편집 진입을 한컴 동작에 맞춰 보정했습니다.
- `셀 너비를 같게` / `셀 높이를 같게`, Shift+마우스 개별 셀 크기 조절, 이후 일반 컬럼/행 resize 조합을 선택 범위와 셀 segment 기준으로 동작하도록 정리했습니다.
- `Alt+C` 모양복사를 한컴 호환 단축키로 추가하고, 툴바/메뉴 연결 및 붙여넣기 후 자동 해제되는 일회성 동작으로 정리했습니다.
- `셀보호2.hwp` / `셀보호2.hwpx` / 한컴 PDF 기준으로 셀 안 여백, 표 속성, 렌더링 회귀를 검증했습니다.

## 주요 변경

- Studio 표 입력 처리
  - 마우스 드래그 셀 선택
  - 보호 셀 hover/click 상태 전환
  - 표 전체 선택 뒤 셀 내부 진입
  - Shift 개별 셀 segment resize와 일반 resize clamp 분리
- 표 모델/렌더링
  - local resize row/column hint 추가
  - 보상 resize 시 표 common width/height 보존
  - 셀 안 여백 on/off에 따른 텍스트 리플로우 보정
- 메뉴/단축키
  - `Alt+C` 모양복사
  - 툴바 모양복사 아이콘 실제 기능 연결
  - 일회성 모양복사 상태 정리
- 회귀 테스트
  - `tests/issue_493_cell_attrs.rs`에 셀 속성, 안 여백, 표 크기 보존, local/global resize 조합 회귀 추가

## 검증

- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm test`
- `wasm-pack build --target web --out-dir pkg`

