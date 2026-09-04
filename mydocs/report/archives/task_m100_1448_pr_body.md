## 요약

Closes #1448

- rhwp-studio에 미저장 문서 자동 백업 저장소와 autosave manager를 추가했습니다.
- 앱 시작 시 복구 가능한 draft가 있으면 복구/삭제/나중에 선택 대화상자를 표시합니다.
- 복구본은 원본을 자동으로 덮어쓰지 않고 dirty 상태로 열리며, HWPX 출처 draft는 `.hwp` 복구본으로 열리도록 정책을 정리했습니다.
- 새 문서, 기존 HWP 문서, HWPX 출처 문서 복구를 실제 E2E로 검증했습니다.

## 주요 변경

- 자동 백업
  - `rhwpStudioAutosave` IndexedDB 저장소 추가
  - dirty/document-mutated/document-changed 이벤트 기반 debounce 저장
  - 정상 저장/문서 교체/변경 버림 시 draft 정리
- 복구 UI
  - 앱 시작 시 복구 후보 안내
  - `복구`, `삭제`, `나중에` 선택 지원
  - 원본 파일 자동 덮어쓰기 금지 안내
- HWPX 출처 정책
  - autosave draft는 HWP export 결과이므로 HWPX 출처도 `.hwp` 복구본으로 열림
  - 복구 후보 표시에서 `HWPX → HWP 복구본` 문구 표시
- 테스트
  - autosave store/manager/recovery format 단위 테스트 추가
  - 새 문서/HWP/HWPX 복구 E2E 추가

## 검증

- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cd rhwp-studio && npm test`
- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm run build`
- `cd rhwp-studio && CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" VITE_URL=http://localhost:7700 node e2e/autosave-recovery.test.mjs --mode=headless`
- `git diff --check`
